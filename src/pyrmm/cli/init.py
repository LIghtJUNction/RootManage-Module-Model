from pathlib import Path
import click

@click.command()
@click.argument("project_path", default=".")
@click.option("--yes", "-y", is_flag=True, help="跳过确认提示")
@click.option("--basic", "-b", "rtype", flag_value="basic", default=True, help="初始化一个基本的RMM项目")
@click.option("--lib", "-l", "rtype", flag_value="library", default=False, help="初始化一个RMM库项目(模块)")
@click.option("--ravd", "-r", "rtype", flag_value="ravd", default=False, help="初始化一个RMM Android Virtrual Device (RAVD)(测试模块用安卓虚拟系统)")
def init(project_path: str, yes: bool, rtype: str) -> None:
    """初始化RMM项目
    
    PROJECT_PATH: 项目路径 (默认为当前目录)
    """
    # 延迟导入 - 只在需要时导入
    from pyrmm.usr.lib.project import RmmProject
    
    rpath = Path(project_path).resolve()
    
    click.echo(f"正在初始化 {rtype} 类型的RMM项目到: {rpath}")
    
    match rtype:
        case "basic":
            result = RmmProject.init_basic(rpath)
            click.echo(f"✅ {result['message']}")
        case "library":
            result = RmmProject.init_library(rpath)
            click.echo(f"✅ {result['message']}")
        case _:
            click.echo("❌ 无效的模块类型。")
            return
    
    click.echo(f"✅ 项目 '{rpath.name}' 初始化完成！")


