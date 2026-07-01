import struct
import os

# 动态获取图标输出路径（脚本在 src-tauri 目录下）
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
ICO_PATH = os.path.join(SCRIPT_DIR, "icons", "icon.ico")

# Minimal valid ICO: 1 entry, 16x16, 32bpp
header = struct.pack('<HHH', 0, 1, 1)  # ICO, 1 image
# ICO directory entry: width, height, colors, reserved, planes, bpp, size, offset
w, h = 16, 16
bpp = 32
pixel_data_size = w * h * 4
mask_size = ((w + 31) // 32) * 4 * h
bmp_info_size = 40
image_size = bmp_info_size + pixel_data_size + mask_size
entry = struct.pack('<BBBBHHII', w, h, 0, 0, 1, bpp, image_size, 22)

# BMP info header (BITMAPINFOHEADER)
bmp_header = struct.pack('<IiiHHIIiiII', 40, w, h*2, 1, bpp, 0, pixel_data_size + mask_size, 0, 0, 0, 0)

# All-blue pixels (BGRA)
pixels = bytes([0xFF, 0x80, 0x00, 0xFF] * (w * h))

# AND mask (all zeros = fully opaque)
mask = bytes(mask_size)

with open(ICO_PATH, 'wb') as f:
    f.write(header + entry + bmp_header + pixels + mask)

print(f"icon.ico created at {ICO_PATH}")
