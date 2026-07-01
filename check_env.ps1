# Local Image Search - Environment Check (PowerShell)
# Better UTF-8 support than .bat files

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Environment Check" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$allPassed = $true

# 1. Python
Write-Host "[1/5] Checking Python..." -ForegroundColor Yellow
try {
    $pyVersion = python --version 2>&1
    Write-Host "  [OK] $pyVersion" -ForegroundColor Green
} catch {
    Write-Host "  [FAIL] Python not installed" -ForegroundColor Red
    $allPassed = $false
}
Write-Host ""

# 2. Dependencies
Write-Host "[2/5] Checking Python packages..." -ForegroundColor Yellow
python -c "import torch, transformers, numpy, PIL; print('  [OK] Core packages installed')" 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) {
    Write-Host "  [FAIL] Missing packages" -ForegroundColor Red
    Write-Host "  Run setup.bat or setup.ps1" -ForegroundColor Yellow
    $allPassed = $false
} else {
    Write-Host "  [OK] Core packages installed" -ForegroundColor Green
}
Write-Host ""

# 3. GPU
Write-Host "[3/5] Checking GPU..." -ForegroundColor Yellow
python -c "import torch; print('  [OK] GPU:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else '[WARN] No GPU (will use CPU)')"
Write-Host ""

# 4. Model files
Write-Host "[4/5] Checking model files..." -ForegroundColor Yellow
if (-not (Test-Path "models\siglip2-large")) {
    Write-Host "  [FAIL] Missing: models\siglip2-large" -ForegroundColor Red
    $allPassed = $false
} elseif (-not (Test-Path "models\clip-large")) {
    Write-Host "  [FAIL] Missing: models\clip-large" -ForegroundColor Red
    $allPassed = $false
} else {
    Write-Host "  [OK] Model files found" -ForegroundColor Green
}
Write-Host ""

# 5. Port
Write-Host "[5/5] Checking port 8765..." -ForegroundColor Yellow
$portInUse = netstat -ano | Select-String ":8765"
if ($portInUse) {
    Write-Host "  [WARN] Port 8765 in use" -ForegroundColor Yellow
    Write-Host "  Close the program or restart" -ForegroundColor Gray
} else {
    Write-Host "  [OK] Port available" -ForegroundColor Green
}
Write-Host ""

if ($allPassed) {
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "  [OK] All checks passed!" -ForegroundColor Green
    Write-Host "  Ready to run LocalImageSearch.exe" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
} else {
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "  [FAIL] Some checks failed" -ForegroundColor Red
    Write-Host "  Fix the issues above" -ForegroundColor Yellow
    Write-Host "========================================" -ForegroundColor Cyan
}

Write-Host ""
pause
