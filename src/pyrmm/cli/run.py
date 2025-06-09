import click

@click.command()
@click.argument("script_name", required=False)
def run(script_name: str | None) -> None:
    """灵感来自npm"""
    # 延迟导入 - 减少启动时间
    import subprocess
    from pathlib import Path
    from pyrmm.usr.lib.project import RmmProject
    project_info = RmmProject.project_info(Path.cwd())
    if not project_info:
        click.echo("❌ 当前目录不是一个有效的RMM项目")
        return

    if not script_name:
        click.echo("❌ 请指定要运行的脚本名称")
        return    # 处理scripts配置，支持数组格式 [[scripts]] 和字典格式 [scripts]
    scripts_config = project_info.get("scripts", [])
    scripts_dict: dict[str, str] = {}
    
    if isinstance(scripts_config, list):
        # 数组格式：[[scripts]]
        for script_item in scripts_config:
            if isinstance(script_item, dict):
                # 合并字典项
                scripts_dict.update(script_item)  # type: ignore
    elif isinstance(scripts_config, dict):
        # 字典格式：[scripts]
        scripts_dict = scripts_config  # type: ignore

    if script_name not in scripts_dict:
        click.echo(f"❌ 脚本 '{script_name}' 未定义！")
        click.echo("可用脚本列表:")
        for name in scripts_dict.keys():
            click.echo(f"  - {name}")
        return

    script_command = scripts_dict[script_name]
    click.echo(f"🔄 正在运行脚本: {script_name} (命令: {script_command})") 
    try:
        subprocess.run(script_command, shell=True, check=True)
    except subprocess.CalledProcessError as e:
        click.echo(f"❌ 脚本执行失败: {e}")