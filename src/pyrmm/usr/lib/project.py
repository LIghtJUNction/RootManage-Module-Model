from pathlib import Path
import toml
import shutil
from typing import Any
from pyrmm.usr.lib.fs import RmmFileSystem
from pyrmm.usr.lib.config import Config
from .base import RmmBaseMeta, RmmBase
from .version import VersionGenerator
from .git import RmmGit

class RmmProjectMeta(RmmBaseMeta):
    """Meta class for RMM Project"""
    # 项目信息缓存    
    _project_cache: dict[str, dict[str, Any]] = {}
    _project_mtime: dict[str, float] = {}
    
    @property
    def META(cls) -> dict[str, Any]:
        """Get the project metadata."""
        meta: dict[str, str | dict[str, str]] = Config.META
        projects: str | dict[str, str] = meta.get("projects", {})
        if isinstance(projects, str):
            raise AttributeError(f"项目配置错误!： '{projects}' 请检查：{RmmFileSystem.META}")
        return projects
    
    def get_config_key(cls) -> str:
        """获取配置键名"""
        return "projects"
    
    def get_reserved_key(cls) -> str:
        """获取保留关键字"""
        return "rmm"  # 移除 last 保留关键字
    
    def get_item_config(cls, item_name: str) -> dict[str, str | dict[str, str]]:
        """获取项目配置"""
        return cls.project_info(cls.project_path(item_name))
    
    def _set_item_config(cls, name: str, value: dict[str, str | dict[str, str]]) -> None:
        """设置项目配置"""
        try:
            project_path = cls.project_path(name)
            project_info = cls.project_info(project_path)
            if project_info:
                project_info.update(value)
                # 将更新后的信息写入项目元数据文件
                meta_file = project_path / "rmmproject.toml"
                with open(meta_file, 'w', encoding='utf-8') as f:
                    toml.dump(project_info, f)
                
                # 清理缓存，确保下次读取时获取最新数据
                cache_key = str(meta_file.resolve())
                if cache_key in cls._project_cache:
                    del cls._project_cache[cache_key]
                if cache_key in cls._project_mtime:
                    del cls._project_mtime[cache_key]
        except Exception as e:
            print(f"设置项目配置时出现错误: {e}")
    
    def _delete_item_config(cls, name: str) -> None:
        """删除项目配置"""
        try:
            # 尝试获取项目路径
            try:
                project_path = cls.project_path(name)
                # 如果路径存在，删除项目目录及其内容
                if project_path.exists():
                    shutil.rmtree(project_path)
                    print(f"项目目录 '{project_path}' 已删除")
            except (KeyError, FileNotFoundError):
                # 路径不存在或项目不在配置中，这是正常情况，不需要报错
                pass
            
            # 从配置中移除项目记录（无论路径是否存在）
            projects = Config.META.get("projects", {})
            if isinstance(projects, dict) and name in projects:
                del projects[name]
                Config.projects = projects
                print(f"项目 '{name}' 已从配置中移除")
        except Exception as e:
            print(f"删除项目时出现未知错误: {e}")

    def project_path(cls, project_name: str) -> Path:
        """Get the path of a project by its name."""
        projects = cls.META
        if project_name in projects:
            projectpath: Path = Path(projects[project_name])
            if projectpath.exists():
                return projectpath
            else:
                raise FileNotFoundError(f"项目路径不存在: {projectpath}")
        else:
            raise KeyError(f"项目 '{project_name}' 不存在于配置中。")
    
    @classmethod
    def project_info(cls, project_path: Path) ->  dict[str, str | dict[str, str]]:
        """Get the project information from the project path with caching."""
        if not project_path.exists():
            raise FileNotFoundError(f"项目路径不存在: {project_path}")
        
        # 读取项目的元数据文件
        meta_file = project_path / "rmmproject.toml"
        if not meta_file.exists():
            raise FileNotFoundError(f"项目元数据文件不存在: {meta_file}")
        
        # 使用文件路径作为缓存键
        cache_key = str(meta_file.resolve())
        
        # 检查文件修改时间
        current_mtime = meta_file.stat().st_mtime
        
        # 如果缓存中有数据且文件未修改，直接返回缓存
        if (cache_key in cls._project_cache and 
            cache_key in cls._project_mtime and 
            cls._project_mtime[cache_key] == current_mtime):
            return cls._project_cache[cache_key]
        
        # 读取文件并更新缓存
        with open(meta_file, 'r', encoding='utf-8') as f:
            project_info = toml.load(f)
        
        # 更新缓存
        cls._project_cache[cache_key] = project_info
        cls._project_mtime[cache_key] = current_mtime
        return project_info
        
    def __getattr__(cls, item: str):
        """Get an attribute from the project metadata."""
        if item == cls.get_reserved_key():
            raise KeyError(f"项目 '{cls.get_reserved_key()}' 是保留关键字! 请使用实际项目名称。")
        try:
            project_info = cls.project_info(cls.project_path(item))
            if project_info:
                return project_info
            else:
                raise AttributeError(f"项目 '{item}' 的信息未找到。")
        except (KeyError, FileNotFoundError):
            raise AttributeError(f"项目 '{item}' 不存在。")
        
    def __setattr__(cls, name: str, value: dict[str, str | dict[str, str]]) -> None:
        """Set an attribute in the project metadata."""
        if name == cls.get_reserved_key():
            raise KeyError(f"项目 '{cls.get_reserved_key()}' 是保留关键字! 请使用实际项目名称。")
        
        try:
            # 获取项目路径
            project_path = cls.project_path(name)
            if not project_path.exists():
                raise FileNotFoundError(f"项目 '{name}' 不存在。")
            
            # 获取当前项目元数据
            current_info = cls.project_info(project_path)
                        
            # 使用 update 方法更新元数据
            current_info.update(value)

            # 保存更新后的数据到 TOML 文件
            meta_file = project_path / "rmmproject.toml"
            with open(meta_file, 'w', encoding='utf-8') as f:
                toml.dump(current_info, f)
            
            # 清理缓存以确保下次读取时获取最新数据
            cache_key = str(meta_file.resolve())
            if cache_key in cls._project_cache:
                del cls._project_cache[cache_key]
            if cache_key in cls._project_mtime:
                del cls._project_mtime[cache_key]
                
        except (KeyError, FileNotFoundError) as e:
            raise AttributeError(f"无法设置项目 '{name}' 的属性: {str(e)}")
        
        
