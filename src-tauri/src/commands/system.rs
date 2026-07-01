use serde::Deserialize;
use std::sync::Mutex;
use tauri::State;

#[derive(Deserialize)]
pub struct OpenFileRequest {
    pub path: String,
}

/// 用系统默认程序打开文件
#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    open::that(&path).map_err(|e| format!("打开文件失败: {}", e))
}

/// Diagnostic: test CLIP text encoding
#[tauri::command]
pub fn test_clip_text(
    text: String,
    clip_model: State<'_, Mutex<crate::models::clip::ClipModel>>,
) -> Result<String, String> {
    let clip = clip_model.lock().map_err(|e| e.to_string())?;
    let vector = clip.encode_text(&text)?;
    Ok(format!("text={}, dim={}, sample={:.4}", text, vector.len(), vector[0]))
}

/// Diagnostic: test CLIP image encoding with a given path
#[tauri::command]
pub fn test_clip_encode(
    path: String,
    clip_model: State<'_, Mutex<crate::models::clip::ClipModel>>,
) -> Result<String, String> {
    let clip = clip_model.lock().map_err(|e| e.to_string())?;
    let vector = clip.encode_image(&path)?;
    Ok(format!("vector dim={}, sample={:.4}", vector.len(), vector[0]))
}

/// Read model config from config/model_config.json, with availability detection
#[tauri::command]
pub fn get_model_config() -> Result<serde_json::Value, String> {
    // 1. Try exe directory
    let mut base_dir = None;
    if let Ok(exe) = std::env::current_exe() {
        base_dir = exe.parent().map(|p| p.to_path_buf());
    }
    if base_dir.is_none() {
        base_dir = std::env::current_dir().ok();
    }

    // 2. Read config file (if exists)
    let mut config = serde_json::json!({"model_variant": "clip"});
    if let Some(ref dir) = base_dir {
        let cfg = dir.join("config").join("model_config.json");
        if let Ok(s) = std::fs::read_to_string(&cfg) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                config = v;
            }
        }
    }

    // 3. Detect model availability
    if let Some(ref dir) = base_dir {
        let models = dir.join("models");
        let siglip2_exists = models.join("siglip2-large").join("config.json").exists();
        let cliplarge_exists = models.join("clip-large").join("config.json").exists();
        config["siglip2_available"] = serde_json::Value::Bool(siglip2_exists);
        config["cliplarge_available"] = serde_json::Value::Bool(cliplarge_exists);
    } else {
        config["siglip2_available"] = serde_json::Value::Bool(false);
        config["cliplarge_available"] = serde_json::Value::Bool(false);
    }

    Ok(config)
}

/// Write model config to config/model_config.json
#[tauri::command]
pub fn set_model_config(config: serde_json::Value) -> Result<String, String> {
    // 1. Find config directory (prefer exe directory)
    let config_dir = if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            Some(exe_dir.join("config"))
        } else {
            None
        }
    } else {
        std::env::current_dir().ok().map(|d| d.join("config"))
    };

    let config_dir = config_dir.ok_or_else(|| "Cannot determine config directory".to_string())?;
    let config_path = config_dir.join("model_config.json");

    // Ensure config directory exists
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let json_str = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write(&config_path, json_str)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(format!("Model config updated. Restart app to apply changes."))
}
