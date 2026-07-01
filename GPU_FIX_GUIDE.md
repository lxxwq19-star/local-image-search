# GPU 未使用问题排查与修复

## 症状
- 索引进度条在涨，但速度很慢
- 任务管理器里 GPU 占用为 0%
- CPU 占用很高（100% 或接近）

## 根因
`setup.bat` 默认安装的 `torch` 是 **CPU 版本**，不含 CUDA 支持。

验证方法：在新电脑上双击运行 `check_gpu.bat`，如果显示 `CUDA available: False` 就是这个问题。

---

## 修复步骤

### 方法 1：运行 `FIX_GPU.bat`（推荐，一键修复）

1. 双击 `FIX_GPU.bat`
2. 等待下载（约 2.5 GB，需要 5-15 分钟）
3. 完成后重新启动 `LocalImageSearch.exe`

### 方法 2：手动安装 GPU 版 torch

```bat
python -m pip uninstall -y torch torchvision
python -m pip install torch torchvision --index-url https://download.pytorch.org/whl/cu121
```

---

## 如果 `FIX_GPU.bat` 也失败

### 可能原因 1：没有 NVIDIA 显卡
- 检查设备管理器 → 显示适配器
- 如果没有 NVIDIA 显卡，只能用 CPU 模式（速度慢 5-10 倍）

### 可能原因 2：有 NVIDIA 显卡但没有装驱动
- 去 https://www.nvidia.com/Download/index.aspx 下载驱动
- 安装后重启，再运行 `FIX_GPU.bat`

### 可能原因 3：显存不足（低于 4GB）
- 应用会自动降级到 CPU 模式
- 需要更换显卡

---

## 验证修复成功

运行 `check_gpu.bat`，应输出：
```
CUDA available: True
GPU name: NVIDIA GeForce ...
```

然后启动应用，索引进度应该明显变快，GPU 占用应上升到 30-90%。

---

## 国内网络加速（下载 torch 很慢的话）

```bat
python -m pip install torch torchvision --index-url https://download.pytorch.org/whl/cu121 --proxy=http://代理地址:端口
```

或者先下载离线包：
- torch cu121: https://download.pytorch.org/whl/cu121/torch/
- torchvision cu121: https://download.pytorch.org/whl/cu121/torchvision/
