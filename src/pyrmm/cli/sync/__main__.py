import click
from pathlib import Path
from typing import Any
from pyrmm.config import Config as RmmConfig, RmmProject

pass_config = click.make_pass_decorator(RmmConfig, ensure=True)

@click.group()
@pass_config  
def sync(rmmc: RmmConfig) -> None:
    """Pyrmm 同步命令组 - 同步和刷新项目配置"""
    pass

@sync.command()
@click.option('--path', '-p', type=click.Path(exists=True, file_okay=False, dir_okay=True, path_type=Path),
              help='指定要扫描的路径，默认扫描已知项目路径的父目录')
@click.option('--auto-add', '-a', is_flag=True, default=False,
              help='自动添加新发现的项目，不询问用户')
@click.option('--auto-remove', '-r', is_flag=True, default=False,
              help='自动移除无效项目，不询问用户')
@pass_config
def projects(rmmc: RmmConfig, path: Path | None, auto_add: bool, auto_remove: bool):
    """同步项目配置与文件系统
    
    扫描指定路径或已知项目路径，检查项目的有效性：
    - 检查配置中的项目是否仍然存在且有效
    - 发现新的RMM项目并询问是否添加到配置
    - 移除无效的项目配置
    """
    click.echo("🔄 开始同步项目配置...")
    
    # 临时修改click的行为以支持自动模式
    original_confirm = click.confirm
    
    def auto_confirm_add(message: str, default: bool | None = None, **kwargs: Any) -> bool:
        if "Add these projects" in message and auto_add:
            click.echo(f"✅ {message} [自动确认: 是]")
            return True
        elif "Remove invalid project" in message and auto_remove:
            click.echo(f"✅ {message} [自动确认: 是]")
            return True
        else:
            return original_confirm(message, default=default, **kwargs)
    
    # 临时替换click.confirm
    click.confirm = auto_confirm_add
    
    try:
        # 调用RmmProject的同步方法
        updated_projects = RmmProject.sync(scan_path=path)
        
        if updated_projects:
            click.echo(f"✅ 同步完成！当前有 {len(updated_projects)} 个项目：")
            for name, project_path in updated_projects.items():
                click.echo(f"  📁 {name}: {project_path}")
        else:
            click.echo("ℹ️  没有找到任何RMM项目")
            
        # 显示最后使用的项目
        if hasattr(rmmc, 'projects') and rmmc.projects.get('last'):
            click.echo(f"📌 最后使用的项目: {rmmc.projects['last']}")
            
    except Exception as e:
        click.echo(f"❌ 同步项目时出错: {e}", err=True)
    finally:
        # 恢复原始的click.confirm
        click.confirm = original_confirm

@sync.command()
@click.option('--reset', '-r', is_flag=True, 
              help='重置配置为默认值')
@click.option('--verify', '-v', is_flag=True, default=True,
              help='验证配置的完整性')
