import click

@click.command("del")
@click.argument("keys", nargs=-1)
def delete(keys: list[str] | None = None):
    """Delete configuration keys"""
    from pyrmm.usr.lib.config import Config
    if not keys:
        keys = dir(Config)
    for key in keys:
        delattr(Config, key)
        print(f"已删除配置项: {key}")