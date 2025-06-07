from __future__ import annotations
from pathlib import Path
from typing import Any, Literal, cast
import click
from pyrmm.__about__ import __version__
import os
import toml
import shutil

class Config:
    """
    Configuration class for the Pyrmm application.
    """
    #region RMMC
    _instance: Config | None = None

    projects: dict[str, str]

    DEFAULTS : dict[str,str] = {
        'username': 'Your Name',
        'email': 'dev@rmmp.com',
        'rmmroot': str(Path.home() / "data" / "adb" / '.rmm'),        
    }
    @property
    def root(self) -> Path:
        """
        Get the root path of the RMM configuration.
        """
        return self.rmmroot

    @property
    def _data(self) -> Path:
        """
        Get the data path of the RMM configuration.
        """
        return self.rmmroot / 'data'

    @property
    def _tmp(self) -> Path:
        """
        Get the temporary path of the RMM configuration.
        """
        return self.rmmroot / 'tmp'

    @property
    def _cache(self) -> Path:
        """
        Get the cache path of the RMM configuration.
        """
        return self.rmmroot / 'cache'




    def __new__(cls,rmmroot: str | Path | None = None) -> Config:
        if cls._instance is None:
            cls._instance = super().__new__(cls)
        return cls._instance

    def __init__(self,rmmroot: str | Path | None = None) -> None:
        if not rmmroot:
            self.rmmroot: Path = Path(os.getenv('RMM_ROOT', str(Path.home() / "data" / "adb" / '.rmm')))
        self.load()  # Load configuration from file

    def load(self) -> dict[str, Any]:
        """
        Load the configuration from a file.
        """
        # 确保目录存在
        self.rmmroot.mkdir(parents=True, exist_ok=True)
        self._data.mkdir(parents=True, exist_ok=True)
        self._cache.mkdir(parents=True, exist_ok=True)
        self._tmp.mkdir(parents=True, exist_ok=True)

        rmmenv: Path = self.rmmroot / 'rmm.env'

        if rmmenv.exists():
            try:
                envdict: dict[str, Any] = toml.load(rmmenv)
                object.__setattr__(self, '_envdict', envdict)
                for key, value in self._envdict.items():
                    # 处理 Path 对象的加载
                    if key == 'rmmroot':
                        value = Path(value)
                    # 直接设置属性，避免触发保存
                    object.__setattr__(self, key, value)
            except Exception as e:
                click.echo(f"Error loading configuration file: {e}")
                # 如果加载失败，使用默认配置
                envdict = {
                    'rmmroot': str(self.rmmroot),
                    'rmmdata': str(self._data),
                    'rmmcache': str(self._cache),
                    'rmmtmp': str(self._tmp),
                    'version': __version__,
                }
                object.__setattr__(self, '_envdict', envdict)
        else:
            # 初始化配置文件
            envdict = {
                'rmmroot': str(self.rmmroot),
                'rmmdata': str(self._data),
                'rmmcache': str(self._cache),
                'rmmtmp': str(self._tmp),
                'version': __version__,
            }
            object.__setattr__(self, '_envdict', envdict)

            rmmenv.touch(exist_ok=True)  # 创建文件
            
            with open(rmmenv, 'w', encoding='utf-8') as f:
                toml.dump(self._envdict, f)
            
            # 设置初始属性
            for key, value in self._envdict.items():
                if key == 'rmmroot':
                    value = Path(value)
                object.__setattr__(self, key, value)

        object.__setattr__(self, '_initialized', True)
        return self._envdict

    def save(self) -> None:
        """
        Save the current configuration to a file.
        """
        # 确保 rmmroot 是 Path 对象
        if isinstance(self.rmmroot, str):
            self.rmmroot = Path(self.rmmroot)
        
        self.rmmroot.mkdir(parents=True, exist_ok=True)
        rmmenv: Path = self.rmmroot / 'rmm.env'
        
        # 获取所有非私有属性
        for key, value in self.__dict__.items():
            if not key.startswith('_'):
                # 处理 Path 对象
                if isinstance(value, Path):
                    self._envdict[key] = str(value)
                else:
                    self._envdict[key] = value

        with open(rmmenv, 'w', encoding='utf-8') as f:
            toml.dump(self._envdict, f)
            
        print("Configuration saved successfully.")

    def __setattr__(self, name: str, value: Any) -> None:
        """
        Set an attribute in the configuration and save it.
        """
        # 允许在初始化期间设置私有属性
        if name.startswith('_'):
            super().__setattr__(name, value)
            return
        
        # 为特定属性提供明确的类型转换
        if name == 'projects':
            if isinstance(value, dict):
                # 确保项目字典类型为 dict[str, str]  
                value = cast(dict[str, str], value)

        elif name == 'username':
            if not isinstance(value, str):
                raise TypeError("Username must be a string.")
            value = value.strip()
        
        elif name == "email":
            if not isinstance(value, str):
                raise TypeError("Email must be a string.")
            value = value.strip()
            if '@' not in value:
                raise ValueError("Invalid email address format.")
            if '.' not in value.split('@')[-1]:
                raise ValueError("Invalid email address format, missing domain.")
            if len(value) < 5:
                raise ValueError("Email address is too short, must be at least 5 characters long.")
        
        # 直接设置属性
        super().__setattr__(name, value)
        # 如果不是私有属性且已初始化，保存配置
        if getattr(self, '_initialized', False):
            print(f"Set configuration: {name}={value}")
            self.save()

    def __getattr__(self, name: str) -> Any:
        """
        Get an attribute from the configuration.
        """
        # 如果是私有属性，直接抛出 AttributeError
        if name.startswith('_'):
            raise AttributeError(f"'{self.__class__.__name__}' object has no attribute '{name}'")
            # 首先检查是否在 _envdict 中
        if hasattr(self, '_envdict') and name in self._envdict:
            value = self._envdict[name]
            # 处理 Path 对象
            if name == 'rmmroot' and isinstance(value, str):
                value = Path(value)
            return value
        
        # 如果是 projects 属性，初始化为空字典而不是返回 RmmProject 实例
        if name == 'projects':
            self.projects = {}
            return self.projects


        if name == "username":
            self.username = "Your Name"
            click.echo("Username not set. Defaulting to 'Your Name'. Please set it using `config.username = 'Your Name'`.")
            return self.username
        if name == "email":
            self.email = "dev@rmmp.com"
            click.echo("Email not set. Defaulting to 'Your Email'. Please set it using `config.email = 'Your Email'`.")
            return self.email
        # 如果没有找到，询问用户是否设置
        if click.confirm(f"Configuration '{name}' not found. Do you want to set it?", abort=False):
            value = click.prompt(f"Please enter the value for '{name}'", type=str)
            setattr(self, name, value)
            return value
        # 如果用户选择不设置，返回 None 或抛出异常
        raise AttributeError(f"Configuration '{name}' not found and user chose not to set it.")

    def __str__(self) -> str:
        """
        String representation of the configuration.
        """
        info = f"[*]RMMROOT: {self.rmmroot}\n"
        info += f"[*]Version: {__version__}\n"
        info += "[*]author: LIghtJUNction\n"
        info += "[*]config:\n"
        for key, value in self._envdict.items():
            if key.startswith('_'):
                continue
            if isinstance(value, Path):
                value = str(value)
            info += f"  {key}: {value}\n"

        return info

    def __repr__(self) -> str:
        """
        Official string representation of the configuration.
        """
        return f"<Config rmmroot={self.rmmroot}, version={__version__}, initialized={getattr(self, '_initialized', False)} \n\n envdict={self._envdict}>"

    def __dir__(self) -> list[str]:
        """
        List all attributes of the configuration.
        """
        return sorted(key for key in self.__dict__.keys() if not key.startswith('_') and key != 'rmmroot' and key != '_envdict')

    def __delattr__(self, name: str) -> None:
        """
        Delete an attribute from the configuration and save it.
        """
        # 检查是否是私有属性或重要属性
        if name.startswith('_'):
            raise AttributeError(f"Cannot delete private attribute '{name}'")
        
        # 检查是否是核心属性
        protected_attrs = {'rmmroot', 'version'}
        if name in protected_attrs:
            if not click.confirm(f"Are you sure you want to delete protected attribute '{name}'?", default=False):
                return
        
        # 检查属性是否存在
        if not hasattr(self, name):
            raise AttributeError(f"'{self.__class__.__name__}' object has no attribute '{name}'")
        
        # 从实例属性中删除
        super().__delattr__(name)
        
        # 从环境字典中删除
        if name in self._envdict:
            del self._envdict[name]
        
        print(f"Deleted configuration: {name}")
        
        # 保存配置
        if getattr(self, '_initialized', False):
            self.save()

    @staticmethod
    def is_default(key: str) -> bool:
        """
        Check if a configuration key is set to its default value.
        """
        if key not in Config.DEFAULTS:
            return False
        
        current_value = getattr(Config._instance, key, None)
        return current_value == Config.DEFAULTS[key]


    @staticmethod
    def has_default(key: str) -> bool:
        """
        Check if the configuration has default values.
        """
        return key in Config.DEFAULTS

    def set_default(self, key: str) -> None:
        """
        Set a configuration key to its default value.
        """
        if key in Config.DEFAULTS:
            setattr(self, key, Config.DEFAULTS[key])
            print(f"Set {key} to default value: {Config.DEFAULTS[key]}")
        else:
            raise KeyError(f"No default value for '{key}' in Config.DEFAULTS")


    @classmethod
    def by_default(cls, key: str) -> Config:
        """
        rmmroot : defaults to Path.home() / "data" / "adb" / '.rmm'
        """
        rmmroot : Path = Path(cls.DEFAULTS.get('rmmroot', str(Path.home() / "data" / "adb" / '.rmm')))
        return cls(rmmroot=rmmroot)
  




