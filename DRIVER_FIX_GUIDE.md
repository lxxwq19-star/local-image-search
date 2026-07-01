# 显卡驱动问题诊断与修复指南

## 问题描述
即使安装了正确的 GPU 版 PyTorch，如果显卡驱动版本太旧或不兼容，CUDA 也无法工作，导致索引运行在 CPU 上。

## 诊断步骤

### 1. 运行驱动检查脚本
双击运行 `CHECK_DRIVER.bat`，查看输出：

**正常情况示例：**
```
PyTorch version: 2.1.0+cu121
CUDA available: True
CUDA version: 12.1
Device count: 1
GPU 0: NVIDIA GeForce RTX 4060

[nvidia-smi output]
+-----------------------------------------------------------------------------+
| NVIDIA-SMI 546.33       Driver Version: 546.33       CUDA Version: 12.3     |
+-------------------------------+----------------------+----------------------+
```

**异常情况示例：**
```
PyTorch version: 2.1.0+cpu
CUDA available: False
[WARNING] nvidia-smi not found in PATH
```

## 常见问题与解决方案

### 问题 1: nvidia-smi 找不到
**原因**：显卡驱动未安装或环境变量未配置

**解决方案**：
1. 访问 https://www.nvidia.com/Download/index.aspx
2. 选择你的显卡型号和操作系统
3. 下载并安装最新驱动
4. 重启电脑
5. 再次运行 `CHECK_DRIVER.bat` 验证

### 问题 2: nvidia-smi 显示但 CUDA available: False
**原因**：PyTorch 安装的 CPU 版本，或驱动版本太旧

**解决方案**：
1. 检查驱动版本（nvidia-smi 显示 "Driver Version"）
2. CUDA 12.1 需要驱动版本 >= 530.30.02
3. 如果驱动太旧，更新驱动
4. 运行 `FIX_GPU.bat` 重新安装 GPU 版 PyTorch

### 问题 3: 驱动版本太旧
**症状**：nvidia-smi 显示驱动版本 < 530

**解决方案**：
1. 访问 NVIDIA 官网下载最新驱动
2. 或使用 GeForce Experience 自动更新
3. 重启电脑
4. 验证驱动版本

### 问题 4: PyTorch 检测不到 CUDA
**可能原因**：
- 安装了 CPU 版 PyTorch
- 驱动版本不匹配
- 环境变量冲突

**解决方案**：
1. 运行 `DIAGNOSE_GPU.bat` 查看详细诊断
2. 运行 `FIX_GPU.bat` 重新安装 GPU 版 PyTorch
3. 更新显卡驱动
4. 重启电脑

## 驱动版本要求

| CUDA 版本 | 最低驱动版本 (Windows) |
|-----------|----------------------|
| CUDA 12.1 | 530.30.02 |
| CUDA 12.0 | 528.33 |
| CUDA 11.8 | 522.06 |

**推荐**：安装最新版驱动（通常支持多个 CUDA 版本）

## 验证修复

修复后，运行以下命令验证：

```bash
python -c "import torch; print('CUDA available:', torch.cuda.is_available()); print('GPU:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'None')"
```

**期望输出**：
```
CUDA available: True
GPU: NVIDIA GeForce RTX xxxx
```

## 日志文件位置

如果问题仍然存在，检查以下日志文件：
- `clip_server.log` - Python 服务器日志
- `clip_server_err.log` - Python 错误日志
- `CHECK_DRIVER.bat` 的输出 - 驱动状态

## 联系支持

如果以上步骤都无法解决问题，请提供：
1. `CHECK_DRIVER.bat` 的完整输出
2. `clip_server.log` 的内容
3. 显卡型号
4. 操作系统版本
