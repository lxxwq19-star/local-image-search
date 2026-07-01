use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Command, Child, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use serde::Deserialize;

const SERVER_HOST: &str = "127.0.0.1";
const SERVER_PORT: u16 = 8765;

/// CLIP model wrapper using a persistent Python TCP server.
/// Model loads ONCE at startup → ~600MB one-time cost, not per-request.
///
/// ## Thread safety
/// A single `Mutex<TcpStream>` serialises all requests so that each
/// request's send+receive are atomic — no other thread can interleave its I/O
/// while a request is in flight. This prevents broken-pipe and response-
/// interleaving crashes when multiple searches race during indexing.
pub struct ClipModel {
    // Guards the TCP connection — a request holds this lock for the
    // entire send+receive cycle so it can never be preempted.
    io: Mutex<TcpStream>,
    _child: Option<Child>,
    request_counter: AtomicU64,
    /// Whether CLIP is available. If false, all encode calls return Err.
    clip_enabled: bool,
    /// Error message from startup (shown to user when clip_enabled is false).
    startup_error: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ServerResponse {
    #[serde(rename = "type")]
    pub resp_type: String,
    #[serde(default)]
    pub id: Option<u64>,
    #[serde(default)]
    pub vector: Option<Vec<f32>>,
    #[serde(default)]
    pub results: Option<Vec<EncodeImageResult>>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(rename = "translatedText")]
    #[serde(default)]
    pub translated_text: Option<String>,
    #[serde(default)]
    pub siglip2_vector: Option<Vec<f32>>,
    #[serde(default)]
    pub cliplarge_vector: Option<Vec<f32>>,
}

#[derive(Deserialize, Debug)]
pub struct EncodeImageResult {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub vector: Option<Vec<f32>>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub siglip2_vector: Option<Vec<f32>>,
    #[serde(default)]
    pub cliplarge_vector: Option<Vec<f32>>,
}

impl ClipModel {
    /// Start the persistent Python TCP server and connect to it.
    /// Reads MODEL_VARIANT from config file to select CLIP vs EVA02.
    pub fn new() -> Result<Self, String> {
        let _ = find_model_dir().map_err(|e| e.to_string())?;

        // Read model variant from config file
        let model_variant = read_model_config().unwrap_or_else(|| "clip".to_string());
        eprintln!("[CLIP] Model variant: {}", model_variant);

        // Find server: could be a bundled executable or a .py script
        let server_path = find_server_path().ok_or_else(|| "clip_server not found".to_string())?;

        eprintln!("[CLIP] Server path: {}", server_path);

        // Determine how to launch the server
        let (cmd_program, server_arg) = if is_python_script(&server_path) {
            // It's a .py script → need Python
            let python_cmd = find_python_command(&model_variant)?;
            eprintln!("[CLIP] Using Python: {}", python_cmd);
            (python_cmd, server_path)
        } else {
            // It's a bundled standalone executable → run directly
            eprintln!("[CLIP] Using bundled executable (no Python needed)");
            (server_path.clone(), String::new())
        };

        eprintln!("[CLIP] Starting TCP server: {} {}...", cmd_program, server_arg);

        // Determine working directory: directory containing the server (exe or .py)
        let work_dir = std::path::Path::new(&server_path).parent().map(|p| p.to_path_buf());

        // Open log files for capturing Python stdout/stderr
        let log_dir = if let Some(ref wd) = work_dir {
            wd.clone()
        } else if let Ok(exe) = std::env::current_exe() {
            exe.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| std::path::PathBuf::from("."))
        } else {
            std::path::PathBuf::from(".")
        };

        let out_log_path = log_dir.join("clip_server.log");
        let err_log_path = log_dir.join("clip_server_err.log");

        let out_log = std::fs::File::create(&out_log_path)
            .map_err(|e| format!("Failed to create stdout log {}: {}", out_log_path.display(), e))?;
        let err_log = std::fs::File::create(&err_log_path)
            .map_err(|e| format!("Failed to create stderr log {}: {}", err_log_path.display(), e))?;
        eprintln!("[CLIP] stdout -> {}", out_log_path.display());
        eprintln!("[CLIP] stderr -> {}", err_log_path.display());

