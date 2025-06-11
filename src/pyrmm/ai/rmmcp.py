from pathlib import Path
import os
import toml
from typing import Any
from mcp.server.fastmcp import FastMCP

class RmMcp(FastMCP):
    """
    RmMcp class that extends FastMCP with configuration management capabilities.
    """
    
    def __init__(self, name: str = "RmMcp", host: str = "localhost", port: int = 8000, *args, **kwargs):
        super().__init__(name, *args, **kwargs)
        self.host = host
        self.port = port
        self.cache: dict[str, Any] = {
            "username": "LIghtJUNction[dev]",
            "email": "lightjuntion.me@gmail.com",
            "version": "0.2.3",
            "projects": {}
        }
        self._cache_last_modified: float = 0

    @property
    def ROOT(self) -> Path:
        """Get the root directory for RMM configuration."""
        return Path(os.getenv("RMM_ROOT", Path().home() / "data" / "adb" / ".rmm"))

    @property
    def META_FILE(self) -> Path:
        """Get the path to the metadata TOML file."""
        return self.ROOT / "meta.toml"

    @property
    def _true_last_modified(self) -> float:
        """Get the actual last modified time of the metadata file."""
        try:
            return self.META_FILE.stat().st_mtime
        except FileNotFoundError:
            return 0

    @property
    def is_changed(self) -> bool:
        """
        Check if the cache has changed since the last load.
        """
        true_modified = self._true_last_modified
        if self._cache_last_modified != true_modified:
            self._cache_last_modified = true_modified
            return True
        return False

    @property
    def META(self) -> dict[str, Any]:
        """
        Get the metadata from the TOML file, loading it if necessary.
        """
        if self._cache_last_modified == 0 or self._true_last_modified == 0 or self.is_changed:
            try:
                with open(self.META_FILE, "r", encoding="utf-8") as f:
                    self.cache = toml.load(f)
            except FileNotFoundError:
                # If file doesn't exist, create the directory and use default cache
                self.ROOT.mkdir(parents=True, exist_ok=True)        
        return self.cache
    
    @property
    def projects(self) -> dict[str, Any]:
        """
        Get the projects from the metadata.
        """
        return self.META.get("projects", {"tip": "暂未发现任何模块，提示用户使用rmm init 新建一个项目"})
    
    def project_path(self, project_id: str) -> Path:
        """
        Get the path of a project.
        """
        project_path = self.projects.get(project_id)
        if project_path:
            return Path(project_path)
        else:
            return Path("")

    def project_info(self, project_id: str) -> dict[str, Any]:
        """
        Get the project information from the metadata.
        """
        project_path = self.project_path(project_id)
        project_info_file: Path = project_path / "rmmproject.toml"
        if project_info_file.exists():
            try:
                with open(project_info_file, "r", encoding="utf-8") as f:
                    return toml.load(f)
            except Exception as e:
                print(f"读取项目 {project_id} 信息失败: {e}")
                return {}
        else:
            print(f"项目 {project_id} 的信息文件不存在: {project_info_file}")
            return {f"项目 {project_id} 的信息文件不存在": str(project_info_file)}
        

        
mcp = RmMcp("RmMcp")


def with_project_directory(project_name: str | None = None):
    """
    装饰器/上下文管理器：切换到项目目录并在完成后恢复原目录
    
    参数:
        project_name: 可选的项目名称，如果未提供则使用第一个可用项目
    
    返回:
        返回一个上下文管理器，提供项目路径和工作目录切换
    """
    from contextlib import contextmanager
    
    @contextmanager
    def project_context():
        original_cwd = os.getcwd()
        work_dir = None
        project_path = None
        
        try:
            projects = mcp.projects
            
            # 确定工作目录
            if project_name and project_name in projects:
                project_path = mcp.project_path(project_name)
                if not project_path.exists():
                    raise FileNotFoundError(f"项目路径不存在: {project_path}")
                work_dir = str(project_path)
            elif projects and len(projects) > 0 and "tip" not in projects:
                # 如果没有指定项目但存在项目，使用第一个项目
                first_project = next(iter(projects.keys()))
                project_path = mcp.project_path(first_project)
                work_dir = str(project_path)
            else:
                # 没有项目时使用根目录
                work_dir = str(mcp.ROOT.parent)
            
            # 切换到项目目录
            os.chdir(work_dir)
            
            # 返回上下文信息
            yield {
                "work_dir": work_dir,
                "project_path": project_path,
                "project_name": project_name,
                "original_cwd": original_cwd
            }
            
        finally:
            # 恢复原始工作目录
            os.chdir(original_cwd)
    
    return project_context()

