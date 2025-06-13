from pathlib import Path
import os
import toml
from typing import Any
from contextlib import contextmanager
from argparse import ArgumentParser
from mcp.server.fastmcp import FastMCP


class RmMcp(FastMCP):
    """
    RmMcp class that extends FastMCP with configuration management capabilities.
    """
    
    def __init__(self, name: str = "RmMcp", host: str = "localhost", port: int = 8000):
        super().__init__(name)
        self.host = host
        self.port = port
        # 默认值而已，可以改
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


# 创建全局 MCP 实例
mcp = RmMcp("RmMcp")


@contextmanager
def with_project_directory(project_name: str | None = None):
    """
    上下文管理器：切换到项目目录并在完成后恢复原目录
    
    参数:
        project_name: 可选的项目名称，如果未提供则使用第一个可用项目
    
    返回:
        返回一个上下文管理器，提供项目路径和工作目录切换
    """
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


# 测试工具函数
@mcp.tool()
def echo(message: str = "world") -> str:
    """
    回显消息
    :param message: 要回显的消息
    :return: 回显的消息
    """
    return f"Echo: {message}"


@mcp.tool()
def getRmmMeta():
    """
    获取 RMM (Magisk 模块项目管理) 的元数据。
    """
    return mcp.META


@mcp.tool()
def getRmmProjects():
    """
    获取 RMM (Magisk 模块项目管理) 的项目列表。
    """
    return mcp.projects


@mcp.tool()
def getRmmRoot():
    """
    获取 RMM 配置的根目录。
    """
    return str(mcp.ROOT)


@mcp.tool()
def testProjectContext(project_name: str | None = None):
    """
    测试项目上下文管理器功能
    
    参数:
        project_name: 可选的项目名称
    
    返回:
        上下文切换的测试结果
    """
    original_cwd = os.getcwd()
    
    try:
        with with_project_directory(project_name) as ctx:
            current_cwd = os.getcwd()
            return {
                "success": True,
                "original_cwd": original_cwd,
                "current_cwd": current_cwd,
                "context_info": ctx,
                "test_result": "上下文管理器工作正常"
            }
    except Exception as e:
        return {
            "success": False,
            "error": str(e),
            "original_cwd": original_cwd,
            "current_cwd": os.getcwd()
        }














def parse_args():
    """解析命令行参数"""
    parser = ArgumentParser(description="Run the RMM MCP server.")
    parser.add_argument("--transport", "-t", type=str, default="stdio", 
                       choices=["stdio", "sse"], help="传输方式 (default: stdio)")
    parser.add_argument("--port", "-p", type=int, default=8000, 
                       help="SSE 端口 (default: 8000)")
    parser.add_argument("--host", "-H", type=str, default="localhost", 
                       help="SSE 主机 (default: localhost)")
    parser.add_argument("--verbose", "-v", action="store_true", 
                       help="启用详细日志")
    return parser.parse_args()


def start_mcp_server(transport: str = "stdio", host: str = "localhost", port: int = 8000, verbose: bool = False):
    """
    启动 MCP 服务器的入口函数
    
    参数:
        transport: 传输方式 ("stdio" 或 "sse")
        host: 服务器主机地址 (仅用于 sse 模式)
        port: 服务器端口 (仅用于 sse 模式)
        verbose: 是否启用详细日志
    """
    mcp.host = host
    mcp.port = port
    
    if verbose:
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


def rmmcp():
    """
    主入口函数，用于 project.scripts 配置
    """
    args = parse_args()
    print("🚀 启动 RMM MCP 服务器... 输入rmmcp -h 查看帮忙")

    start_mcp_server(
        transport=args.transport,
        host=args.host,
        port=args.port,
        verbose=args.verbose
    )


if __name__ == "__main__":
    rmmcp()