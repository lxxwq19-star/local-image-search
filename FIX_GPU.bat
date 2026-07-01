@echo off
chcp 65001 >nul
echo ================================================
echo  FIX GPU: REINSTALL TORCH WITH CUDA SUPPORT
echo ================================================
echo.

REM Check if running as admin
net session >nul 2>&1
if errorlevel 1 (
    echo [WARN] Not running as administrator
    echo        Some operations may fail
    echo.
)

echo [STEP 1/4] Uninstalling CPU-only torch...
python -m pip uninstall -y torch torchvision
echo.

echo [STEP 2/4] Installing torch with CUDA 12.1 support...
echo          This will download ~2.5 GB, please wait...
echo.
python -m pip install torch torchvision --index-url https://download.pytorch.org/whl/cu121
echo.

echo [STEP 3/4] Verifying GPU support...
python -c "import torch; print('CUDA available:', torch.cuda.is_available()); print('torch version:', torch.__version__)"
echo.

echo [STEP 4/4] Testing model loading...
cd /d %~dp0
python -c "
import torch
if not torch.cuda.is_available():
    print('[FAIL] CUDA still not available!')
    print('Please install NVIDIA driver first.')
    exit(1)
print('[OK] CUDA available:', torch.cuda.get_device_name(0))
print('[OK] VRAM:', round(torch.cuda.get_device_properties(0).total_memory/1024**3, 1), 'GB')
"
echo.

echo ================================================
echo  DONE. Now run LocalImageSearch.exe
echo  GPU should be used for indexing.
echo ================================================
pause