@mcp.tool()
def getRMMETA():
    """
    获取 RMM (Magisk 模块项目管理) 的元数据。
    """
    return mcp.META

@mcp.tool()
def getRMProjects():
    """
    获取 RMM (Magisk 模块项目管理) 的项目列表。
    """
    return mcp.projects

@mcp.tool()
def getRMMRoot():
    """
    获取 RMM 配置的根目录。
    """
    return str(mcp.ROOT)

@mcp.tool()
def getProjectInfo(project_name: str):
    """
    获取特定项目的信息。
    
    参数:
        project_name: 要获取信息的项目名称。
    
    返回:
        包含项目信息的字典，如果项目不存在则返回错误消息。
    """
    projects = mcp.projects
    if project_name in projects:
        project_path: Path = mcp.project_path(project_name)
        if project_path.exists():
            project_info = mcp.project_info(project_name)
            return project_info
        else:
            return {"error": f"项目 {project_name} 的路径不存在: {project_path}"}

@mcp.tool()
def initNewProject(project_name: str, template: str = "basic", project_path: str | None = None):
    """
    使用 rmm init 命令初始化新的 RMM 项目。
    
    参数:
        project_name: 新项目的名称
        template: 项目模板 (basic, library, ravd)
        project_path: 可选的自定义项目路径,禁止选择在非RMM_ROOT目录外的路径
    
    返回:
        rmm init 命令的执行结果
    """
    from pyrmm.cli.rmmcore import cli_with_output
    
    # 构建参数列表
    args = ["init"]
    
    # 添加模板标志
    if template == "lib":
        args.append("--lib")
    elif template == "ravd":
        args.append("--ravd")
    else:  # basic 是默认的，使用 --basic 或者不添加参数
        args.append("--basic")
    
    # 添加项目名称
    args.append(project_name)
    
    # 添加项目路径
    if project_path:
        # 检查这个路径是否在 RMM_ROOT 目录下
        safe_project_path = Path(project_path).resolve()
        if not safe_project_path.is_relative_to(mcp.ROOT):
            return {
                "success": False,
                "stdout": "",
                "stderr": f"项目路径 {project_path} 必须在 RMM_ROOT 目录下: {mcp.ROOT}",
                "command": f"rmm {' '.join(args)} {project_path}",
                "method": "rust_extension",
                "error": f"项目路径不合法: {project_path}，必须在 RMM_ROOT 目录下: {mcp.ROOT}"
            }
        args.append(str(safe_project_path))
    else:
        # 如果没有提供路径，自动构建项目路径到 RMM_ROOT 目录下
        if not mcp.ROOT.exists():
            mcp.ROOT.mkdir(parents=True, exist_ok=True)
        safe_project_path = mcp.ROOT / "data" / "rmmps" / project_name
        args.append(str(safe_project_path))
    
    # 使用 Rust 扩展执行命令
    try:
        # 执行命令并获取详细输出
        result_output = cli_with_output(args)
        
        # 使用实际的命令输出
        if result_output and isinstance(result_output, str):
            stdout_message = result_output
        else:
            stdout_message = f"项目 {project_name} 初始化成功"
        
        return {
            "success": True,
            "stdout": stdout_message,
            "stderr": "",
            "command": f"rmm {' '.join(args)}",
            "method": "rust_extension_with_output"
        }
    except Exception as rust_error:
        return {
            "success": False,
            "stdout": "",
            "stderr": str(rust_error),
            "command": f"rmm {' '.join(args)}",
            "method": "rust_extension",                
            "error": f"Rust 扩展执行失败: {rust_error}"
        }

