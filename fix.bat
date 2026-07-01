@echo off
chcp 65001 >nul
title Local Image Search - Quick Fix

echo ================================
echo   Quick Fix Tool
echo ================================
echo.

:: Kill existing processes
echo [1/3] Killing existing processes...
taskkill /F /IM python.exe /T 2>nul
taskkill /F /IM LocalImageSearch.exe /T 2>nul
echo   [OK] Done
echo.

:: Free port
echo [2/3] Freeing port 8765...
for /f "tokens=5" %%a in ('netstat -ano ^| findstr :8765') do (
    taskkill /F /PID %%a 2>nul
)
echo   [OK] Done
echo.

:: Install missing deps
echo [3/3] Installing missing dependencies...
python -m pip install onnxruntime safetensors
echo   [OK] Done
echo.

echo ================================
echo   [OK] Quick fix complete!
echo   Try running LocalImageSearch.exe again
echo ================================
echo.
pause
