# 部署检查清单

## ✅ 部署前检查

### 1. 系统环境
- [ ] Windows 10/11 64位
- [ ] 16GB+ 内存
- [ ] 10GB+ 可用硬盘空间
- [ ] NVIDIA GPU（可选，CPU模式也可用）

### 2. Python 环境
- [ ] Python 3.10+ 已安装
- [ ] Python 已添加到 PATH
- [ ] pip 可正常使用

### 3. 依赖安装
- [ ] 运行 `setup.bat` 安装依赖
- [ ] 或手动运行 `pip install -r requirements.txt`
- [ ] 验证安装：`python -c "import torch, transformers, numpy"`

### 4. 模型文件
- [ ] `models/siglip2-large/` 目录存在
- [ ] `models/clip-large/` 目录存在
- [ ] 模型文件完整（无损坏）

### 5. 环境检查
- [ ] 运行 `check_env.bat` 检查环境
- [ ] 所有检查项显示 ✅

---

## 🚀 部署步骤

1. **解压备份**
   - 将 `local-image-search2_stable_20260626.tar` 解压到目标目录
   - 例如：`D:\local-image-search2\`

2. **安装 Python 依赖**
   ```bat
   cd /d D:\local-image-search2
   setup.bat
   ```

3. **检查环境**
   ```bat
   check_env.bat
   ```

4. **运行应用**
   - 双击 `LocalImageSearch.exe`

---

## 🔍 故障排查

### 问题：应用启动后无法搜索

**排查步骤：**

1. **手动启动 Python 后端**
   ```bat
   cd /d D:\local-image-search2
   python clip_server.py
   ```
   观察错误信息

2. **检查日志文件**
   - `clip_server.log` - 服务端日志
   - `clip_server_err.log` - 错误日志

3. **常见错误**

   | 错误信息 | 原因 | 解决方法 |
   |----------|------|----------|
   | `No module named 'torch'` | PyTorch 未安装 | 运行 `setup.bat` |
   | `No module named 'transformers'` | transformers 未安装 | 运行 `setup.bat` |
   | `Model directory not found` | 模型文件缺失 | 检查备份是否完整解压 |
   | `Address already in use` | 端口被占用 | 关闭占用程序或重启 |
   | `CUDA out of memory` | 显存不足 | 关闭其他 GPU 程序或使用 CPU 模式 |
   | `dtype parameter error` | 代码版本旧 | 更新到最新 `clip_server.py` |

---

## 📦 部署包内容

完整的部署包应包含：

```
local-image-search2/
├── LocalImageSearch.exe      ✅ 主程序
├── clip_server.py           ✅ Python 后端
├── models/                   ✅ 模型文件
│   ├── siglip2-large/
│   ├── clip-large/
│   ├── clip_text.onnx
│   └── clip_vision.onnx
├── config/                   ✅ 配置文件
├── requirements.txt          ✅ 依赖清单
├── setup.bat                 ✅ 一键安装
├── check_env.bat             ✅ 环境检查
└── DEPLOY.md                 ✅ 部署文档
```

**不需要的文件：**
- `src/` - 前端源码
- `src-tauri/` - Rust 源码
- `node_modules/` - Node.js 依赖
- `*.log` - 日志文件
- `__pycache__/` - Python 缓存

---

## 💡 优化建议

1. **使用 SSD** - 模型加载速度提升 3-5 倍
2. **NVIDIA GPU** - 搜索速度提升 5-10 倍
3. **关闭杀毒软件实时防护** - 避免误报和性能影响
4. **定期清理日志** - 避免日志文件过大

---

## 📞 技术支持

如果遇到无法解决的问题：

1. 运行 `check_env.bat` 并截图结果
2. 手动运行 `python clip_server.py` 并截图错误信息
3. 提供系统信息和 Python 版本
