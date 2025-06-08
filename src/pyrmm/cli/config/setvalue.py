import click

@click.command("set")
@click.argument("kvpairs", nargs=-1)
def setvalue(kvpairs: list[str]):
    """Set a configuration key to a value"""
    from pyrmm.usr.lib.config import Config
    for pair in kvpairs:
        key, value = pair.split('=', 1)
        setattr(Config, key, value)
        print(f"已设置配置项 {key} 的值为 {value}")