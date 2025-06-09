#!/usr/bin/env python3
"""
RMMR CLI wrapper - Rust implementation entry point
"""

def cli():
    """Main entry point for rmmr command"""
    try:
        # 导入 Rust 模块
        import sys
        import os
        
        # 获取当前目录路径
        current_dir = os.path.dirname(os.path.abspath(__file__))
        
        # 动态导入 Rust 扩展模块
        import importlib.util
        
        # 尝试导入不同版本的 .pyd 文件
        pyd_files = [
            f"rmmr.cp{sys.version_info.major}{sys.version_info.minor}-win_amd64.pyd"
        ]
        
        rust_module = None
        for pyd_file in pyd_files:
            pyd_path = os.path.join(current_dir, pyd_file)
            if os.path.exists(pyd_path):
                spec = importlib.util.spec_from_file_location("rmmr", pyd_path)
                if spec and spec.loader:
                    rust_module = importlib.util.module_from_spec(spec)
                    spec.loader.exec_module(rust_module)
                    break
        
        if rust_module is None:
            print(f"❌ 错误: 找不到对应 Python {sys.version_info.major}.{sys.version_info.minor} 的 Rust 模块")
            print(f"🔍 在目录 '{current_dir}' 中查找的文件:")
            for pyd_file in pyd_files:
                print(f"   - {pyd_file}")
            print("💡 请确保已正确编译 Rust 扩展模块")
            sys.exit(1)
        
        # 直接传递命令行参数给 Rust 模块
        # 构建干净的参数列表，排除 Python 包装器相关的参数
        clean_args = ["rmmr"]
        
        # 获取真实的命令行参数
        import sys
        original_argv = sys.argv[:]
        
        # 只保留实际的命令行参数
        for arg in original_argv[1:]:
            # 跳过包含路径信息的参数
            if not (arg.endswith('.exe') or 
                   'python' in arg.lower() or 
                   'scripts' in arg.lower() or
                   arg == '-c' or
                   'import' in arg or
                   'pyrmm' in arg):
                clean_args.append(arg)
        
        # 调用 Rust 模块的 cli 函数，传递清理后的参数
        rust_module.cli(clean_args)
        
    except ImportError as e:
        print(f"❌ 导入错误: {e}")
        print("💡 请确保 Rust 扩展模块已正确编译和安装")
        sys.exit(1)
    except SystemExit:
        # 正常的系统退出，不需要额外处理
        raise
    except Exception as e:
        print(f"❌ 运行错误: {e}")
        sys.exit(1)

if __name__ == "__main__":
    cli()