@pass_config  
def config(rmmc: RmmConfig, reset: bool, verify: bool):
    """同步和验证配置文件
    
    检查配置文件的完整性，确保所有必要的配置项都存在且有效。
    可以选择重置配置为默认值。
    """
    click.echo("🔄 开始同步配置文件...")
    
    if reset:
        if click.confirm("⚠️  确定要重置所有配置为默认值吗？这将丢失所有自定义设置！", default=False):
            try:
                # 重置为默认配置
                for key in list(rmmc.__dict__.keys()):
                    if not key.startswith('_') and key != 'rmmroot':
                        delattr(rmmc, key)
                
                # 设置默认值
                for key, default_value in RmmConfig.DEFAULTS.items():
                    setattr(rmmc, key, default_value)
                
                click.echo("✅ 配置已重置为默认值")
                
            except Exception as e:
                click.echo(f"❌ 重置配置时出错: {e}", err=True)
                return
    
    if verify:
        click.echo("🔍 验证配置完整性...")
        
        # 检查必要的配置项
        required_configs = ['username', 'email']
        missing_configs: list[str] = []
        
        for config_key in required_configs:
            try:
                value = getattr(rmmc, config_key)
                if config_key == 'username' and value == "Your Name":
                    missing_configs.append(f"{config_key} (使用默认值)")
                elif config_key == 'email' and value == "dev@rmmp.com":
                    missing_configs.append(f"{config_key} (使用默认值)")
            except AttributeError:
                missing_configs.append(config_key)
        
        # 检查路径是否存在
        # 使用公共方法而不是私有属性
        paths_to_check = [
            ('rmmroot', rmmc.rmmroot),
        ]
        
        # 尝试访问目录路径
        try:
            data_path = rmmc.rmmroot / 'data'
            cache_path = rmmc.rmmroot / 'cache' 
            tmp_path = rmmc.rmmroot / 'tmp'
            
            paths_to_check.extend([
                ('data', data_path),
                ('cache', cache_path),
                ('tmp', tmp_path)
            ])
        except Exception:
            pass
        
        missing_paths: list[str] = []
        for path_name, path_obj in paths_to_check:
            if not path_obj.exists():
                missing_paths.append(f"{path_name}: {path_obj}")
        
        # 报告结果
        if missing_configs:
            click.echo("⚠️  发现未配置或使用默认值的配置项:")
            for config_item in missing_configs:
                click.echo(f"  • {config_item}")
            click.echo("💡 使用 'rmm config set key=value' 来设置这些配置")
        
        if missing_paths:
            click.echo("⚠️  发现不存在的路径（将自动创建）:")
            for path_item in missing_paths:
                click.echo(f"  • {path_item}")
            
            # 重新加载配置以创建缺失的目录
            rmmc.load()
            click.echo("✅ 缺失的目录已创建")
        
        if not missing_configs and not missing_paths:
            click.echo("✅ 配置验证通过，所有配置项都正常")

@sync.command()
@click.option('--path', '-p', type=click.Path(exists=True, file_okay=False, dir_okay=True, path_type=Path),
              help='指定要扫描项目的路径')
@click.option('--auto', '-a', is_flag=True, default=False,
              help='自动模式，不询问用户确认')
@click.option('--verify-only', '-v', is_flag=True, default=False,
              help='仅验证，不进行实际修改')
@pass_config
def all(rmmc: RmmConfig, path: Path | None, auto: bool, verify_only: bool):
    """同步所有配置和项目
    
    执行完整的同步操作：
    1. 验证配置文件完整性
    2. 同步项目配置与文件系统
    3. 清理无效配置
    """
    click.echo("🚀 开始完整同步操作...")
    # Step 1: 同步配置
    click.echo("\n📋 步骤 1: 同步配置文件")
    ctx = click.get_current_context()
    ctx.invoke(config, reset=False, verify=True)
    
    if verify_only:
        click.echo("\n📋 步骤 2: 验证项目配置（仅检查，不修改）")
        # 这里可以添加项目验证逻辑，但不实际修改
        try:
            current_projects = rmmc.projects if hasattr(rmmc, 'projects') else {}
            valid_count = 0
            invalid_count = 0
            
            for name, project_path in current_projects.items():
                if name == 'last':
                    continue
                if RmmProject.is_rmmp(project_path):
                    valid_count += 1
                    click.echo(f"  ✅ {name}: {project_path}")
                else:
                    invalid_count += 1
                    click.echo(f"  ❌ {name}: {project_path} (无效)")
            
            click.echo(f"\n📊 项目验证结果: {valid_count} 个有效项目, {invalid_count} 个无效项目")
            
        except Exception as e:
            click.echo(f"❌ 验证项目时出错: {e}", err=True)
    else:
        # Step 2: 同步项目
        click.echo("\n📁 步骤 2: 同步项目配置")
        ctx.invoke(projects, path=path, auto_add=auto, auto_remove=auto)
    
    click.echo("\n🎉 完整同步操作完成！")

if __name__ == '__main__':
    sync()