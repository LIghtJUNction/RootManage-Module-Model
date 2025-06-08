import click

@click.group()
def clean():
    """Pyrmm Clean Command group"""
    pass


from pyrmm.cli.clean.dist import dist
clean.add_command(dist)

from pyrmm.cli.clean.tags import tags
clean.add_command(tags)