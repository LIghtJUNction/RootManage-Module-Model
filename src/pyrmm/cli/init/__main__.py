from pathlib import Path
import click
import toml

from pyrmm.__about__ import __version__

@click.group()
@click.argument('type', type=click.Choice(['magisk', 'ksu','apatch'], case_sensitive=False), default='magisk')
@click.argument('id', type=str, default='MyModule', help='Module ID (default: MyModule)')
def init(type: str, name: str, id: str):
    """Pyrmm Init Command group"""
    if id == 'MyModule':
        id = click.prompt('请输入模块 ID', default='MyModule', show_default=True, type=str)
    click.echo(f"Initializing {type} module: {name} (ID: {id})")
    # 创建目录

    CWD = Path.cwd()

    RMM_VERSION: Path = CWD / id / ".rmm_version"
    RMM_VERSION.write_text(__version__)

    RWW_PROJECT: Path = CWD / id / "rmmproject.toml"
    rmmp_meta : dict[str, dict[str, str | list[dict[str, str]] | dict[str,str]]] = {
        'rmmproject': {
            'name': name,
            'id': id,
            'type': type,
            'requires-rmm': f">={__version__}",
            'description': f'{name} Module',
            'authors':  [
                {
                    'name': 'Your Name',
                    'email': 'your.email@example.com'
                }
            ],
            'license': 'None',
            'readme': 'README.MD',
            'changelog': 'CHANGELOG.MD',
            'dependencies': [],
            'scripts': {
                "build": "rmm build",
            }
        }
    }






