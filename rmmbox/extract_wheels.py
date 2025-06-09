#!/usr/bin/env python3
"""
简化构建脚本：使用已有的轮子文件，提取 .pyd/.so 和 .pyi 文件到正确位置
"""

import os
import shutil
import zipfile
import glob
from pathlib import Path


def extract_files_from_wheel(wheel_path, lib_dir, src_dir):
    """从轮子文件中提取 .pyd/.so 和 .pyi 文件"""
    print(f"Extracting files from {wheel_path}")
    
    with zipfile.ZipFile(wheel_path, 'r') as zip_ref:
        for file_info in zip_ref.filelist:
            filename = file_info.filename
            
            # 提取 .pyd/.so 文件到 lib 目录
            if filename.endswith('.pyd') or filename.endswith('.so'):
                zip_ref.extract(file_info, lib_dir)
                
                # 移动到正确位置 (移除可能的子目录结构)
                src_path = os.path.join(lib_dir, filename)
                dst_path = os.path.join(lib_dir, os.path.basename(filename))
                
                if src_path != dst_path:
                    os.makedirs(os.path.dirname(dst_path), exist_ok=True)
                    shutil.move(src_path, dst_path)
                    
                    # 清理空目录
                    try:
                        parent_dir = os.path.dirname(src_path)
                        while parent_dir != lib_dir:
                            os.rmdir(parent_dir)
                            parent_dir = os.path.dirname(parent_dir)
                    except OSError:
                        pass  # 目录不为空或其他错误，忽略
                
                print(f"Extracted binary: {os.path.basename(filename)}")
            
            # 提取 .pyi 存根文件到 src 目录
            elif filename.endswith('.pyi'):
                zip_ref.extract(file_info, src_dir)
                
                # 移动到正确位置 (移除可能的子目录结构)
                src_path = os.path.join(src_dir, filename)
                dst_path = os.path.join(src_dir, os.path.basename(filename))
                
                if src_path != dst_path:
                    os.makedirs(os.path.dirname(dst_path), exist_ok=True)
                    shutil.move(src_path, dst_path)
                    
                    # 清理空目录
                    try:
                        parent_dir = os.path.dirname(src_path)
                        while parent_dir != src_dir:
                            os.rmdir(parent_dir)
                            parent_dir = os.path.dirname(parent_dir)
                    except OSError:
                        pass  # 目录不为空或其他错误，忽略
                
                print(f"Extracted stub: {os.path.basename(filename)}")


def process_module(module_path):
    """处理单个模块的轮子文件"""
    module_name = os.path.basename(module_path)
    print(f"\n=== Processing module: {module_name} ===")
    
    # 查找轮子文件
    wheel_dirs = [
        os.path.join(module_path, "dist"),
        os.path.join(module_path, "target", "wheels")
    ]
    
    wheel_files = []
    for wheel_dir in wheel_dirs:
        if os.path.exists(wheel_dir):
            wheel_files.extend(glob.glob(os.path.join(wheel_dir, "*.whl")))
    
    if not wheel_files:
        print(f"No wheel files found for {module_name}")
        return True
    
    # 设置目标目录
    script_dir = os.path.dirname(os.path.abspath(__file__))
    target_lib_dir = os.path.join(os.path.dirname(script_dir), "src", "pyrmm", "usr", "lib")
    target_src_dir = os.path.join(os.path.dirname(script_dir), "src", "pyrmm")
    
    os.makedirs(target_lib_dir, exist_ok=True)
    os.makedirs(target_src_dir, exist_ok=True)
    
    # 处理每个轮子文件
    for wheel_file in wheel_files:
        print(f"Processing: {os.path.basename(wheel_file)}")
        extract_files_from_wheel(wheel_file, target_lib_dir, target_src_dir)
    
    return True


def main():
    """主函数"""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    rmmbox_dir = script_dir
    
    print(f"Processing modules in: {rmmbox_dir}")
    
    success_count = 0
    total_count = 0
    
    # 遍历所有子目录
    for item in os.listdir(rmmbox_dir):
        item_path = os.path.join(rmmbox_dir, item)
        
        # 跳过文件和特殊目录  
        if not os.path.isdir(item_path) or item.startswith('.') or item == '__pycache__':
            continue
        
        total_count += 1
        
        if process_module(item_path):
            success_count += 1
        else:
            print(f"Failed to process module: {item}")
    
    print(f"\n=== Processing Summary ===")
    print(f"Successfully processed: {success_count}/{total_count} modules")
    
    # 显示结果
    target_lib_dir = os.path.join(os.path.dirname(script_dir), "src", "pyrmm", "usr", "lib")
    target_src_dir = os.path.join(os.path.dirname(script_dir), "src", "pyrmm")
    
    print(f"\nExtracted files:")
    if os.path.exists(target_lib_dir):
        lib_files = os.listdir(target_lib_dir)
        if lib_files:
            print(f"Binary files in {target_lib_dir}:")
            for f in lib_files:
                if f.endswith(('.pyd', '.so')):
                    print(f"  - {f}")
    
    if os.path.exists(target_src_dir):
        src_files = [f for f in os.listdir(target_src_dir) if f.endswith('.pyi')]
        if src_files:
            print(f"Stub files in {target_src_dir}:")
            for f in src_files:
                print(f"  - {f}")
    
    return 0 if success_count == total_count else 1


if __name__ == "__main__":
    exit(main())
