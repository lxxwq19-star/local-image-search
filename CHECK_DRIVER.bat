@echo off
chcp 65001 >nul
echo ================================================
echo 检查显卡驱动和CUDA状态
echo ================================================
echo.

REM 检查Python环境
where python
if %errorlevel% neq 0 (
    echo [ERROR] Python not found in PATH
    echo Please run setup.bat first
    pause
    exit /b 1
)

echo [1] Checking Python and PyTorch...
python -c "import torch; print('PyTorch version:', torch.__version__); print('CUDA available:', torch.cuda.is_available()); print('CUDA version:', torch.version.cuda if torch.cuda.is_available() else 'N/A')"

echo.
echo [2] Checking GPU device...
python -c "import torch; print('Device count:', torch.cuda.device_count()); [print(f'GPU {i}:', torch.cuda.get_device_name(i)) for i in range(torch.cuda.device_count())]"

echo.
echo [3] Checking NVIDIA driver (nvidia-smi)...
where nvidia-smi
if %errorlevel% equ 0 (
    nvidia-smi
) else (
    echo [WARNING] nvidia-smi not found in PATH
    echo This usually means NVIDIA driver is not installed or not in PATH
    echo.
    echo Typical locations to check:
    echo   C:\Program Files\NVIDIA Corporation\NVSMI\nvidia-smi.exe
    echo.
    if exist "C:\Program Files\NVIDIA Corporation\NVSMI\nvidia-smi.exe" (
        echo [FOUND] nvidia-smi exists at default location
        "C:\Program Files\NVIDIA Corporation\NVSMI\nvidia-smi.exe"
    )
)

echo.
echo [4] Checking DirectML / GPU info via PowerShell...
powershell -Command "Get-CimInstance Win32_VideoController | Select-Object Name, DriverVersion, Status | Format-Table -AutoSize"

echo.
echo [5] Testing actual GPU computation...
python -c "
import torch
print('Testing GPU computation...')
if torch.cuda.is_available():
    x = torch.randn(1000, 1000).cuda()
    y = torch.mm(x, x)
    print('SUCCESS: GPU computation works!')
    print('Result shape:', y.shape)
else:
    print('FAIL: CUDA not available, cannot test GPU computation')
"

echo.
echo ================================================
echo Diagnosis complete.
echo ================================================
echo.
echo If you see 'CUDA available: False' or 'nvidia-smi' not found:
echo   1. Install/update NVIDIA driver from: https://www.nvidia.com/Download/index.aspx
echo   2. Restart computer after driver installation
echo   3. Run this script again to verify
echo.
pause
