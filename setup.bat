@echo off
chcp 65001 >nul
setlocal EnableDelayedExpansion

:: ============================================================
:: Local Image Search 2 - Python Environment Setup Script
:: Version: 2.0 (2026-06-30)
:: ============================================================

title Local Image Search 2 - Python Setup
echo ==========================================
echo   Local Image Search 2 - Python Setup
echo   Version: 2.0 (2026-06-30)
echo ==========================================
echo.

:: ----------------------------------------------------------
:: Step 0: Pre-flight checks
:: ----------------------------------------------------------
echo [STEP 0] Pre-flight checks...
echo.

:: Check Python installation
python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Python not found in PATH
    echo.
    echo Please install Python 3.8 - 3.11 from:
    echo   https://www.python.org/downloads/
    echo.
    echo IMPORTANT: Check "Add Python to PATH" during installation!
    echo.
    pause
    exit /b 1
)

for /f "tokens=2" %%i in ('python --version 2^>^&1') do set PYTHON_VERSION=%%i
echo [OK] Python %PYTHON_VERSION% detected

:: Check Python version (warn if 3.12+)
echo %PYTHON_VERSION% | findstr /r "^3\.[12][0-9]" >nul
if %errorlevel% equ 0 (
    echo [WARN] Python %PYTHON_VERSION% detected - some packages may not support 3.12+
    echo        Recommended: Python 3.10 or 3.11
    echo.
    pause
)

:: Check if running on Windows
ver | findstr /i "Windows" >nul
if %errorlevel% neq 0 (
    echo [WARN] This script is designed for Windows
)

:: Check NVIDIA GPU (optional, for diagnostic only)
echo.
echo [INFO] Checking GPU...
nvidia-smi >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] NVIDIA GPU detected (driver installed)
    nvidia-smi | findstr /i "CUDA Version"
) else (
    echo [INFO] NVIDIA GPU not detected or driver not installed
    echo        Will install CPU version of PyTorch
    echo        For GPU acceleration, install NVIDIA driver from:
    echo        https://www.nvidia.com/Download/index.aspx
)
echo.

:: ----------------------------------------------------------
:: Step 1: Configure pip mirrors (for China users)
:: ----------------------------------------------------------
echo [STEP 1] Configuring pip mirrors...
echo.

pip config set global.index-url https://pypi.tuna.tsinghua.edu.cn/simple
pip config set global.trusted-host pypi.tuna.tsinghua.edu.cn
pip config set global.extra-index-url https://mirrors.aliyun.com/pypi/simple

:: Set HuggingFace mirror (for China users)
set HF_ENDPOINT=https://hf-mirror.com
echo [OK] Pip mirrors configured (Tsinghua + Aliyun)
echo [OK] HuggingFace mirror set: %HF_ENDPOINT%
echo.

:: ----------------------------------------------------------
:: Step 2: Uninstall existing torch (prevent conflicts)
:: ----------------------------------------------------------
echo [STEP 2] Removing existing PyTorch installation...
echo.

python -m pip uninstall -y torch torchvision 2>nul
if %errorlevel% equ 0 (
    echo [OK] Existing PyTorch removed
) else (
    echo [INFO] No existing PyTorch installation found (fresh install)
)
echo.

:: ----------------------------------------------------------
:: Step 3: Install PyTorch (try GPU first, fallback to CPU)
:: ----------------------------------------------------------
echo [STEP 3] Installing PyTorch...
echo.

:: Try GPU version with CUDA 12.1
echo [3.1] Trying GPU version (CUDA 12.1)...
echo       Download size: ~2.5 GB
echo       This may take 5-10 minutes...
echo.

python -m pip install torch torchvision --index-url https://download.pytorch.org/whl/cu121

if %errorlevel% equ 0 (
    echo.
    echo [OK] PyTorch (GPU/CUDA 12.1) installed successfully!
    set TORCH_MODE=GPU
    goto :verify_torch
)

:: If failed, try CUDA 11.8
echo.
echo [WARN] CUDA 12.1 installation failed, trying CUDA 11.8...
echo.

python -m pip install torch torchvision --index-url https://download.pytorch.org/whl/cu118

if %errorlevel% equ 0 (
    echo.
    echo [OK] PyTorch (GPU/CUDA 11.8) installed successfully!
    set TORCH_MODE=GPU
    goto :verify_torch
)

:: If both failed, install CPU version
echo.
echo [WARN] GPU installation failed, installing CPU version...
echo         Note: Search will be slower without GPU acceleration
echo.

python -m pip install torch torchvision --index-url https://download.pytorch.org/whl/cpu

if %errorlevel% equ 0 (
    echo.
    echo [OK] PyTorch (CPU) installed successfully!
    set TORCH_MODE=CPU
    goto :verify_torch
) else (
    echo.
    echo [ERROR] PyTorch installation failed completely!
    echo         Please check your internet connection and try again
    echo         Or manually install with: pip install torch torchvision
    pause
    exit /b 1
)

