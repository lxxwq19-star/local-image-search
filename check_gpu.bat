@echo off
chcp 65001 >nul
echo ================================================
echo  CHECK GPU & TORCH INSTALLATION
echo ================================================
echo.

REM Check Python
python --version 2>nul
if errorlevel 1 (
    echo [FAIL] Python not found in PATH
    pause
    exit /b 1
)

echo [1/5] Checking PyTorch installation...
python -c "import torch; print('torch version:', torch.__version__); print('CUDA available:', torch.cuda.is_available()); print('CUDA version:', torch.version.cuda if torch.cuda.is_available() else 'N/A')"
echo.

echo [2/5] Checking GPU...
python -c "import torch; print('GPU count:', torch.cuda.device_count()); print('GPU name:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'NONE')"
echo.

echo [3/5] Checking NVIDIA driver...
nvidia-smi 2>nul
if errorlevel 1 (
    echo [WARN] nvidia-smi not found - driver may not be installed
) else (
    echo [OK] NVIDIA driver detected
)
echo.

echo [4/5] Checking installed torch package...
python -m pip show torch 2>nul | findstr "Version Location"
echo.

echo [5/5] Recommendation:
python -c "import torch; print('>>> torch is CPU-only' if not torch.cuda.is_available() else '>>> torch is GPU-ready')"
echo.

echo ================================================
echo  If CUDA available = False, run FIX_GPU.bat
echo ================================================
pause
