import click

@click.command()
def sign():
    """ 配置签名 """
    click.echo("此命令用于配置签名。请使用 --help 查看详细用法。")
    