        let mut cmd = Command::new(&cmd_program);
        // If using .py script, pass it as arg; if standalone exe, no arg needed
        if !server_arg.is_empty() {
            cmd.arg(&server_arg);
        }
        cmd.stdin(Stdio::null())
            .stdout(Stdio::from(out_log))
            .stderr(Stdio::from(err_log))
            .env("PYTHONIOENCODING", "utf-8")
            .env("PYTHONUTF8", "1")
            .env("MODEL_VARIANT", &model_variant);

        // Set working directory so Python can find models relative to script location
        if let Some(ref wd) = work_dir {
            cmd.current_dir(wd);
            eprintln!("[CLIP] Working directory set to: {}", wd.display());
        }

        let mut child = cmd.spawn()
            .map_err(|e| format!("Failed to start CLIP server (cmd={}): {}", cmd_program, e))?;

        // Connect to the TCP server — retry for up to 15s while Python loads models
        let addr = format!("{}:{}", SERVER_HOST, SERVER_PORT);
        let io = (|| -> Result<std::net::TcpStream, String> {
            let start = std::time::Instant::now();
            let max_wait = std::time::Duration::from_secs(45);
            let mut attempt = 0;
            loop {
                attempt += 1;
                match TcpStream::connect(&addr) {
                    Ok(s) => {
                        eprintln!("[CLIP] Connected to TCP server at {} (after {} attempts, {}ms)",
                            addr, attempt, start.elapsed().as_millis());
                        return Ok(s);
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused && start.elapsed() < max_wait => {
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        continue;
                    }
                    Err(e) => {
                        return Err(format!("Failed to connect to CLIP server at {} after {}ms: {}", addr, start.elapsed().as_millis(), e));
                    }
                }
            }
        })()?;

        let model = Self {
            io: Mutex::new(io),
            _child: Some(child),
            request_counter: AtomicU64::new(1),
            clip_enabled: true,
            startup_error: None,
        };

        Ok(model)
    }

    /// Create a disabled CLIP model (when Python/CLIP server is unavailable).
    /// All encode calls will return the error message passed here.
    pub fn new_fallback(error: String) -> Self {
        // Create a local loopback connection that always works as a stub
        let stub = std::net::TcpListener::bind("127.0.0.1:0")
            .and_then(|listener| {
                let addr = listener.local_addr()?;
                std::net::TcpStream::connect(addr)
            })
            .unwrap_or_else(|_| {
                std::net::TcpStream::connect("127.0.0.1:1")
                    .expect("Cannot create stub TCP connection")
            });

        Self {
            io: Mutex::new(stub),
            _child: None,
            request_counter: AtomicU64::new(1),
            clip_enabled: false,
            startup_error: Some(error),
        }
    }

    /// Encode a text query. Returns a 512-dim CLIP vector.
    pub fn encode_text(&self, text: &str) -> Result<Vec<f32>, String> {
        if !self.clip_enabled {
            return Err(self.startup_error.clone().unwrap_or_else(|| "CLIP unavailable".into()));
        }
        let request = serde_json::json!({
            "id": self.request_counter.fetch_add(1, Ordering::SeqCst),
            "type": "encode_text",
            "text": text,
        });
        let resp = self.do_request(&request)?;
        if let Some(err) = resp.error {
            return Err(err);
        }
        resp.vector.ok_or_else(|| "No vector in response".into())
    }

    /// Encode a single image. Returns a 512-dim CLIP vector.
    pub fn encode_image(&self, image_path: &str) -> Result<Vec<f32>, String> {
        if !self.clip_enabled {
            return Err(self.startup_error.clone().unwrap_or_else(|| "CLIP unavailable".into()));
        }
        let request = serde_json::json!({
            "id": self.request_counter.fetch_add(1, Ordering::SeqCst),
            "type": "encode_images",
            "paths": [image_path],
        });
        let resp = self.do_request(&request)?;
        let results = resp.results.ok_or_else(|| "No results in response".to_string())?;
        let first = results.into_iter().next().ok_or_else(|| "Empty results".to_string())?;
        if let Some(err) = first.error {
            return Err(err);
        }
        first.vector.ok_or_else(|| "No vector in result".into())
    }

