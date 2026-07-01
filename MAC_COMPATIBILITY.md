# macOS 兼容性说明

## 支持的 Mac 机型

| 机型 | 架构 | GPU | 支持状态 | 性能 |
|------|------|-----|---------|------|
| MacBook Pro M3/M4 | ARM64 | Apple Silicon (MPS) | ✅ 完全支持 | ⭐⭐⭐⭐⭐ 最快 |
| MacBook Pro M2 | ARM64 | Apple Silicon (MPS) | ✅ 完全支持 | ⭐⭐⭐⭐⭐ 最快 |
| MacBook Pro M1 | ARM64 | Apple Silicon (MPS) | ✅ 完全支持 | ⭐⭐⭐⭐ 很快 |
| MacBook Air M1/M2/M3 | ARM64 | Apple Silicon (MPS) | ✅ 完全支持 | ⭐⭐⭐⭐ 很快 |
| Mac Mini M1/M2 | ARM64 | Apple Silicon (MPS) | ✅ 完全支持 | ⭐⭐⭐⭐⭐ 最快 |
| Mac Studio M1/M2/M3 Ultra | ARM64 | Apple Silicon (MPS) | ✅ 完全支持 | ⭐⭐⭐⭐⭐ 极快 |
| Mac Pro M2 Ultra | ARM64 | Apple Silicon (MPS) | ✅ 完全支持 | ⭐⭐⭐⭐⭐ 极快 |
| Intel Mac (2019 及以前) | x86_64 | AMD/NVIDIA/Intel HD | ❌ 不支持 | - |

---

## 为什么不支持 Intel Mac？

1. **Apple 已停产 Intel Mac**：2023 年起所有 Mac 都用 Apple Silicon
2. **Intel Mac GPU 加速有限**：MPS 在 Intel Mac 上支持不完整，性能提升不大
3. **构建复杂度**：需要交叉编译 Universal 二进制，构建时间翻倍
4. **用户基数小**：2026 年 Intel Mac 用户占比已低于 10%

---

## 如何判断你的 Mac 是否支持

点击左上角 苹果图标 → **关于本机**：

- 如果看到 **"Apple M1/M2/M3/M4"** → ✅ 支持
- 如果看到 **"Intel Core i5/i7/i9"** → ❌ 不支持

---

## Intel Mac 用户怎么办？

### 选项 1：使用 Bootcamp 装 Windows（不推荐）
- 需要重启切换系统
- 很麻烦

### 选项 2：使用 Parallels 虚拟机（可行）
- 在虚拟机里跑 Windows 版
- 性能损失约 20-30%

### 选项 3：等未来版本（可能）
- 如果有很多用户反馈，我会考虑构建 Universal 版本
- 请在 GitHub Issues 里提需求

---

## 技术细节

### MPS (Metal Performance Shaders)
- macOS 12.3+ 内置
- PyTorch 1.12+ 支持
- 性能：约 CUDA 的 70-80%（M1 Max 对比 RTX 3060）
- 内存：统一内存架构，模型和图片共享内存

### 为什么不用 ONNX Runtime (Mac)？
- ONNX Runtime 对 MPS 支持不完整
- PyTorch MPS 后端更稳定
- 性能差距不大

---

## 反馈

如果你有 Intel Mac 且希望支持，请在 GitHub 提 Issue：
https://github.com/lxxwq19-star/local-image-search/issues
