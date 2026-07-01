use crate::db;
use crate::index::{SimpleIndex, DualIndex};
use crate::models::clip::ClipModel;
use tauri::{Emitter, Manager};
use walkdir::WalkDir;
use serde_json::json;
use sha2::{Sha256, Digest};

const METADATA_BATCH: usize = 50;
const CLIP_BATCH: usize = 50; // Images per CLIP encode batch (GPU models handle larger batches efficiently)

#[tauri::command]
pub async fn index_images(app: tauri::AppHandle, directory: String, force_reencode: Option<bool>) -> Result<String, String> {
    let reencode = force_reencode.unwrap_or(true); // Default: always re-encode CLIP vectors
    let folder_path = directory.trim_end_matches(|c| c == '\\' || c == '/').to_string();
    println!("[INDEX] force_reencode={}, will re-encode CLIP vectors: {}", reencode, reencode);
    println!("[INDEX] Starting index for directory: {}", folder_path);

    app.emit("index-progress", json!({
        "counted": 0, "indexed": 0, "errors": 0,
        "status": "started", "dir": directory
    })).ok();

    let dir_path = std::path::Path::new(&directory);
    if !dir_path.exists() {
        return Err(format!("Directory does not exist: {}", directory));
    }
    if !dir_path.is_dir() {
        return Err(format!("Path is not a directory: {}", directory));
    }

    let db_state = app.state::<std::sync::Mutex<rusqlite::Connection>>();
    let supported_exts = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "tif"];

    // ===== Phase 1: Scan + metadata =====
    let mut all_image_paths: Vec<String> = Vec::new();
    for entry_result in WalkDir::new(&directory).follow_links(true).into_iter() {
        let entry = match entry_result {
            Ok(e) => e,
            Err(e) => { println!("[INDEX] WalkDir error: {}", e); continue; }
        };
        let path = entry.path();
        if !path.is_file() { continue; }
        let ext = path.extension()
            .and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        if !supported_exts.contains(&ext.as_str()) { continue; }
        all_image_paths.push(path.to_string_lossy().to_string());
    }

    let total_files = all_image_paths.len();
    println!("[INDEX] Found {} image files", total_files);

    if total_files == 0 {
        app.emit("index-progress", json!({
            "counted": 0, "indexed": 0, "errors": 0, "status": "completed"
        })).ok();
        return Ok("No image files found".to_string());
    }

    let mut indexed_meta = 0usize;
    let mut errors = 0usize;
    let mut scanned = 0usize;

    for batch in all_image_paths.chunks(METADATA_BATCH) {
        for path_str in batch {
            scanned += 1;
            if scanned % 10 == 0 || scanned == 1 {
                app.emit("index-progress", json!({
                    "counted": total_files, "indexed": indexed_meta, "errors": errors,
                    "status": "scanning",
                    "scanned": scanned,
                })).ok();
            }

            let hash = match compute_file_hash(path_str) {
                Ok(h) => h,
                Err(e) => {
                    println!("[INDEX] Hash error {}: {}", path_str, e);
                    errors += 1; continue;
                }
            };

            // Skip unchanged files
            {
                let conn = db_state.lock().map_err(|e| e.to_string())?;
                if let Ok(Some(existing)) = db::get_image_by_path(&conn, path_str) {
                    if existing.hash == hash { continue; }
                }
            }

            let (width, height) = match get_image_dimensions(path_str) {
                Ok(d) => d,
                Err(e) => {
                    println!("[INDEX] Dims error {}: {}", path_str, e);
                    errors += 1; continue;
                }
            };

            {
                let conn = db_state.lock().map_err(|e| e.to_string())?;
                let info = db::ImageInfo {
                    id: None, path: path_str.clone(), hash: hash.clone(),
                    width: Some(width as i32), height: Some(height as i32),
                    main_colors: None, clip_vector: None, clip_vector_siglip2: None,
                    exif_camera_make: None, exif_camera_model: None,
                    exif_aperture: None, exif_iso: None,
                    exif_shutter_speed: None, exif_focal_length: None,
                    exif_taken_at: None,
                    indexed_at: Some(std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64),
                    folder_path: Some(folder_path.clone()),
                };
                db::insert_or_update_image(&conn, &info).map_err(|e| e.to_string())?;
            }
            indexed_meta += 1;
        }
    }
    println!("[INDEX] Phase 1 done: {} indexed, {} errors", indexed_meta, errors);

    // ===== Phase 2+3: CLIP encoding + memory index (run on background thread to avoid blocking UI) =====
    let (tx, _rx) = std::sync::mpsc::channel::<Result<String, String>>();
    
    // Clone handles needed by the background thread
    let bg_app = app.clone();
    let bg_reencode = reencode;
    let bg_total_files = total_files;
    let bg_indexed_meta = indexed_meta;
    let bg_folder_path = folder_path.clone();
    let bg_errors = errors;

    std::thread::spawn(move || {
        let result = run_clip_encoding(&bg_app, bg_reencode, bg_total_files, bg_indexed_meta, bg_folder_path, bg_errors, reencode);
        let _ = tx.send(result);
    });

    // Return immediately — don't wait for the background thread.
    // Progress is emitted via Tauri events; completion will be emitted from the thread.
    Ok("Phase 1 complete. CLIP encoding running in background...".to_string())
}