    /// Encode a batch of images. Returns one result per path (Ok = vector, Err = error message).
    pub fn encode_images_batch(
        &self,
        paths: &[String],
    ) -> Result<Vec<Result<Vec<f32>, String>>, String> {
        if !self.clip_enabled {
            return Err(self.startup_error.clone().unwrap_or_else(|| "CLIP unavailable".into()));
        }
        if paths.is_empty() {
            return Ok(vec![]);
        }
        let request = serde_json::json!({
            "id": self.request_counter.fetch_add(1, Ordering::SeqCst),
            "type": "encode_images",
            "paths": paths,
        });
        let resp = self.do_request(&request)?;
        let results = resp.results.ok_or_else(|| "No results".to_string())?;
        Ok(results
            .into_iter()
            .map(|r| {
                if let Some(err) = r.error {
                    Err(err)
                } else {
                    r.vector.ok_or_else(|| "No vector".into())
                }
            })
            .collect())
    }

    /// Encode a text query using CLIP-L/14. Returns a 768-dim vector.
    pub fn encode_text_clip_large(&self, text: &str) -> Result<Vec<f32>, String> {
        if !self.clip_enabled {
            return Err(self.startup_error.clone().unwrap_or_else(|| "CLIP unavailable".into()));
        }
        let request = serde_json::json!({
            "id": self.request_counter.fetch_add(1, Ordering::SeqCst),
            "type": "encode_text_clip_large",
            "text": text,
        });
        let resp = self.do_request(&request)?;
        if let Some(err) = resp.error {
            return Err(err);
        }
        resp.vector.ok_or_else(|| "No vector in response".into())
    }

    /// Encode a single image with SigLIP2. Returns a vector directly from the response.
    pub fn encode_image_siglip2(&self, image_path: &str) -> Result<Vec<f32>, String> {
        if !self.clip_enabled {
            return Err(self.startup_error.clone().unwrap_or_else(|| "CLIP unavailable".into()));
        }
        let request = serde_json::json!({
            "id": self.request_counter.fetch_add(1, Ordering::SeqCst),
            "type": "encode_image_siglip2",
            "paths": [image_path],
        });
        let resp = self.do_request(&request)?;
        if let Some(err) = resp.error {
            return Err(err);
        }
        // SigLIP2 returns the vector directly in the top-level `vector` field
        resp.vector.ok_or_else(|| "No vector in response".into())
    }

    /// Batch encode images with both CLIP-L/14 and SigLIP2 models.
    /// Returns a Vec of Results, each containing a pair (siglip2_vector, cliplarge_vector).
    pub fn encode_both_batch(
        &self,
        paths: &[String],
    ) -> Result<Vec<Result<(Vec<f32>, Vec<f32>), String>>, String> {
        if !self.clip_enabled {
            return Err(self.startup_error.clone().unwrap_or_else(|| "CLIP unavailable".into()));
        }
        if paths.is_empty() {
            return Ok(vec![]);
        }
        let request = serde_json::json!({
            "id": self.request_counter.fetch_add(1, Ordering::SeqCst),
            "type": "encode_both",
            "paths": paths,
        });
        let resp = self.do_request(&request)?;
        if let Some(err) = resp.error {
            return Err(err);
        }
        let results = resp.results.ok_or_else(|| "No results".to_string())?;
        Ok(results
            .into_iter()
            .map(|r| {
                if let Some(err) = r.error {
                    return Err(err);
                }
                let siglip2 = r.siglip2_vector.ok_or_else::<String, _>(|| "No siglip2_vector in result".into())?;
                let cliplarge = r.cliplarge_vector.ok_or_else::<String, _>(|| "No cliplarge_vector in result".into())?;
                Ok((siglip2, cliplarge))
            })
            .collect())
    }

    /// Translate Chinese → English using the clip_server's built-in dictionary + MyMemory API.
    pub fn translate_text(&self, text: &str) -> Result<String, String> {
        let request = serde_json::json!({
            "id": self.request_counter.fetch_add(1, Ordering::SeqCst),
            "type": "translate",
            "text": text,
        });
        let resp = self.do_request(&request)?;
        if let Some(err) = resp.error {
            return Err(err);
        }
        resp.translated_text.ok_or_else(|| "No translated_text in response".into())
    }

