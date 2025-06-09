from abc import abstractmethod
from typing import Any
import click

class RmmBaseMeta(type):
    """基础元类，提供配置管理的通用功能"""
    
    @property
    @abstractmethod
    def META(cls) -> dict[str, Any]:
        """获取元数据配置 - 由子类实现"""
        pass
    
    @abstractmethod
    def get_config_key(cls) -> str:
        """获取配置键名 - 由子类实现"""
        pass
    
    @abstractmethod
    def get_reserved_key(cls) -> str:
        """获取保留关键字 - 由子类实现"""
        pass
    
    @abstractmethod
    def get_item_config(cls, item_name: str) -> dict[str, Any]:
        """获取指定项目的配置 - 由子类实现"""
        pass
    
    def __getattr__(cls, item: str):
        """获取配置项"""
        try:
            return cls.get_item_config(item)
        except KeyError:
            raise AttributeError(f"配置项 '{item}' 未找到。")
    
    def __setattr__(cls, name: str, value: Any) -> None:
        """设置配置项"""
        # 如果是内部属性，直接设置
        if name.startswith('_') or hasattr(type, name):
            super().__setattr__(name, value)
            return
        
        # 配置项属性，由子类处理具体的保存逻辑
        cls._set_item_config(name, value)
    
    def __delattr__(cls, name: str) -> None:
        """删除配置项"""
        # 如果是内部属性，直接删除
        if name.startswith('_'):
            return super().__delattr__(name)
        
        # 配置项属性，由子类处理具体的删除逻辑
        cls._delete_item_config(name)
    
    @abstractmethod
    def _set_item_config(cls, name: str, value: Any) -> None:
        """设置配置项的具体实现 - 由子类实现"""
        pass
    
    @abstractmethod
    def _delete_item_config(cls, name: str) -> None:
        """删除配置项的具体实现 - 由子类实现"""
        pass


class RmmBase(metaclass=RmmBaseMeta):
    """RMM 基础类，提供通用的配置管理功能"""
    
    @classmethod
    @abstractmethod
    def is_valid_item(cls, item_name: str) -> bool:
        """检查指定项目是否有效 - 由子类实现"""
        pass
    
    @classmethod
    @abstractmethod
    def get_sync_prompt(cls, item_name: str) -> str:
        """获取同步提示信息 - 由子类实现"""
        pass
    
    @classmethod
    def sync_item(cls, item_name: str):
        """同步配置项"""
        if not cls.is_valid_item(item_name):
            prompt = cls.get_sync_prompt(item_name)
            if click.confirm(prompt, default=True):
                delattr(cls, item_name)
