import click

@click.command()
@click.option("--all", is_flag=True, help="清理所有已注册的RMM项目，而不仅仅是当前目录的项目")
def dist(all: bool):
    """清理构建输出目录"""
    from pyrmm.usr.lib.project import RmmProject
    from pyrmm.usr.lib.config import Config
    from pathlib import Path

    if all:
        # 清理所有已注册的项目
        projects = Config.projects
        if not projects or isinstance(projects, str):
            click.echo("没有找到任何RMM项目。")
            return

        click.echo(f"🧹 清理所有已注册项目的构建输出目录...")
        for project_name in projects:
            try:
                project_path = RmmProject.project_path(project_name)
                RmmProject.clean_dist(project_path)
                click.echo(f"✅ 项目 {project_name} 的构建输出目录已清理。")
            except Exception as e:
                click.echo(f"❌ 清理项目 {project_name} 的构建输出目录失败: {e}")
    else:
        # 仅清理当前目录的项目（如果是RMM项目）
        current_path = Path.cwd()
        
        if not RmmProject.is_rmmproject(current_path):
            click.echo("❌ 当前目录不是一个有效的RMM项目。")
            click.echo("💡 提示：使用 --all 参数清理所有已注册的项目，或切换到RMM项目目录。")
            return
        
        try:
            RmmProject.clean_dist(current_path)
            click.echo(f"✅ 当前项目 {current_path.name} 的构建输出目录已清理。")
        except Exception as e:
            click.echo(f"❌ 清理当前项目构建输出目录失败: {e}")


