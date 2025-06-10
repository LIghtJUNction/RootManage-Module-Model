#!/usr/bin/env python3
"""
RMM 项目构建脚本

该脚本负责：
1. 构建 Rust CLI 扩展
2. 将编译结果放置到正确位置
3. 构建主 Python 包
"""

import os
import sys
import subprocess
import shutil
import platform
from pathlib import Path

def run_command(cmd, cwd=None, check=True):
    """运行命令并处理错误"""
    print(f"运行命令: {' '.join(cmd) if isinstance(cmd, list) else cmd}")
    if cwd:
        print(f"工作目录: {cwd}")
    
    result = subprocess.run(
        cmd,
        cwd=cwd,
        shell=isinstance(cmd, str),
        capture_output=True,
        text=True,
        check=False
    )
    
    if result.stdout:
        print(result.stdout)
    if result.stderr:
        print(result.stderr, file=sys.stderr)
    
    if check and result.returncode != 0:
        sys.exit(result.returncode)
    
    return result

def get_platform_extension():
    """获取当前平台的动态库扩展名"""
    system = platform.system()
    if system == "Windows":
        return ".pyd"
    elif system == "Darwin":
        return ".dylib"
    else:
        return ".so"

def build_rust_extension():
    """构建 Rust CLI 扩展"""
    print("🔨 构建 Rust CLI 扩展...")
    
    cli_dir = Path("src/pyrmm/cli")
    if not cli_dir.exists():
        print("❌ CLI 目录不存在")
        sys.exit(1)
    
    # 构建 Rust 扩展
    run_command(["maturin", "build", "--release"], cwd=cli_dir)
    
    # 查找构建输出
    target_dir = cli_dir / "target" / "release"
    extension = get_platform_extension()
    
    # 查找编译产物
    built_files = []
    if extension == ".pyd":
        # Windows
        built_files = list(target_dir.glob("*.pyd"))
    elif extension == ".dylib":
        # macOS
        built_files = list(target_dir.glob("libpyrmm_cli*.dylib"))
    else:
        # Linux
        built_files = list(target_dir.glob("libpyrmm_cli*.so"))
    
    if not built_files:
        print(f"❌ 未找到编译产物 (*{extension})")
        return False
    
    # 复制到目标位置
    target_file = cli_dir / f"pyrmm_cli{extension}"
    shutil.copy2(built_files[0], target_file)
    print(f"✅ 复制 {built_files[0]} -> {target_file}")
    
    return True

def build_python_package():
    """构建 Python 包"""
    print("📦 构建 Python 包...")
    
    # 清理旧的构建文件
    dist_dir = Path("dist")
    if dist_dir.exists():
        shutil.rmtree(dist_dir)
    
    # 构建包
    run_command([sys.executable, "-m", "build"])
    
    print("✅ Python 包构建完成")

def develop_mode():
    """开发模式构建"""
    print("🔧 开发模式构建...")
    
    cli_dir = Path("src/pyrmm/cli")
    
    # 使用 maturin develop 进行开发构建
    run_command(["maturin", "develop"], cwd=cli_dir)
    
    print("✅ 开发模式构建完成")

def clean():
    """清理构建文件"""
    print("🧹 清理构建文件...")
    
    # 清理目录列表
    clean_dirs = [
        "dist",
        "build",
        "src/pyrmm.egg-info",
        "src/pyrmm/cli/target",
    ]
    
    # 清理文件模式
    clean_patterns = [
        "src/pyrmm/cli/*.pyd",
        "src/pyrmm/cli/*.so",
        "src/pyrmm/cli/*.dylib",
    ]
    
    for dir_path in clean_dirs:
        path = Path(dir_path)
        if path.exists():
            shutil.rmtree(path)
            print(f"删除目录: {path}")
    
    for pattern in clean_patterns:
        for file_path in Path(".").glob(pattern):
            file_path.unlink()
            print(f"删除文件: {file_path}")
    
    print("✅ 清理完成")

def main():
    """主函数"""
    import argparse
    
    parser = argparse.ArgumentParser(description="RMM 项目构建脚本")
    parser.add_argument("command", choices=["build", "develop", "clean"], 
                       help="构建命令")
    parser.add_argument("--rust-only", action="store_true",
                       help="只构建 Rust 扩展")
    
    args = parser.parse_args()
    
    if args.command == "clean":
        clean()
    elif args.command == "develop":
        develop_mode()
    elif args.command == "build":
        if args.rust_only:
            build_rust_extension()
        else:
            # 完整构建流程
            if build_rust_extension():
                build_python_package()
            else:
                print("❌ Rust 扩展构建失败")
                sys.exit(1)

if __name__ == "__main__":
    main()
