# Local Image Search 2

本地图片语义搜索引擎 - 使用 AI 模型理解图片内容，支持以图搜图和自然语言搜图。

---

## ✨ 功能特性

- **🔍 以图搜图**：上传一张图片，找出视觉上相似的图片
- **💬 语义搜图**：用自然语言描述你想找的图片（如"日落风景"、"猫咪照片"）
- **⚡ GPU 加速**：
  - Windows: NVIDIA GPU (CUDA)，索引速度提升 10x+
  - macOS: Apple Silicon GPU (MPS)，原生加速
- **📁 文件夹索引**：批量索引本地文件夹，建立向量数据库
- **🎯 实时搜索**：输入即搜，无需等待

---

## 🖥️ 支持的平台

| 平台 | 架构 | GPU 加速 | 状态 |
|------|------|---------|------|
| Windows 10/11 | x86_64 | ✅ NVIDIA CUDA | ✅ 完全支持 |
| macOS (M1/M2/M3) | ARM64 | ✅ Apple Silicon MPS | ✅ 完全支持 |
| macOS (Intel) | x86_64 | ❌ | ❌ 不支持 |
| Linux | x86_64 | ✅ NVIDIA CUDA | 🔄 计划中 |

> **Mac 用户请注意**：只支持 Apple Silicon (M1/M2/M3) Mac。
> Intel Mac 用户请参考 [MAC_COMPATIBILITY.md](MAC_COMPATIBILITY.md)

---

## 🚀 快速开始

### Windows 用户

#### 1. 安装 Python
下载并安装 Python 3.10（推荐）：
- 官网：https://www.python.org/downloads/
- **重要**：安装时勾选 "Add Python to PATH"

#### 2. 安装依赖
双击运行 `setup.bat`，脚本会自动安装所有依赖。

#### 3. 启动应用
双击 `LocalImageSearch.exe` 启动应用。

---

### macOS (Apple Silicon) 用户

