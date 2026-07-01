# 部署包完整清单

## ✅ 必需文件

### 主程序
- [x] `LocalImageSearch.exe` - Tauri 主程序 (14MB)

### Python 后端
- [x] `clip_server.py` - Python TCP 服务端
- [x] `clip_server_tta_backup.py` - 备份（可删除）

### 模型文件 (7.1GB)
- [x] `models/siglip2-large/` - SigLIP2 模型
- [x] `models/clip-large/` - CLIP-L/14 模型
- [x] `models/clip_text.onnx` - ONNX 文本模型
- [x] `models/clip_vision.onnx` - ONNX 视觉模型

### 配置文件
- [x] `config/model_config.json` - 模型配置

### 依赖管理
- [x] `requirements.txt` - Python 依赖清单
- [x] `setup.bat` - 依赖安装脚本（英文版）
- [x] `setup.ps1` - 依赖安装脚本（PowerShell）
- [x] `check_env.bat` - 环境检查（英文版）
- [x] `check_env.ps1` - 环境检查（PowerShell）
- [x] `test_deploy.py` - 部署测试工具

### 文档
- [x] `README.md` - 项目说明
- [x] `DEPLOY.md` - 部署指南（英文）
- [x] `DEPLOY_CHECKLIST.md` - 部署检查清单
- [x] `ENCODING_FIX.md` - 乱码解决方案

---

## ❌ 不需要的文件

### 开发源码（运行不需要）
- [ ] `src/` - 前端源码
- [ ] `src-tauri/` - Rust 源码
- [ ] `node_modules/` - Node.js 依赖
- [ ] `package.json` - npm 配置
- [ ] `package-lock.json` - npm 锁文件

### 临时文件
- [ ] `*.log` - 日志文件
- [ ] `__pycache__/` - Python 缓存
- [ ] `*.pyc` - Python 编译文件
- [ ] `.git/` - Git 仓库

---

## 📦 打包建议

### 完整包（包含模型）
- 文件：`local-image-search2_full.zip`
- 大小：~7.5GB
- 包含：`models/` 目录
- 适用：新电脑首次部署

### 精简包（不含模型）
- 文件：`local-image-search2_app.zip`
- 大小：~50MB
- 不含：`models/` 目录
- 适用：已有模型的电脑，或需要重新下载模型

---

## 🚀 部署步骤（新电脑）

1. **解压备份**
   ```
   local-image-search2_full.zip → D:\local-image-search2\
   ```

2. **安装 Python**
   - 下载：https://www.python.org/downloads/
   - 勾选 "Add Python to PATH"

3. **安装依赖**
   - 双击 `setup.bat`（英文版，无乱码）
   - 或右键 `setup.ps1` → "使用 PowerShell 运行"

4. **检查环境**
   - 双击 `check_env.bat`
   - 或运行 `python test_deploy.py`

5. **运行**
   - 双击 `LocalImageSearch.exe`

---

## 🔧 常见问题快速修复

### 问题 1：乱码
**原因**：Windows 控制台不支持中文
**解决**：使用英文版 `.bat` 或 `.ps1`

### 问题 2：依赖缺失
**原因**：`onnxruntime` 或 `safetensors` 未安装
**解决**：运行 `fix.bat` 或 `setup.bat`

### 问题 3：端口被占用
**原因**：之前的进程未完全退出
**解决**：运行 `fix.bat` 或重启电脑

### 问题 4：模型加载失败
**原因**：模型文件损坏或缺失
**解决**：重新解压备份，或手动下载模型

### 问题 5：GPU 未检测到
**原因**：NVIDIA 驱动未安装
**解决**：安装驱动 from https://www.nvidia.com/Download/

---

## 📊 系统要求

| 项目 | 最低配置 | 推荐配置 |
|------|---------|---------|
| 操作系统 | Windows 10 64位 | Windows 11 64位 |
| Python | 3.10+ | 3.12 |
| 内存 | 16GB | 32GB |
| 显卡 | 无（CPU模式） | NVIDIA GPU 8GB+ |
| 硬盘 | 10GB | 10GB SSD |
| 显存 | - | 8GB+ |

---

## ✅ 验证清单

部署完成后，验证以下功能：

- [ ] 应用能启动
- [ ] 模型状态显示"已加载"
- [ ] 可以添加文件夹
- [ ] 可以索引图片
- [ ] 文本搜索可用
- [ ] 以图搜图可用
- [ ] 无错误日志

---

## 📞 技术支持

如遇问题，提供以下信息：

1. `check_env.bat` 的输出截图
2. `python test_deploy.py` 的输出
3. `clip_server.log` 的内容
4. 系统信息和 Python 版本