@mcp.tool()
def buildProject(project_name: str | None = None, debug: bool = False, skip_shellcheck: bool = False):
    """
    使用 rmm build 命令构建 RMM 项目。
    
    参数:
        project_name: 可选的项目名称 (如果未提供，则构建当前目录)
        debug: 启用调试模式
        skip_shellcheck: 跳过 shell 脚本检查
    
    返回:
        rmm build 命令的执行结果
    """
    from pyrmm.cli.rmmcore import cli_with_output
    
    # 使用项目路径切换执行命令
    with with_project_directory(project_name) as ctx:
        try:
            # 构建参数列表
            args = ["build"]
            if debug:
                args.append("--debug")
            if skip_shellcheck:
                args.append("--skip-shellcheck")
            
            # 执行 rmm build 命令并捕获返回值
            result_output = cli_with_output(args)
            
            # cli_with_output 返回实际的命令输出
            if result_output and isinstance(result_output, str):
                stdout_message = result_output
            else:
                stdout_message = "项目构建成功"
            
            return {
                "success": True,
                "stdout": stdout_message,
                "stderr": "",
                "command": f"rmm {' '.join(args)}",
                "work_dir": ctx["work_dir"],
                "method": "rust_extension_with_output"
            }
        except Exception as rust_error:
            return {
                "success": False,
                "stdout": "",
                "stderr": str(rust_error),
                "command": f"rmm {' '.join(args)}",
                "work_dir": ctx["work_dir"],
                "method": "rust_extension",
                "error": f"Rust 扩展执行失败: {rust_error}"
            }

@mcp.tool()
def checkProjectSyntax(project_name: str | None = None):
    """
    使用 rmm check 命令检查项目语法。
    
    参数:
        project_name: 项目名称
    
    返回:
        语法检查的执行结果
    """
    from pyrmm.cli.rmmcore import cli_with_output
    
    # 使用项目路径切换执行命令
    with with_project_directory(project_name) as ctx:
        try:
            # 执行命令并获取详细输出
            result_output = cli_with_output(["check"])
            
            # 使用实际的命令输出
            if result_output and isinstance(result_output, str):
                stdout_message = result_output
            else:
                stdout_message = "语法检查完成"
            
            return {
                "success": True,
                "stdout": stdout_message,
                "stderr": "",
                "command": "rmm check",
                "work_dir": ctx["work_dir"],
                "method": "rust_extension_with_output"
            }
        except Exception as rust_error:
            return {
                "success": False,
                "stdout": "",
                "stderr": str(rust_error),
                "command": "rmm check",
                "work_dir": ctx["work_dir"],
                "method": "rust_extension",
                "error": f"Rust 扩展执行失败: {rust_error}"
            }

@mcp.tool()
def cleanProject(project_name: str | None = None, deep: bool = False):
    """
    使用 rmm clean 命令清理项目构建产物。
    
    参数:
        project_name: 可选的项目名称
        deep: 执行深度清理 (删除所有构建产物)
    
    返回:
        清理项目的执行结果
    """
    from pyrmm.cli.rmmcore import cli_with_output
    
    # 使用项目路径切换执行命令
    with with_project_directory(project_name) as ctx:
        try:
            # 构建参数列表
            args = ["clean"]
            if deep:
                args.append("--deep")
            
            # 执行命令并获取详细输出
            result_output = cli_with_output(args)
            
            # 使用实际的命令输出
            if result_output and isinstance(result_output, str):
                stdout_message = result_output
            else:
                stdout_message = "项目清理成功"
            
            return {
                "success": True,
                "stdout": stdout_message,
                "stderr": "",
                "command": f"rmm {' '.join(args)}",
                "work_dir": ctx["work_dir"],
                "method": "rust_extension_with_output"
            }
        except Exception as rust_error:
            return {
                "success": False,
                "stdout": "",
                "stderr": str(rust_error),
                "command": f"rmm {' '.join(args)}",
                "work_dir": ctx["work_dir"],
                "method": "rust_extension",
                "error": f"Rust 扩展执行失败: {rust_error}"
            }

