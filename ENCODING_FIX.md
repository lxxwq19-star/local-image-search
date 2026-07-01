# 乱码问题解决方案

## 问题：setup.bat 显示乱码

**原因**：Windows 控制台（cmd.exe）对 UTF-8 支持不好，中文会显示为乱码

**解决方案**：已创建无中文的英文版本 bat 文件

---

## 更新后的文件

### 1. setup.bat（英文版）
- 无中文，不会乱码
- 双击运行即可

### 2. check_env.bat（英文版）
- 无中文，不会乱码
- 双击运行检查环境

### 3. setup.ps1（PowerShell 版）
- 支持中文显示
- 右键选择"使用 PowerShell 运行"

### 4. check_env.ps1（PowerShell 版）
- 支持中文显示
- 右键选择"使用 PowerShell 运行"

---

## 推荐使用顺序

### 方法 1：使用 .bat 文件（推荐）
1. 双击 `setup.bat` - 安装依赖
2. 双击 `check_env.bat` - 检查环境
3. 双击 `LocalImageSearch.exe` - 运行

### 方法 2：使用 PowerShell（支持中文）
1. 右键 `setup.ps1` → "使用 PowerShell 运行"
2. 右键 `check_env.ps1` → "使用 PowerShell 运行"
3. 双击 `LocalImageSearch.exe` - 运行

---

## 如果仍然乱码

### 临时解决：修改控制台编码
```bat
chcp 65001
setup.bat
```

### 永久解决：修改注册表
1. 打开注册表编辑器（`regedit`）
2. 导航到：`HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Command Processor`
3. 新建字符串值：`Autorun`
4. 设置数值数据：`chcp 65001 >nul`

---

## 其他可能的问题

### 1. PowerShell 执行策略限制
如果 PowerShell 脚本无法运行，执行：
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### 2. Python 未添加到 PATH
如果提示"python 不是内部或外部命令"：
1. 重新安装 Python
2. 勾选 "Add Python to PATH"
3. 或手动添加 Python 安装目录到系统 PATH

### 3. 依赖安装失败
如果 `pip install` 失败，手动安装：
```bat
pip install torch torchvision transformers accelerate onnxruntime numpy Pillow tokenizers huggingface-hub safetensors
```
