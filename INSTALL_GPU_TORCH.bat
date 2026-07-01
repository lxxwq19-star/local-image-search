@echo off
chcp 65001 >nul
echo ================================================
echo Uninstall CPU torch, install GPU torch
echo ================================================
echo.

echo [1] Uninstalling existing torch (CPU or GPU)...
python -m pip uninstall -y torch torchvision
if %errorlevel% neq 0 (
    echo [WARN] Uninstall had warnings (maybe not installed yet)
)

echo.
echo [2] Installing GPU version of PyTorch (CUDA 12.1)...
echo     Download size: ~2.5 GB
echo     This may take 5-10 minutes...
echo.

python -m pip install torch torchvision --index-url https://download.pytorch.org/whl/cu121

if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Installation failed!
    echo Possible causes:
    echo   1. Network issue - check internet connection
    echo   2. NVIDIA driver not installed - run CHECK_DRIVER.bat
    echo.
    pause
    exit /b 1
)

echo.
echo [3] Verifying installation...
python -c "import torch; print('torch version:', torch.__version__); print('CUDA available:', torch.cuda.is_available()); print('GPU:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'NOT DETECTED'); exit(0 if torch.cuda.is_available() else 1)"

if %errorlevel% equ 0 (
    echo.
    echo ================================================
    echo SUCCESS: GPU torch installed!
    echo You can now run LocalImageSearch.exe
    echo ================================================
) else (
    echo.
    echo ================================================
    echo ERROR: CUDA still not detected!
    echo Possible causes:
    echo   1. NVIDIA driver not installed or too old
    echo   2. GPU is not NVIDIA
    echo Run CHECK_DRIVER.bat for diagnosis
    echo ================================================
)

echo.
pause
