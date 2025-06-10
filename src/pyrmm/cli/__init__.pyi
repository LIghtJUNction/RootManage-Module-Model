"""
Type stubs for pyrmm.cli module

This module provides the main CLI interface for RMM (Root Module Manager).
"""

from typing import Optional, List

def cli(args: Optional[List[str]] = None) -> None:
    """
    Main CLI entry point for RMM.
    
    This function provides a high-performance CLI interface for Magisk/APatch/KernelSU 
    module development, implemented in Rust for optimal performance.
    
    Args:
        args: Optional list of command line arguments. 
              If None, arguments will be read from sys.argv.
              
    Commands:
        init: Initialize a new RMM project
        build: Build RMM project (coming soon)
        sync: Sync project dependencies (coming soon) 
        test: Test RMM modules (coming soon)
        publish: Publish to GitHub releases (coming soon)
        check: Check GitHub connectivity and project status (coming soon)
    
    Global Options:
        -v, --verbose: Enable verbose output
        -q, --quiet: Quiet mode, only show errors
        -h, --help: Show help message
        --version: Show version information
    
    Environment Variables:
        RMM_ROOT: RMM metadata storage location (default: ~/data/adb/.rmm/)
        GITHUB_ACCESS_TOKEN: GitHub access token for authentication
    
    Raises:
        SystemExit: On CLI errors, help display, or version display
        ImportError: If the Rust extension module is not properly built
    
    Examples:
        >>> from pyrmm.cli import cli
        >>> cli(["--help"])
        >>> cli(["init", "my-project"])
        >>> cli(["init", "--basic", "project-name"])
    """
    ...