class RmmProject(RmmBase, metaclass=RmmProjectMeta):
    """RMM Project class"""
    
    @classmethod
    def add_project(cls, project_name: str, project_path: str) -> None:
        """Add an existing project to the configuration"""
        project_path_obj = Path(project_path)
        
        # 验证项目路径存在且是 RMM 项目
        if not project_path_obj.exists():
            raise FileNotFoundError(f"项目路径不存在: {project_path}")
        
        if not cls.is_rmmproject(project_path_obj):
            raise ValueError(f"路径 {project_path} 不是一个有效的 RMM 项目")
        
        # 获取当前项目配置
        projects = Config.META.get("projects", {})
        if isinstance(projects, dict):
            projects[project_name] = str(project_path_obj.resolve())
            Config.projects = projects
        else:
            raise AttributeError("项目配置格式错误")
    
    @classmethod
    def is_valid_item(cls, item_name: str) -> bool:
        """Check if the given project name corresponds to a valid RMM project."""
        try:
            project_path = cls.project_path(item_name)
            return RmmProject.is_rmmproject(project_path)
        except (KeyError, FileNotFoundError):
            return False

    @classmethod
    def get_sync_prompt(cls, item_name: str) -> str:
        """获取同步提示信息"""
        return f"项目 '{item_name}' 不是一个有效的 RMM 项目。移除？"

    @classmethod
    def init(cls, project_path: Path):
        """Initialize a new RMM project with Git repository detection."""
        project_name = project_path.name
        
        # 确保项目目录存在
        project_path.mkdir(parents=True, exist_ok=True)
          # Git 仓库检测
        git_repo_info = RmmGit.get_repo_info(project_path)
        git_info = {}
        username = Config.username
        repo_name = project_name
        is_in_repo_root = False
        
        if git_repo_info:
            # 优先使用 origin 远程，如果没有则使用第一个可用的
            remote_info = git_repo_info.remotes.get('origin')
            if not remote_info and git_repo_info.remotes:
                remote_info = next(iter(git_repo_info.remotes.values()))
                
            if remote_info and remote_info.username and remote_info.repo_name:
                username = remote_info.username
                repo_name = remote_info.repo_name
                
                # 如果当前 Config.username 是默认值，自动更新为检测到的用户名
                if Config.username == "username":
                    Config.username = username
                    print(f"📝 自动更新配置用户名: {username}")
                
                git_info = {
                    'git_root': str(git_repo_info.root_path),
                    'remote_url': remote_info.url,
                    'username': username,
                    'repo_name': repo_name
                }
                
                # 判断项目是否在仓库根目录
                is_in_repo_root = (project_path.resolve() == git_repo_info.root_path.resolve())
                
                print(f"检测到 Git 仓库: {remote_info.url}")
                print(f"用户名: {username}, 仓库名: {repo_name}")
                print(f"项目位置: {'仓库根目录' if is_in_repo_root else '子目录'}")
        
        # 创建项目信息
        if is_in_repo_root and git_info:
            # 项目在仓库根目录，使用检测到的仓库信息
            github_url = f"https://github.com/{username}/{repo_name}"
            update_json_url = f"https://raw.githubusercontent.com/{username}/{repo_name}/main/update.json"
        else:
            # 项目在子目录或没有 Git 仓库，使用默认配置
            github_url = f"https://github.com/{Config.username}/{project_name}"
            update_json_url = f"https://raw.githubusercontent.com/{Config.username}/{project_name}/main/update.json"
        
        project_info: dict[str, Any] = {
            "id": project_name,
            "name": project_name,
            "requires_rmm": f">={Config.version}",
            "versionCode": str(project_path.resolve()),
            "updateJson": update_json_url,
            "readme": "README.MD",
            "changelog": "CHANGELOG.MD",
            "lecense": "LICENSE",
            "urls": {
                "github": github_url
            },
            "dependencies": [
                {
                    "dep?": "?version",
                }
            ],
            "authors": [
                {
                    "name": Config.username,
                    "email": Config.email
                }
            ],
            "scripts": [
                {
                    "build": "rmm build",
                }
            ],
        }        # 如果检测到 Git 信息，添加到项目信息中
        if git_info:
            git_info["is_in_repo_root"] = str(is_in_repo_root)
            project_info["git"] = git_info
          # 将项目信息写入项目元数据文件
        meta_file = project_path / "rmmproject.toml"
        with open(meta_file, 'w', encoding='utf-8') as f:
            toml.dump(project_info, f)
          # 创建 module.prop 文件
        module_prop: Path = project_path / "module.prop"        # 获取作者信息
        authors = project_info.get("authors", [{}])
        # 安全地获取作者名称，确保返回字符串类型
        if authors and isinstance(authors[0], dict):
            author_dict = authors[0]
            author_name_value = author_dict.get("name", Config.username)
            if isinstance(author_name_value, str):
                author_name: str = author_name_value
            else:
                author_name: str = "username"  # 使用默认值
        else:
            # 从 Config.username 获取，但确保是字符串类型
            config_username = Config.username
            if isinstance(config_username, str):
                author_name: str = config_username
            else:
                author_name: str = "username"  # 使用默认值
        
        # 使用版本生成器生成版本信息
        version_info = VersionGenerator.generate("", project_path)
        
        module_prop_content = {
            "id": str(project_info.get("id", project_name)),
            "name": str(project_info.get("name", project_name)),
            "version": version_info["version"],
            "versionCode": version_info["versionCode"],
            "author": str(author_name),
            "description": f"RMM项目 {project_name}",
            "updateJson": str(project_info.get("updateJson", ""))
        }
          # 写入 module.prop 文件（使用标准的key=value格式）
        with open(module_prop, 'w', encoding='utf-8') as f:
            for key, value in module_prop_content.items():
                f.write(f"{key}={value}\n")
            
          # 创建必要的文件
        cls._create_project_files(project_path, project_name, author_name)
        
        # 将项目路径添加到配置中
        projects = Config.META.get("projects", {})
        if isinstance(projects, dict):
            projects[project_name] = str(project_path.resolve())
            Config.projects = projects

        return project_info

    @staticmethod
    def is_rmmproject(project_path: Path) -> bool:
        """Check if the given path is a valid RMM project."""
        meta_file = project_path / "rmmproject.toml"
        return meta_file.exists() and meta_file.is_file()

    @classmethod
    def sync(cls, project_name: str):
        """Sync a project by its name."""
        # 首先检查项目有效性
        if not cls.is_valid_item(project_name):
            cls.sync_item(project_name)  # 这会处理无效项目的删除
            return
        
        # 对于有效项目，更新版本信息
        try:
            project_path = cls.project_path(project_name)
            
            # 使用 VersionGenerator 来生成并更新版本信息
            from .version import VersionGenerator
            
            # 读取当前版本
            try:
                project_info = cls.project_info(project_path)
                
                _current_version : str | dict[str , str]= project_info.get("version", "v1.0.0")
    
                current_version: str = _current_version if isinstance(_current_version, str) else "v1.0.0"
            except Exception:
                current_version: str = "v1.0.0"

            # 自动判断升级类型并更新版本
            version_info = VersionGenerator.auto_bump(current_version, project_path)
            VersionGenerator.update_project_files(project_path, version_info)
            
        except Exception as e:
            print(f"❌ 同步项目 {project_name} 时出错: {e}")
            # 如果版本更新失败，回退到基础同步
            cls.sync_item(project_name)

    @classmethod
    def init_basic(cls, project_path: Path):
        """Initialize a basic RMM project."""
        cls.init(project_path)
        system_dir = project_path / "system"
        system_dir.mkdir(exist_ok=True)
        return {"message": "RMM basic project initialized."}

    @classmethod
    def init_library(cls, project_path: Path):
        """Initialize a RMM library project."""
        cls.init(project_path)
        # 这里可以添加特定于库项目的初始化逻辑
        # 例如，创建特定的目录结构或文件
        lib_dir = project_path / "lib"
        lib_dir.mkdir(exist_ok=True)
        return {"message": "RMM library project initialized."}

    @classmethod
    def clean_dist(cls, project_path: Path) -> None:
        """清理项目的构建输出目录
        
        Args:
            project_path: 项目路径
        """
        import shutil
        
        # 构建输出目录通常在 .rmmp/dist 下
        dist_dir = project_path / ".rmmp" / "dist"
        
        if dist_dir.exists():
            try:
                shutil.rmtree(dist_dir)
                print(f"🧹 已清理构建输出目录: {dist_dir}")
            except Exception as e:
                print(f"❌ 清理构建输出目录失败: {e}")
                raise
        else:
            print(f"ℹ️  构建输出目录不存在: {dist_dir}")

    @classmethod
    def _create_project_files(cls, project_path: Path, project_name: str, author_name: str) -> None:
        """创建项目必要的文件：README.MD、CHANGELOG.MD、LICENSE
        
        Args:
            project_path: 项目路径
            project_name: 项目名称
            author_name: 作者名称
        """


        from .basic import README
        # 创建 README.MD
        readme_content = README.replace("{project_name}", project_name).replace(
            "{author_name}", author_name)

        from .basic import CHANGELOG
        # 创建 CHANGELOG.MD
        changelog_content = CHANGELOG

        from .basic import LICENSE
        # 创建 LICENSE 文件内容
        license_content = LICENSE
        
        # 写入文件
        files_to_create = [
            ("README.MD", readme_content),
            ("CHANGELOG.MD", changelog_content),
            ("LICENSE", license_content)
        ]
        
        for filename, content in files_to_create:
            file_path = project_path / filename
            
            # 如果文件不存在才创建，避免覆盖现有文件
            if not file_path.exists():
                try:
                    with open(file_path, 'w', encoding='utf-8') as f:
                        f.write(content)
                    print(f"✅ 创建文件: {filename}")
                except Exception as e:
                    print(f"⚠️  警告: 创建文件 {filename} 失败: {e}")
            else:
                print(f"ℹ️  文件已存在，跳过: {filename}")


