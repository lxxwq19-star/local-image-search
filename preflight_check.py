#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
预部署检查工具 - 在部署前检查系统是否满足要求
"""

import sys
import os
import platform

def print_header():
    print("=" * 70)
    print("  本地图片搜索 - 预部署检查工具")
    print("=" * 70)

def check_os():
    print("\n[1/5] 操作系统")
    os_name = platform.system()
    os_version = platform.version()
    print(f"  系统: {os_name}")
    print(f"  版本: {os_version}")
    
    if os_name != "Windows":
        print("  ⚠️  警告: 仅测试过 Windows 10/11")
        return False
    print("  ✅ 操作系统符合要求")
    return True

def check_python():
    print("\n[2/5] Python 版本")
    version = sys.version_info
    print(f"  当前版本: {version.major}.{version.minor}.{version.micro}")
    
    if version.major < 3 or (version.major == 3 and version.minor < 10):
        print("  ❌ Python 版本过低，需要 3.10+")
        print("     下载: https://www.python.org/downloads/")
        return False
    print("  ✅ Python 版本符合要求")
    return True

def check_memory():
    print("\n[3/5] 内存")
    try:
        import ctypes
        kernel32 = ctypes.windll.kernel32
        mem = kernel32.GlobalMemoryStatusEx()
        total_mb = mem.dwTotalPhys / 1024 / 1024
        print(f"  总内存: {total_mb:.0f} MB ({total_mb/1024:.1f} GB)")
        
        if total_mb < 16 * 1024:
            print("  ⚠️  警告: 内存不足 16GB，性能可能受影响")
            return False
        print("  ✅ 内存符合要求")
        return True
    except:
        print("  ⚠️  无法检测内存")
        return True

def check_disk():
    print("\n[4/5] 硬盘空间")
    try:
        import shutil
        total, used, free = shutil.disk_usage(".")
        free_gb = free / 1024**3
        print(f"  可用空间: {free_gb:.1f} GB")
        
        if free_gb < 10:
            print("  ❌ 可用空间不足 10GB")
            return False
        print("  ✅ 硬盘空间充足")
        return True
    except:
        print("  ⚠️  无法检测硬盘空间")
        return True

def check_gpu():
    print("\n[5/5] GPU")
    try:
        import torch
        if torch.cuda.is_available():
            gpu_name = torch.cuda.get_device_name(0)
            gpu_mem = torch.cuda.get_device_properties(0).total_memory / 1024**3
            print(f"  GPU: {gpu_name}")
            print(f"  显存: {gpu_mem:.1f} GB")
            if gpu_mem < 6:
                print("  ⚠️  显存不足 6GB，可能无法加载双模型")
                return False
            print("  ✅ GPU 符合要求")
            return True
        else:
            print("  ⚠️  未检测到 NVIDIA GPU")
            print("     将使用 CPU 模式（速度慢 5-10 倍）")
            return True
    except:
        print("  ⚠️  无法检测 GPU（可能未安装 PyTorch）")
        return True

def main():
    print_header()
    
    results = []
    results.append(("操作系统", check_os()))
    results.append(("Python", check_python()))
    results.append(("内存", check_memory()))
    results.append(("硬盘", check_disk()))
    results.append(("GPU", check_gpu()))
    
    print("\n" + "=" * 70)
    print("  检查结果")
    print("=" * 70)
    for name, ok in results:
        status = "✅ 通过" if ok else "❌ 失败"
        print(f"  {name}: {status}")
    
    all_passed = all(ok for _, ok in results)
    print("\n" + "=" * 70)
    if all_passed:
        print("  ✅ 系统符合要求，可以部署")
        print("\n  下一步:")
        print("    1. 运行 setup.bat 安装依赖")
        print("    2. 运行 check_env.bat 检查环境")
        print("    3. 运行 LocalImageSearch.exe")
    else:
        print("  ❌ 系统不符合要求，请升级硬件或软件")
    print("=" * 70)
    
    return 0 if all_passed else 1

if __name__ == "__main__":
    sys.exit(main())
