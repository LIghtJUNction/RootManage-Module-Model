# Author : LIghtJUNction
from pathlib import Path
import click
from pyrmm.usr.lib.fs import RmmFileSystem

@click.command()
@click.option('--project-path', '-p',"_project_path" ,type=click.Path(exists=True, file_okay=False, dir_okay=True), default='.', help='项目路径，默认为当前目录')
@click.option('--yes', '-y', is_flag=True, help='自动确认所有提示，跳过交互式确认')
@click.option('--verbose', '-v', is_flag=True, help='显示详细输出')
def basic(_project_path: str , yes: bool, verbose: bool) -> None:
    """静态检测项目的所有 shell 脚本！"""
    project_path: Path = Path(_project_path).resolve()

    project_name: str = project_path.name

    click.echo(f"🔍 正在检测项目: {project_name} ({project_path})"
               f"\n  - 自动确认: {'开启' if yes else '关闭'}"
               f"\n  - 详细模式: {'开启' if verbose else '关闭'}")
    # 检查是否安装shellcheck
    try:
        import subprocess
        result = subprocess.run(['shellcheck', '--version'], capture_output=True, text=True)
        if result.returncode != 0:
            click.echo("❌ 错误: 未安装 shellcheck，请先安装 shellcheck。")
            return
    except FileNotFoundError:
        click.echo("❌ 错误: 未找到 shellcheck，请先安装 shellcheck。")
        click.echo("安装方法：\n  - rmm install shellcheck\n  - 或者使用系统包管理器安装 到任意$PATH路径即可")
        return
    
    