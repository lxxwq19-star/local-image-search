# -*- mode: python ; coding: utf-8 -*-

# PyInstaller spec for clip_server.py
# Creates a standalone executable with Python + all dependencies bundled
# Usage: pyinstaller pyinstaller.spec --onefile

from PyInstaller.building.build_main import Analysis, PYZ, EXE, COLLECT
from PyInstaller import log as logging

logging.logger.setLevel(logging.WARN)

block_cipher = None

# Main analysis
a = Analysis(
    ['clip_server.py'],
    pathex=[],
    binaries=[],
    datas=[
        # Include any data files needed at runtime
        # (models are downloaded at runtime, not bundled)
    ],
    hiddenimports=[
        'torch',
        'torchvision',
        'transformers',
        'numpy',
        'PIL',
        'Pillow',
        'onnxruntime',
        'tokenizers',
        'huggingface_hub',
        'safetensors',
        'requests',
        'urllib3',
        'json',
        'socket',
        'struct',
        'threading',
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[
        # Exclude unnecessary large packages to reduce size
        'torch.cuda',      # CPU-only build: exclude CUDA libs
        'torch._C._cuda',
    ],
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.zipfiles,
    a.datas,
    [],
    name='clip_server',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=True,         # Keep console for server stdout/stderr
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,     # Auto-detect (x86_64 or arm64 on macOS)
    codesign_identity=None,
    entitlements_file=None,
    icon=None,
)