@mcp.tool()
def syncProjects(
    project_name: str | None = None,
    projects_only: bool = False,
    search_paths: list[str] | None = None,
    max_depth: int | None = None
):
    """
    使用 rmm sync 命令同步项目元数据。
    
    参数:
        project_name: 可选的项目名称 (如果未提供，则尝试同步所有项目)
        projects_only: 仅同步项目列表（发现新项目，移除无效项目），跳过依赖同步
        search_paths: 指定搜索项目的路径列表
        max_depth: 搜索项目的最大目录深度
    
    返回:
        同步项目的执行结果
    """
    from pyrmm.cli.rmmcore import cli_with_output
    
    try:
        # 构建参数列表
        args = ["sync"]
        
        # 添加项目名称（如果提供）
        if project_name:
            args.append(project_name)
        
        # 添加 --projects 标志
        if projects_only:
            args.append("--projects")
        
        # 添加搜索路径（可多次使用）
        if search_paths:
            for path in search_paths:
                args.extend(["--search-path", path])
        
        # 添加最大深度
        if max_depth is not None:
            args.extend(["--max-depth", str(max_depth)])
        
        # 执行命令并获取详细输出
        result_output = cli_with_output(args)
        
        # 使用实际的命令输出
        if result_output and isinstance(result_output, str):
            stdout_message = result_output
        else:
            stdout_message = "项目同步成功"
        
        return {
            "success": True,
            "stdout": stdout_message,
            "stderr": "",
            "command": f"rmm {' '.join(args)}",
            "method": "rust_extension_with_output"
        }
    except Exception as rust_error:
        return {
            "success": False,
            "stdout": "",
            "stderr": str(rust_error),
            "command": f"rmm {' '.join(args)}",
            "method": "rust_extension",
            "error": f"Rust 扩展执行失败: {rust_error}"
        }

@mcp.tool()
def publishRelease(project_name: str | None = None, draft: bool = False, prerelease: bool = False):
    """
    使用 rmm publish 命令发布版本到 GitHub。
    
    参数:
        project_name: 可选的项目名称
        draft: 创建为草稿版本
        prerelease: 标记为预发布版本
    
    返回:
        rmm publish 命令的执行结果
    """
    from pyrmm.cli.rmmcore import cli_with_output
    
    # 使用项目路径切换执行命令
    with with_project_directory(project_name) as ctx:
        try:
            # 构建参数列表
            args = ["publish"]
            if draft:
                args.append("--draft")
            if prerelease:
                args.append("--prerelease")
            
            # 执行命令并获取详细输出
            result_output = cli_with_output(args)
            
            # 使用实际的命令输出
            if result_output and isinstance(result_output, str):
                stdout_message = result_output
            else:
                stdout_message = "版本发布成功"
            
            return {
                "success": True,
                "stdout": stdout_message,
                "stderr": "",
                "command": f"rmm {' '.join(args)}",
                "work_dir": ctx["work_dir"],
                "method": "rust_extension_with_output"
            }
        except Exception as rust_error:
            return {
                "success": False,
                "stdout": "",
                "stderr": str(rust_error),
                "command": f"rmm {' '.join(args)}",
                "work_dir": ctx["work_dir"],
                "method": "rust_extension",
                "error": f"Rust 扩展执行失败: {rust_error}"
            }

