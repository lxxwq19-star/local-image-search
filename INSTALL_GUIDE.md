# Local Image Search 2 - 完整安装指南

## 系统要求

### 最低配置
- **操作系统**: Windows 10/11 64位
- **Python**: 3.8 - 3.11（3.12 可能不兼容某些包）
- **内存**: 8 GB RAM
- **存储**: 5 GB 可用空间（用于模型下载）

### 推荐配置（GPU 加速）
- **显卡**: NVIDIA GPU（RTX 2060 或更高）
- **显存**: 6 GB 以上
- **驱动**: NVIDIA Driver 530+（支持 CUDA 12.1）
- **内存**: 16 GB RAM

---

## 快速安装（推荐）

### 1. 安装 Python
下载并安装 Python 3.10（推荐）：
- 官网：https://www.python.org/downloads/
- **重要**：安装时勾选 "Add Python to PATH"

验证安装：
```bash
python --version
# 应该显示 Python 3.8 - 3.11
```

### 2. 安装 NVIDIA 驱动（GPU 用户）
如果使用 NVIDIA 显卡，必须安装驱动：
- 官网：https://www.nvidia.com/Download/index.aspx
- 或使用 GeForce Experience 自动更新

验证驱动：
```bash
nvidia-smi
# 应该显示驱动版本和 GPU 信息
```

### 3. 运行安装脚本
双击运行 `setup.bat`，脚本会自动：
- 检测 Python 环境
- 安装 GPU 版 PyTorch（如果可用）
- 安装所有依赖包
- 验证安装是否成功

### 4. 验证安装
运行以下命令验证：
```bash
python -c "import torch; print('CUDA available:', torch.cuda.is_available()); print('GPU:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'CPU mode')"
```

**期望输出（GPU）**：
```
CUDA available: True
GPU: NVIDIA GeForce RTX xxxx
```

**期望输出（CPU）**：
```
CUDA available: False
GPU: CPU mode
```

### 5. 启动应用
双击 `LocalImageSearch.exe` 启动应用。

---

## 手动安装（高级用户）

如果脚本安装失败，可以手动安装：

### Step 1: 安装 PyTorch

**GPU 版本（推荐）**：
```bash
# CUDA 12.1（最新）
python -m pip install torch torchvision --index-url https://download.pytorch.org/whl/cu121

# CUDA 11.8（旧显卡）
python -m pip install torch torchvision --index-url https://download.pytorch.org/whl/cu118
```

**CPU 版本（无 GPU）**：
```bash
python -m pip install torch torchvision --index-url https://download.pytorch.org/whl/cpu
```

### Step 2: 安装其他依赖
```bash
pip config set global.index-url https://pypi.tuna.tsinghua.edu.cn/simple
set HF_ENDPOINT=https://hf-mirror.com
python -m pip install transformers\<5.0.0 Pillow numpy onnxruntime tokenizers huggingface-hub safetensors
```

### Step 3: 验证
```bash
python -c "import torch; import transformers; print('torch:', torch.__version__); print('transformers:', transformers.__version__); print('CUDA:', torch.cuda.is_available())"
```

---

## 依赖包说明

### 核心依赖（必需）
| 包名 | 版本 | 用途 | 安装源 |
|------|------|------|--------|
| `torch` | >=2.0.0 | PyTorch 深度学习框架 | PyTorch CUDA 索引 |
| `torchvision` | >=0.15.0 | 图像处理工具 | PyTorch CUDA 索引 |
| `transformers` | >=4.30.0,<5.0.0 | HuggingFace 模型加载 | PyPI |
| `Pillow` | >=9.0.0 | 图像读写 | PyPI |
| `numpy` | >=1.24.0 | 向量运算 | PyPI |

### 可选依赖
| 包名 | 版本 | 用途 | 备注 |
|------|------|------|------|
| `onnxruntime` | >=1.16.0 | ONNX 模型推理 | CPU 模式备用 |
| `tokenizers` | >=0.13.0 | 文本分词 | CLIP 文本编码 |
| `huggingface-hub` | >=0.20.0 | 模型下载 | 从 HuggingFace 下载 |
| `safetensors` | >=0.3.0 | 安全模型格式 | 新版 transformers 需要 |

### 关于 PyTorch CUDA 索引
**重要**：PyTorch 在 PyPI 默认提供 CPU 版本。要安装 GPU 版本，必须使用 PyTorch 官方 CUDA 索引：

