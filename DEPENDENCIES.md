# Python 依赖详解

本文档详细说明 `local-image-search2` 项目的所有 Python 依赖包，包括版本要求、用途、安装源和常见问题。

---

## 核心依赖（必需）

### 1. PyTorch (`torch`)
- **版本**: >=2.0.0
- **用途**: 深度学习框架，用于加载和运行 AI 模型
- **安装源**: 
  - GPU 版: `https://download.pytorch.org/whl/cu121`
  - CPU 版: `https://download.pytorch.org/whl/cpu`
- **大小**: ~2.5 GB (GPU 版)
- **注意事项**: 
  - **必须从 PyTorch 官方源安装**，PyPI 默认只提供 CPU 版
  - 安装命令: `pip install torch --index-url https://download.pytorch.org/whl/cu121`
  - 验证安装: `python -c "import torch; print(torch.cuda.is_available())"`

**常见问题**:
- Q: 安装了 torch 但 `torch.cuda.is_available()` 返回 `False`？
  - A: 安装了 CPU 版，运行 `FIX_GPU.bat` 重新安装 GPU 版
- Q: 安装速度太慢？
  - A: 使用国内镜像或确保网络稳定，GPU 版约 2.5 GB

---

### 2. TorchVision (`torchvision`)
- **版本**: >=0.15.0
- **用途**: 图像处理工具，与 PyTorch 配套使用
- **安装源**: 与 PyTorch 相同（自动匹配版本）
- **大小**: ~50 MB
- **注意事项**: 必须与 `torch` 一起安装，版本要兼容

**安装命令**:
```bash
pip install torchvision --index-url https://download.pytorch.org/whl/cu121
```

---

### 3. Transformers (`transformers`)
- **版本**: >=4.30.0, <5.0.0
- **用途**: HuggingFace 模型库，用于加载 SigLIP2 和 CLIP 模型
- **安装源**: PyPI（默认）
- **大小**: ~50 MB
- **注意事项**: 
  - **必须固定 <5.0.0**，因为 5.x 版本 API 有破坏性变更
  - SigLIP2 需要 `transformers>=4.30.0`
  - CLIP 需要 `transformers>=4.25.0`

**常见问题**:
- Q: 报错 `AttributeError: 'BaseModelOutputWithPooling' object has no attribute 'float'`？
  - A: 安装了 transformers 5.x，运行 `pip install transformers<5.0.0` 降级
- Q: 模型下载失败？
  - A: 设置 HF 镜像: `set HF_ENDPOINT=https://hf-mirror.com`

---

### 4. Pillow (`Pillow`)
- **版本**: >=9.0.0
- **用途**: 图像读写库，用于加载和预处理图片
- **安装源**: PyPI（默认）
- **大小**: ~40 MB
- **注意事项**: 必须安装，否则无法读取图片文件

**安装命令**:
```bash
pip install Pillow
```

---

### 5. NumPy (`numpy`)
- **版本**: >=1.24.0
- **用途**: 向量运算库，用于处理图像特征向量
- **安装源**: PyPI（默认）
- **大小**: ~30 MB
- **注意事项**: 必须与 PyTorch 版本兼容

---

## 可选依赖

### 6. ONNX Runtime (`onnxruntime`)
- **版本**: >=1.16.0
- **用途**: ONNX 模型推理引擎（CPU 模式备用）
- **安装源**: PyPI（默认）
- **大小**: ~50 MB
- **注意事项**: 
  - 可选，但推荐安装（提供 CPU 模式回退）
  - GPU 模式不需要此包

---

### 7. Tokenizers (`tokenizers`)
- **版本**: >=0.13.0
- **用途**: 文本分词器，用于 CLIP 文本编码
- **安装源**: PyPI（默认）
- **大小**: ~10 MB
- **注意事项**: `transformers` 会自动安装此包

---

### 8. HuggingFace Hub (`huggingface-hub`)
- **版本**: >=0.20.0
- **用途**: 从 HuggingFace 下载模型
- **安装源**: PyPI（默认）
- **大小**: ~5 MB
- **注意事项**: 必须安装，否则无法下载模型

**常见问题**:
- Q: 模型下载失败？
  - A: 设置 HF 镜像: `set HF_ENDPOINT=https://hf-mirror.com`

---

### 9. SafeTensors (`safetensors`)
- **版本**: >=0.3.0
- **用途**: 安全模型格式支持
- **安装源**: PyPI（默认）
- **大小**: ~5 MB
- **注意事项**: 新版 `transformers` 需要此包

---

## 依赖关系图

```
local-image-search2
├── torch (>=2.0.0)           # 核心深度学习框架
│   └── torchvision (>=0.15.0) # 图像处理
├── transformers (>=4.30.0,<5.0.0) # 模型加载
│   ├── tokenizers (>=0.13.0)  # 文本分词
│   ├── huggingface-hub (>=0.20.0) # 模型下载
│   └── safetensors (>=0.3.0)  # 安全模型格式
├── Pillow (>=9.0.0)          # 图像读写
├── numpy (>=1.24.0)          # 向量运算
└── onnxruntime (>=1.16.0)    # ONNX 推理（可选）
```

