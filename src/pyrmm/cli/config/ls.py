import click

@click.command()
@click.argument("project_name", required=False)
def ls(project_name: str | None = None):
    """List all configuration keys"""
    from pyrmm.usr.lib.project import RmmProject
    from pyrmm.usr.lib.config import Config
    if project_name:
        try:
            # 尝试获取项目信息
            project_path = RmmProject.project_path(project_name)
            project_info = RmmProject.project_info(project_path)
            
            print(f"项目 '{project_name}' 的配置信息:")
            for key, value in project_info.items():
                print(f"  {key}: {value}")
        except (KeyError, FileNotFoundError, AttributeError) as e:
            print(f"项目 '{project_name}' 不存在或无法访问: {e}")
        except Exception as e:
            print(f"获取项目信息时出错: {e}")
    else:
        # 获取 Config 的公共配置属性
        print("系统配置:")
        config_attrs = ['username', 'email', 'version', 'projects']
        for attr in config_attrs:
            try:
                value = getattr(Config, attr)
                print(f"  {attr}: {value}")
            except AttributeError:
                print(f"  {attr}: <未设置>")