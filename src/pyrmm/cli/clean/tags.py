import click

@click.command()
@click.option("--all", is_flag=True, help="清理所有已注册的RMM项目，而不仅仅是当前目录的项目")
@click.option("--dry-run", is_flag=True, help="仅查看将要删除的标签，不实际执行删除操作")
@click.option("--force", is_flag=True, help="跳过确认提示，直接删除孤立标签")
def tags(all: bool, dry_run: bool, force: bool):
    """清理RMM项目的Git孤立标签"""
    from pyrmm.usr.lib.project import RmmProject
    from pyrmm.usr.lib.config import Config
    from pyrmm.usr.lib.git import RmmGit
    from pathlib import Path
    import os

    # 获取 GitHub API token（可选）
    github_token = os.getenv('GITHUB_ACCESS_TOKEN')

    if all:
        # 清理所有已注册的项目
        projects = Config.projects
        if not projects or isinstance(projects, str):
            click.echo("没有找到任何RMM项目。")
            return

        click.echo(f"🧹 清理所有已注册项目的孤立Git标签...")
        
        for project_name in projects:
            try:
                project_path = RmmProject.project_path(project_name)
                
                # 检查是否为Git仓库
                if not RmmGit.find_git_root(project_path):
                    click.echo(f"⚠️  项目 {project_name} 不是Git仓库，跳过。")
                    continue
                
                click.echo(f"\n🔍 检查项目: {project_name}")
                
                # 查找孤立标签
                orphan_tags = RmmGit.find_orphan_tags(project_path, token=github_token)
                
                if not orphan_tags:
                    click.echo(f"✅ 项目 {project_name} 没有发现孤立标签。")
                    continue
                
                click.echo(f"🏷️  发现 {len(orphan_tags)} 个孤立标签:")
                for tag in orphan_tags:
                    click.echo(f"  - {tag}")
                
                if dry_run:
                    click.echo(f"🔍 (dry-run) 将删除以上标签")
                    continue
                
                # 确认删除
                if not force:
                    if not click.confirm(f"确定要删除项目 {project_name} 的这些孤立标签吗？"):
                        click.echo(f"⏭️  跳过项目 {project_name}")
                        continue
                
                # 执行清理
                success_tags, failed_tags = RmmGit.clean_orphan_tags(project_path, token=github_token)
                
                if success_tags:
                    click.echo(f"✅ 项目 {project_name} 成功删除 {len(success_tags)} 个孤立标签。")
                if failed_tags:
                    click.echo(f"❌ 项目 {project_name} 删除失败 {len(failed_tags)} 个标签: {', '.join(failed_tags)}")
                    
            except Exception as e:
                click.echo(f"❌ 处理项目 {project_name} 时出错: {e}")
    else:
        # 仅清理当前目录的项目（如果是RMM项目）
        current_path = Path.cwd()
        
        if not RmmProject.is_rmmproject(current_path):
            click.echo("❌ 当前目录不是一个有效的RMM项目。")
            click.echo("💡 提示：使用 --all 参数清理所有已注册的项目，或切换到RMM项目目录。")
            return
        
        # 检查是否为Git仓库
        git_root = RmmGit.find_git_root(current_path)
        if not git_root:
            click.echo("❌ 当前项目不是Git仓库。")
            return
        
        click.echo(f"🔍 检查当前项目的孤立Git标签...")
        
        try:
            # 查找孤立标签
            orphan_tags = RmmGit.find_orphan_tags(current_path, token=github_token)
            
            if not orphan_tags:
                click.echo("✅ 没有发现孤立标签。")
                return
            
            click.echo(f"🏷️  发现 {len(orphan_tags)} 个孤立标签:")
            for tag in orphan_tags:
                click.echo(f"  - {tag}")
            
            if dry_run:
                click.echo("🔍 (dry-run) 将删除以上标签")
                click.echo("💡 提示：移除 --dry-run 参数来实际执行删除操作")
                return
            
            # 确认删除
            if not force:
                if not click.confirm("确定要删除这些孤立标签吗？"):
                    click.echo("⏭️  操作已取消")
                    return
            
            # 执行清理
            click.echo("🧹 开始清理孤立标签...")
            success_tags, failed_tags = RmmGit.clean_orphan_tags(current_path, token=github_token)
            
            if success_tags:
                click.echo(f"✅ 成功删除 {len(success_tags)} 个孤立标签:")
                for tag in success_tags:
                    click.echo(f"  ✓ {tag}")
            
            if failed_tags:
                click.echo(f"❌ 删除失败 {len(failed_tags)} 个标签:")
                for tag in failed_tags:
                    click.echo(f"  ✗ {tag}")
            
            if not failed_tags:
                click.echo("🎉 所有孤立标签已成功清理！")
                
        except Exception as e:
            click.echo(f"❌ 清理孤立标签时出错: {e}")
