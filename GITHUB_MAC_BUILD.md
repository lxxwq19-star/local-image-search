# GitHub 云编译 Mac 版本指南

本文档说明如何使用 GitHub Actions 在云端编译 `local-image-search2` 的 macOS 版本。

---

## ⚠️ 重要注意事项

### 代码签名问题（macOS 特有）
**问题**：macOS 应用必须经过代码签名才能在别人的 Mac 上运行，否则会被 Gatekeeper 拦截。

**解决方案**：
1. **有 Apple Developer 账号**：配置签名证书（推荐）
2. **没有证书**：构建未签名版本（只能自己用，或需要用户手动绕过 Gatekeeper）

---

## 🚀 快速开始

### 步骤 1: 推送代码到 GitHub

如果你的代码还没推送到 GitHub：

```bash
# 1. 初始化 git（如果还没有）
cd D:\local-image-search2
git init

# 2. 添加所有文件
git add .

# 3. 提交
git commit -m "Initial commit: Local Image Search 2"

# 4. 添加远程仓库（替换为你的仓库 URL）
git remote add origin https://github.com/lxxwq19-star/local-image-search2.git

# 5. 推送到 GitHub
git push -u origin main
```

### 步骤 2: 启用 GitHub Actions

1. 访问你的 GitHub 仓库：https://github.com/lxxwq19-star/local-image-search2
2. 点击 **Settings** 标签
3. 左侧菜单找到 **Actions** → **General**
4. 确保 **Allow all actions and reusable workflows** 被选中
5. 点击 **Save**

### 步骤 3: 触发构建

GitHub Actions 会在以下情况自动触发：
- 推送到 `main` 或 `master` 分支
- 创建以 `v` 开头的标签（如 `v1.0.0`）
- 手动触发（在 Actions 页面点击 **Run workflow**）

**手动触发**：
1. 访问仓库的 **Actions** 标签
2. 选择 **Build macOS App** 工作流
3. 点击 **Run workflow** 按钮
4. 选择分支，点击确认

---

## 🔧 配置代码签名（推荐）

如果要在别人的 Mac 上运行，必须签名。

### 选项 1: 使用 Apple Developer 证书（推荐）

**需要**：
- Apple Developer 账号（$99/年）
- 创建 Developer ID 证书
- 配置 GitHub Secrets

**步骤**：

1. **创建证书**（在 Mac 上）：
   ```bash
   # 申请 Developer ID Certificate
   # 访问 https://developer.apple.com/
   ```

2. **导出证书为 .p12 文件**

3. **添加 GitHub Secrets**：
   - 访问仓库 **Settings** → **Secrets and variables** → **Actions**
   - 添加以下 Secrets：
     - `APPLE_CERTIFICATE`: .p12 证书文件（Base64 编码）
     - `APPLE_CERTIFICATE_PASSWORD`: 证书密码
     - `APPLE_SIGNING_IDENTITY`: 签名身份（如 `Developer ID Application: Your Name (TEAMID)`）
     - `APPLE_ID`: Apple ID 邮箱
     - `APPLE_PASSWORD`: App-specific password

4. **更新工作流文件**，在 `Build Tauri app` 步骤前添加签名配置

---

### 选项 2: 构建未签名版本（测试用）

如果只是为了自己测试，可以构建未签名版本：

1. **修改工作流文件**，在 `Build Tauri app` 步骤添加环境变量：
   ```yaml
   - name: Build Tauri app
     run: |
       cd src-tauri
       cargo tauri build --no-bundle
   ```

2. **用户需要手动绕过 Gatekeeper**：
   ```bash
   # 在终端运行（替换路径）
   xattr -cr /Applications/LocalImageSearch.app
   codesign --force --deep --sign - /Applications/LocalImageSearch.app
   ```

---

## 📥 下载构建产物

构建完成后：

1. 访问仓库的 **Actions** 标签
2. 点击最近的构建记录
3. 在页面底部找到 **Artifacts** 区域
4. 下载 `LocalImageSearch-macos-<run_number>.zip`
5. 解压后得到 `.dmg` 安装器或 `.app` 应用

---

## 🛠️ 工作流说明

### 构建环境
- **Runner**: `macos-latest`（GitHub 托管的 macOS 虚拟机）
- **Xcode**: 预装在 runner 中
- **Rust**: 通过 `dtolnay/rust-toolchain` 安装
- **Node.js**: v20
- **Python**: v3.10

### 构建步骤
1. 检出代码
2. 安装 Node.js 和前端依赖
3. 安装 Python 和依赖（**注意**：torch 不打包，用户需自行安装）
4. 安装 Rust 和编译 Tauri
5. 构建前端（npm run build）
6. 构建 Tauri 应用（cargo tauri build）
7. 上传 `.dmg` 或 `.app` 作为 Artifact

### 关于 Python 依赖
**重要**：为了减小应用体积，工作流中**不打包** PyTorch（约 2.5 GB）。

**用户需要**：
1. 安装 Python 3.10
2. 运行 `setup_mac.sh`（需要创建）安装依赖
3. 或首次运行时自动安装

---

## 📝 需要创建的文件

### 1. macOS 安装脚本 (`setup_mac.sh`)

创建 `setup_mac.sh` 供 Mac 用户安装 Python 依赖：

```bash
#!/bin/bash
echo "Local Image Search - macOS Setup"
echo "================================"

# Check Python
if ! command -v python3 &> /dev/null; then
    echo "[ERROR] Python 3 not found"
    echo "Please install Python 3.10 from https://www.python.org/downloads/macos/"
    exit 1
fi

echo "[OK] Python $(python3 --version)"

# Install PyTorch (CPU version for macOS)
echo "[1/2] Installing PyTorch..."
pip3 install torch torchvision --index-url https://download.pytorch.org/whl/cpu

# Install other dependencies
echo "[2/2] Installing other dependencies..."
pip3 install transformers\<5.0.0 Pillow numpy onnxruntime tokenizers huggingface-hub safetensors

echo.
echo "[SUCCESS] Setup complete!"
echo "You can now run LocalImageSearch.app"
```

---

## 🐛 常见问题

### Q1: 构建失败，提示 "torch not found"
**A**: 工作流中没有安装 PyTorch（为了减小体积）。用户需要在自己的 Mac 上运行 `setup_mac.sh` 安装。

### Q2: 下载的 .app 无法打开（"无法验证开发者"）
**A**: 未签名。参考上面的"选项 2"手动签名，或配置代码签名。

### Q3: 构建超时
**A**: macOS runner 有 6 小时超时限制。如果超时：
- 减少 Python 依赖（不安装可选包）
- 使用 pip cache
- 分步构建（先构建前端，再构建后端）

### Q4: .dmg 找不到
**A**: Tauri 可能没有生成 .dmg。检查工作流日志，查看 `src-tauri/target/release/bundle/` 目录。

---

## 📋 完整部署清单

- [ ] 代码推送到 GitHub
- [ ] 启用 GitHub Actions
- [ ] 创建 `setup_mac.sh` 安装脚本
- [ ] （可选）配置代码签名 Secrets
- [ ] 推送代码触发构建
- [ ] 下载并测试构建产物
- [ ] 创建 GitHub Release（如果是正式版本）

---

## 🔗 参考链接

- [Tauri 2.0 文档](https://v2.tauri.app/)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Apple Developer 文档](https://developer.apple.com/documentation/)
- [Tauri code signing 指南](https://v2.tauri.app/plugin/code-signing/)

---

## 📝 更新日志

- **2026-07-01**: 初始版本
