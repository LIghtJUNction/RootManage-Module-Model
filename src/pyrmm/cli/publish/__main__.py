import click

@click.group()
@click.option("--token",envvar = "GITHUB_ACCESS_TOKEN", help="GitHub访问令牌，默认从环境变量GITHUB_ACCESS_TOKEN获取")
@click.option("--yes", "-y", is_flag=True, help="自动同意所有确认提示，不需要用户交互")
@click.pass_context
def publish(ctx: click.Context, token: str, yes: bool):
    """发布RMM项目的命令行工具"""
    # 将token和yes标志存储在context中，供子命令使用
    ctx.ensure_object(dict)
    ctx.obj['token'] = token
    ctx.obj['yes'] = yes



from pyrmm.cli.publish.github import github
publish.add_command(github)

from pyrmm.cli.publish.repo import repo
publish.add_command(repo)



if __name__ == "__main__":
    publish()