---

## 安装顺序

推荐的安装顺序（避免依赖冲突）：

1. **PyTorch + TorchVision** (必须先安装，且要从 CUDA 索引安装)
   ```bash
   pip install torch torchvision --index-url https://download.pytorch.org/whl/cu121
   ```

2. **Transformers + 其他依赖**
   ```bash
   pip install transformers\<5.0.0 Pillow numpy onnxruntime tokenizers huggingface-hub safetensors
   ```

3. **验证安装**
   ```bash
   python -c "import torch; import transformers; import PIL; import numpy; print('All OK')"
   ```

---

## 完整安装命令

### GPU 版本（推荐）
```bash
# 1. 配置镜像（中国用户）
pip config set global.index-url https://pypi.tuna.tsinghua.edu.cn/simple
set HF_ENDPOINT=https://hf-mirror.com

# 2. 安装 PyTorch (GPU)
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu121

# 3. 安装其他依赖
pip install transformers\<5.0.0 Pillow numpy onnxruntime tokenizers huggingface-hub safetensors
```

### CPU 版本（无 GPU）
```bash
# 1. 配置镜像（中国用户）
pip config set global.index-url https://pypi.tuna.tsinghua.edu.cn/simple
set HF_ENDPOINT=https://hf-mirror.com

# 2. 安装 PyTorch (CPU)
pip install torch torchvision --index-url https://download.pytorch.org/whl/cpu

# 3. 安装其他依赖
pip install transformers\<5.0.0 Pillow numpy onnxruntime tokenizers huggingface-hub safetensors
```

---

## 版本兼容性表

| 组件 | 推荐版本 | 最低版本 | 最高版本 | 备注 |
|------|---------|---------|---------|------|
| Python | 3.10 | 3.8 | 3.11 | 3.12 可能不兼容 |
| torch | 2.1.0 | 2.0.0 | 最新 | GPU 版需匹配 CUDA |
| torchvision | 0.16.0 | 0.15.0 | 最新 | 与 torch 版本匹配 |
| transformers | 4.36.0 | 4.30.0 | <5.0.0 | 5.x 有 API 破坏 |
| Pillow | 10.0.0 | 9.0.0 | 最新 | - |
| numpy | 1.26.0 | 1.24.0 | <2.0.0 | 2.x 有 API 变化 |

---

## 故障排除

### 问题 1: PyTorch CUDA 不可用
**症状**: `torch.cuda.is_available()` 返回 `False`

**原因**:
1. 安装了 CPU 版 PyTorch
2. NVIDIA 驱动未安装或太旧
3. CUDA 版本不匹配

**解决方案**:
1. 运行 `FIX_GPU.bat` 重新安装 GPU 版
2. 更新 NVIDIA 驱动到 530+
3. 运行 `CHECK_DRIVER.bat` 诊断

---

### 问题 2: Transformers API 错误
**症状**: `AttributeError` 或 `TypeError` 来自 `transformers`

**原因**: 安装了 transformers 5.x

**解决方案**:
```bash
pip uninstall -y transformers
pip install transformers\<5.0.0
```

---

### 问题 3: 模型下载失败
**症状**: `ConnectionError` 或 `TimeoutError` 当下载模型时

**原因**: HuggingFace 官网在国内访问不稳定

**解决方案**:
```bash
set HF_ENDPOINT=https://hf-mirror.com
```
然后重新运行应用（会自动下载模型）

---

### 问题 4: 内存不足
**症状**: `OutOfMemoryError` 或应用崩溃

**原因**: 模型太大，内存不够

**解决方案**:
1. 使用 CPU 模式（自动回退）
2. 关闭其他占用内存的应用
3. 升级内存到 16 GB 以上

---

## 验证脚本

运行以下 Python 脚本验证所有依赖是否正确安装：

```python
import sys

print("Python version:", sys.version)
print()

packages = [
    ("torch", "PyTorch"),
    ("torchvision", "TorchVision"),
    ("transformers", "Transformers"),
    ("PIL", "Pillow"),
    ("numpy", "NumPy"),
    ("onnxruntime", "ONNX Runtime"),
    ("tokenizers", "Tokenizers"),
    ("huggingface_hub", "HuggingFace Hub"),
    ("safetensors", "SafeTensors"),
]

for module, name in packages:
    try:
        mod = __import__(module)
        version = getattr(mod, "__version__", "unknown")
        print(f"[OK] {name}: {version}")
    except ImportError:
        print(f"[FAIL] {name}: NOT INSTALLED")

print()
print("Verification complete.")
```

保存为 `verify_deps.py` 并运行：
```bash
python verify_deps.py
```

---

## 更新日志

- **2026-06-30**: 更新依赖版本，修复 PyTorch CUDA 索引问题
- **2026-06-28**: 添加 GPU 诊断和修复工具
- **2026-06-25**: 初始版本

---

## 参考链接

- PyTorch 官网: https://pytorch.org/
- Transformers 文档: https://huggingface.co/docs/transformers/
- Pillow 文档: https://pillow.readthedocs.io/
- NumPy 文档: https://numpy.org/doc/
