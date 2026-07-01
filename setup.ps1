# Local Image Search - Setup Script (PowerShell)
# Better UTF-8 support than .bat files

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Local Image Search - Python Setup" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check Python
Write-Host "[1/5] Checking Python..." -ForegroundColor Yellow
try {
    $pyVersion = python --version 2>&1
    Write-Host "  [OK] $pyVersion" -ForegroundColor Green
} catch {
    Write-Host "  [ERROR] Python not found" -ForegroundColor Red
    Write-Host "  Download: https://www.python.org/downloads/" -ForegroundColor Yellow
    Write-Host "  Check 'Add Python to PATH' during install" -ForegroundColor Yellow
    pause
    exit 1
}
Write-Host ""

# Set mirrors
Write-Host "[2/5] Setting up mirrors..." -ForegroundColor Yellow
pip config set global.index-url https://pypi.tuna.tsinghua.edu.cn/simple 2>&1 | Out-Null
pip config set global.extra-index-url https://mirrors.aliyun.com/pypi/simple 2>&1 | Out-Null
$env:HF_ENDPOINT = "https://hf-mirror.com"
Write-Host "  [OK] Mirrors configured" -ForegroundColor Green
Write-Host ""

# Upgrade pip
Write-Host "[3/5] Upgrading pip..." -ForegroundColor Yellow
python -m pip install --upgrade pip -q 2>&1 | Out-Null
Write-Host "  [OK] pip upgraded" -ForegroundColor Green
Write-Host ""

# Install dependencies
Write-Host "[4/5] Installing dependencies..." -ForegroundColor Yellow
Write-Host "  This may take 5-10 minutes..." -ForegroundColor Gray
python -m pip install -r requirements.txt
if ($LASTEXITCODE -ne 0) {
    Write-Host "  [ERROR] Installation failed" -ForegroundColor Red
    Write-Host "  Try manual install: pip install -r requirements.txt" -ForegroundColor Yellow
    pause
    exit 1
}
Write-Host "  [OK] All packages installed" -ForegroundColor Green
Write-Host ""

# Check GPU
Write-Host "[5/5] Checking GPU..." -ForegroundColor Yellow
python -c "import torch; print('  [OK] GPU:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else '[WARN] No NVIDIA GPU (will use CPU)')"
Write-Host ""

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  [OK] Setup Complete!" -ForegroundColor Green
Write-Host "  Double-click LocalImageSearch.exe to run" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
pause
