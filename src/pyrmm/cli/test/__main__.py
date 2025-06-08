import click

@click.group()
def test():
    """Pyrmm Test Command group"""
    pass

from pyrmm.cli.test.github import github
test.add_command(github)