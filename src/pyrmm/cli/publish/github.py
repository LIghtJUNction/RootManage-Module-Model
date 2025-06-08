import click

@click.command()
@click.argument("project_path", default=".", required=False)
@click.option("--tag", "-t", help="Release标签名，默认使用项目版本 (例如: v1.0.0)")
@click.option("--name", "-n", help="Release名称，默认使用标签名")
@click.option("--body", "-b", help="Release描述")
@click.option("--draft", is_flag=True, help="创建为草稿")
@click.option("--prerelease", is_flag=True, help="标记为预发布版本")
@click.option("--dry-run", is_flag=True, help="模拟运行，不实际创建Release和上传文件")
@click.pass_context
def github(ctx: click.Context, project_path: str, tag: str, name: str, body: str,
          draft: bool, prerelease: bool, dry_run: bool) -> None:    
    """发布到GitHub"""
    from pyrmm.usr.lib.project import RmmProject
    from pyrmm.usr.lib.git import RmmGit
    from pyrmm.usr.lib.proxy import ProxyManager
    from pathlib import Path

    token = ctx.obj.get('token', None)
    auto_yes = ctx.obj.get('yes', False)
    
    if auto_yes:
        click.echo("🤖 自动模式: 已启用 --yes 参数，将自动同意所有确认提示")
    
    try:# 安全检查：防止将GitHub token误用为tag
        if tag and (tag.startswith('ghp_') or tag.startswith('github_pat_') or len(tag) > 50):
            click.echo("🚨 安全警告：检测到可能的GitHub token！")
            click.echo("💡 您是否想要使用 --token 参数而不是 --tag？")
            click.echo("📋 正确用法:")
            click.echo("   rmm publish --token YOUR_TOKEN github .")
            click.echo("   rmm publish --token YOUR_TOKEN github --tag v1.0.0 .")
            click.echo("❌ 为了安全考虑，操作已取消。")
            return
        
        # 解析项目路径
        if project_path == ".":
            project_dir = Path.cwd()
        else:
            project_dir = Path(project_path).resolve()
            if not project_dir.exists():
                # 尝试作为项目名解析
                try:
                    project_dir = RmmProject.project_path(project_path)
                except Exception:
                    click.echo(f"❌ 项目路径不存在: {project_path}")
                    return
        
        click.echo(f"🔍 项目目录: {project_dir}")
        
        # 检查项目是否为Git仓库
        git_info = RmmGit.get_repo_info(project_dir)
        if not git_info:
            click.echo("❌ 当前目录不是Git仓库")
            return
        
        # 检查是否有origin远程仓库
        if 'origin' not in git_info.remotes:
            click.echo("❌ 未找到origin远程仓库")
            return
        
        origin_info = git_info.remotes['origin']
        if not origin_info.username or not origin_info.repo_name:
            click.echo(f"❌ 无法解析GitHub仓库信息: {origin_info.url}")
            return
        
        click.echo(f"📦 GitHub仓库: {origin_info.username}/{origin_info.repo_name}")        # 获取GitHub token
        github_token: str | None = token
        if not github_token:
            github_token = ctx.obj.get('token', None)
            if not github_token:
                click.echo(" rmm test github --TOKEN YOUR_GITHUB_ACCESS_TOKEN")
                click.echo("❌ 未提供GitHub访问令牌。请设置GITHUB_ACCESS_TOKEN环境变量或使用--token参数")
                click.echo("💡 GitHub token 需要以下权限:")
                click.echo("   - repo (完整仓库权限)")
                click.echo("   - contents:write (写入内容)")
                click.echo("   - metadata:read (读取元数据)")
                click.echo("🔗 创建token: https://github.com/settings/tokens/new")
                return
        
        # 验证GitHub token权限
        click.echo("🔑 验证GitHub访问权限...")
        if not RmmGit.check_repo_exists(origin_info.username, origin_info.repo_name, github_token):
            click.echo(" rmm test github --TOKEN YOUR_GITHUB_ACCESS_TOKEN")
            click.echo("❌ 无法访问GitHub仓库，请检查:")
            click.echo("   1. 仓库是否存在且可访问")
            click.echo("   2. GitHub token 是否有效")
            click.echo("   3. Token 是否有足够权限 (repo权限)")
            click.echo("🔗 检查token权限: https://github.com/settings/tokens")
            return
          # 检查仓库状态
        if not git_info.is_clean:
            click.echo("⚠️  警告: Git仓库有未提交的更改")
            if not auto_yes and not click.confirm("继续发布？"):
                return
        
        # 检查构建输出目录
        dist_dir = project_dir / ".rmmp" / "dist"
        if not dist_dir.exists():
            click.echo("❌ 构建输出目录不存在: .rmmp/dist/")
            click.echo("请先运行构建命令: rmm build")
            return
          # 收集要上传的文件
        asset_files: list[Path] = []
        for file_path in dist_dir.rglob("*"):
            if file_path.is_file():
                asset_files.append(file_path)
        
        if not asset_files:
            click.echo("❌ 构建输出目录为空: .rmmp/dist/")
            return
        
        click.echo(f"📁 找到 {len(asset_files)} 个文件待上传:")
        for asset in asset_files:
            click.echo(f"  - {asset.relative_to(dist_dir)}")        # 确定标签名
        if not tag:
            try:
                # 尝试从项目配置获取版本
                project_info = RmmProject.project_info(project_dir)
                if 'version' in project_info and project_info['version']:
                    version: str  = project_info['version'] if isinstance(project_info['version'], str) else "1.0.0"
                    # 确保版本号以v开头，但不重复添加
                    if not version.startswith('v'):
                        tag = f"v{version}"
                    else:
                        tag = version
                else:
                    tag = "v1.0.0"
            except Exception:
                tag = "v1.0.0"

        # 确定release名称
        if not name:
            name = tag
          # 确定release描述
        if not body:
            # 尝试获取最新提交信息
            commit_info = RmmGit.get_commit_info(project_dir)
            if commit_info:
                body = f"Release {tag}\n\n最新提交: {commit_info['message']}"
            else:
                body = f"Release {tag}"
          # 添加代理下载链接到 release 描述中
        if asset_files:
            # 为每个文件生成代理下载链接
            proxy_sections: list[str] = []
            
            for asset_file in asset_files:
                # 构造GitHub下载URL
                download_url = f"https://github.com/{origin_info.username}/{origin_info.repo_name}/releases/download/{tag}/{asset_file.name}"
                
                # 生成代理下载链接
                proxy_links = ProxyManager.generate_proxy_download_links(project_dir, download_url)
                if proxy_links:
                    # 为多个文件时，修改标题以包含文件名
                    if len(asset_files) > 1:
                        proxy_links = proxy_links.replace("## 🚀 加速下载链接", f"## 🚀 {asset_file.name} 加速下载")
                    proxy_sections.append(proxy_links)
            
            # 将所有代理链接添加到描述中
            if proxy_sections:
                body = f"{body}\n\n" + "\n\n".join(proxy_sections)
        
        click.echo(f"🏷️  标签: {tag}")
        click.echo(f"📋 名称: {name}")
        click.echo(f"📝 描述: {body}")
        
        if dry_run:
            click.echo("🔍 模拟运行模式 - 不会实际创建Release或上传文件")
            click.echo(f"📊 模拟发布到: {origin_info.username}/{origin_info.repo_name}")
            click.echo("✅ 模拟运行完成")
            return        # 确认发布
        if not auto_yes and not click.confirm(f"确定要发布到 {origin_info.username}/{origin_info.repo_name}？"):
            return
        
        # 检查release是否已存在
        existing_release = RmmGit.get_release_by_tag(
            origin_info.username, 
            origin_info.repo_name, 
            tag, 
            github_token
        )
        
        if existing_release:
            click.echo(f"⚠️  Release {tag} 已存在")
            if not auto_yes and not click.confirm("是否要上传文件到现有Release？"):
                return
            release_info = existing_release
            release_info = existing_release
        else:
            # 创建新release
            click.echo(f"🚀 创建Release: {tag}")
            release_info = RmmGit.create_release(
                origin_info.username,
                origin_info.repo_name,
                tag,
                name,
                body,
                draft,
                prerelease,
                github_token            
                )
            
            if not release_info:
                click.echo("❌ 创建Release失败")
                click.echo("\n💡 可能的解决方案:")
                click.echo("   1. 检查GitHub token权限 (需要 repo 权限)")
                click.echo("   2. 确认标签不重复")
                click.echo("   3. 检查网络连接")
                click.echo("   4. 手动在GitHub上创建Release后重新运行")
                click.echo(f"\n🔗 手动创建Release: https://github.com/{origin_info.username}/{origin_info.repo_name}/releases/new")
                return
            
            click.echo(f"✅ Release创建成功: {release_info['html_url']}")
        
        # 上传文件
        click.echo("📤 开始上传文件...")
        success = RmmGit.upload_release_assets(
            origin_info.username,
            origin_info.repo_name,
            tag,
            asset_files,
            github_token
        )
        
        if success:
            click.echo(f"🎉 发布成功! 访问: {release_info['html_url']}")
        else:
            click.echo("❌ 文件上传失败")
            
    except Exception as e:
        click.echo(f"❌ 发布过程中出错: {e}")
        import traceback
        traceback.print_exc()