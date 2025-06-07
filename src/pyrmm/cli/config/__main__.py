import click
from pyrmm.config import Config as RmmConfig

pass_config = click.make_pass_decorator(RmmConfig, ensure=True)
@click.group()
@pass_config
def config(rmmc: RmmConfig):
    """Pyrmm Config Command group"""
    pass

@config.command()
@click.option('--projects', '-p' ,"range", flag_value="projects-only" , help="仅显示项目列表")
@click.option('--rmmroot', '-r',"range", flag_value="rmmroot-only", help="仅显示RMMROOT")
@click.option('--all', '-a',"range", flag_value="all", help="显示所有配置")
@click.option('--info', '-i',"range", flag_value="info", help="显示关键信息",default=True)
@pass_config
def ls(rmmc: RmmConfig, range: str):
    """列出当前配置"""
    match range:
        case "projects-only":
            click.echo("当前项目列表:")
            for project in rmmc.projects:
                click.echo(f" - {project} : <Path>{rmmc.projects[project]}</Path>")
        case "rmmroot-only":
            click.echo(f"当前 RMMROOT: {rmmc.rmmroot}")
        case "all":
            click.echo("当前配置:")
            click.echo(f"RMMROOT: {rmmc.rmmroot}")
            click.echo("项目列表:")
            for project in rmmc.projects:
                click.echo(f" - {project} : <Path>{rmmc.projects[project]}</Path>")
        case "info":
            click.echo("关键信息:")
            click.echo(f"RMMROOT: {rmmc.rmmroot}")
            click.echo(f"用户名(使用rmm config set username=xxx修改): {rmmc.username}") if rmmc.username == "Your Name" else click.echo(f"用户名: {rmmc.username}")
            click.echo(f"电子邮件(使用rmm config set email=xxx修改): {rmmc.email}") if rmmc.email == "dev@rmmp.com" else click.echo(f"电子邮件: {rmmc.email}")

        case _:
            click.echo("无效的范围选项")


@config.command()
@click.argument('config_pairs', nargs=-1, required=True, 
                metavar='KEY=VALUE...')
@pass_config
def set(rmmc: RmmConfig, config_pairs: tuple[str, ...]):
    """设置配置项，格式为 KEY=VALUE KEY2=VALUE2"""
    if not config_pairs:
        click.echo("错误：请提供至少一个配置项")
        return
    pair: str = ""
    for pair in config_pairs:
        if '=' not in pair:
            click.echo(f"错误：无效格式 '{pair}'，应为 KEY=VALUE")
            continue
        key: str 
        value: str
        key, value= pair.split('=', 1)  # 只分割第一个=号
        key = key.strip()
        value = value.strip()
        setattr(rmmc, key, value)
        click.echo(f"已设置 {key} = {value}")
    

@config.command('del')
@click.argument('configs', nargs=-1, required=True, 
                metavar='KEY...')
@click.option('--use-default','-u',"use", is_flag=True, help="删除后改为使用默认值")
@pass_config
def delete(rmmc: RmmConfig, configs: tuple[str, ...],use: bool):
    """删除配置项
    例如: rmm config del username email
    这将删除用户名和电子邮件配置项
    """
    for key in configs:
        if hasattr(rmmc, key):
            if rmmc.has_default(key) and use:
                rmmc.set_default(key)
            else:
                delattr(rmmc, key)
            click.echo(f"已删除 {key}")
        else:
            click.echo(f"未找到配置项 {key}")



if __name__ == '__main__':
    config()