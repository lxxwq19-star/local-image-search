# Local Image Search 2

本地图片语义搜索引擎 - 使用 AI 模型理解图片内容，支持以图搜图和自然语言搜图。

---

## ✨ 功能特性

- **🔍 以图搜图**：上传一张图片，找出视觉上相似的图片
- **💬 语义搜图**：用自然语言描述你想找的图片（如"日落风景"、"猫咪照片"）
- **⚡ GPU 加速**：支持 NVIDIA GPU 加速，索引速度提升 10x+
- **📁 文件夹索引**：批量索引本地文件夹，建立向量数据库
- **🎯 实时搜索**：输入即搜，无需等待

---

## 🚀 快速开始

### 1. 安装 Python
下载并安装 Python 3.10（推荐）：
- 官网：https://www.python.org/downloads/
- **重要**：安装时勾选 "Add Python to PATH"

### 2. 安装依赖
双击运行 `setup.bat`，脚本会自动安装所有依赖。

### 3. 启动应用
双击 `LocalImageSearch.exe` 启动应用。

---

## 📚 文档索引

| 文档 | 用途 |
|------|------|
| `INSTALL_GUIDE.md` | **完整安装指南**（推荐阅读） |
| `README.md` | 本文件（项目概述） |
| `DRIVER_FIX_GUIDE.md` | 显卡驱动问题修复指南 |
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

### Q: 应用启动后黑屏或闪退？
**A**: 查看 `clip_server.log` 和 `clip_server_err.log` 查看错误信息。通常是因为 Python 依赖未正确安装，运行 `setup.bat` 重新安装。

### Q: 搜索索引很慢（CPU 模式）？
**A**: 检查是否安装了 GPU 版 PyTorch：
```bash
python -c "import torch; print('CUDA available:', torch.cuda.is_available())"
```
如果显示 `False`，运行 `FIX_GPU.bat` 安装 GPU 版本。

### Q: `nvidia-smi` 命令找不到？
**A**: NVIDIA 驱动未安装。访问 https://www.nvidia.com/Download/index.aspx 下载并安装驱动，然后重启电脑。

### Q: 安装依赖很慢？
**A**: `setup.bat` 已配置清华和阿里镜像源。如果仍慢，检查网络连接或手动设置镜像：
```bash
pip config set global.index-url https://pypi.tuna.tsinghua.edu.cn/simple
```

---

## 📋 系统要求

### 最低配置
- Windows 10/11 64位
- Python 3.8 - 3.11
- 8 GB RAM
- 5 GB 可用存储空间

### 推荐配置（GPU 加速）
- NVIDIA GPU（RTX 2060 或更高）
- 6 GB 以上显存
- NVIDIA Driver 530+
- 16 GB RAM

---

## 🔧 技术栈

- **前端**: Tauri (Rust + WebView)
- **后端**: Python TCP 服务器
- **AI 模型**:
  - SigLIP2（图像编码，1024 维）
  - CLIP-L/14（文本编码，768 维）
- **向量数据库**: 本地 JSON 存储
- **GPU 加速**: PyTorch CUDA

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

**最后更新**: 2026-06-30
