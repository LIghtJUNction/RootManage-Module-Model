"""
RmmCore 类型存根文件
为 Rust 实现的 RmmCore 类提供 Python 类型提示
"""

from __future__ import annotations
from typing import Any


class RmmCore:
    """
    RMM (Root Module Manager) 核心管理类
    
    提供项目配置文件管理、Git 集成、缓存机制等功能
    """
    
    def __init__(self) -> None:
        """创建新的 RmmCore 实例"""
        ...
    
    def get_rmm_root(self) -> str:
        """
        获取 RMM_ROOT 路径
        
        Returns:
            RMM 根目录路径字符串
        """
        ...
    
    def get_meta_config(self) -> dict[str, Any]:
        """
        获取 meta.toml 配置内容
        
        Returns:
            包含邮箱、用户名、版本和项目列表的字典
            
        Raises:
            RuntimeError: 当配置文件不存在或解析失败时
        """
        ...    
    def update_meta_config(self, email: str, username: str, version: str, projects: dict[str, str]) -> None:
        """
        更新 meta.toml 配置文件
        
        Args:
            email: 用户邮箱
            username: 用户名
            version: 版本号
            projects: 项目名到路径的映射字典
            
        Raises:
            RuntimeError: 当写入失败时
        """
        ...
    
    def update_meta_config_from_dict(self, config: dict[str, Any]) -> None:
        """
        从字典更新 meta.toml 配置文件
        
        Args:
            config: 包含配置信息的字典，需包含 email, username, version, projects 字段
            
        Raises:
            RuntimeError: 当写入失败时
        """
        ...
    
    def get_meta_value(self, key: str) -> Any | None:
        """
        获取 meta.toml 中指定键的值
        
        Args:
            key: 要获取的配置键名
            
        Returns:
            配置值，如果键不存在则返回 None
        """
        ...
    
    def get_project_path(self, project_name: str) -> str | None:
        """
        根据项目名获取项目路径
        
        Args:
            project_name: 项目名称
            
        Returns:
            项目路径字符串，如果项目不存在则返回 None
        """
        ...
    
    def check_projects_validity(self) -> dict[str, bool]:
        """
        检查所有项目的有效性
        
        Returns:
            项目名到有效性状态的映射字典
        """
        ...
    
    def scan_projects(self, scan_path: str, max_depth: int | None = None) -> list[dict[str, Any]]:
        """
        扫描指定路径下的项目
        
        Args:
            scan_path: 要扫描的路径
            max_depth: 最大扫描深度，None 表示无限制
            
        Returns:
            包含项目信息的字典列表，每个字典包含 name, path, is_valid 等字段
        """
        ...
    
    def sync_projects(self, scan_paths: list[str], max_depth: int | None = None) -> None:
        """
        同步项目列表到 meta.toml
        
        Args:
            scan_paths: 要扫描的路径列表
            max_depth: 最大扫描深度
            
        Raises:
            RuntimeError: 当同步失败时
        """
        ...
    
    def get_project_config(self, project_path: str) -> dict[str, Any]:
        """
        读取项目的 rmmproject.toml 配置
        
        Args:
            project_path: 项目路径
            
        Returns:
            项目配置字典
            
        Raises:
            RuntimeError: 当配置文件不存在或解析失败时
        """
        ...
    
    def update_project_config(self, project_path: str, config: dict[str, Any]) -> None:
        """
        更新项目的 rmmproject.toml 配置
        
        Args:
            project_path: 项目路径
            config: 项目配置字典
            
        Raises:
            RuntimeError: 当写入失败时
        """
        ...
    
    def get_module_prop(self, project_path: str) -> dict[str, Any]:
        """
        读取项目的 module.prop 文件
        
        Args:
            project_path: 项目路径
            
        Returns:
            模块属性字典
            
        Raises:
            RuntimeError: 当文件不存在或解析失败时
        """
        ...
    
    def update_module_prop(self, project_path: str, prop: dict[str, Any]) -> None:
        """
        更新项目的 module.prop 文件
        
        Args:
            project_path: 项目路径
            prop: 模块属性字典
            
        Raises:
            RuntimeError: 当写入失败时
        """
        ...
    
    def get_rmake_config(self, project_path: str) -> dict[str, Any]:
        """
        读取项目的 Rmake.toml 配置
        
        Args:
            project_path: 项目路径
            
        Returns:
            构建配置字典
            
        Raises:
            RuntimeError: 当配置文件不存在或解析失败时
        """
        ...
    
    def update_rmake_config(self, project_path: str, config: dict[str, Any]) -> None:
        """
        更新项目的 Rmake.toml 配置
        
        Args:
            project_path: 项目路径
            config: 构建配置字典
            
        Raises:
            RuntimeError: 当写入失败时
        """
        ...
    
    def get_git_info(self, path: str) -> dict[str, Any]:
        """
        获取指定路径的 Git 信息
        
        Args:
            path: 要分析的路径
            
        Returns:
            包含 Git 信息的字典，包含以下字段：
            - repo_root: Git 仓库根目录
            - relative_path: 相对于仓库根目录的路径
            - branch: 当前分支名
            - remote_url: 远程仓库 URL
            - has_uncommitted_changes: 是否有未提交的更改
            - last_commit_hash: 最后一次提交的哈希值
            - last_commit_message: 最后一次提交的消息
            
        Raises:
            RuntimeError: 当路径不在 Git 仓库中时
        """
        ...
    
    def remove_project_from_meta(self, project_name: str) -> bool:
        """
        从 meta.toml 中移除指定项目
        
        Args:
            project_name: 要移除的项目名
            
        Returns:
            如果项目存在并被移除则返回 True，否则返回 False
        """
        ...
    
    def remove_projects_from_meta(self, project_names: list[str]) -> list[str]:
        """
        从 meta.toml 中移除多个项目
        
        Args:
            project_names: 要移除的项目名列表
            
        Returns:
            实际被移除的项目名列表（JSON 字符串格式）
        """
        ...
    
    def remove_invalid_projects(self) -> list[str]:
        """
        移除所有无效的项目
        
        Returns:
            被移除的无效项目名列表（JSON 字符串格式）
        """
        ...

    def get_cache_stats(self) -> dict[str, bool | int]:
        """
        获取缓存统计信息
        
        Returns:
            包含缓存统计的字典：
            - meta_cached: meta 配置是否已缓存
            - project_count: 已缓存的项目数量
        """
        ...
    
    def clear_all_cache(self) -> None:
        """清理所有缓存"""
        ...
    
    def cleanup_expired_cache(self) -> None:
        """清理过期的缓存项"""
        ...
    
    # 工具方法
    def create_default_meta(self, email: str, username: str, version: str) -> dict[str, Any]:
        """
        创建默认的 meta.toml 配置
        
        Args:
            email: 用户邮箱
            username: 用户名
            version: 版本号
            
        Returns:
            默认的 meta 配置字典
        """
        ...
    
    def create_default_project(self, project_id: str, username: str, email: str) -> dict[str, Any]:
        """
        创建默认的项目配置
        
        Args:
            project_id: 项目 ID
            username: 用户名
            email: 用户邮箱
            
        Returns:
            默认的项目配置字典
        """
        ...
    
    def create_default_module_prop(self, module_id: str, username: str) -> dict[str, Any]:
        """
        创建默认的 module.prop 配置
        
        Args:
            module_id: 模块 ID
            username: 用户名
            
        Returns:
            默认的模块属性字典
        """
        ...
    
    def create_default_rmake(self) -> dict[str, Any]:
        """
        创建默认的 Rmake.toml 配置
        
        Returns:
            默认的构建配置字典        """
        ...



# 导出的类型
__all__ = [
    'RmmCore',
]
