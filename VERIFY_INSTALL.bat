@echo off
chcp 65001 >nul
echo ================================================
echo Local Image Search 2 - Installation Verifier
echo ================================================
echo.

set ERROR_COUNT=0

echo [1] Checking Python...
python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [FAIL] Python not found
    set /a ERROR_COUNT+=1
) else (
    for /f "tokens=2" %%i in ('python --version 2^>^&1') do echo [OK] Python %%i
)
echo.

echo [2] Checking PyTorch...
python -c "import torch; print('[OK] PyTorch', torch.__version__); print('     CUDA available:', torch.cuda.is_available())" 2>nul
if %errorlevel% neq 0 (
    echo [FAIL] PyTorch not installed or broken
    set /a ERROR_COUNT+=1
)
echo.

echo [3] Checking transformers...
python -c "import transformers; print('[OK] transformers', transformers.__version__)" 2>nul
if %errorlevel% neq 0 (
    echo [FAIL] transformers not installed or broken
    set /a ERROR_COUNT+=1
)
echo.

echo [4] Checking other dependencies...
python -c "import PIL; import numpy; import onnxruntime; import tokenizers; import huggingface_hub; import safetensors; print('[OK] All optional dependencies installed')" 2>nul
if %errorlevel% neq 0 (
    echo [WARN] Some optional dependencies missing (non-critical)
)
echo.

echo [5] GPU Status...
python -c "import torch; print('CUDA available:', torch.cuda.is_available()); print('GPU:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'CPU mode')" 2>nul
echo.

echo [6] NVIDIA Driver...
nvidia-smi >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] NVIDIA driver installed
    nvidia-smi | findstr /i "Driver Version"
) else (
    echo [INFO] NVIDIA driver not detected (CPU mode only)
)
echo.

echo ================================================
if %ERROR_COUNT% equ 0 (
    echo [SUCCESS] All checks passed!
    echo You can now run LocalImageSearch.exe
) else (
    echo [ERROR] %ERROR_COUNT% check(s) failed!
    echo Please run setup.bat to fix the issues
)
echo ================================================
echo.
pause
