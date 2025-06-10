"""
RMM CLI 模块

这个模块导入并暴露 Rust 实现的 CLI 功能，提供高性能的命令行界面。
"""

import sys
from pathlib import Path
from typing import Optional, List, Any, Callable
print("TEST")
def _find_rust_module() -> Optional[Any]:
    """查找并导入 Rust 编译的模块
    
    Returns:
        Rust 模块对象，如果找不到则返回 None
    """
    current_dir = Path(__file__).parent
    
    # 首先尝试直接导入（如果在 site-packages 中）
    try:
        import rmmcore  # type: ignore
        return rmmcore
    except ImportError:
        pass
    
    # 获取当前 Python 版本信息
    import platform
    python_version = f"cp{sys.version_info.major}{sys.version_info.minor}"
    platform_tag = platform.machine().lower()
    if platform_tag == "amd64":
        platform_tag = "win_amd64"
    elif platform_tag == "x86":
        platform_tag = "win32"
    
    # 构建可能的文件名
    possible_names = [
        f"rmmcore.{python_version}-{platform_tag}.pyd",  # Windows specific
        f"rmmcore.{python_version}-{platform_tag}.so",   # Linux/macOS specific
        "rmmcore.pyd",  # Generic Windows
        "rmmcore.so",   # Generic Unix
        "rmmcore.dylib"  # macOS
    ]
    
    for module_name in possible_names:
        module_file = current_dir / module_name
        if module_file.exists():
            # 动态导入模块
            import importlib.util
            spec = importlib.util.spec_from_file_location("rmmcore", module_file)
            if spec and spec.loader:
                module = importlib.util.module_from_spec(spec)
                sys.modules["rmmcore"] = module
                spec.loader.exec_module(module)
                return module
    
    return None

# 尝试导入 Rust 模块
_rust_module = _find_rust_module()

if _rust_module is None:
    raise ImportError(
        "无法找到 Rust CLI 扩展。请确保已正确编译了 Rust 扩展模块。\n"
        "运行 'maturin develop' 来构建扩展。"
    )

# 导出 CLI 函数
cli: Callable[[Optional[List[str]]], None] = _rust_module.cli

__all__ = ['cli']
