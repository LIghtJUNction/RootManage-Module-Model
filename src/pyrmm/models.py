"""
Type definitions for RMM project configuration and data structures.

This file defines the type structure for RMM projects, configuration files,
and core data models used throughout the system.
"""

from typing import Dict, List, Optional, TypedDict, Literal
from pathlib import Path


class Author(TypedDict):
    """Author information for RMM projects."""
    name: str
    email: str


class Dependency(TypedDict):
    """Project dependency specification."""
    name: str
    version: str


class Script(TypedDict):
    """Build script configuration."""
    name: str
    command: str


class Urls(TypedDict):
    """Project URLs configuration."""
    github: str


class GitInfo(TypedDict):
    """Git repository information."""
    git_root: str
    remote_url: str
    username: str
    repo_name: str
    is_in_repo_root: bool


class BuildConfig(TypedDict, total=False):
    """Build configuration options."""
    prebuild: Optional[str]
    build: Optional[str]
    postbuild: Optional[str]


class ProjectConfig(TypedDict):
    """RMM project configuration (rmmproject.toml)."""
    id: str
    name: str
    description: Optional[str]
    requires_rmm: str
    versionCode: str
    updateJson: str
    readme: str
    changelog: str
    license: str
    dependencies: List[Dependency]
    authors: List[Author]
    scripts: List[Script]
    urls: Urls
    build: Optional[BuildConfig]
    git: Optional[GitInfo]


class RmakeBuildConfig(TypedDict, total=False):
    """Rmake build configuration."""
    prebuild: Optional[List[str]]
    build: Optional[List[str]]
    postbuild: Optional[List[str]]
    exclude: Optional[List[str]]
    include: Optional[List[str]]


class RmakePackageConfig(TypedDict, total=False):
    """Rmake package configuration."""
    zip_name: Optional[str]
    tar_name: Optional[str]
    compression: Optional[str]


class RmakeConfig(TypedDict):
    """Rmake configuration (.rmmp/Rmake.toml)."""
    build: RmakeBuildConfig
    package: Optional[RmakePackageConfig]
    scripts: Optional[Dict[str, str]]


class RmmConfig(TypedDict):
    """Main RMM configuration (meta.toml)."""
    email: str
    username: str
    version: str
    projects: Dict[str, str]  # project_name -> project_path
    github_token: Optional[str]  # From environment variable


# CLI Command Types
ProjectType = Literal["basic", "library"]
InitOptions = TypedDict('InitOptions', {
    'project_path': str,
    'yes': bool,
    'basic': bool,
    'lib': bool,
    'ravd': bool,
})

BuildOptions = TypedDict('BuildOptions', {
    'project_name': Optional[str],
    'path': Optional[str],
    'output': Optional[str],
    'clean': bool,
    'verbose': bool,
    'debug': bool,
})

# Environment Configuration
class RmmPaths(TypedDict):
    """RMM directory structure."""
    root: Path          # RMM_ROOT or ~/data/adb/.rmm/
    cache: Path         # cache/
    tmp: Path           # tmp/
    data: Path          # data/
    bin: Path           # bin/
    meta_file: Path     # meta.toml


# Module.prop structure
class ModuleProp(TypedDict):
    """Magisk/APatch/KernelSU module.prop structure."""
    id: str
    name: str
    version: str
    versionCode: str
    author: str
    description: str
    updateJson: str