@mcp.tool()
def listDevices():
    """
    使用 rmm device list 命令列出已连接的 ADB 设备。
    
    返回:
        已连接设备的列表
    """
    from pyrmm.cli.rmmcore import cli_with_output
    
    try:
        # 执行命令并获取详细输出
        result_output = cli_with_output(["device", "list"])
        
        # 使用实际的命令输出
        if result_output and isinstance(result_output, str):
            stdout_message = result_output
        else:
            stdout_message = "设备列表获取成功"
        
        return {
            "success": True,
            "stdout": stdout_message,
            "stderr": "",
            "command": "rmm device list",
            "method": "rust_extension_with_output"
        }
    except Exception as rust_error:
        return {
            "success": False,
            "stdout": "",
            "stderr": str(rust_error),
            "command": "rmm device list",
            "method": "rust_extension",
            "error": f"Rust 扩展执行失败: {rust_error}"
        }

@mcp.tool()
def getDeviceInfo(device_id: str | None = None):
    """
    获取已连接设备的详细信息。
    
    参数:
        device_id: 可选的设备 ID (如果未提供，则使用第一个可用设备)
    
    返回:
        设备信息和状态
    """
    from pyrmm.cli.rmmcore import cli_with_output
    
    try:
        # 构建参数列表
        args = ["device", "info"]
        if device_id:
            args.append(device_id)
        else:
            # 如果没有提供设备ID，先获取设备列表
            devices_result = cli_with_output(["device", "list"])
            if devices_result:
                # 这里可以解析设备列表并选择第一个设备
                # 暂时返回提示信息
                return {
                    "success": False,
                    "stdout": "",
                    "stderr": "请提供设备ID",
                    "command": "rmm device info",
                    "method": "rust_extension",
                    "error": "需要指定设备ID"
                }
        
        # 执行命令并获取详细输出
        result_output = cli_with_output(args)
        
        # 使用实际的命令输出
        if result_output and isinstance(result_output, str):
            stdout_message = result_output
        else:
            stdout_message = "设备信息获取成功"
        
        return {
            "success": True,
            "stdout": stdout_message,
            "stderr": "",
            "command": f"rmm {' '.join(args)}",
            "method": "rust_extension_with_output"
        }
    except Exception as rust_error:
        return {
            "success": False,
            "stdout": "",
            "stderr": str(rust_error),
            "command": f"rmm device info {device_id or ''}",
            "method": "rust_extension",
            "error": f"Rust 扩展执行失败: {rust_error}"
        }

@mcp.tool()
def installModule(project_name: str | None = None, device_id: str | None = None):
    """
    使用 rmm device install 命令将模块安装到设备。
    
    参数:
        project_name: 可选的项目名称
        device_id: 目标设备 ID
    
    返回:
        安装的执行结果
    """
    from pyrmm.cli.rmmcore import cli_with_output
    
    # 使用项目路径切换执行命令
    with with_project_directory(project_name) as ctx:
        try:
            # 构建参数列表
            args = ["device", "install"]
            if device_id:
                args.append(device_id)
            
            # 需要指定模块路径，这里假设使用构建后的模块
            # 实际实现中可能需要先检查构建产物
            module_path = f"{ctx['work_dir']}/.rmmp/dist/module.zip"
            args.append(module_path)
            
            # 执行命令并获取详细输出
            result_output = cli_with_output(args)
            
            # 使用实际的命令输出
            if result_output and isinstance(result_output, str):
                stdout_message = result_output
            else:
                stdout_message = "模块安装成功"
            
            return {
                "success": True,
                "stdout": stdout_message,
                "stderr": "",
                "command": f"rmm {' '.join(args)}",
                "work_dir": ctx["work_dir"],
                "method": "rust_extension_with_output"
            }
        except Exception as rust_error:
            return {
                "success": False,
                "stdout": "",
                "stderr": str(rust_error),
                "command": f"rmm device install {device_id or ''} {project_name or ''}",
                "work_dir": ctx["work_dir"],
                "method": "rust_extension",
                "error": f"Rust 扩展执行失败: {rust_error}"
            }

