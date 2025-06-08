import click
from pyrmm.__about__ import __version__
@click.group()
@click.option('-p', '--profile', help='指定配置文件')
@click.option('-t' , '--token' , envvar="GITHUB_ACCESS_TOKEN", help='指定GITHUB访问令牌')
@click.version_option(version=__version__, message=f'Pyrmm CLI version {__version__}')
@click.help_option('-h', '--help', help='显示帮助信息')
@click.option('--debug/--no-debug', default=False, help='启用调试模式')
@click.pass_context
def cli(ctx: click.Context, profile: str, token: str, debug: bool):
    """Pyrmm : Magisk Apath kernelsu module devkit"""
    # 确保上下文对象存在
    ctx.ensure_object(dict)
    
    # 将参数存储到上下文中，供子命令使用
    ctx.obj['profile'] = profile
    ctx.obj['token'] = token
    ctx.obj['debug'] = debug
    
    if debug:
        click.echo(f"调试模式已启用，配置文件: {profile}")

# import sub command groups
from pyrmm.cli.build import build
cli.add_command(build)
"""
构建 模块
"""

from pyrmm.cli.init import init
cli.add_command(init)
"""
初始化 Pyrmm 模块 项目
"""

from pyrmm.cli.sync import sync
cli.add_command(sync)
"""
同步 & 刷新 Pyrmm 模块 项目
"""

from pyrmm.cli.config import config
cli.add_command(config)
"""
配置 Pyrmm 模块 项目
"""

from pyrmm.cli.publish import publish
cli.add_command(publish)
"""
发布 Pyrmm 模块 项目
"""

from pyrmm.cli.clean import clean
cli.add_command(clean)
"""
清理 Pyrmm 模块 项目
"""

from pyrmm.cli.test import test
cli.add_command(test)
"""
测试 Pyrmm 模块 项目
"""






if __name__ == '__main__':
    cli()