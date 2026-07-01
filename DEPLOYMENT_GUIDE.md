# Local Image Search 2 - 完整部署清单

## 📋 目录
1. [系统要求](#系统要求)
2. [必需软件](#必需软件)
3. [Python 依赖](#python-依赖)
4. [模型文件](#模型文件)
5. [环境变量](#环境变量)
6. [部署步骤](#部署步骤)
7. [验证安装](#验证安装)
8. [故障排查](#故障排查)

---

## 系统要求

### 最低配置
| 组件 | 要求 |
|------|------|
| **操作系统** | Windows 10 版本 1809+ / Windows 11 |
| **CPU** | x86_64 架构，支持 AVX2 |
| **内存** | 16 GB RAM |
| **硬盘** | 10 GB 可用空间（含模型） |
| **显卡** | 可选（无显卡也能运行，但速度慢 5-10 倍） |

### 推荐配置（最佳体验）
| 组件 | 要求 |
|------|------|
| **操作系统** | Windows 11 23H2+ |
| **CPU** | Intel i7-10700 / AMD R7 3700X 或更高 |
| **内存** | 32 GB RAM |
| **硬盘** | NVMe SSD，20 GB 可用空间 |
| **显卡** | NVIDIA GPU，8 GB+ 显存（RTX 3060 或更高） |
| **驱动** | NVIDIA Driver 535+ |

---

## 必需软件

### 1. Python 3.10 - 3.13
- **下载**：https://www.python.org/downloads/windows/
- **版本要求**：3.10, 3.11, 3.12, 或 3.13
- **安装注意**：
  - ✅ 勾选 **"Add Python to PATH"**
  - ✅ 勾选 **"Install pip"**
  - ✅ 勾选 **"Install py launcher"**（推荐）
- **验证安装**：
  ```bat
  python --version
  pip --version
  ```

### 2. Microsoft Visual C++ Redistributable
- **下载**：https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist
- **版本**：2022 或更新
- **位数**：x64（64位）
- **说明**：大多数 Windows 电脑已预装，如果运行时有 `VCRUNTIME140.dll` 错误，则需要安装

### 3. CUDA Toolkit（仅 NVIDIA GPU 用户）
- **说明**：**不需要手动安装！**
- **原因**：`torch` 包已自带 CUDA 运行时（12.4），会自动使用 GPU
- **前提**：需要安装 NVIDIA 显卡驱动（535+ 版本）
- **验证 GPU 可用**：
  ```bat
  python -c "import torch; print(torch.cuda.is_available())"
  ```
  应返回 `True`

---

## Python 依赖

### 完整依赖清单（requirements.txt）

```txt
# 核心依赖（必需）
torch>=2.0.0
torchvision>=0.15.0
transformers>=4.30.0
numpy>=1.24.0
Pillow>=9.0.0

# 可选依赖（推荐安装，提升兼容性）
onnxruntime>=1.16.0
tokenizers>=0.13.0
huggingface-hub>=0.20.0
safetensors>=0.3.0
```

### 依赖说明

| 包名 | 版本要求 | 用途 | 大小 | 必需 |
|------|----------|------|------|------|
| `torch` | ≥2.0.0 | PyTorch 深度学习框架 | ~200 MB | ✅ 必需 |
| `torchvision` | ≥0.15.0 | 视觉模型支持 | ~50 MB | ✅ 必需 |
| `transformers` | ≥4.30.0 | SigLIP2 / CLIP-L/14 模型加载 | ~100 MB | ✅ 必需 |
| `numpy` | ≥1.24.0 | 向量运算 | ~30 MB | ✅ 必需 |
| `Pillow` | ≥9.0.0 | 图片处理 | ~10 MB | ✅ 必需 |
| `onnxruntime` | ≥1.16.0 | ONNX 模型推理（备用） | ~50 MB | 💡 推荐 |
| `tokenizers` | ≥0.13.0 | 文本分词器 | ~5 MB | 💡 推荐 |
| `huggingface-hub` | ≥0.20.0 | HuggingFace 工具库 | ~10 MB | 💡 推荐 |
| `safetensors` | ≥0.3.0 | 安全模型加载格式 | ~5 MB | 💡 推荐 |

### 精确版本（本机测试通过）

```
torch==2.6.0+cu124
torchvision==0.21.0+cu124
transformers==5.12.1
numpy==2.4.4
onnxruntime==1.27.0
tokenizers==0.22.2
Pillow==12.2.0
huggingface-hub==1.21.0
safetensors==0.8.0
```

### 安装命令

**方法 1：使用 setup.bat（推荐）**
```bat
cd /d D:\local-image-search2
setup.bat
```

**方法 2：手动安装**
```bat
cd /d D:\local-image-search2
python -m pip install -r requirements.txt
```

**方法 3：国内镜像加速**
```bat
pip config set global.index-url https://pypi.tuna.tsinghua.edu.cn/simple
python -m pip install -r requirements.txt
```

---

## 模型文件

### 完整模型清单

| 模型 | 路径 | 大小 | 用途 |
|------|------|------|------|
| **SigLIP2-Large** | `models/siglip2-large/` | ~3.5 GB | 以图搜图（图像编码器） |
| **CLIP-L/14** | `models/clip-large/` | ~3.6 GB | 语义搜图（文本编码器） |
| **CLIP ONNX** | `models/clip_text.onnx` | ~500 MB | 备用文本编码器（ONNX） |
| **CLIP ONNX** | `models/clip_vision.onnx` | ~1.2 GB | 备用图像编码器（ONNX） |

### 模型文件结构

```
D:\local-image-search2\models\
├── siglip2-large\
│   ├── config.json
│   ├── model.safetensors
│   ├── preprocessor_config.json
│   └── tokenizer.json
├── clip-large\
│   ├── config.json
│   ├── model.safetensors
│   ├── preprocessor_config.json
│   └── tokenizer.json
├── clip_text.onnx
└── clip_vision.onnx
```

### 模型下载

**如果备份不完整，可以单独下载模型：**

```bat
# 下载 SigLIP2 模型
python download_siglip2.py

# 下载 CLIP-L/14 模型（从 HuggingFace）
python -c "from transformers import AutoModel, AutoProcessor; AutoModel.from_pretrained('openai/clip-vit-large-patch14'); AutoProcessor.from_pretrained('openai/clip-vit-large-patch14')"
```

---

## 环境变量

### 可选环境变量

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `MODEL_VARIANT` | `siglip2` | 模型变体（`siglip2` 或 `clip-large`） |
| `PYTHONIOENCODING` | `utf-8` | Python 输出编码 |
| `PYTHONUTF8` | `1` | 强制 Python 使用 UTF-8 |
| `HF_ENDPOINT` | `https://huggingface.co` | HuggingFace 镜像站 |

### 设置方法

```bat
# 临时设置（当前会话）
set MODEL_VARIANT=siglip2
set HF_ENDPOINT=https://hf-mirror.com

# 永久设置（系统级）
setx MODEL_VARIANT siglip2
setx HF_ENDPOINT https://hf-mirror.com
```

---

## 部署步骤

### 完整部署流程（新电脑）

#### 第 1 步：解压备份
```bat
# 将 local-image-search2_stable_20260626.tar 解压到目标目录
# 例如：D:\local-image-search2\
```

#### 第 2 步：安装 Python
1. 下载 Python 3.12：https://www.python.org/downloads/windows/
2. 运行安装程序
3. ✅ 勾选 **"Add Python to PATH"**
4. ✅ 勾选 **"Install pip"**
5. 点击 **"Install Now"**

#### 第 3 步：安装依赖
```bat
cd /d D:\local-image-search2
setup.bat
```

#### 第 4 步：验证安装
```bat
check_env.bat
```
确保所有检查项显示 ✅

#### 第 5 步：运行
双击 `LocalImageSearch.exe`

---

## 验证安装

### 验证脚本（check_env.bat）

运行 `check_env.bat`，应该看到：

```
[OK] Python 3.12.0
[OK] PyTorch 2.6.0+cu124
[OK] Transformers 5.12.1
[OK] Model files found
[OK] Port 8765 is free
[OK] All checks passed!
```

### 手动验证

```bat
# 1. 验证 Python
python --version

# 2. 验证 PyTorch（含 GPU）
python -c "import torch; print('CUDA available:', torch.cuda.is_available()); print('GPU:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'None')"

# 3. 验证 Transformers
python -c "from transformers import AutoModel; print('Transformers OK')"

# 4. 验证模型文件
dir models\siglip2-large\config.json
dir models\clip-large\config.json

# 5. 测试后端启动
cd /d D:\local-image-search2
python clip_server.py
# 应该看到：
# [SERVER] TCP server listening on 127.0.0.1:8765
```

---

## 故障排查

### 常见问题

#### Q1: `python` 命令不识别
**原因**：Python 未添加到 PATH  
**解决**：
1. 重新安装 Python，勾选 "Add Python to PATH"
2. 或手动添加：将 `C:\Python312\` 和 `C:\Python312\Scripts\` 添加到系统 PATH

#### Q2: `No module named 'torch'`
**原因**：PyTorch 未安装  
**解决**：
```bat
python -m pip install torch torchvision
```

#### Q3: `Cannot load library: VCRUNTIME140.dll`
**原因**：Visual C++ 运行库缺失  
**解决**：下载并安装 [Microsoft Visual C++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist)

#### Q4: 后端启动失败（端口 8765 被占用）
**原因**：其他程序占用了端口  
**解决**：
```bat
# 查找占用端口的进程
netstat -ano | find ":8765"

# 杀掉进程
taskkill /F /PID <PID>
```

#### Q5: GPU 未启用（CUDA not available）
**原因**：
1. NVIDIA 驱动未安装或版本太旧
2. PyTorch 安装了 CPU 版本

**解决**：
1. 更新 NVIDIA 驱动：https://www.nvidia.com/Download/index.aspx
2. 重新安装 PyTorch（GPU 版）：
```bat
python -m pip uninstall torch torchvision
python -m pip install torch torchvision --index-url https://download.pytorch.org/whl/cu124
```

#### Q6: 模型加载失败（`Model directory not found`）
**原因**：模型文件缺失或路径错误  
**解决**：
1. 检查 `models/` 目录是否存在
2. 重新解压备份文件
3. 或手动下载模型

#### Q7: 内存不足（`CUDA out of memory`）
**原因**：GPU 显存不足  
**解决**：
1. 关闭占用显存的程序（游戏、浏览器等）
2. 使用 CPU 模式（速度慢但可用）：
   - 编辑 `config/model_config.json`
   - 设置 `"device": "cpu"`

---

## 完整文件清单

### 运行必需文件

```
D:\local-image-search2\
├── LocalImageSearch.exe          # 主程序（14 MB）
├── clip_server.py                # Python 后端（50 KB）
├── models\                       # 模型文件（7.1 GB）
│   ├── siglip2-large\
│   ├── clip-large\
│   ├── clip_text.onnx
│   └── clip_vision.onnx
├── config\                       # 配置文件（1 KB）
│   └── model_config.json
└── requirements.txt              # 依赖清单（200 B）
```

### 部署工具（可选）

```
├── setup.bat                     # 依赖安装脚本
├── setup.ps1                     # 依赖安装脚本（PowerShell）
├── check_env.bat                 # 环境检查脚本
├── check_env.ps1                 # 环境检查脚本（PowerShell）
├── test_exe.bat                  # exe 测试脚本
├── fix.bat                       # 一键修复脚本
└── DEPLOY.md                     # 部署指南
```

---

## 总结

### 最少安装（必需）
1. ✅ Python 3.10+（添加到 PATH）
2. ✅ PyTorch + Transformers（`pip install torch transformers`）
3. ✅ 模型文件（`models/` 目录）

### 推荐安装（最佳体验）
1. ✅ Python 3.12
2. ✅ 所有依赖（`setup.bat`）
3. ✅ NVIDIA GPU（8 GB+ 显存）
4. ✅ NVMe SSD（提升索引速度）

### 不需要安装
- ❌ Rust / Cargo（exe 已编译）
- ❌ Node.js / npm（exe 已编译）
- ❌ CUDA Toolkit（PyTorch 自带）
- ❌ Git（除非需要从源码编译）

---

**文档版本**：2026-06-30  
**适用版本**：Local Image Search 2.0+（双模型架构）