@mcp.tool()
def testModule(project_name: str | None = None, device_id: str | None = None, interactive: bool = False):
    """
    使用 rmm test 命令在设备上测试模块。
    
    参数:
        project_name: 可选的项目名称
        device_id: 目标设备 ID (adb 设备)
        interactive: 启用交互模式
    
    返回:
        rmm test 命令的执行结果
    """
    from pyrmm.cli.rmmcore import cli_with_output
    
    # 使用项目路径切换执行命令
    with with_project_directory(project_name) as ctx:
        try:
            # 构建参数列表
            args = ["device", "test"]
            if device_id:
                args.append(device_id)
            if interactive:
                args.append("--interactive")
            
            # 执行命令并获取详细输出
            result_output = cli_with_output(args)
            
            # 使用实际的命令输出
            if result_output and isinstance(result_output, str):
                stdout_message = result_output
            else:
                stdout_message = "模块测试完成"
            
            return {
                "success": True,
                "stdout": stdout_message,
                "stderr": "",
                "command": f"rmm {' '.join(args)}",
                "work_dir": ctx["work_dir"],
                "method": "rust_extension_with_output"
            }
        except Exception as rust_error:
            return {
                "success": False,
                "stdout": "",
                "stderr": str(rust_error),
                "command": f"rmm device test {device_id or ''} {project_name or ''}",
                "work_dir": ctx["work_dir"],
                "method": "rust_extension",
                "error": f"Rust 扩展执行失败: {rust_error}"
            }

@mcp.tool()
def runCustomScript(project_name: str | None = None, script_type: str = "service"):
    """
    使用 rmm run 命令在项目中运行自定义脚本。
    
    参数:
        project_name: 可选的项目名称
        script_type: 要运行的脚本类型 (service, post_fs_data, late_start)
    
    返回:
        运行脚本的执行结果
    """
    from pyrmm.cli.rmmcore import cli_with_output
    
    # 使用项目路径切换执行命令
    with with_project_directory(project_name) as ctx:
        try:
            # 构建参数列表
            args = ["run", script_type]
            
            # 执行命令并获取详细输出
            result_output = cli_with_output(args)
            
            # 使用实际的命令输出
            if result_output and isinstance(result_output, str):
                stdout_message = result_output
            else:
                stdout_message = f"脚本 {script_type} 运行成功"
            
            return {
                "success": True,
                "stdout": stdout_message,
                "stderr": "",
                "command": f"rmm {' '.join(args)}",
                "work_dir": ctx["work_dir"],
                "method": "rust_extension_with_output"
            }
        except Exception as rust_error:
            return {
                "success": False,
                "stdout": "",
                "stderr": str(rust_error),
                "command": f"rmm run {script_type}",
                "work_dir": ctx["work_dir"],
                "method": "rust_extension",
                "error": f"Rust 扩展执行失败: {rust_error}"
            }

@mcp.tool()
def generateCompletion(shell: str = "powershell"):
    """
    使用 rmm completion 命令生成 shell 补全脚本。
    
    参数:
        shell: 目标 shell (bash, zsh, fish, powershell)
    
    返回:
        生成的补全脚本
    """
    from pyrmm.cli.rmmcore import cli_with_output
    
    try:
        # 构建参数列表
        args = ["completion", shell]
        
        # 执行命令并获取详细输出
        result_output = cli_with_output(args)
        
        # 使用实际的命令输出
        if result_output and isinstance(result_output, str):
            stdout_message = result_output
        else:
            stdout_message = f"{shell} 补全脚本生成成功"
        
        return {
            "success": True,
            "stdout": stdout_message,
            "stderr": "",
            "command": f"rmm {' '.join(args)}",
            "shell": shell,
            "method": "rust_extension_with_output"
        }
    except Exception as rust_error:
        return {
            "success": False,
            "stdout": "",
            "stderr": str(rust_error),
            "command": f"rmm completion {shell}",
            "shell": shell,
            "method": "rust_extension",
            "error": f"Rust 扩展执行失败: {rust_error}"
        }