    /// Shutdown the persistent server cleanly (sends TCP shutdown).
    pub fn shutdown(&self) -> Result<(), String> {
        let request = serde_json::json!({
            "id": 0,
            "type": "shutdown",
        });
        let _ = self.do_request(&request);
        Ok(())
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    /// Holds `io` lock for the full send+receive cycle — the core of our thread safety.
    fn do_request(&self, request: &serde_json::Value) -> Result<ServerResponse, String> {
        let mut stream = self.io.lock().map_err(|e| e.to_string())?;

        // SEND
        let line = serde_json::to_string(request).map_err(|e| e.to_string())?;
        let line = line + "\n";
        stream
            .write_all(line.as_bytes())
            .map_err(|e| format!("Write: {}", e))?;
        stream.flush().map_err(|e| format!("Flush: {}", e))?;

        // RECEIVE
        let resp = Self::read_response(&mut stream)?;

        Ok(resp)
    }

    /// Read one JSON object from the TCP stream. Caller must hold `io`.
    fn read_response(stream: &mut TcpStream) -> Result<ServerResponse, String> {
        let mut buf = vec![0u8; 8192];
        let mut line_buf = String::new();
        loop {
            match stream.read(&mut buf) {
                Ok(0) => return Err("CLIP server closed connection unexpectedly".into()),
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]);
                    line_buf.push_str(&chunk);
                    // Try to parse a complete JSON line
                    if let Some(idx) = line_buf.find('\n') {
                        let line = line_buf[..idx].trim().to_string();
                        line_buf = line_buf[idx + 1..].to_string();
                        if line.is_empty() { continue; }
                        if !line.starts_with('{') { continue; }
                        match serde_json::from_str::<ServerResponse>(&line) {
                            Ok(resp) => return Ok(resp),
                            Err(e) => {
                                eprintln!("[CLIP] Parse error: {} | line: {}", e, line);
                                continue;
                            }
                        }
                    }
                }
                Err(e) => return Err(format!("Read: {}", e)),
            }
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Find the models directory containing clip_text.onnx / clip_vision.onnx.
/// Priority: exe directory > current directory > parent of current directory
fn find_model_dir() -> Result<String, Box<dyn std::error::Error>> {
    // 1. Try exe directory (most reliable for installed app — double-click scenario)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let m = exe_dir.join("models");
            if m.join("clip_text.onnx").exists() {
                eprintln!("[CLIP] Found models at: {}", m.display());
                return Ok(m.to_string_lossy().to_string());
            }
        }
    }
    // 2. Fallback: current directory (dev mode: cargo run)
    let cwd = std::env::current_dir()?;
    for base in [&cwd, cwd.parent().unwrap_or(&cwd)] {
        let m = base.join("models");
        if m.join("clip_text.onnx").exists() {
            eprintln!("[CLIP] Found models at: {}", m.display());
            return Ok(m.to_string_lossy().to_string());
        }
    }
    Err("Could not find models directory with clip_text.onnx".into())
}

/// Find the clip_server executable or script.
/// Priority order:
///   1. Bundled standalone executable (from PyInstaller) next to the app binary
///   2. clip_server.py next to the app binary (for dev mode / system Python)
///   3. Same lookups in current directory and its parent
pub fn find_server_path() -> Option<String> {
    // Helper: check if a path exists and is executable
    fn is_executable(p: &std::path::Path) -> bool {
        if !p.exists() {
            return false;
        }
        // On Unix, check executable bit; on Windows, check .exe extension
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            p.metadata().map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
        }
        #[cfg(not(unix))]
        {
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            ext == "exe" || ext == "" || p.metadata().map(|m| !m.permissions().readonly()).unwrap_or(true)
        }
    }

    // Look in exe directory (most reliable for installed app)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            // 1a. Bundled standalone executable next to main binary (macOS: Contents/MacOS/)
            let bundled = exe_dir.join("clip_server");
            if is_executable(&bundled) {
                eprintln!("[CLIP] Found bundled server executable at: {}", bundled.display());
                return Some(bundled.to_string_lossy().to_string());
            }

            // 1b. In Resources dir (macOS: Contents/Resources/clip_server)
            // exe_dir = Contents/MacOS/, so parent() = Contents/
            let resources = exe_dir.parent().map(|p| p.join("Resources"));
            if let Some(ref r) = resources {
                let bundled_res = r.join("clip_server");
                if is_executable(&bundled_res) {
                    eprintln!("[CLIP] Found bundled server in Resources: {}", bundled_res.display());
                    return Some(bundled_res.to_string_lossy().to_string());
                }
            }

            // 1c. clip_server.py next to exe (dev mode)
            let py = exe_dir.join("clip_server.py");
            if py.exists() {
                eprintln!("[CLIP] Found server script at: {}", py.display());
                return Some(py.to_string_lossy().to_string());
            }
        }
    }

    // 2. Fallback: current directory (dev mode: cargo run)
    if let Ok(cwd) = std::env::current_dir() {
        let bundled = cwd.join("clip_server");
        if is_executable(&bundled) {
            eprintln!("[CLIP] Found bundled server at: {}", bundled.display());
            return Some(bundled.to_string_lossy().to_string());
        }
        let py = cwd.join("clip_server.py");
        if py.exists() {
            return Some(py.to_string_lossy().to_string());
        }
        if let Some(parent) = cwd.parent() {
            let py = parent.join("clip_server.py");
            if py.exists() {
                return Some(py.to_string_lossy().to_string());
            }
            let bundled = parent.join("clip_server");
            if is_executable(&bundled) {
                return Some(bundled.to_string_lossy().to_string());
            }
        }
    }
    None
}

