import time

# 记录开始时间
_start_time = time.perf_counter()

# 直接导入 cli 并导出
from pyrmm.cli.__main__ import cli

# 计算加载时间
_load_time = time.perf_counter() - _start_time
print(f"CLI加载时间: {_load_time}秒")

__all__ = ['cli']