@mcp.tool()
def getRMMConfig():
    """
    使用 rmm config list 命令获取 RMM 配置。
    
    返回:
        当前 RMM 配置
    """
    from pyrmm.cli.rmmcore import cli_with_output
    
    try:
        # 执行命令并获取详细输出
        result_output = cli_with_output(["config", "list"])
        
        # 使用实际的命令输出
        if result_output and isinstance(result_output, str):
            stdout_message = result_output
        else:
            stdout_message = "配置获取成功"
        
        return {
            "success": True,
            "stdout": stdout_message,
            "stderr": "",
            "command": "rmm config list",
            "method": "rust_extension_with_output"
        }
    except Exception as rust_error:
        return {
            "success": False,
            "stdout": "",
            "stderr": str(rust_error),
            "command": "rmm config list",
            "method": "rust_extension",
            "error": f"Rust 扩展执行失败: {rust_error}"
        }

@mcp.tool()
def setRMMConfig(key: str, value: str):
    """
    使用 rmm config set 命令设置 RMM 配置。
    
    参数:
        key: 配置键
        value: 配置值
    
    返回:
        设置配置的执行结果
    """
    from pyrmm.cli.rmmcore import cli_with_output
    
    try:
        # 构建参数列表
        args = ["config", "set", key, value]
        
        # 执行命令并获取详细输出
        result_output = cli_with_output(args)
        
        # 使用实际的命令输出
        if result_output and isinstance(result_output, str):
            stdout_message = result_output
        else:
            stdout_message = f"配置 {key} 设置成功"
        
        return {
            "success": True,
            "stdout": stdout_message,
            "stderr": "",
            "command": f"rmm {' '.join(args)}",
            "method": "rust_extension_with_output"
        }
    except Exception as rust_error:        return {
            "success": False,
            "stdout": "",
            "stderr": str(rust_error),
            "command": f"rmm config set {key} {value}",
            "method": "rust_extension",
            "error": f"Rust 扩展执行失败: {rust_error}"
        }
# Resources for providing documentation and help
@mcp.resource("docs://rmm-cli-help")
def rmmHelp():
    """
    RMM (Root Module Manager) 完整帮助文档和使用指南
    """
    from pyrmm.ai.resources import RMMCLIHELP
    return RMMCLIHELP

@mcp.resource("docs://magisk-module-guide")
def magiskModuleGuide():
    """
    Magisk 模块开发指南和最佳实践
    """
    from pyrmm.ai.resources import MODULEDEVGUIDE
    return MODULEDEVGUIDE

@mcp.resource("docs://shell-script-best-practices")
def shellScriptBestPractices():
    """
    Shell 脚本编写最佳实践和 ShellCheck 规范
    """
    from pyrmm.ai.resources import SHELLSCRIPTBESTPRACTICES
    return SHELLSCRIPTBESTPRACTICES

def start_mcp_server(transport: str = "stdio", host: str = "localhost", port: int = 8000, verbose: bool = False):
    """
    启动 MCP 服务器的入口函数
    
    参数:
        transport: 传输方式 ("stdio" 或 "sse")
        host: 服务器主机地址 (仅用于 sse 模式)
        port: 服务器端口 (仅用于 sse 模式)
        verbose: 是否启用详细日志
    """
    
    #region 注册mcp功能！！！
    mcp.host = host
    mcp.port = port
    print("🚀 启动 RMM MCP 服务器...")
    print(f"📡 传输方式: {transport}")
    if transport == "sse":
        print(f"📍 地址: {host}:{port}")
    try:
        if transport == "stdio":
            mcp.run(transport="stdio")
        elif transport == "sse":
            mcp.run(transport="sse")
        else:
            raise ValueError(f"不支持的传输方式: {transport}")
    except KeyboardInterrupt:
        if verbose:
            print("\n👋 MCP 服务器已停止")
    except Exception as e:
        if verbose:
            print(f"❌ MCP 服务器错误: {e}")
        raise


if __name__ == "__main__":
    start_mcp_server()