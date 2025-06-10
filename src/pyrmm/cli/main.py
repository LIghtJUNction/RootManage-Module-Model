#!/usr/bin/env python3
"""
RMM CLI 入口点

这个脚本提供了 rmm 命令的主入口点，正确处理命令行参数。
"""

import sys
from typing import List, Optional

def main(args: Optional[List[str]] = None) -> None:
    """
    主入口点函数
    
    Args:
        args: 可选的命令行参数列表，如果为 None 则从 sys.argv 获取
    """
    # 如果没有提供参数，从 sys.argv 获取，但排除脚本名
    if args is None:
        args = sys.argv[1:]
      # 导入 CLI 函数
    from pyrmm.cli import cli
    
    # 调用 Rust 实现的 CLI 函数，传递过滤后的参数
    cli(args)

if __name__ == "__main__":
    main()
