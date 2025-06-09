def __getattr__(name):
    if name == "cli":
        from .__main__ import cli
        return cli
    elif name == "rmmr":
        # 直接导入编译好的 Rust 模块 (.pyd 文件在当前目录)
        try:
            from . import rmmr as rust_module
            return rust_module
        except ImportError:
            raise ImportError("无法导入 rmmr Rust 模块，请确保 rmmr.*.pyd 文件存在")
    else:
        raise AttributeError(f"module '{__name__}' has no attribute '{name}'")
    