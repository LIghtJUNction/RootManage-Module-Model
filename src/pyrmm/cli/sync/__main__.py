import click
from pyrmm.usr.lib.project import RmmProject
from pyrmm.usr.lib.config import Config

@click.group()
@click.option("--update","-U", is_flag=True, help=" 如果依赖有升级，将依赖更新到最新版本（包括rmm自己） ")
def sync(update: bool):
    """RMM Sync CLI commands"""
    pass


@sync.command()
def project():
    """同步RMM项目"""
    click.echo("同步RMM项目...")
    projects: str | dict[str, str] = Config.projects
    if not projects or isinstance(projects, str):
        click.echo("没有找到任何RMM项目。")
        return
    for project in projects:
        click.echo(f"正在同步项目: {project}")
        try:
            RmmProject.sync(project)
            click.echo(f"项目 {project} 同步成功。")
        except Exception as e:
            click.echo(f"项目 {project} 同步失败: {e}")