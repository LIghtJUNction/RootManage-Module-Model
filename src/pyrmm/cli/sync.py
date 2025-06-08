import click
from pathlib import Path
from typing import Any
from pyrmm.usr.lib.project import RmmProject
from pyrmm.usr.lib.config import Config
from pyrmm.usr.lib.proxy import ProxyManager

@click.command()
@click.argument("project_name", required=False)
@click.option("--update", "-U", is_flag=True, help="如果依赖有升级，将依赖更新到最新版本（包括rmm自己）")
@click.option("--all", "-a", "sync_all", is_flag=True, help="同步所有项目")
@click.option("--proxy", is_flag=True, help="获取GitHub代理地址列表并保存到项目元数据")
def sync(project_name: str | None, update: bool, sync_all: bool, proxy: bool) -> None:    
    """同步RMM项目
    
    PROJECT_NAME: 要同步的项目名称 (可选，如果不指定则需要使用 --all 参数)
    """    # 处理代理选项
    def handle_proxy_update(project_name: str) -> None:
        """获取代理列表并更新项目元数据"""
        if proxy or sync_all:
            try:
                click.echo("🌐 正在获取GitHub代理列表...")
                
                # 获取项目路径
                project_path = RmmProject.project_path(project_name)
                if not project_path:
                    click.echo(f"❌ 找不到项目 {project_name} 的路径")
                    return
                
                # 获取代理列表并保存到文件
                proxies, proxy_file = ProxyManager.get_and_save_proxies(project_path)
                
                if proxies:
                    # 获取项目元数据文件路径
                    meta_file = project_path / "rmmproject.toml"
                    if meta_file.exists():
                        # 读取现有元数据
                        import toml
                        with open(meta_file, 'r', encoding='utf-8') as f:
                            project_info = toml.load(f)
                        
                        # 设置代理文件的相对路径
                        relative_proxy_path = proxy_file.relative_to(project_path)
                        project_info["github_proxies"] = str(relative_proxy_path)
                        
                        # 保存更新的元数据
                        with open(meta_file, 'w', encoding='utf-8') as f:
                            toml.dump(project_info, f)
                        
                        click.echo(f"✅ 已获取到 {len(proxies)} 个GitHub代理节点")
                        click.echo(f"📁 代理数据已保存到: {relative_proxy_path}")
                        click.echo(f"🚀 最快代理: {proxies[0].url} (速度: {proxies[0].speed})")
                    else:
                        click.echo(f"❌ 找不到项目 {project_name} 的元数据文件")
                else:
                    click.echo("⚠️  未获取到有效的代理节点")
            except Exception as e:
                click.echo(f"❌ 获取代理列表失败: {e}")
    
    if sync_all:
        # 同步所有项目
        click.echo("同步所有RMM项目...")
        projects: str | dict[str, str] = Config.projects        
        if not projects or isinstance(projects, str):
            click.echo("没有找到任何RMM项目。")
            return
        for project in projects:
            click.echo(f"正在同步项目: {project}")
            try:
                # 处理代理更新
                handle_proxy_update(project)
                
                # 同步项目（版本更新现在在 RmmProject.sync 中处理）
                RmmProject.sync(project)
                click.echo(f"✅ 项目 {project} 同步成功。")
            except Exception as e:
                click.echo(f"❌ 项目 {project} 同步失败: {e}")

    elif project_name:
        # 同步指定项目
        click.echo(f"正在同步项目: {project_name}")
        try:
            # 处理代理更新
            handle_proxy_update(project_name)
            
            # 同步项目（版本更新现在在 RmmProject.sync 中处理）
            RmmProject.sync(project_name)
            click.echo(f"✅ 项目 {project_name} 同步成功。")        
        except Exception as e:
            click.echo(f"❌ 项目 {project_name} 同步失败: {e}")
    else:
        # 检查当前目录是否是一个 RMM 项目但未注册
        current_path = Path.cwd()
        if RmmProject.is_rmmproject(current_path):
            # 检查项目是否已经注册
            try:
                project_info: dict[str, Any] = RmmProject.project_info(current_path)
                project_name_from_config = project_info.get("name", current_path.name)
                # 检查配置中是否已有此项目
                projects: str | dict[str, str] = Config.projects
                if isinstance(projects, dict) and project_name_from_config not in projects:
                    # 自动注册项目
                    click.echo(f"发现未注册的 RMM 项目: {project_name_from_config}")
                    click.echo(f"项目路径: {current_path}")
                    
                    try:
                        RmmProject.add_project(project_name_from_config, str(current_path))
                        click.echo(f"✅ 项目 {project_name_from_config} 已自动注册。")
                        # 注册后同步项目
                        click.echo(f"正在同步新注册的项目: {project_name_from_config}")
                        
                        RmmProject.sync(project_name_from_config)
                        click.echo(f"✅ 项目 {project_name_from_config} 同步成功。")
                        return
                    except Exception as e:
                        click.echo(f"❌ 自动注册失败: {e}")

                elif isinstance(projects, dict) and project_name_from_config in projects:
                    try:
                        RmmProject.sync(project_name_from_config)
                        click.echo(f"✅ 项目 {project_name_from_config} 同步成功。")
                        return
                    except Exception as e:
                        click.echo(f"❌ 项目同步失败: {e}")
            except Exception as e:
                click.echo(f"❌ 检查当前项目时出错: {e}")
        
        # 没有指定项目名称也没有使用 --all 参数，且当前目录不是 RMM 项目
        click.echo("请指定要同步的项目名称，或使用 --all 参数同步所有项目。")
        click.echo("或者在 RMM 项目目录中运行此命令以自动检测和同步项目。")
        click.echo("使用 'rmm sync --help' 查看帮助信息。")