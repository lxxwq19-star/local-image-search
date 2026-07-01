import os
import sys
from huggingface_hub import snapshot_download

# 设置镜像站（加速国内下载）
os.environ['HF_ENDPOINT'] = 'https://hf-mirror.com'

# 动态获取脚本所在目录，避免硬编码路径
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
MODELS_DIR = os.path.join(SCRIPT_DIR, "models", "siglip2-large")

print("=" * 60)
print("开始下载 SigLIP2-Large 模型...")
print(f"模型: google/siglip2-large-patch16-256")
print(f"保存到: {MODELS_DIR}/")
print("=" * 60)

try:
    snapshot_download(
        repo_id='google/siglip2-large-patch16-256',
        local_dir=MODELS_DIR,
        local_dir_use_symlinks=False,  # 不创建符号链接，直接下载文件
        resume_download=True,  # 断点续传
        max_workers=4,  # 多线程下载
    )
    print("\n" + "=" * 60)
    print("✅ 下载完成！")
    print("=" * 60)
except Exception as e:
    print(f"\n❌ 下载失败: {e}")
    sys.exit(1)
