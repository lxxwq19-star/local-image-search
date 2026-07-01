#!/bin/bash
# Local Image Search 2 - macOS Setup Script
# This script installs Python dependencies for macOS

set -e

echo "=========================================="
echo "  Local Image Search 2 - macOS Setup"
echo "=========================================="
echo ""

# ----------------------------------------------------------
# Step 1: Check Python
# ----------------------------------------------------------
echo "[STEP 1] Checking Python..."

if ! command -v python3 &> /dev/null; then
    echo "[ERROR] Python 3 not found!"
    echo ""
    echo "Please install Python 3.10 from:"
    echo "  https://www.python.org/downloads/macos/"
    echo ""
    echo "Or use Homebrew:"
    echo "  brew install python@3.10"
    echo ""
    exit 1
fi

PYTHON_VERSION=$(python3 --version 2>&1 | awk '{print $2}')
echo "[OK] Python $PYTHON_VERSION detected"
echo ""

# ----------------------------------------------------------
# Step 2: Configure pip mirrors (for China users)
# ----------------------------------------------------------
echo "[STEP 2] Configuring pip mirrors (optional)..."
echo ""

read -p "Are you in China? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    python3 -m pip config set global.index-url https://pypi.tuna.tsinghua.edu.cn/simple
    python3 -m pip config set global.trusted-host pypi.tuna.tsinghua.edu.cn
    export HF_ENDPOINT=https://hf-mirror.com
    echo "[OK] Pip mirrors configured (Tsinghua)"
else
    echo "[INFO] Using default PyPI mirrors"
fi
echo ""

# ----------------------------------------------------------
# Step 3: Install PyTorch (CPU version for macOS)
# ----------------------------------------------------------
echo "[STEP 3] Installing PyTorch (CPU version)..."
echo "  Download size: ~200 MB"
echo ""

python3 -m pip install torch torchvision --index-url https://download.pytorch.org/whl/cpu

if [ $? -eq 0 ]; then
    echo "[OK] PyTorch installed successfully!"
else
    echo "[ERROR] PyTorch installation failed!"
    exit 1
fi
echo ""

# ----------------------------------------------------------
# Step 4: Install other dependencies
# ----------------------------------------------------------
echo "[STEP 4] Installing other dependencies..."
echo ""

python3 -m pip install "transformers>=4.30.0,<5.0.0" Pillow numpy onnxruntime tokenizers huggingface-hub safetensors

if [ $? -eq 0 ]; then
    echo "[OK] All dependencies installed!"
else
    echo "[ERROR] Some dependencies failed to install!"
    exit 1
fi
echo ""

# ----------------------------------------------------------
# Step 5: Verify installation
# ----------------------------------------------------------
echo "[STEP 5] Verifying installation..."
echo ""

python3 << EOF
import sys
print("Python version:", sys.version)
print()

packages = [
    ("torch", "PyTorch"),
    ("transformers", "Transformers"),
    ("PIL", "Pillow"),
    ("numpy", "NumPy"),
]

for module, name in packages:
    try:
        mod = __import__(module)
        version = getattr(mod, "__version__", "unknown")
        print(f"[OK] {name}: {version}")
    except ImportError:
        print(f"[FAIL] {name}: NOT INSTALLED")
        sys.exit(1)

print()
print("All dependencies installed successfully!")
EOF

if [ $? -eq 0 ]; then
    echo ""
    echo "=========================================="
    echo "  [SUCCESS] Setup complete!"
    echo "=========================================="
    echo ""
    echo "  You can now run LocalImageSearch.app"
    echo ""
else
    echo ""
    echo "[ERROR] Verification failed!"
    exit 1
fi

echo ""
read -p "Press any key to exit..."
