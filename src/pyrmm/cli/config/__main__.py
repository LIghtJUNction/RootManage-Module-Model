import click
from pyrmm.usr.lib.config import Config
from pyrmm.usr.lib.project import RmmProject


@click.group()
def config():
    """Pyrmm Config Command group"""
    pass

@config.command()
@click.argument("project_name", required=False)
def ls(project_name: str | None = None):
    """List all configuration keys"""
    if project_name:
        info = getattr(RmmProject, project_name, None)
        if info:
            public_attrs = dir(info)
            for attr in public_attrs:
                value = getattr(info, attr)
                print(f"  {attr}: {value}")
        else:
            print(f"项目 '{project_name}' 不存在。")
    else:
        # 获取 Config 的属性列表
        public_attrs = dir(Config)
        # 打印每个属性
        for attr in public_attrs:
            value = getattr(Config, attr)
            print(f"  {attr}: {value}")


@config.command("del")
@click.argument("keys", nargs=-1)
def delete(keys: list[str] | None = None):
    """Delete configuration keys"""
    if not keys:
        keys = dir(Config)
    for key in keys:
        delattr(Config, key)
        print(f"已删除配置项: {key}")


@config.command("set")
@click.argument("kvpairs", nargs=-1)
def setvalue(kvpairs: list[str]):
    """Set a configuration key to a value"""
    for pair in kvpairs:
        key, value = pair.split('=', 1)
        setattr(Config, key, value)
        print(f"已设置配置项 {key} 的值为 {value}")