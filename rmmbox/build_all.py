#!/usr/bin/env python3
"""
构建脚本：遍历 rmmbox 下所有子文件夹，构建轮子并提取 .pyd 文件到 src/pyrmm/usr/lib/
"""

import os
import subprocess
import sys
import shutil
import zipfile
import glob
from pathlib import Path


def run_command(cmd, cwd=None):
    """运行命令并处理错误"""
    print(f"Running: {cmd} in {cwd or os.getcwd()}")
    result = subprocess.run(cmd, shell=True, cwd=cwd, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"Error running command: {cmd}")
        print(f"stdout: {result.stdout}")
        print(f"stderr: {result.stderr}")
        return False
    print(f"Success: {result.stdout}")
    return True


def extract_pyd_from_wheel(wheel_path, target_dir):
    """从轮子文件中提取 .pyd 文件"""
    print(f"Extracting .pyd files from {wheel_path}")
    
    with zipfile.ZipFile(wheel_path, 'r') as zip_ref:
        for file_info in zip_ref.filelist:
            if file_info.filename.endswith('.pyd'):
                # 提取 .pyd 文件
                zip_ref.extract(file_info, target_dir)
                
                # 移动到正确位置 (移除可能的子目录结构)
                src_path = os.path.join(target_dir, file_info.filename)
                dst_path = os.path.join(target_dir, os.path.basename(file_info.filename))
                
                if src_path != dst_path:
                    os.makedirs(os.path.dirname(dst_path), exist_ok=True)
                    shutil.move(src_path, dst_path)
                    
                    # 清理空目录
                    try:
                        parent_dir = os.path.dirname(src_path)
                        while parent_dir != target_dir:
                            os.rmdir(parent_dir)
                            parent_dir = os.path.dirname(parent_dir)
                    except OSError:
                        pass  # 目录不为空或其他错误，忽略
                
                print(f"Extracted: {os.path.basename(file_info.filename)}")


def build_module(module_path):
    """构建单个模块"""
    module_name = os.path.basename(module_path)
    print(f"\n=== Building module: {module_name} ===")
    
    # 检查是否有 pyproject.toml
    pyproject_path = os.path.join(module_path, "pyproject.toml")
    if not os.path.exists(pyproject_path):
        print(f"Skipping {module_name}: no pyproject.toml found")
        return True
    
    # 创建虚拟环境
    if not run_command("uv venv", cwd=module_path):
        return False
    
    # 同步依赖
    if not run_command("uv sync", cwd=module_path):
        return False
    
    # 构建轮子
    if not run_command("uv build", cwd=module_path):
        return False
    
    # 查找生成的轮子文件
    dist_dir = os.path.join(module_path, "dist")
    if not os.path.exists(dist_dir):
        print(f"Warning: dist directory not found for {module_name}")
        return True
    
    wheel_files = glob.glob(os.path.join(dist_dir, "*.whl"))
    if not wheel_files:
        print(f"Warning: no wheel files found for {module_name}")
        return True
    
    # 提取 .pyd 文件
    target_lib_dir = os.path.join(os.path.dirname(os.path.dirname(module_path)), "src", "pyrmm", "usr", "lib")
    os.makedirs(target_lib_dir, exist_ok=True)
    
    for wheel_file in wheel_files:
        extract_pyd_from_wheel(wheel_file, target_lib_dir)
    
    return True


def main():
    """主函数"""
    # 获取 rmmbox 目录路径
    script_dir = os.path.dirname(os.path.abspath(__file__))
    rmmbox_dir = script_dir
    
    print(f"Building all modules in: {rmmbox_dir}")
    
    # 遍历所有子目录
    success_count = 0
    total_count = 0
    
    for item in os.listdir(rmmbox_dir):
        item_path = os.path.join(rmmbox_dir, item)
        
        # 跳过文件和特殊目录
        if not os.path.isdir(item_path) or item.startswith('.') or item == '__pycache__':
            continue
        
        total_count += 1
        
        if build_module(item_path):
            success_count += 1
        else:
            print(f"Failed to build module: {item}")
    
    print(f"\n=== Build Summary ===")
    print(f"Successfully built: {success_count}/{total_count} modules")
    
    if success_count == total_count:
        print("\n=== Committing changes ===")
        # 提交更改
        repo_root = os.path.dirname(rmmbox_dir)
        run_command("git add .", cwd=repo_root)
        run_command('git commit -m "Auto-build: Update .pyd files"', cwd=repo_root)
        print("Changes committed successfully!")
        return 0
    else:
        print("Some modules failed to build. Please check the errors above.")
        return 1


if __name__ == "__main__":
    sys.exit(main())
