# -*- mode: python ; coding: utf-8 -*-
#
# PyInstaller spec for clip_server.py
# Creates an onedir bundle (folder mode) with Python + all dependencies
# onedir is MORE RELIABLE than onefile for large packages like torch
#
# Usage: pyinstaller pyinstaller.spec
# Output: dist/clip_server/clip_server (executable + deps in same folder)

from PyInstaller.building.build_main import Analysis, PYZ, EXE, COLLECT

block_cipher = None

# On macOS: do NOT exclude torch.cuda — macOS uses MPS (Metal Performance Shaders),
# which is a different backend from CUDA. Excluding torch.cuda can cause
# dynamic library loading issues with MPS.

a = Analysis(
    ['clip_server.py'],
    pathex=[],
    binaries=[],
    datas=[],
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
        'concurrent.futures',
        'safetensors.torch',
        'transformers.models.clip',
        'transformers.models.siglip2',
        'transformers.models.eva',
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[
        # Only exclude very large unnecessary packages
        # torch.cuda is NOT excluded — macOS MPS needs parts of it
    ],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
    optimize=0,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

# EXE without the binaries (onedir mode)
exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,   # ← onedir mode: binaries go in COLLECT, not EXE
    name='clip_server',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=True,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
    icon=None,
)

# COLLECT: puts exe + all binaries + datas into a folder
coll = COLLECT(
    exe,
    a.binaries,
    a.zipfiles,
    a.datas,
    strip=False,
    upx=True,
    upx_exclude=[],
    name='clip_server',   # Output folder: dist/clip_server/
)