```bash
# 正确方式（GPU 版）
pip install torch --index-url https://download.pytorch.org/whl/cu121

# 错误方式（CPU 版）
pip install torch  # 从 PyPI 安装，只有 CPU 版本
```

---

## 常见问题

### Q1: 安装后 CUDA 不可用（`CUDA available: False`）
**原因**：
1. 安装了 CPU 版 PyTorch
2. NVIDIA 驱动未安装或太旧
3. 显卡不支持 CUDA

**解决方案**：
1. 运行 `FIX_GPU.bat` 重新安装 GPU 版
2. 更新 NVIDIA 驱动
3. 运行 `CHECK_DRIVER.bat` 诊断

### Q2: `nvidia-smi` 命令找不到
**原因**：NVIDIA 驱动未安装或环境变量未配置

**解决方案**：
1. 安装 NVIDIA 驱动（从官网或 GeForce Experience）
2. 重启电脑
3. 如果仍不行，手动添加 `C:\Program Files\NVIDIA Corporation\NVSMI` 到 PATH

### Q3: 安装速度太慢
**原因**：默认 PyPI 源在国内访问慢

**解决方案**：
```bash
pip config set global.index-url https://pypi.tuna.tsinghua.edu.cn/simple
pip config set global.extra-index-url https://mirrors.aliyun.com/pypi/simple
```

### Q4: `transformers` 版本冲突
**原因**：安装了 transformers 5.x（API 不兼容）

**解决方案**：
```bash
python -m pip uninstall -y transformers
python -m pip install transformers\<5.0.0
```

### Q5: 内存不足
**原因**：模型太大，内存不够

**解决方案**：
1. 使用 CPU 模式（自动回退）
2. 关闭其他占用内存的应用
3. 升级内存到 16 GB 以上

### Q6: 应用启动后黑屏或闪退
**原因**：
1. Python 服务器未启动
2. 模型加载失败
3. 依赖包版本不兼容

**解决方案**：
1. 查看 `clip_server.log` 和 `clip_server_err.log`
2. 运行 `setup.bat` 重新安装依赖
3. 确保 Python 在 PATH 中

---

## 验证清单

安装完成后，运行以下命令验证：

```bash
# 1. 检查 Python
python --version

# 2. 检查 PyTorch 和 CUDA
python -c "import torch; print('torch:', torch.__version__); print('CUDA:', torch.cuda.is_available())"

# 3. 检查 transformers
python -c "import transformers; print('transformers:', transformers.__version__)"

# 4. 检查其他依赖
python -c "import PIL; import numpy; import onnxruntime; print('All dependencies OK')"

# 5. 完整诊断
CHECK_DRIVER.bat
```

所有检查都应该通过，否则参考对应的解决方案。

---

## 卸载与清理

如果需要完全重新安装：

```bash
# 卸载所有 Python 依赖
python -m pip uninstall -y torch torchvision transformers Pillow numpy onnxruntime tokenizers huggingface-hub safetensors

# 删除模型缓存（可选）
rmdir /s /q %USERPROFILE%\.cache\huggingface

# 删除日志文件（可选）
del clip_server.log clip_server_err.log
```

然后重新运行 `setup.bat`。

---

## 技术支持

如果遇到无法解决的问题：
1. 运行 `CHECK_DRIVER.bat` 并保存输出
2. 查看 `clip_server.log` 错误信息
3. 记录你的系统配置（OS、Python 版本、GPU 型号、驱动版本）
4. 联系开发者并提供以上信息

---

## 附录：PyTorch CUDA 版本对照表

| CUDA 版本 | PyTorch 安装命令 | 驱动最低版本 |
|-----------|-----------------|-------------|
| CUDA 12.1 | `--index-url https://download.pytorch.org/whl/cu121` | 530.30.02 |
| CUDA 11.8 | `--index-url https://download.pytorch.org/whl/cu118` | 522.06 |
| CPU | `--index-url https://download.pytorch.org/whl/cpu` | N/A |

**推荐**：使用 CUDA 12.1（最新），除非你的显卡驱动太旧。

---

## 更新日志

- **2026-06-30**: 更新安装指南，修复 PyTorch CUDA 索引问题
- **2026-06-28**: 添加 GPU 诊断和修复工具
- **2026-06-25**: 初始版本
