@echo off
chcp 65001 >nul
title Local Image Search - Environment Check

echo ================================
echo   Environment Check
echo ================================
echo.

:: 1. Python
echo [1/5] Checking Python...
python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo   [FAIL] Python not installed
    goto :fail
)
python --version
echo.

:: 2. Dependencies
echo [2/5] Checking Python packages...
python -c "import torch, transformers, numpy, PIL; print('  [OK] Core packages installed')" 2>nul
if %errorlevel% neq 0 (
    echo   [FAIL] Missing packages
    echo   Run setup.bat to install
    goto :fail
)
echo.

:: 3. GPU
echo [3/5] Checking GPU...
python -c "import torch; print('  [OK] GPU:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else '[WARN] No GPU (will use CPU)')"
echo.

:: 4. Model files
echo [4/5] Checking model files...
if not exist "models\siglip2-large" (
    echo   [FAIL] Missing: models\siglip2-large
    goto :fail
)
if not exist "models\clip-large" (
    echo   [FAIL] Missing: models\clip-large
    goto :fail
)
echo   [OK] Model files found
echo.

:: 5. Port
echo [5/5] Checking port 8765...
netstat -ano | findstr :8765 >nul 2>&1
if %errorlevel% equ 0 (
    echo   [WARN] Port 8765 in use
    echo   Close the program or restart
) else (
    echo   [OK] Port available
)
echo.

echo ================================
echo   [OK] All checks passed!
echo   Ready to run LocalImageSearch.exe
echo ================================
goto :end

:fail
echo.
echo ================================
echo   [FAIL] Check failed
echo   Fix the issues above
echo ================================

:end
pause