:verify_torch
:: Verify torch installation
echo.
echo [STEP 3.5] Verifying PyTorch installation...
echo.

python -c "import torch; print('  PyTorch version:', torch.__version__); print('  CUDA available:', torch.cuda.is_available()); exit(0 if torch.cuda.is_available() else 0)" 2>&1

if %errorlevel% equ 0 (
    echo [OK] PyTorch verification passed!
    if "%TORCH_MODE%"=="GPU" (
        python -c "import torch; print('  GPU detected:', torch.cuda.get_device_name(0))"
    ) else (
        echo [INFO] Running in CPU mode (no GPU acceleration)
    )
) else (
    echo [WARN] PyTorch installed but CUDA not available (CPU mode)
)
echo.

:: ----------------------------------------------------------
:: Step 4: Install other dependencies
:: ----------------------------------------------------------
echo [STEP 4] Installing other dependencies...
echo.

echo [4.1] Installing transformers (pinned ^<5.0.0)...
python -m pip install "transformers>=4.30.0,<5.0.0" --no-cache-dir

if %errorlevel% neq 0 (
    echo [ERROR] transformers installation failed!
    pause
    exit /b 1
)

echo.
echo [4.2] Installing image processing libraries...
python -m pip install Pillow numpy --no-cache-dir

if %errorlevel% neq 0 (
    echo [ERROR] Pillow/numpy installation failed!
    pause
    exit /b 1
)

echo.
echo [4.3] Installing optional dependencies...
python -m pip install onnxruntime tokenizers huggingface-hub safetensors --no-cache-dir

if %errorlevel% neq 0 (
    echo [WARN] Some optional dependencies failed (non-critical)
)
echo.

:: ----------------------------------------------------------
:: Step 5: Final verification
:: ----------------------------------------------------------
echo [STEP 5] Final verification...
echo.

python -c "
import sys
print('Python version:', sys.version)
print()

try:
    import torch
    print('[OK] torch', torch.__version__)
    print('     CUDA available:', torch.cuda.is_available())
    if torch.cuda.is_available():
        print('     GPU:', torch.cuda.get_device_name(0))
        print('     GPU memory:', torch.cuda.get_device_properties(0).total_memory / 1024**3, 'GB')
except Exception as e:
    print('[FAIL] torch not installed')
    sys.exit(1)

print()
try:
    import transformers
    print('[OK] transformers', transformers.__version__)
except Exception as e:
    print('[FAIL] transformers not installed')
    sys.exit(1)

print()
try:
    import PIL
    print('[OK] Pillow', PIL.__version__)
except Exception as e:
    print('[FAIL] Pillow not installed')
    sys.exit(1)

print()
try:
    import numpy
    print('[OK] numpy', numpy.__version__)
except Exception as e:
    print('[FAIL] numpy not installed')
    sys.exit(1)

print()
print('All core dependencies installed successfully!')
"

if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Verification failed!
    echo         Some dependencies are missing or broken
    echo         Try running this script again
    pause
    exit /b 1
)

echo.

:: ----------------------------------------------------------
:: Step 6: Create launcher scripts (optional)
:: ----------------------------------------------------------
echo [STEP 6] Creating helper scripts...
echo.

:: Create a simple test script
echo import torch > test_gpu.py
echo print("CUDA available:", torch.cuda.is_available()) >> test_gpu.py
echo if torch.cuda.is_available(): >> test_gpu.py
echo     print("GPU:", torch.cuda.get_device_name(0)) >> test_gpu.py
echo     x = torch.randn(1000, 1000).cuda() >> test_gpu.py
echo     print("GPU test passed!") >> test_gpu.py
echo else: >> test_gpu.py
echo     print("Running in CPU mode") >> test_gpu.py

echo [OK] Created test_gpu.py (run: python test_gpu.py)
echo.

:: ----------------------------------------------------------
:: Done!
:: ----------------------------------------------------------
echo ==========================================
echo   [SUCCESS] Setup complete!
echo ==========================================
echo.
if "%TORCH_MODE%"=="GPU" (
    echo   GPU acceleration: ENABLED
    echo   PyTorch version: with CUDA support
    echo.
) else (
    echo   GPU acceleration: DISABLED (CPU mode)
    echo   Note: Search will be slower without GPU
    echo   To enable GPU: Install NVIDIA driver and run setup.bat again
    echo.
)
echo   Next steps:
echo   1. Double-click LocalImageSearch.exe to start
echo   2. If app fails to start, check clip_server.log
echo.
echo   Helper scripts created:
echo   - test_gpu.py     : Test GPU support
echo   - CHECK_DRIVER.bat: Diagnose driver issues
echo   - FIX_GPU.bat     : Fix GPU installation
echo.
pause
endlocal
