# filepath: c:\Users\light\Documents\GitHub\RootManageModuleModel\src\pyrmm\cli\rmmcore.pyi
"""
Type stubs for the Rust CLI module (rmmcore)

This file provides type information for the Rust-implemented CLI functions.
"""

from typing import Optional, List

def cli(args: Optional[List[str]] = None) -> None:
    """
    Main CLI function implemented in Rust.
    
    Args:
        args: Optional list of command line arguments.
              If None, arguments will be read from sys.argv.
    
    Raises:
        SystemExit: On CLI errors or when help/version is requested.
    
    Examples:
        >>> cli(["--help"])
        >>> cli(["init", "my-project"])
        >>> cli(["init", "--help"])
    """
    ...
