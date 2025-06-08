from __future__ import annotations
from collections.abc import Iterable
from pathlib import Path
from .fs import RmmFileSystem
from ...__about__ import __version__
import toml


class ConfigMeta(type):
    _cache: dict[str, str | dict[str, str]] | None = None
    _lm : float = 0
    """文件上次修改时间"""
    _cm : float = 0
    """现在读取到的文件修改时间"""
    @classmethod
    def __save__(cls, meta: dict[str, str | dict[str, str]]):
        """Save the metadata to the metadata directory."""
        with open(RmmFileSystem.META, "w") as f:
            toml.dump(meta, f)
    @property
    def META(cls) -> dict[str, str | dict[str, str]]:
        """Return the metadata directory path of the Pyrmm application."""
        cls._cm = Path(RmmFileSystem.META).stat().st_mtime
        if cls._cache is not None and cls._lm == cls._cm:
            return cls._cache
        with open(RmmFileSystem.META, "r") as f:
            meta: dict[str,str | dict[str,str]] = toml.load(f)
        cls._lm = cls._cm
        return meta
    def __getattr__(cls, item: str) -> str | dict[str, str]:
        """Get an attribute from the metadata. 仅当读取不存在的值时"""
        meta = cls.META
        if item in meta:
            return meta[item]
        raise AttributeError(f"找不到配置项： '{item}' 请检查：{RmmFileSystem.META} , 当前配置项：{meta.keys()}")
    def __setattr__(cls, item: str, value: str | dict[str, str]):
        """Set an attribute in the metadata."""
        # 如果是内部属性，直接设置
        if item.startswith('_'):
            super().__setattr__(item, value)
            return
        # 配置项属性，保存到文件
        meta = cls.META.copy()  # 创建副本避免修改缓存
        meta[item] = value
        cls.__save__(meta)
    def __delattr__(cls, name: str) -> None:
        """Delete an attribute from the metadata."""
        # 如果是内部属性，直接删除
        if name.startswith('_'):
            return super().__delattr__(name)
        # 配置项属性，从文件中删除
        meta = cls.META.copy()  # 创建副本避免修改缓存
        if name in meta:
            del meta[name]
            cls.__save__(meta)
            cls._cache = None
            cls._lm = 0  # 重置上次修改时间
        else:
            raise AttributeError(f"配置项 '{name}' 不存在")

    def __dir__(cls) -> Iterable[str]:
        """Return the list of attributes in the metadata."""
        meta = cls.META

        return list(i for i in meta.keys() if not i.startswith('_') and i != "init")
    

class Config(metaclass = ConfigMeta):
    """
    Configuration class for the Pyrmm application.
    """
    @classmethod    
    def init(cls):
        """Initialize the configuration by ensuring the metadata directory exists."""
        RmmFileSystem.init()
        meta = cls.META.copy()  # 创建副本避免修改缓存
        meta.update(
            {
                "username": "username",
                "email": "email",
                "version": __version__,
                "projects": {}
            }
        )
        cls.__save__(meta)


