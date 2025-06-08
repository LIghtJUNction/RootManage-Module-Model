import click

@click.group()
def config():
    """Pyrmm Config Command group"""
    pass

from pyrmm.cli.config.ls import ls
config.add_command(ls)

from pyrmm.cli.config.setvalue import setvalue
config.add_command(setvalue)

from pyrmm.cli.config.delete import delete
config.add_command(delete)