#### 1. 下载 Mac 版本
前往 [GitHub Releases](https://github.com/lxxwq19-star/local-image-search/releases) 或 [Actions 构建页面](https://github.com/lxxwq19-star/local-image-search/actions) 下载 `.dmg` 安装包。

#### 2. 安装
双击 `.dmg` 文件，将 `LocalImageSearch.app` 拖到 `Applications` 文件夹。

#### 3. 首次运行（绕过 Gatekeeper）
**不要直接双击打开**，右键点击 `LocalImageSearch.app` → **打开**，在弹窗中点击 **"打开"**。

#### 4. 安装 Python 依赖（首次运行自动提示）
```bash
pip3 install torch torchvision transformers Pillow numpy
```
> **注意**：Mac 版本已打包了独立 Python 服务，大多数情况下不需要手动安装依赖。

---

## 📚 文档索引

| 文档 | 用途 |
|------|------|
| `INSTALL_GUIDE.md` | **完整安装指南**（推荐阅读） |
| `README.md` | 本文件（项目概述） |
| `MAC_COMPATIBILITY.md` | **Mac 兼容性说明**（Mac 用户必读） |
| `DRIVER_FIX_GUIDE.md` | 显卡驱动问题修复指南（Windows） |
| `GITHUB_MAC_BUILD.md` | GitHub Actions 云编译 Mac 版本指南 |
| `requirements.txt` | Python 依赖列表 |

---

## 🛠️ 工具脚本

| 脚本 | 用途 |
|------|------|
| `setup.bat` | **主安装脚本**（双击运行） |
| `VERIFY_INSTALL.bat` | 验证安装是否成功 |
| `CHECK_DRIVER.bat` | 检查显卡驱动和 CUDA 状态 |
| `FIX_GPU.bat` | 修复 GPU 版本 PyTorch 安装 |
| `INSTALL_GPU_TORCH.bat` | 仅重新安装 GPU 版 PyTorch |

---

## 🐛 常见问题

### Windows 用户

#### Q: 应用启动后黑屏或闪退？
**A**: 查看 `clip_server.log` 和 `clip_server_err.log` 查看错误信息。通常是因为 Python 依赖未正确安装，运行 `setup.bat` 重新安装。

#### Q: 搜索索引很慢（CPU 模式）？
**A**: 检查是否安装了 GPU 版 PyTorch：
```bash
python -c "import torch; print('CUDA available:', torch.cuda.is_available())"
```
如果显示 `False`，运行 `FIX_GPU.bat` 安装 GPU 版本。

#### Q: `nvidia-smi` 命令找不到？
**A**: NVIDIA 驱动未安装。访问 https://www.nvidia.com/Download/index.aspx 下载并安装驱动，然后重启电脑。

#### Q: 安装依赖很慢？
**A**: `setup.bat` 已配置清华和阿里镜像源。如果仍慢，检查网络连接或手动设置镜像：
```bash
pip config set global.index-url https://pypi.tuna.tsinghua.edu.cn/simple
```

---

### macOS (Apple Silicon) 用户

#### Q: "LocalImageSearch" 无法打开，因为无法验证开发者？
**A**: macOS Gatekeeper 拦截了未签名的应用。解决方法：
1. 不要直接双击打开
2. 右键点击 `LocalImageSearch.app` → **打开**
3. 在弹窗中点击 **"打开"**
4. 以后就可以直接双击打开了

#### Q: 索引速度慢？
**A**: 检查是否启用了 MPS GPU 加速：
1. 打开 `~/Library/Logs/clip_server.log`（如果有的话）
2. 查找 `[STARTUP] MPS available: True`
3. 如果显示 `False`，说明在用 CPU，速度会慢 5-10 倍
4. 确保你用的是 Apple Silicon Mac（不是 Intel Mac）

#### Q: 应用闪退？
**A**: 可能是内存不足（模型需要约 2-3 GB 内存）。关闭一些应用后再试。
如果是 Intel Mac，本版本不支持，请参考 [MAC_COMPATIBILITY.md](MAC_COMPATIBILITY.md)。

#### Q: 下载模型很慢？
**A**: 首次运行会下载 AI 模型（约 2-3 GB）。确保网络畅通，或设置 HF 镜像：
```bash
export HF_ENDPOINT=https://hf-mirror.com
```

---

## 📋 系统要求

### Windows

#### 最低配置
- Windows 10/11 64位
- Python 3.8 - 3.11
- 8 GB RAM
- 5 GB 可用存储空间

#### 推荐配置（GPU 加速）
- NVIDIA GPU（RTX 2060 或更高）
- 6 GB 以上显存
- NVIDIA Driver 530+
- 16 GB RAM

---

### macOS (Apple Silicon)

#### 最低配置
- macOS 12.3 (Monterey) 或更高
- Apple Silicon Mac (M1/M2/M3/M4)
- 8 GB RAM
- 5 GB 可用存储空间

#### 推荐配置（最佳性能）
- M2/M3 Max/Pro 或 M4 系列
- 16 GB RAM 或更多
- 足够的 SSD 空间存放向量索引

> **如何确认是 Apple Silicon？**  
> 点击左上角苹果图标 → **关于本机**，看到 "Apple M1/M2/M3" 即表示支持。

---

## 🔧 技术栈

- **前端**: Tauri (Rust + WebView)
- **后端**: Python TCP 服务器
- **AI 模型**:
  - SigLIP2（图像编码，1024 维）
  - CLIP-L/14（文本编码，768 维）
- **向量数据库**: 本地 JSON 存储
- **GPU 加速**:
  - Windows: PyTorch CUDA (NVIDIA)
  - macOS: PyTorch MPS (Apple Silicon)

---

## 📝 开发文档

详细的技术文档和 API 说明，请参考：
- `INSTALL_GUIDE.md` - 完整安装指南
- `DRIVER_FIX_GUIDE.md` - 驱动问题修复
- 源代码注释

---

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

---

## 📄 许可证

MIT License

---

## 📧 联系方式

如有问题，请提交 GitHub Issue 或联系开发者。

---

**最后更新**: 2026-07-01
