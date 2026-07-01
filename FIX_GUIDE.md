# 新电脑部署修复说明

## 🔧 问题根因

新电脑上 `pip install` 安装了 **`transformers` 5.12.1**（预发布版），其 API 有变更：
- `model.get_image_features()` 返回 `BaseModelOutputWithPooling` 对象，不是 tensor
- 导致 `.float()` 调用崩溃 → 向量索引无法进行

---

## ✅ 修复方案

### 修复 1：固定 `transformers` 版本
`requirements.txt` 已更新为：
```
transformers>=4.30.0,<5.0.0
```
避免安装不稳定的 5.x 预发布版。

### 修复 2：增强 `clip_server.py` 兼容性
所有编码函数已更新，兼容所有 `transformers` 版本：
- 优先使用 `model.get_image_features()` （返回 tensor）
- 如果返回的是对象，自动提取 `.image_embeds` / `.pooler_output`
- 安全调用 `.float()`，避免 AttributeError

---

## 📦 修复包内容

**文件**：`local-image-search2_fix_20260630.tar`（70 KB）

**包含文件**：
| 文件 | 说明 |
|------|------|
| `clip_server.py` | 修复后的 Python 后端 |
| `requirements.txt` | 固定 transformers<5.0.0 |
| `setup.bat` | 依赖安装脚本（更新） |
| `check_env.bat` | 环境检查（英文版，无乱码） |
| `test_exe.bat` | 测试脚本（英文版，无乱码） |
| `DEPLOYMENT_GUIDE.md` | 完整部署指南 |

---

## 🚀 在新电脑上应用修复

### 方法 1：覆盖文件（推荐）

1. 将 `local-image-search2_fix_20260630.tar` 解压到 `D:\local-image-search2\`
2. 覆盖现有文件
3. **以管理员身份**打开命令提示符，执行：
   ```bat
   cd /d D:\local-image-search2
   python -m pip install --upgrade -r requirements.txt
   ```
4. 等待 `transformers` 降级完成（约 1 分钟）
5. 双击 `LocalImageSearch.exe` 重新启动

---

### 方法 2：重新运行 `setup.bat`

1. 覆盖文件（同上）
2. 双击 `setup.bat`
3. 脚本会自动升级/降级依赖到正确版本
4. 双击 `LocalImageSearch.exe`

---

## 🔍 验证修复是否成功

### 1. 检查 `transformers` 版本
```bat
python -c "import transformers; print(transformers.__version__)"
```
**应显示**：`4.x.x`（不是 `5.x.x`）

### 2. 手动启动 Python 后端测试
```bat
cd /d D:\local-image-search2
python clip_server.py
```
**应显示**：
```
[SigLIP2] ✅ Model loaded on GPU
[CLIP-L/14] ✅ Model loaded on GPU
[SERVER] TCP server listening on 127.0.0.1:8765
```

### 3. 测试向量索引
- 打开应用
- 添加图片文件夹
- 点击"开始索引"
- **进度条应该开始增长**（之前是卡住不动的）

---

## 📋 如果还是无法索引

**查看日志**：
```
D:\local-image-search2\clip_server.log
D:\local-image-search2\clip_server_err.log
```

**手动测试编码函数**：
```bat
cd /d D:\local-image-search2
python -c "
from clip_server import encode_image_siglip2, encode_text_clip_large
import numpy as np
vec = encode_image_siglip2('test.jpg')
print('Image vector dim:', len(vec))
vec2 = encode_text_clip_large('test')
print('Text vector dim:', len(vec2))
"
```

把错误信息发给我，我继续排查。

---

## 📝 完整部署步骤（新电脑）

如果没有备份，需要从头部署：

1. **安装 Python 3.12**：https://www.python.org/downloads/
   - 勾选 "Add Python to PATH"

2. **解压代码**到 `D:\local-image-search2\`

3. **安装依赖**：
   ```bat
   cd /d D:\local-image-search2
   setup.bat
   ```

4. **下载模型**（如果没有 `models/` 目录）：
   ```bat
   python download_siglip2.py
   ```
   > 需要下载 ~7GB 模型文件，确保网络稳定

5. **运行应用**：
   ```bat
   LocalImageSearch.exe
   ```

---

## 🎯 预期效果

修复后，应用应该可以：
- ✅ 正常启动（无错误提示）
- ✅ 索引进度条正常增长
- ✅ "以图搜图"返回正确结果（SigLIP2）
- ✅ "语义搜图"返回正确结果（CLIP-L/14）

---

**修复完成时间**：2026-06-30 10:32  
**修复包位置**：`D:\FileManager\其他\local-image-search2_fix_20260630.tar`