/// Check if a path is a Python script (ends with .py)
fn is_python_script(path: &str) -> bool {
    path.ends_with(".py")
}

/// Read model variant from config file.
/// Config file location: <exe-dir>/config/model_config.json
/// Fallback: <current-dir>/config/model_config.json
/// Default: "clip" (CLIP-B/32 ONNX)
fn read_model_config() -> Option<String> {
    // 1. Try exe directory
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let cfg = exe_dir.join("config").join("model_config.json");
            if let Ok(s) = std::fs::read_to_string(&cfg) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                    if let Some(variant) = v.get("model_variant").and_then(|v| v.as_str()) {
                        return Some(variant.to_string());
                    }
                }
            }
        }
    }
    // 2. Fallback: current directory
    if let Ok(cwd) = std::env::current_dir() {
        let cfg = cwd.join("config").join("model_config.json");
        if let Ok(s) = std::fs::read_to_string(&cfg) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                if let Some(variant) = v.get("model_variant").and_then(|v| v.as_str()) {
                    return Some(variant.to_string());
                }
            }
        }
    }
    None
}

/// Resolve pyw/py launcher to the actual python.exe path.
/// pyw.exe launches pythonw.exe (GUI subsystem) which breaks CUDA/Winsock.
/// We need python.exe (console subsystem) with CREATE_NO_WINDOW flag instead.
fn resolve_pythonw_path(launcher: &str) -> String {
    use std::process::Command;

    // Run: pyw -c "import sys; print(sys.executable, end='')"
    if let Ok(out) = Command::new(launcher)
        .arg("-c")
        .arg("import sys; print(sys.executable, end='')")
        .output()
    {
        if out.status.success() {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !path.is_empty() {
                // If we got pythonw.exe, replace with python.exe
                let fixed = path.replace("pythonw.exe", "python.exe");
                eprintln!("[CLIP] Resolved {} -> {}", launcher, fixed);
                return fixed;
            }
        }
    }
    // Fallback: return the launcher name and hope for the best
    launcher.to_string()
}

/// Try to resolve a Python command to its windowless variant (pythonw instead of python).
/// On Windows, pythonw.exe doesn't create a console window.
/// Strategy: resolve the command to a full path, then replace python.exe → pythonw.exe.
fn resolve_pythonw(cmd: &str) -> String {
    if cfg!(not(target_os = "windows")) {
        return cmd.to_string();
    }

    // For "py" → try "pyw" (the windowless launcher)
    if cmd == "py" || cmd == "pyw" {
        let pyw_cmd = if cmd == "py" { "pyw" } else { cmd };
        if std::process::Command::new(pyw_cmd).arg("--version").output().is_ok() {
            eprintln!("[CLIP] Using windowless Python launcher: {}", pyw_cmd);
            return pyw_cmd.to_string();
        }
    }

    // For bare "python"/"python3": try "pythonw"/"pythonw3" as a command name
    // (Windows will resolve it to the same directory as python.exe)
    if cmd == "python" || cmd == "python3" {
        let w = format!("{}w", cmd);
        // Only use pythonw if it actually works (has torch+transformers)
        if std::process::Command::new(&w).arg("--version").output().is_ok() {
            eprintln!("[CLIP] Using windowless Python command: {}", w);
            return w;
        }
    }

    // Full path: try to replace python.exe → pythonw.exe
    let lower = cmd.to_lowercase();
    if lower.ends_with("python.exe") {
        let w_path = cmd.replace("python.exe", "pythonw.exe")
                       .replace("PYTHON.EXE", "pythonw.exe");
        if std::path::Path::new(&w_path).exists() {
            eprintln!("[CLIP] Using windowless Python: {}", w_path);
            return w_path;
        }
    }

    // Fallback: return original command
    cmd.to_string()
}

