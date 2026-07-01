@echo off
echo ===============================================
echo    LocalImageSearch.exe Test Tool
echo ===============================================
echo.
echo Current dir: %~dp0
pushd "%~dp0"

echo [1/4] Checking Python...
python --version >nul 2>&1
if errorlevel 1 (
    echo [FAIL] Python NOT found!
    echo Install Python 3.10+ from python.org
    echo   Check "Add Python to PATH" during install
    pause
    popd
    exit /b 1
)
python --version
echo [OK] Python found

echo.
echo [2/4] Checking PyTorch...
python -c "import torch; print(torch.__version__)" >nul 2>&1 || (
    echo [FAIL] PyTorch NOT installed!
    echo Run setup.bat first
    pause
    popd
    exit /b 1
)
python -c "import torch; print('PyTorch', torch.__version__)"
echo [OK]

echo.
echo [3/4] Checking Transformers...
python -c "import transformers; print(transformers.__version__)" >nul 2>&1 || (
    echo [FAIL] Transformers NOT installed!
    echo Run setup.bat first
    pause
    popd
    exit /b 1
)
python -c "import transformers; print('Transformers', transformers.__version__)"
echo [OK]

echo.
echo [4/4] Checking model files...
if not exist "models\siglip2-large" (
    echo [FAIL] SigLIP2 model missing: models\siglip2-large\
    pause
    popd
    exit /b 1
)
if not exist "models\clip-large" (
    echo [FAIL] CLIP-L/14 model missing: models\clip-large\
    pause
    popd
    exit /b 1
)
echo [OK] Model files OK
echo   - models\siglip2-large\
echo   - models\clip-large\

echo.
echo ===============================================
echo Testing Python backend startup...
echo ===============================================
echo.

netstat -ano | find ":8765" | find "LISTENING" >nul 2>&1
if not errorlevel 1 (
    echo Port 8765 busy, killing existing process...
    for /f "tokens=5" %%a in ('netstat -ano ^| find ":8765" ^| find "LISTENING"') do taskkill /F /PID %%a >nul 2>&1
    ping -n 3 127.0.0.1 >nul 2>&1
)

echo Starting Python backend (max 60s)...
start /b "" python clip_server.py

set count=0
:loop
set /a count+=1
if %count% gtr 60 goto fail
netstat -ano | find ":8765" | find "LISTENING" >nul 2>&1
if not errorlevel 1 goto ok
set /a mod=%count% %% 5
if %mod% equ 0 echo   waiting... (%count%/60)
ping -n 2 127.0.0.1 >nul 2>&1
goto :loop

:fail
echo.
echo [FAIL] Backend failed to start in 60s!
echo Possible causes:
echo   1. Run setup.bat to install dependencies
echo   2. Re-extract backup file
echo   3. Close programs using GPU memory
pause
popd
exit /b 1

:ok
echo [OK] Backend started! (%count%s)

echo.
echo Stopping test backend...
for /f "tokens=5" %%a in ('netstat -ano ^| find ":8765" ^| find "LISTENING"') do taskkill /F /PID %%a >nul 2>&1
echo [OK] Stopped

popd

echo.
echo ===============================================
echo   ALL TESTS PASSED!
echo   Double-click LocalImageSearch.exe to run
echo ===============================================
pause
