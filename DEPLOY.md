# Local Image Search - Deployment Guide

## Quick Start

### 1. Extract Backup
Extract `local-image-search2_stable_20260626.tar` to target directory
Example: `D:\local-image-search2\`

### 2. Install Python
- Download Python 3.10+: https://www.python.org/downloads/
- **IMPORTANT**: Check "Add Python to PATH" during install

### 3. Install Dependencies
Double-click `setup.bat` (English version, no encoding issues)

Or manually:
```bat
cd /d D:\local-image-search2
python -m pip install -r requirements.txt
```

### 4. Check Environment
Double-click `check_env.bat` to verify setup

### 5. Run
Double-click `LocalImageSearch.exe`

---

## Required Python Packages

```
torch>=2.0.0
torchvision>=0.15.0
transformers>=4.30.0
accelerate>=0.20.0
onnxruntime>=1.16.0
tokenizers>=0.13.0
Pillow>=9.0.0
numpy>=1.24.0
huggingface-hub>=0.20.0
safetensors>=0.3.0
```

---

## Troubleshooting

### Python backend won't start

Manual test:
```bat
cd /d D:\local-image-search2
python clip_server.py
```

Check logs:
- `clip_server.log`
- `clip_server_err.log`

### Missing modules error

Run:
```bat
python -m pip install -r requirements.txt
```

### Port 8765 in use

Find and kill process:
```bat
netstat -ano | findstr :8765
taskkill /PID <PID> /F
```

### GPU not detected

Check NVIDIA driver:
```bat
nvidia-smi
```

If not found, install driver from:
https://www.nvidia.com/Download/index.aspx

---

## File List

```
local-image-search2/
├── LocalImageSearch.exe    (main program)
├── clip_server.py          (Python backend)
├── models/                 (AI models - required)
│   ├── siglip2-large/
│   ├── clip-large/
│   ├── clip_text.onnx
│   └── clip_vision.onnx
├── config/                 (config files)
├── requirements.txt        (dependency list)
├── setup.bat               (install script - English)
├── check_env.bat           (environment check - English)
└── DEPLOY.md               (this file)
```

---

## System Requirements

- Windows 10/11 64-bit
- Python 3.10+
- 16GB+ RAM (32GB recommended)
- NVIDIA GPU 8GB+ VRAM (optional, CPU mode works)
- 10GB+ free disk space