/// Find Python command with torch (and transformers for PyTorch models).
/// Priority:
///   1. PYTHON_PATH env var (user override)
///   2. Python in PATH that has torch (+ transformers for PyTorch models)
///   3. Fallback: "python" in PATH (let clip_server.py handle missing deps)
fn find_python_command(model_variant: &str) -> Result<String, String> {
    use std::process::Command;



    // PyTorch models need transformers, ONNX models don't
    let needs_transformers = matches!(model_variant, "clip-large" | "siglip2" | "eva02");

    // 1. PYTHON_PATH env override
    if let Ok(p) = std::env::var("PYTHON_PATH") {
        if !p.is_empty() {
            eprintln!("[CLIP] Using PYTHON_PATH override: {}", p);
            return Ok(resolve_pythonw(&p));
        }
    }

    // 2. Try `python` and `python3` in PATH, check for torch + transformers
    for cmd in ["python", "python3"] {
        if Command::new(cmd).arg("--version").output().is_ok() {
            // Check if torch is installed
            let torch_check = Command::new(cmd)
                .arg("-c")
                .arg("import torch; print(torch.__version__)")
                .output();
            if let Ok(out) = torch_check {
                if out.status.success() {
                    let version = String::from_utf8_lossy(&out.stdout).trim().to_string();

                    // For PyTorch models, also check transformers
                    if needs_transformers {
                        let tf_check = Command::new(cmd)
                            .arg("-c")
                            .arg("import transformers; print(transformers.__version__)")
                            .output();
                        if let Ok(tf_out) = tf_check {
                            if tf_out.status.success() {
                                let tf_ver = String::from_utf8_lossy(&tf_out.stdout).trim().to_string();
                                eprintln!("[CLIP] Found Python with torch={} transformers={}: {}", version, tf_ver, cmd);
                                return Ok(resolve_pythonw(cmd));
                            }
                        }
                        // torch found but transformers missing — keep looking
                        eprintln!("[CLIP] Python '{}' has torch but no transformers, trying next...", cmd);
                        continue;
                    }

                    eprintln!("[CLIP] Found Python with torch: {} (torch {})", cmd, version);
                    return Ok(resolve_pythonw(cmd));
                }
            }
        }
    }

    // 3. Try py launcher (Windows) then pyw
    for launcher in ["pyw", "py"] {
        if Command::new(launcher).arg("--version").output().is_ok() {
            if let Ok(out) = Command::new(launcher)
                .arg("-c")
                .arg("import torch; print(torch.__version__)")
                .output()
            {
                if out.status.success() {
                    if needs_transformers {
                        if let Ok(tf_out) = Command::new(launcher)
                            .arg("-c")
                            .arg("import transformers; print(transformers.__version__)")
                            .output()
                        {
                            if tf_out.status.success() {
                                // Resolve pyw/py -> actual python.exe path
                                let py_path = resolve_pythonw_path(launcher);
                                eprintln!("[CLIP] Found Python ({}) with torch+transformers -> {}", launcher, py_path);
                                return Ok(py_path);
                            }
                        }
                    } else {
                        eprintln!("[CLIP] Found Python ({}) with torch", launcher);
                        return Ok(launcher.to_string());
                    }
                }
            }
        }
    }

    // 4. Fallback: use "python" and let server print error
    eprintln!("[CLIP] WARNING: Could not find Python with torch. Falling back to 'python'.");
    Ok(resolve_pythonw("python"))
}
