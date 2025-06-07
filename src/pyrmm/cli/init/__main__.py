from pathlib import Path
import click
from pyrmm.usr.lib.project import RmmProject

@click.group()
def init() -> None:
    """Pyrmm Init Command group"""
    pass

@init.command()
@click.option("--yes", "-y", is_flag=True, help="跳过确认提示")
@click.option("--rpath", "-r", default=".", help="项目路径")
@click.option("--basic","-b","rtype",flag_value="basic",default=True, help="初始化一个基本的RMM项目")
@click.option("--lib","-l","rtype",flag_value="library",default=False, help="初始化一个RMM库项目(模块)")
def module(yes: bool, rpath: str, rtype: str) -> None:
    """ 初始化RMM模块 """
    Rpath = Path(rpath).resolve()
    match rtype:
        case "basic":
            RmmProject.init_basic(Rpath)
        case "library":
            RmmProject.init_library(Rpath)
        case _:            click.echo("无效的模块类型。")

    pass 


@init.command()
def ravd():
    """ 初始化RAVD: RMM ANDROID VIRTUAL DEVICE """
    click.echo("RAVD 正在开发中...")


