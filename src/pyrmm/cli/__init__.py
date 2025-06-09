def __getattr__(name):
    if name == "cli":
        from .__main__ import cli
        return cli
    elif name == "rmmr":
        # 直接导入编译好的 Rust 模块
        try:
            import rmmr as rust_module
            return rust_module
        except ImportError:
            # 如果导入失败，回退到 Python 包装器
            from .rmmr import cli
            return cli
    else:
        raise AttributeError(f"module '{__name__}' has no attribute '{name}'")
    