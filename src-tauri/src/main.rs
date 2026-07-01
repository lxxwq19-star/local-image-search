#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;
mod index;
mod db;

use std::sync::Mutex;

fn main() {
    // Initialize database + tables
    let db = match db::init_db()
        .and_then(|conn| { db::init_index_paths_table(&conn)?; Ok(conn) })
    {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize in-memory indices (dual model: CLIP-L/14 for text, SigLIP2 for image)
    let mut dual_index = index::DualIndex::new();

    // Load existing vectors from DB into memory indices
    let all_images = db::get_all_indexed_images(&db).unwrap_or_default();
    eprintln!("[STARTUP] get_all_indexed_images returned {} images", all_images.len());

    let mut loaded_text = 0usize;
    let mut loaded_image = 0usize;
    for img in &all_images {
        let id = match img.id { Some(v) => v, None => continue };

        // CLIP-L/14 vectors (for text search, stored in clip_vector)
        if let Some(ref blob) = img.clip_vector {
            if blob.len() % 4 == 0 {
                let count = blob.len() / 4;
                let mut vec = Vec::with_capacity(count);
                for i in 0..count {
                    let start = i * 4;
                    let arr: [u8; 4] = [blob[start], blob[start+1], blob[start+2], blob[start+3]];
                    vec.push(f32::from_le_bytes(arr));
                }
                if !vec.is_empty() {
                    dual_index.text_index.add(id, vec);
                    loaded_text += 1;
                }
            }
        }

        // SigLIP2 vectors (for image search, stored in clip_vector_siglip2)
        if let Some(ref blob) = img.clip_vector_siglip2 {
            if blob.len() % 4 == 0 {
                let count = blob.len() / 4;
                let mut vec = Vec::with_capacity(count);
                for i in 0..count {
                    let start = i * 4;
                    let arr: [u8; 4] = [blob[start], blob[start+1], blob[start+2], blob[start+3]];
                    vec.push(f32::from_le_bytes(arr));
                }
                if !vec.is_empty() {
                    dual_index.image_index.add(id, vec);
                    loaded_image += 1;
                }
            }
        }
    }
    eprintln!("[STARTUP] Loaded {} text vectors (CLIP-L/14), {} image vectors (SigLIP2)",
        loaded_text, loaded_image);

    // Initialize CLIP model — if Python/CLIP is unavailable, use a disabled
    // fallback so the window still opens and the user sees an error message.
    let clip_model = match models::clip::ClipModel::new() {
        Ok(model) => {
            eprintln!("[STARTUP] CLIP model ready");
            model
        }
        Err(e) => {
            eprintln!("[STARTUP] CLIP unavailable ({}). Running in fallback mode.", e);
            eprintln!("[STARTUP] Semantic search and indexing will not work until Python + CLIP are installed.");
            models::clip::ClipModel::new_fallback(e)
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(Mutex::new(db))
        .manage(Mutex::new(dual_index))
        .manage(Mutex::new(clip_model))
        .invoke_handler(tauri::generate_handler![
            commands::index::index_images,
            commands::search::search_by_text,
            commands::search::search_by_image,
            commands::db::get_image_info,
            commands::index::get_index_status,
            commands::system::open_file,
            commands::system::test_clip_encode,
            commands::system::get_model_config,
            commands::system::set_model_config,
            // Path management
            commands::paths::get_paths,
            commands::paths::add_path,
            commands::paths::delete_path,
            commands::paths::toggle_path,
            commands::paths::rebuild_all_index,
        ])
        .run(tauri::generate_context!());
    eprintln!("[STARTUP] Application window closed normally");
}
