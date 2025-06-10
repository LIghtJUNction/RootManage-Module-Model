"""
Type stubs for pyrmm package

Root Module Manager (RMM) - A high-performance toolkit for Magisk/APatch/KernelSU module development.
"""

__version__: str

# CLI module
from . import cli

__all__ = ["cli", "__version__"]
