import click

@click.group()
@click.argument('pairs', nargs=-1, required=False)
@click.option('-r', '--read', 'read_mode', is_flag=True, help='读取模式')
@click.option('-w', '--write', 'write_mode', is_flag=True, help='写入模式')
@click.pass_context
def config(ctx: click.Context, pairs: tuple[str], read_mode: bool, write_mode: bool):
    """Pyrmm Config Command group"""
    ctx.ensure_object(dict)
    ctx.obj['pairs'] = pairs
    ctx.obj['read_mode'] = read_mode
    ctx.obj['write_mode'] = write_mode

    if write_mode:
        click.echo("写入配置:")
        for pair in pairs:
            if '=' not in pair:
                raise click.BadParameter(f"写入模式需要 key=value 格式，得到: '{pair}'")
            key, value = pair.split('=', 1)
            click.echo(f"  {key} = {value}")
    
    elif read_mode:
        click.echo("读取配置:")
        for key in pairs:
            if '=' in key:
                raise click.BadParameter(f"读取模式只需要键名，得到: '{key}'")
            click.echo(f"  {key} = (配置值)")
    else:
        raise click.UsageError("请指定 -r (读取) 或 -w (写入) 模式")



if __name__ == '__main__':
    config()