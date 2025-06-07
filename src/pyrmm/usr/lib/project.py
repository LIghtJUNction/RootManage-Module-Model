from pathlib import Path
import toml
import shutil
from pyrmm.usr.lib.fs import RmmFileSystem
from pyrmm.usr.lib.config import Config

class RmmProjectMeta(type):
    """Meta class for RMM Project"""
    @property
    def META(cls):
        """Get the project metadata."""
        meta: dict[str, str | dict[str, str]] = Config.META
        projects: str | dict[str, str] = meta.get("projects", {"last":"None"})
        if isinstance(projects, str):
            raise AttributeError(f"项目配置错误!： '{projects}' 请检查：{RmmFileSystem.META}")
        return projects
    
    def project_path(cls, project_name: str) -> Path:
        """Get the path of a project by its name."""
        projects = cls.META
        if project_name == "last":
            raise KeyError("项目 'last' 是保留关键字! 请使用实际项目名称。")
        if project_name in projects:
            projectpath: Path = Path(projects[project_name])
            if projectpath.exists():
                return projectpath
            else:
                raise FileNotFoundError(f"项目路径不存在: {projectpath}")
        else:
            raise KeyError(f"项目 '{project_name}' 不存在于配置中。")
        
    def project_info(cls, project_path: Path):
        """Get the project information from the project path."""
        if not project_path.exists():
            raise FileNotFoundError(f"项目路径不存在: {project_path}")
        # 读取项目的元数据文件
        meta_file = project_path / "project.meta"
        if not meta_file.exists():
            raise FileNotFoundError(f"项目元数据文件不存在: {meta_file}")        
        with open(meta_file, 'r', encoding='utf-8') as f:
            return toml.load(f)

    def __getattr__(cls, item: str):
        """Get an attribute from the project metadata."""
        project_info = cls.project_info(cls.project_path(item))
        if project_info:
            return project_info
        else:
            raise AttributeError(f"项目 '{item}' 的信息未找到。")

    def __setattr__(self, name: str, value: dict[str , str | list[dict[str,str]]]) -> None:
        """Set an attribute in the project metadata."""
        project_info = self.project_info(self.project_path(name))
        if project_info:
            project_info.update(value)
        else:
            raise AttributeError(f"项目 '{name}' 的信息未找到。")

    def __delattr__(cls, name: str) -> None:
        """Delete a project."""
        try:
            # 获取项目路径
            project_path = cls.project_path(name)
            
            # 删除项目目录及其内容
            if project_path.exists():
                shutil.rmtree(project_path)
                print(f"项目目录 '{project_path}' 已删除")
            
            # 从配置中移除项目记录
            projects = Config.META.get("projects", {})
            if isinstance(projects, dict) and name in projects:
                del projects[name]
                Config.projects = projects
                print(f"项目 '{name}' 已从配置中移除")
            
        except (KeyError, FileNotFoundError) as e:
            print(f"删除项目时出现错误: {e}")
        except Exception as e:
            print(f"删除项目时出现未知错误: {e}")


class RmmProject(metaclass=RmmProjectMeta):
    """RMM Project class"""
    @classmethod
    def init(cls, project_path: Path):
        """Initialize a new RMM project."""
        project_name = project_path.name
        
        # 确保项目目录存在
        project_path.mkdir(parents=True, exist_ok=True)
        
        # 创建项目信息
        project_info: dict[str, str | dict[str, str] | list[dict[str, str | dict[str, str]]] | list[dict[str, str]]] = {
            "id": project_name,
            "name": project_name,
            "requires_rmm": f">={Config.version}",
            "versionCode": str(project_path.resolve()),
            "updateJson": f"https://raw.githubusercontent.com/{Config.username}/{project_name}/main/update.json",
            "readme": "README.MD",
            "changelog": "CHANGELOG.MD",
            "lecense": "LICENSE",
            "urls": {
                "github": f"https://github.com/{Config.username}/{project_name}"
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
        }
        
        # 将项目信息写入项目元数据文件
        meta_file = project_path / "project.meta"
        with open(meta_file, 'w', encoding='utf-8') as f:
            toml.dump(project_info, f)
        
        # 将项目路径添加到配置中
        projects = Config.META.get("projects", {})
        if isinstance(projects, dict):
            projects[project_name] = str(project_path.resolve())
            Config.projects = projects
        
        return project_info

    @classmethod
    def sync(cls, project_name: str):
        """Sync a project by its name."""
        project_path = cls.project_path(project_name)
        if not project_path.exists():
            delattr(cls, project_name)

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