#[tauri::command]
pub fn get_index_status(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let db_state = app.state::<std::sync::Mutex<rusqlite::Connection>>();
    let conn = db_state.lock().map_err(|e| e.to_string())?;
    let indexed_count = db::get_indexed_count(&conn).map_err(|e| e.to_string())?;

    let index = app.state::<std::sync::Mutex<DualIndex>>();
    let index_size = index.lock().map_err(|e| e.to_string())?.text_index.len();

    Ok(json!({
        "indexed_count": indexed_count,
        "index_size": index_size,
        "status": if indexed_count > 0 { "ready" } else { "empty" },
        "vector_indexed": index_size > 0
    }))
}

fn compute_file_hash(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut hasher = Sha256::new();
    let mut file = std::fs::File::open(path)?;
    std::io::copy(&mut file, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn get_image_dimensions(path: &str) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    match imagesize::size(path) {
        Ok(dims) => Ok((dims.width as u32, dims.height as u32)),
        Err(e) => Err(Box::new(e)),
    }
}

fn serialize_vector(vec: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vec.len() * 4);
    for f in vec {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    bytes
}

fn deserialize_vector(bytes: &[u8]) -> Option<Vec<f32>> {
    if bytes.len() % 4 != 0 {
        return None;
    }
    let count = bytes.len() / 4;
    let mut vec = Vec::with_capacity(count);
    for i in 0..count {
        let start = i * 4;
        let arr: [u8; 4] = bytes[start..start + 4].try_into().ok()?;
        vec.push(f32::from_le_bytes(arr));
    }
    Some(vec)
}

/// Background thread function for Phase 2 (CLIP encoding) and Phase 3 (load to memory).
/// Runs independently so the UI doesn't freeze during slow ONNX inference.
/// Only encodes images from `folder_path` (or all if `reencode` is true).
fn run_clip_encoding(
    app: &tauri::AppHandle,
    reencode: bool,
    total_files: usize,
    indexed_meta: usize,
    folder_path: String,
    mut errors: usize,
    force_reencode: bool,
) -> Result<String, String> {
    eprintln!("[INDEX] Phase 2: Starting CLIP encoding on background thread...");

    let db_state = app.state::<std::sync::Mutex<rusqlite::Connection>>();
    let clip_state = app.state::<std::sync::Mutex<ClipModel>>();

    // Collect paths that need encoding — ONLY from the folder being indexed
    // (not the entire DB, so adding a new folder doesn't re-encode old ones)
    let paths_to_encode: Vec<String> = {
        let conn = db_state.lock().map_err(|e| e.to_string())?;
        let all = db::get_all_indexed_images(&conn).unwrap_or_default();
        // Filter to this folder first
        let folder_images: Vec<_> = if force_reencode {
            all.iter().filter(|img| img.folder_path.as_deref() == Some(&folder_path)).collect()
        } else {
            all.iter().filter(|img| {
                img.folder_path.as_deref() == Some(&folder_path) && img.clip_vector.is_none()
            }).collect()
        };
        let total = all.len();
        let this_folder = folder_images.len();
        let with_vec = folder_images.iter().filter(|img| img.clip_vector.is_some()).count();
        eprintln!("[INDEX] Phase 2: DB total={}, this_folder={}, with_vector={}, reencode={}",
            total, this_folder, with_vec, force_reencode);
        folder_images.into_iter().map(|img| img.path.clone()).collect()
    };

    let remaining = paths_to_encode.len();
    eprintln!("[INDEX] Phase 2: {} images need CLIP encoding", remaining);

    if remaining > 0 {
        let clip = clip_state.lock().map_err(|e| e.to_string())?;
        let mut clipped_errors = 0u32;
        let mut clipped_ok = 0u32;

        for (chunk_idx, chunk) in paths_to_encode.chunks(CLIP_BATCH).enumerate() {
            // Use total_files as the progress bar denominator throughout so the bar
            // never exceeds 100%. Phase 1 fills 0→total_files, Phase 2 fills
            // total_files→total_files (no further fill since Phase 1 = 100%).
            app.emit("index-progress", json!({
                "counted": total_files,
                "indexed": total_files.min(indexed_meta + (clipped_ok as usize)),
                "errors": errors + clipped_errors as usize,
                "status": "encoding",
                "encoded": clipped_ok,
                "remaining": remaining.saturating_sub((chunk_idx * CLIP_BATCH) + chunk.len()),
            })).ok();

            eprintln!("[INDEX] Encoding batch {}/{}, {} paths",
                chunk_idx + 1, (remaining + CLIP_BATCH - 1) / CLIP_BATCH, chunk.len());

            // Use dual-model batch encoding — stores both SigLIP2 and CLIP-L/14 vectors
            let results = match clip.encode_both_batch(chunk) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[INDEX] Dual batch error (will retry individually): {}", e);
                    let mut fallback_results = Vec::new();
                    for p in chunk {
                        let siglip2 = clip.encode_image_siglip2(p);
                        let cliplarge = clip.encode_text_clip_large(p);
                        if siglip2.is_ok() || cliplarge.is_ok() {
                            fallback_results.push(Ok((
                                siglip2.unwrap_or_default(),
                                cliplarge.unwrap_or_default(),
                            )));
                        } else {
                            let err_msg = format!("Both models failed for {}", p);
                            eprintln!("[INDEX] {}", err_msg);
                            fallback_results.push(Err(err_msg));
                        }
                    }
                    fallback_results
                }
            };

            {
                let conn = db_state.lock().map_err(|e| e.to_string())?;
                for (path_str, result) in chunk.iter().zip(results.iter()) {
                    match result {
                        Ok((siglip2_vec, cliplarge_vec)) => {
                            let siglip2_bytes = if !siglip2_vec.is_empty() { Some(serialize_vector(siglip2_vec)) } else { None };
                            let cliplarge_bytes = if !cliplarge_vec.is_empty() { Some(serialize_vector(cliplarge_vec)) } else { None };
                            if let Err(e) = conn.execute(
                                "UPDATE images SET clip_vector = ?1, clip_vector_siglip2 = ?2 WHERE path = ?3",
                                rusqlite::params![cliplarge_bytes, siglip2_bytes, path_str],
                            ) {
                                eprintln!("[INDEX] DB update error {}: {}", path_str, e);
                                clipped_errors += 1;
                            } else {
                                clipped_ok += 1;
                            }
                        }
                        Err(e) => {
                            eprintln!("[INDEX] CLIP error {}: {}", path_str, e);
                            clipped_errors += 1;
                        }
                    }
                }
            }
        }
        errors += clipped_errors as usize;
        eprintln!("[INDEX] Phase 2 done: {} encoded, {} clip_errors", clipped_ok, clipped_errors);
    }

    // ===== Phase 3: Load vectors into in-memory index =====
    {
        let index_state = app.state::<std::sync::Mutex<DualIndex>>();
        let mut idx = index_state.lock().map_err(|e| e.to_string())?;
        idx.text_index.clear();
        idx.image_index.clear();
        let conn = db_state.lock().map_err(|e| e.to_string())?;
        let all = db::get_all_indexed_images(&conn).unwrap_or_default();
        let mut loaded_text = 0usize;
        let mut loaded_image = 0usize;
        for img in &all {
            if let Some(id) = img.id {
                // CLIP-L/14 vectors (for text search)
                if let Some(ref blob) = img.clip_vector {
                    if let Some(vec) = deserialize_vector(blob) {
                        idx.text_index.add(id, vec);
                        loaded_text += 1;
                    }
                }
                // SigLIP2 vectors (for image search)
                if let Some(ref blob) = img.clip_vector_siglip2 {
                    if let Some(vec) = deserialize_vector(blob) {
                        idx.image_index.add(id, vec);
                        loaded_image += 1;
                    }
                }
            }
        }
        eprintln!("[INDEX] Phase 3 done: {} text vectors + {} image vectors loaded", loaded_text, loaded_image);

        // Update indexed_count for this folder — use ACTUAL count of images with clip_vector,
        // not indexed_meta (which only counts Phase 1 metadata changes).
        let actual_count = all
            .iter()
            .filter(|img| {
                img.folder_path.as_deref() == Some(&folder_path) && img.clip_vector.is_some()
            })
            .count() as i64;
        eprintln!(
            "[INDEX] Updating indexed_count for {}: actual_count={} (was indexed_meta={})",
            folder_path, actual_count, indexed_meta
        );
        if let Err(e) = db::update_path_count(&conn, &folder_path, actual_count) {
            eprintln!("[INDEX] Failed to update path count for {}: {}", folder_path, e);
        }
    }

    app.emit("index-progress", json!({
        "counted": total_files,
        "indexed": indexed_meta,
        "errors": errors,
        "status": "completed"
    })).ok();

    eprintln!("[INDEX] Complete: total={}, indexed={}, errors={}", total_files, indexed_meta, errors);
    Ok(format!("Indexed {} images (CLIP vectors included), {} errors", indexed_meta, errors))
}
