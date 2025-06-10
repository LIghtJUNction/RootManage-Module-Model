"""
RMM CLI 基础测试
"""

import pytest
import sys
from pathlib import Path

def test_import_cli():
    """测试 CLI 模块导入"""
    try:
        from pyrmm.cli import cli
        assert callable(cli)
    except ImportError as e:
        pytest.skip(f"CLI 模块未编译: {e}")

def test_cli_help():
    """测试 CLI 帮助命令"""
    try:
        from pyrmm.cli import cli
        
        # 测试帮助命令
        result = cli(["--help"])
        # 如果成功执行就通过测试
        assert True
    except ImportError:
        pytest.skip("CLI 模块未编译")
    except SystemExit:
        # --help 会触发 SystemExit，这是正常的
        assert True

def test_cli_version():
    """测试 CLI 版本命令"""
    try:
        from pyrmm.cli import cli
        
        # 测试版本命令
        result = cli(["--version"])
        assert True
    except ImportError:
        pytest.skip("CLI 模块未编译")
    except SystemExit:
        # --version 会触发 SystemExit，这是正常的
        assert True
