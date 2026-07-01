# LocalImageSearch.exe 启动测试工具 (PowerShell版)
# 右键点击此文件，选择"使用 PowerShell 运行"

Write-Host "=============================================" -ForegroundColor Cyan
Write-Host "   LocalImageSearch.exe 启动测试工具" -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan
Write-Host ""

# 获取脚本所在目录
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir

# [1/4] 检查 Python 后端
Write-Host "[1/4] 检查 Python 后端..." -ForegroundColor Yellow
Write-Host "-----------------------------------------------"

try {
    $pythonVersion = python --version 2>&1
    Write-Host "[OK] Python: $pythonVersion" -ForegroundColor Green
} catch {
    Write-Host "[FAIL] Python 未安装或未添加到 PATH" -ForegroundColor Red
    Write-Host "请安装 Python 3.10+ 并勾选 'Add Python to PATH'" -ForegroundColor Yellow
    Write-Host ""
    Read-Host "按 Enter 键退出"
    exit 1
}

try {
    $torchVersion = python -c "import torch; print('PyTorch', torch.__version__)" 2>&1
    Write-Host "[OK] $torchVersion" -ForegroundColor Green
} catch {
    Write-Host "[FAIL] PyTorch 未安装!" -ForegroundColor Red
    Write-Host "请运行 setup.bat 或 setup.ps1 安装依赖" -ForegroundColor Yellow
    Write-Host ""
    Read-Host "按 Enter 键退出"
    exit 1
}

try {
    $tfVersion = python -c "import transformers; print('Transformers', transformers.__version__)" 2>&1
    Write-Host "[OK] $tfVersion" -ForegroundColor Green
} catch {
    Write-Host "[FAIL] Transformers 未安装!" -ForegroundColor Red
    Write-Host "请运行 setup.bat 或 setup.ps1 安装依赖" -ForegroundColor Yellow
    Write-Host ""
    Read-Host "按 Enter 键退出"
    exit 1
}

Write-Host ""
Write-Host "[2/4] 检查模型文件..." -ForegroundColor Yellow
Write-Host "-----------------------------------------------"

if (-not (Test-Path "models\siglip2-large")) {
    Write-Host "[FAIL] SigLIP2 模型未找到: models\siglip2-large\" -ForegroundColor Red
    Write-Host "请确保备份已完整解压" -ForegroundColor Yellow
    Write-Host ""
    Read-Host "按 Enter 键退出"
    exit 1
} else {
    Write-Host "[OK] SigLIP2 模型文件存在" -ForegroundColor Green
}

if (-not (Test-Path "models\clip-large")) {
    Write-Host "[FAIL] CLIP-L/14 模型未找到: models\clip-large\" -ForegroundColor Red
    Write-Host "请确保备份已完整解压" -ForegroundColor Yellow
    Write-Host ""
    Read-Host "按 Enter 键退出"
    exit 1
} else {
    Write-Host "[OK] CLIP-L/14 模型文件存在" -ForegroundColor Green
}

Write-Host ""
Write-Host "[3/4] 测试 Python 后端启动..." -ForegroundColor Yellow
Write-Host "-----------------------------------------------"

# 检查端口是否被占用
$portInUse = netstat -ano | Select-String ":8765" | Select-String "LISTENING"
if ($portInUse) {
    Write-Host "[WARN] 端口 8765 已被占用" -ForegroundColor Yellow
    Write-Host "正在停止现有进程..." -ForegroundColor Yellow
    
    $lines = netstat -ano | Select-String ":8765" | Select-String "LISTENING"
    foreach ($line in $lines) {
        $parts = $line.Line -split '\s+'
        $pid = $parts[-1]
        taskkill /F /PID $pid 2>&1 | Out-Null
    }
    Write-Host "[OK] 已停止占用端口的进程" -ForegroundColor Green
    Start-Sleep -Seconds 2
}

Write-Host "正在启动 Python 后端 (最多等待 60 秒)..." -ForegroundColor Yellow

# 启动 Python 后端
$pythonProcess = Start-Process -FilePath "python" -ArgumentList "clip_server.py" -WorkingDirectory $ScriptDir -PassThru -WindowStyle Hidden

# 等待后端启动
$started = $false
for ($i = 1; $i -le 60; $i++) {
    Start-Sleep -Seconds 1
    
    $tcpClient = New-Object System.Net.Sockets.TcpClient
    try {
        $tcpClient.Connect("127.0.0.1", 8765)
        if ($tcpClient.Connected) {
            Write-Host "[OK] Python 后端启动成功 (${i}秒)" -ForegroundColor Green
            $started = $true
            $tcpClient.Close()
            break
        }
    } catch {
        # 继续等待
    }
    
    if ($i % 5 -eq 0) {
        Write-Host "  仍在等待... ($i/60)" -ForegroundColor Gray
    }
}

if (-not $started) {
    Write-Host "[FAIL] Python 后端在 60 秒内未能启动" -ForegroundColor Red
    Write-Host "请检查日志文件:" -ForegroundColor Yellow
    Write-Host "  - clip_server.log" -ForegroundColor Yellow
    Write-Host "  - clip_server_err.log" -ForegroundColor Yellow
    Write-Host ""
    Read-Host "按 Enter 键退出"
    exit 1
}

Write-Host ""
Write-Host "[4/4] 停止测试后端..." -ForegroundColor Yellow
Write-Host "-----------------------------------------------"

# 停止测试后端
if (-not $pythonProcess.HasExited) {
    Stop-Process -Id $pythonProcess.Id -Force -ErrorAction SilentlyContinue
    Write-Host "[OK] 测试后端已停止" -ForegroundColor Green
}

Write-Host ""
Write-Host "=============================================" -ForegroundColor Cyan
Write-Host "  所有测试通过! exe 应该可以正常启动" -ForegroundColor Green
Write-Host "=============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "现在可以双击 LocalImageSearch.exe 运行应用了" -ForegroundColor Yellow
Write-Host ""
Read-Host "按 Enter 键退出"
