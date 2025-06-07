from pathlib import Path
import click
from typing import Literal

from pyrmm.config import Config 
from pyrmm.__about__ import __version__
from pyrmm.config import RmmProject

@click.command()
@click.argument('rpath', type=str, default='MyModule')
@click.option('--magisk','-m', "rtype" , flag_value = "magisk" , default=True )
@click.option('--ksu', '-k' ,'rtype', flag_value='ksu')
@click.option('--apatch', '-a', 'rtype', flag_value='apu')
def init(rtype: Literal["magisk","ksu","apu"], rpath: str | Path, name: str | None = None):
    """Initialize a new RMM project"""
    rpath = Path(rpath).resolve()
    thisproject = RmmProject(rpath, rtype)
    rmmconfig = Config()
    
    # 检查配置中的默认值并提示用户设置
    for key in Config.DEFAULTS:
        if Config.is_default(key):
            current_value = Config.DEFAULTS[key]
            click.echo(f"⚠️  {key} is using default value: {current_value}")
            if click.confirm(f"Do you want to set a custom value for {key}?", default=False):
                value = click.prompt(f"Enter a value for {key}", type=str, default=current_value)
                setattr(rmmconfig, key, value)
    
    thisproject.new()
    thisproject.save()

    info = f"""
    RMM Project initialized successfully!
    Project Path: {thisproject.path}
    Project Type: {thisproject.rtype}
    Project ID: {thisproject.id}
    Config Root: {rmmconfig.rmmroot}
    Version: {__version__}
    
    You can now start working on your RMM project!
    """
    
    click.echo(info)





