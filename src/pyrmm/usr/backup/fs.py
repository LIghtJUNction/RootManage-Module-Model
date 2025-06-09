import os
from pathlib import Path
from typing import Literal
import shutil
print("警告！！！ 未加载rust 版本模块！ --- fs.py 模块已加载")
class RmmFileSystemMeta(type):
    @property
    def ROOT(cls) -> Path:
        """Return the root path of the RMM file system."""
        return Path(os.getenv('RMM_ROOT', Path().home() / "data" / "adb" / ".rmm" )).resolve()
    
    @property
    def TMP(cls) -> Path:
        """Return the temporary directory path of the RMM file system."""
        return cls.ROOT / 'tmp'
    
    @property
    def CACHE(cls) -> Path:
        """Return the cache directory path of the RMM file system."""
        return cls.ROOT / 'cache'

    @property
    def DATA(cls) -> Path:
        """Return the data directory path of the RMM file system."""
        return cls.ROOT / 'data'

    @property
    def META(cls) -> Path:
        """Return the metadata directory path of the RMM file system."""
        return cls.ROOT / 'meta.toml'

    def __getattr__(cls, item: str):
        """Get an attribute from the RMM file system."""
        with open(cls.META, 'r') as f:
            import toml
            meta = toml.load(f)
        if item in meta["projects"]:
            return Path(meta["projects"][item])
        raise AttributeError(f"'{cls.__name__}' object has no attribute '{item}'!!!")
class RmmFileSystem(metaclass=RmmFileSystemMeta):
    """RMM File System class"""
    @classmethod
    def init(cls):
        """Ensure that all necessary directories exist."""
        cls.ROOT.mkdir(parents=True, exist_ok=True)
        cls.TMP.mkdir(parents=True, exist_ok=True)
        cls.CACHE.mkdir(parents=True, exist_ok=True)
        cls.DATA.mkdir(parents=True, exist_ok=True)
        cls.META.touch(exist_ok=True)
    @classmethod
    def rm(cls, dir: Literal["ROOT","DATA","TMP","CACHE","META"] = "TMP"):
        """Remove the RMM DIRS."""
        match dir:
            case "ROOT":
                shutil.rmtree(cls.ROOT, ignore_errors=True)
            case "DATA":
                shutil.rmtree(cls.DATA, ignore_errors=True)
            case "TMP":
                shutil.rmtree(cls.TMP, ignore_errors=True)
            case "CACHE":
                shutil.rmtree(cls.CACHE, ignore_errors=True)
            case "META":
                cls.META.unlink(missing_ok=True)
            case _:
                raise ValueError(f"Unknown directory: {dir}")
