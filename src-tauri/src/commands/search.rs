use crate::db;
use crate::index::{SimpleIndex, DualIndex};
use crate::models::clip::ClipModel;
use serde_json::json;
use tauri::Manager;

// Max results per search to avoid IPC buffer overflow.
// Set high enough that it won't affect normal use; if the user has 50k+ images
// they can expect thousands of results.
const MAX_SEARCH_RESULTS: usize = 5000;

#[tauri::command(rename_all = "camelCase")]
pub fn search_by_text(
    app: tauri::AppHandle,
    query: String,
    topK: usize,
) -> Result<serde_json::Value, String> {
    let top_k = topK.min(MAX_SEARCH_RESULTS);
    eprintln!("[SEARCH] topK={}, capped at MAX_SEARCH_RESULTS={}", topK, MAX_SEARCH_RESULTS);
    eprintln!("[SEARCH] search_by_text called: query={}, top_k={}", query, top_k);
    let db_state = app.state::<std::sync::Mutex<rusqlite::Connection>>();
    let conn = db_state.lock().map_err(|e| e.to_string())?;

    let mut images = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    // Get enabled folder paths once — used to filter results by active folders
    let enabled_paths = db::get_enabled_paths(&conn).unwrap_or_default();
    eprintln!("[SEARCH] enabled_paths ({}): {:?}", enabled_paths.len(), enabled_paths);
    let enabled_paths_set: std::collections::HashSet<&str> =
        enabled_paths.iter().map(|p| p.as_str()).collect();

    // Strategy 1: CLIP semantic search (primary, when vectors available)
    let index_state = app.state::<std::sync::Mutex<DualIndex>>();
    let index_empty = {
        let idx = index_state.lock().map_err(|e| e.to_string())?;
        let len = idx.text_index.len();
        eprintln!("[SEARCH] Text index length: {}", len);
        len == 0
    };
    eprintln!("[SEARCH] text_index_empty={}", index_empty);

    if !index_empty {
        let clip_state = app.state::<std::sync::Mutex<ClipModel>>();
        let clip = clip_state.lock().map_err(|e| e.to_string())?;

        // Use CLIP-L/14 for text encoding, search in text_index
        match clip.encode_text_clip_large(&query) {
            Ok(query_vector) => {
                eprintln!("[SEARCH] encode_text_clip_large OK, vec len={}, first3={:?}", query_vector.len(), &query_vector[..3.min(query_vector.len())]);
                drop(clip);
                let idx = index_state.lock().map_err(|e| e.to_string())?;
                // Fetch up to 2x results to account for disabled-path filtering
                let vec_results = idx.text_index.search(&query_vector, top_k * 2);
                eprintln!("[SEARCH] text vec_results count={}, first 3: {:?}", vec_results.len(), &vec_results[..vec_results.len().min(3)]);
                drop(idx);
                for (image_id, similarity) in vec_results {
                    // SigLIP2 的相似度分布与 CLIP 不同，暂不设阈值
                    // if similarity < 0.15 { continue; } // too dissimilar
                    if similarity < -1.0 { continue; } // 仅过滤无效值
                    if let Ok(Some(info)) = db::get_image_by_id(&conn, image_id) {
                        // Filter: skip if this image's folder is disabled
                        if let Some(ref fp) = info.folder_path {
                            if !enabled_paths_set.contains(fp.as_str()) {
                                continue;
                            }
                        }
                        seen_ids.insert(image_id);
                        images.push(json!({
                            "id": image_id,
                            "path": info.path,
                            "similarity": similarity,
                            "width": info.width,
                            "height": info.height,
                        }));
                        if images.len() >= top_k { break; }
                    }
                }
            }
            Err(e) => {
                eprintln!("[SEARCH] CLIP encode_text failed: {}", e);
            }
        }
    }

    // Strategy 2: Filename/keyword match (supplement)
    if images.len() < top_k {
        let like_results = db::search_images_by_text(&conn, &query, top_k * 2).unwrap_or_default();
        for info in &like_results {
            if let Some(id) = info.id {
                if !seen_ids.contains(&id) {
                    // Filter by enabled paths (same as above)
                    if let Some(ref fp) = info.folder_path {
                        if !enabled_paths_set.contains(fp.as_str()) {
                            continue;
                        }
                    }
                    seen_ids.insert(id);
                    let file_name = info.path.rsplit(|c| c == '/' || c == '\\').next().unwrap_or("");
                    let relevance = compute_relevance(&query, file_name);
                    images.push(json!({
                        "id": id,
                        "path": info.path,
                        "similarity": relevance,
                        "width": info.width,
                        "height": info.height,
                    }));
                    if images.len() >= top_k { break; }
                }
            }
        }
    }

    // Sort by similarity desc
    images.sort_by(|a, b| {
        b["similarity"].as_f64().unwrap_or(0.0)
            .partial_cmp(&a["similarity"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(json!({ "query": query, "count": images.len(), "images": images }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn search_by_image(
    app: tauri::AppHandle,
    imagePath: String,
    topK: usize,
) -> Result<serde_json::Value, String> {
    let top_k = topK.min(MAX_SEARCH_RESULTS);
    eprintln!("[SEARCH] search_by_image called: imagePath={}, top_k={}", imagePath, top_k);
    let clip_state = app.state::<std::sync::Mutex<ClipModel>>();
    let clip = clip_state.lock().map_err(|e| e.to_string())?;
    let query_vector = clip.encode_image_siglip2(&imagePath)
        .map_err(|e| format!("Failed to encode image: {}", e))?;
    drop(clip);

    let index_state = app.state::<std::sync::Mutex<DualIndex>>();
    let results = {
        let idx = index_state.lock().map_err(|e| e.to_string())?;
        // Fetch extra to account for disabled-path filtering
        idx.image_index.search(&query_vector, top_k * 2)
    };

    let db_state = app.state::<std::sync::Mutex<rusqlite::Connection>>();
    let conn = db_state.lock().map_err(|e| e.to_string())?;

    // Get enabled folder paths for filtering
    let enabled_paths = db::get_enabled_paths(&conn).unwrap_or_default();
    let enabled_paths_set: std::collections::HashSet<&str> =
        enabled_paths.iter().map(|p| p.as_str()).collect();

    let mut images = Vec::new();
    for (image_id, similarity) in results {
        // SigLIP2 的相似度分布与 CLIP 不同，暂不设阈值
        // if similarity < 0.15 { continue; }
        if similarity < -1.0 { continue; } // 仅过滤无效值
        if let Ok(Some(info)) = db::get_image_by_id(&conn, image_id) {
            // Filter: skip if this image's folder is disabled
            if let Some(ref fp) = info.folder_path {
                if !enabled_paths_set.contains(fp.as_str()) {
                    continue;
                }
            }
            images.push(json!({
                "id": image_id,
                "path": info.path,
                "similarity": similarity,
                "width": info.width,
                "height": info.height,
            }));
        }
    }

    Ok(json!({ "query_image": imagePath, "count": images.len(), "images": images }))
}

/// Compute a simple relevance score for text matching against filename
fn compute_relevance(query: &str, filename: &str) -> f32 {
    let query_lower = query.to_lowercase();
    let file_lower = filename.to_lowercase();
    if file_lower.contains(&query_lower) {
        return 0.95;
    }
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();
    let mut match_count = 0;
    for word in &query_words {
        if !word.is_empty() && file_lower.contains(word) {
            match_count += 1;
        }
    }
    if !query_words.is_empty() && match_count > 0 {
        return 0.5 + (match_count as f32 / query_words.len() as f32) * 0.4;
    }
    let matched_chars: usize = query_lower.chars().filter(|c| file_lower.contains(*c)).count();
    let total_chars = query_lower.chars().filter(|c| c.is_alphanumeric()).count();
    if total_chars > 0 {
        return 0.1 + (matched_chars as f32 / total_chars as f32) * 0.3;
    }
    0.05
}