class RmmProject:
    """
    Configuration class for RMM projects.
    Uses Config to manage project information.
    """
    #region RMMP
    
    @staticmethod
    def is_rmmp(rpath: str | Path) -> bool:
        """
        Check if a project is a RMM project.
        """
        rpath = Path(rpath).resolve()
        return (rpath / 'rmmproject.toml').exists() and (rpath / '.rmm_version').exists()

    @staticmethod
    def find_rmmp(rpath: str | Path) -> list[tuple[str,Path | str]]:
        """
        Find all RMM projects in the specified directory.
        """
        if isinstance(rpath, str):
            rpath = Path(rpath).resolve()
        if not rpath.exists():
            raise FileNotFoundError(f"Path '{rpath}' does not exist.")

        rmmp_projects: list[tuple[str, Path | str]] = []
        for item in rpath.iterdir():
            if item.is_dir() and RmmProject.is_rmmp(item):
                rmmp_projects.append((item.name, item))
        return rmmp_projects


    @staticmethod
    def sync(scan_path: str | Path | None = None) -> dict[str, str]:
        """
        同步 config.projects 和文件系统中实际存在的 RMM 项目

        Args:
            scan_path: 要扫描的路径，如果为 None 则扫描所有已知项目路径的父目录

        Returns:
            dict: 更新后的项目字典
        """
        config = Config()

        # 如果没有指定扫描路径，则扫描已知项目路径的父目录
        if scan_path is None:
            scan_paths : set[Path] = set()
            for project_name, project_path in config.projects.items():
                if project_name != "last":
                    parent_path = Path(project_path).parent
                    scan_paths.add(parent_path)

            # 如果没有已知项目，扫描当前目录
            if not scan_paths:
                scan_paths = {Path.cwd()}
        else:
            scan_paths = {Path(scan_path).resolve()}

        # 收集所有找到的项目
        found_projects: dict[str,str] = {}
        for scan_path in scan_paths:
            try:
                projects_in_path: list[tuple[str, Path | str]] = RmmProject.find_rmmp(scan_path)
                for project_name, project_path in projects_in_path:
                    found_projects[project_name] = str(project_path)
            except FileNotFoundError:
                click.echo(f"Warning: Scan path '{scan_path}' does not exist, skipping.")
                continue
            
        # 获取当前配置中的项目（排除 "last" 键）
        current_projects = {k: v for k, v in config.projects.items() if k != "last"}

        # 检查配置中的项目是否仍然有效
        invalid_projects: list[str] = []
        for project_name, project_path in current_projects.items():
            if not RmmProject.is_rmmp(project_path):
                invalid_projects.append(project_name)
                click.echo(f"Project '{project_name}' at '{project_path}' is no longer a valid RMM project")

        # 移除无效项目
        for project_name in invalid_projects:
            if click.confirm(f"Remove invalid project '{project_name}' from configuration?", default=True):
                del config.projects[project_name]
                # 如果删除的是 last 项目，需要更新
                if config.projects.get("last") == project_name:
                    remaining = [k for k in config.projects.keys() if k != "last"]
                    if remaining:
                        config.projects["last"] = remaining[0]
                    else:
                        config.projects.pop("last", None)

        # 检查新发现的项目
        new_projects: dict[str, str] = {}
        for project_name, project_path in found_projects.items():
            if project_name not in config.projects:
                new_projects[project_name] = project_path

        # 如果有新项目，询问是否添加
        if new_projects:
            click.echo(f"Found {len(new_projects)} new RMM project(s):")
            for name, path in new_projects.items():
                click.echo(f"  - {name}: {path}")

            if click.confirm("Add these projects to configuration?", default=True):
                config.projects.update(new_projects)

                # 如果没有 last 项目，设置第一个新项目为 last
                if "last" not in config.projects and new_projects:
                    config.projects["last"] = list(new_projects.keys())[0]

                click.echo(f"Added {len(new_projects)} project(s) to configuration")        # 保存配置
        config.save()

        return {k: v for k, v in config.projects.items() if k != "last"}

    def __new__(cls,rpath : str | Path,rtype: Literal["magisk", "ksu", "apu"] = "magisk") -> RmmProject:
        """
        Create a new RmmProject instance (not singleton to allow multiple projects).
        """
        return super().__new__(cls)

    def __init__(self,rpath: str | Path , rtype: Literal["magisk", "ksu", "apu"] = "magisk") -> None:
        self._config = Config()
        # 初始化 projects 属性如果不存在
        if not hasattr(self._config, 'projects'):
            self._config.projects = {}
        
        # 设置路径
        if isinstance(rpath, str):
            self._rpath = Path(rpath).resolve()
        else:
            self._rpath = Path(rpath).resolve()
            
        # 检查目录是否不为空（修复逻辑错误）
        if self._rpath.exists() and any(self._rpath.iterdir()):
            if click.confirm(f"The directory '{self._rpath}' is not empty. Do you want to continue?", default=True):
                click.echo(f"Continuing with the existing directory: {self._rpath}")

        # 使用 object.__setattr__ 直接设置属性，避免触发 __getattr__
        object.__setattr__(self, 'id', self._rpath.name)
        object.__setattr__(self, 'rtype', rtype)

    @property
    def path(self) -> Path:
        """Get the project path."""
        return self._rpath

    def __setattr__(self, name: str, value: Any) -> None:
        """
        Set an attribute in the configuration and save it.
        """
        # 允许设置私有属性
        if name.startswith('_'):
            super().__setattr__(name, value)
            return

        # 直接设置属性
        super().__setattr__(name, value)
    def __getattr__(self, name: str) -> Any:
        """
        Get an attribute from the configuration.
        """
        # 如果是私有属性，直接抛出 AttributeError
        if name.startswith('_'):
            raise AttributeError(f"'{self.__class__.__name__}' object has no attribute '{name}'")
        
        # 如果没有找到，询问用户是否设置
        if click.confirm(f"Configuration '{name}' not found. Do you want to set it?", abort=False):
            value = click.prompt(f"Please enter the value for '{name}'", type=str)
            setattr(self, name, value)
            return value
        
        # 如果用户选择不设置，返回 None 或抛出异常
        raise AttributeError(f"Configuration '{name}' not found and user chose not to set it.")

    def new(self) -> None:
        """
        Create a new RMM project configuration.
        """
        if isinstance(self._rpath, str):
            self._rpath = Path(self._rpath)

        # 确保使用绝对路径
        self._rpath = self._rpath.resolve()

        rmm_project_toml = self._rpath / 'rmmproject.toml'
        if rmm_project_toml.exists():

            click.echo(f"Project already exists at {self._rpath}. Loading existing project.")
            self.load(self._rpath)
        else:
            if not self.id:
            # 采用路径最后文件夹名
                self.id = Path(self._rpath).name
            #region 新建项目
            self._config.projects[self.id] = str(self._rpath)
            # 手动保存配置，因为字典内容修改不会触发 __setattr__
            self._config.projects["last"] = self.id  # 设置最后使用的项目
            self._config.save()
            
            # 确保项目目录存在
            self._rpath.mkdir(parents=True, exist_ok=True)

            # 设置项目根路径
            object.__setattr__(self, '_rmmproot', self._rpath)

            self.requires_rmm = f">{__version__}"
            self.description = f"Add description for project {self.id} here."            
            self.authors = [
                {
                    "name": self._config.username if hasattr(self._config, 'username') else "Your Name",
                    "email": self._config.email if hasattr(self._config, 'email') else "Your Email",               
                }
            ]
            self.license = "license"  # 文件名
            self.readme = "README.MD"  # 文件名
            self.changelog = "CHANGELOG.MD"
            self.dependencies: list[str] = []  # 依赖列表
            self.scripts: list[str] = []  # 脚本列表
            self.save()  # 保存配置

    def load(self, rpath: str | Path | None = None):
        #region 加载项目
        if not rpath:
            # 安全地获取last项目，避免KeyError
            if "last" not in self._config.projects:
                raise ValueError("No 'last' project found in configuration. Please specify a path or create a project first.")
            
            last_project = self._config.projects["last"]
            if last_project not in self._config.projects:
                raise ValueError(f"Last project '{last_project}' not found in configuration.")
            
            rpath = self._config.projects[last_project]

        if isinstance(rpath, str):
            rpath = Path(rpath)


        self._rmmproot: Path = rpath
        rmm_project_toml = rpath / 'rmmproject.toml'
        rmm_version: Path = rpath / '.rmm_version'

        if not rmm_project_toml.exists():
            # 不存在：新建
            rmm_project_toml.touch(exist_ok=True)
            rmm_version.touch(exist_ok=True)
            rmm_version.write_text(__version__, encoding='utf-8')            
            if click.confirm(f"Project '{self._rpath}' does not exist. Do you want to create a new project?", default=True):
                project_type = click.prompt(f"Please choose RMM project type.", type=click.Choice(["magisk", "ksu", "apu"]), default="magisk")
                object.__setattr__(self, 'rtype', project_type)
                self.new()

            # 使用项目ID而不是self.name来避免递归
            if hasattr(self, 'id'):
                self._config.projects["last"] = self.id
            else:
                # 如果没有id，使用路径名
                self._config.projects["last"] = Path(self._rpath).name

        else:
            # 存在：加载
            with open(rmm_project_toml, 'r', encoding='utf-8') as f:
                rmm_project_meta = toml.load(f)
                # 这里可以处理项目配置的加载逻辑
            for key, value in rmm_project_meta.items():
                # 处理 Path 对象的加载
                if isinstance(value, str) and os.path.exists(value):
                    value = Path(value)
                object.__setattr__(self, key, value)

    def save(self):
        """
        Save the current project configuration to a file.
        """
        if not hasattr(self, '_rmmproot'):
            raise AttributeError("Project root path is not set. Please load or create a project first.")
        
        rmm_project_toml = self._rmmproot / 'rmmproject.toml'
        
        # 获取所有非私有属性
        rmmp_meta = {key: value for key, value in self.__dict__.items() if not key.startswith('_')}
        
        with open(rmm_project_toml, 'w', encoding='utf-8') as f:
            toml.dump(rmmp_meta, f)
    
    @property
    def ls(self) -> dict[str, str]:
        """Get the projects dictionary."""
        return self._config.projects

    def __dir__(self) -> list[str]:
        """
        List all project attributes.
        """
        return sorted(key for key in self._config.projects.keys() if key != 'last')

    def __str__(self) -> str:
        """
        String representation of the RMM project.
        """
        info = f"[*]RMM Project ID: {self.id}\n"
        info += f"[*]RMM Project Root: {self._rmmproot}\n"
        info += f"[*]RMM Project Type: {self.rtype}\n"
        info += f"[*]RMM Project Requires RMM: {self.requires_rmm}\n"
        info += f"[*]RMM Project Description: {self.description}\n"
        info += f"[*]RMM Project Authors: {self.authors}\n"
        info += f"[*]RMM Project License: {self.license}\n"
        info += f"[*]RMM Project Readme: {self.readme}\n"
        info += f"[*]RMM Project Changelog: {self.changelog}\n"
        info += f"[*]RMM Project Dependencies: {self.dependencies}\n"
        info += f"[*]RMM Project Scripts: {self.scripts}\n"
        info += f"[*]RMM Project Last Used: {self._config.projects.get('last', 'None')}\n"


        return f"RMM Projects: {self._config.projects}"

    def __delattr__(self, name: str) -> None:
        """
        Delete a project from the configuration.
        """
        # 检查是否是私有属性
        if name.startswith('_'):
            raise AttributeError(f"Cannot delete private attribute '{name}'")

        # 检查项目是否存在
        if name not in self._config.projects:
            raise AttributeError(f"Project '{name}' not found")

        # 获取项目路径
        project_path = Path(self._config.projects[name])

        # 询问是否删除项目目录
        if project_path.exists():
            if click.confirm(f"Project directory '{project_path}' exists. Do you want to delete it as well?", default=True):
                try:
                    shutil.rmtree(project_path)
                    click.echo(f"Project directory '{project_path}' has been deleted.")
                except Exception as e:
                    click.echo(f"Failed to delete project directory: {e}")

        # 从项目字典中删除
        del self._config.projects[name]

        # 如果删除的是当前的 last 项目，需要更新 last
        if self._config.projects.get("last") == name:
            # 找到其他项目作为新的 last，如果没有其他项目则设为空
            remaining_projects = [k for k in self._config.projects.keys() if k != "last"]
            if remaining_projects:
                self._config.projects["last"] = remaining_projects[0]
                click.echo(f"Updated last project to: {remaining_projects[0]}")
            else:
                # 如果没有其他项目了，删除 last 键
                if "last" in self._config.projects:
                    del self._config.projects["last"]
                click.echo("No remaining projects. Cleared last project.")

        # 保存配置
        self._config.save()

        # 调用同步函数以确保配置与文件系统一致
        click.echo(f"Project '{name}' has been deleted from configuration.")
        click.echo("Synchronizing project configuration with file system...")
        RmmProject.sync()

