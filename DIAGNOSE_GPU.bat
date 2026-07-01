@echo off
chcp 65001 >nul
echo ================================================
echo  DEEP GPU DIAGNOSTIC
echo ================================================
echo.

echo [1/6] Python path:
where python
python --version
echo.

echo [2/6] PyTorch version and CUDA support:
python -c "
import torch
print('torch version:', torch.__version__)
print('CUDA available:', torch.cuda.is_available())
if torch.cuda.is_available():
    print('CUDA version:', torch.version.cuda)
    print('GPU count:', torch.cuda.device_count())
    print('GPU name:', torch.cuda.get_device_name(0))
    print('GPU memory:', round(torch.cuda.get_device_properties(0).total_memory/1024**3, 1), 'GB')
else:
    print('*** CUDA NOT AVAILABLE - torch is CPU-only ***')
    print('FIX: run FIX_GPU.bat')
"
echo.

echo [3/6] NVIDIA driver check:
nvidia-smi 2>&1 | findstr /C:"NVIDIA-SMI" && (
    echo [OK] NVIDIA driver is installed
) || (
    echo [FAIL] nvidia-smi not found
    echo        NVIDIA driver is NOT installed!
    echo        Download from: https://www.nvidia.com/Download/index.aspx
)
echo.

echo [4/6] Checking if torch was installed from CPU or GPU index:
python -m pip show torch 2>nul | findstr /C:"Location" /C:"Version"
echo.

echo [5/6] Test: force GPU tensor creation:
python -c "
import torch
if torch.cuda.is_available():
    x = torch.randn(1000, 1000).cuda()
    print('[OK] GPU tensor creation succeeded')
    print('     GPU memory used:', round(torch.cuda.memory_allocated(0)/1024**2, 1), 'MB')
    del x
    torch.cuda.empty_cache()
else:
    print('[FAIL] Cannot create GPU tensors - CUDA not available')
"
echo.

echo [6/6] Check clip_server.py device selection:
findstr /C:"device.*cuda" /C:"device.*cpu" "clip_server.py" 2>nul | findstr /C:"cuda" /C:"cpu"
echo.

echo ================================================
echo  SUMMARY:
echo ================================================
python -c "
import torch, sys
if not torch.cuda.is_available():
    print('>>> PROBLEM: torch has NO CUDA support')
    print('>>> FIX: run FIX_GPU.bat as administrator')
    sys.exit(1)
try:
    x = torch.randn(10).cuda()
    print('>>> OK: GPU is working')
    del x
except Exception as e:
    print('>>> PROBLEM: GPU detected but cannot use:', e)
    sys.exit(1)
print('>>> All checks passed - GPU should work')
"
echo.
pause
