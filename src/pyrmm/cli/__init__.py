"""
RMM CLI 模块

这个模块导入并暴露 Rust 实现的 CLI 功能，提供高性能的命令行界面。
"""

import sys
import argparse
from collections.abc import Callable

def _find_rust_module():
    """查找并导入 Rust 编译的模块"""
    try:
        # 直接导入，让Python解释器处理版本匹配
        import rmmcore
        return rmmcore
    except ImportError:
        pass
    
    # 如果直接导入失败，尝试从当前目录导入
    try:
        from . import rmmcore
        return rmmcore
    except ImportError:
        return None

# 尝试导入 Rust 模块
rmmcore = _find_rust_module()

if rmmcore is None:
    raise ImportError(
        "无法找到 Rust CLI 扩展。请确保已正确编译了 Rust 扩展模块。\n"
        "运行 'maturin develop' 来构建扩展。"
    )


# 包装 CLI 函数以处理参数
def cli(args=None):
    """
    CLI 入口函数
    
    Args:
        args: 命令行参数列表，如果为 None 则从 sys.argv 获取
    """
    # 如果没有提供参数，从 sys.argv 获取，但排除脚本名
    if args is None:
        args = sys.argv[1:]
    
    # 所有命令都调用 Rust 实现的 CLI 函数
    return rmmcore.cli(args)

__all__ = ['cli']