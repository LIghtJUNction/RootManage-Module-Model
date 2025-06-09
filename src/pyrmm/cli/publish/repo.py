import click
# import json - 延迟导入以减少启动时间
# import requests - 延迟导入以减少启动时间
from pathlib import Path
from typing import Any

from pyrmm.usr.lib.project import RmmProject
from pyrmm.usr.lib.git import RmmGit

# 默认的模块集合仓库
DEFAULT_REPO_REGISTRY = "RootManage-Module-Model/ModuleRegistry"

@click.command()
@click.argument("project_path", default=".", required=False)
@click.option("--registry", "-r", default=DEFAULT_REPO_REGISTRY, 
              help=f"模块集合仓库，默认为 {DEFAULT_REPO_REGISTRY}")
@click.option("--category", "-c", help="模块分类 (例如: system, tools, optimization)")
@click.option("--description", "-d", help="模块描述")
@click.option("--maintainer", "-m", help="维护者信息")
@click.option("--dry-run", is_flag=True, help="模拟运行，不实际提交")
@click.option("--force-update", is_flag=True, help="强制更新已存在的模块")
@click.pass_context
def repo(ctx: click.Context, project_path: str, registry: str, category: str, 
         description: str, maintainer: str, dry_run: bool, force_update: bool) -> None:
    """向模块集合仓库提交模块
    
    这个命令用于将你的模块提交到中央模块集合仓库，通过 GitHub PR 的方式。
    其他用户可以从这个仓库发现和安装你的模块。
    
    提交流程：
    1. 验证模块格式和配置
    2. 准备模块元数据
    3. Fork 模块集合仓库（如果需要）
    4. 创建或更新模块信息
    5. 提交 Pull Request
    """    
    auto_yes = ctx.obj.get('yes', False)
    token = ctx.obj.get('token', None)
    
    if auto_yes:
        click.echo("🤖 自动模式: 已启用 --yes 参数，将自动同意所有确认提示")
    
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
    
    # 步骤1：验证模块格式和配置
    click.echo("📋 步骤1: 验证模块格式...")
    
    # 检查是否是有效的RMM项目
    try:
        project_info = RmmProject.project_info(project_dir)
        if not project_info:
            click.echo("❌ 无法读取项目配置，请确保这是一个有效的RMM项目")
            return
    except Exception as e:
        click.echo(f"❌ 项目配置验证失败: {e}")
        return
    
    # 必需的字段检查
    required_fields = ['name', 'version', 'id']
    missing_fields = [field for field in required_fields if field not in project_info or not project_info[field]]
    
    if missing_fields:
        click.echo(f"❌ 项目配置缺少必需字段: {', '.join(missing_fields)}")
        click.echo("💡 请在 rmmproject.toml 中补充这些信息")
        return
    
    module_name = project_info['name']
    module_version = project_info['version']
    module_id = project_info['id']
    
    click.echo(f"✅ 模块验证通过: {module_name} v{module_version} (ID: {module_id})")
    
    # 检查是否为Git仓库
    git_info = RmmGit.get_repo_info(project_dir)
    if not git_info:
        click.echo("❌ 项目目录不是Git仓库，无法获取源码信息")
        return
    
    # 获取源码仓库信息
    source_repo_url = None
    if 'origin' in git_info.remotes:
        source_repo_url = git_info.remotes['origin'].url
        click.echo(f"📡 源码仓库: {source_repo_url}")
    else:
        click.echo("⚠️  警告: 没有找到origin远程仓库")
    
    # 步骤2：准备模块元数据
    click.echo("\n📋 步骤2: 准备模块元数据...")
    
    # 构建模块元数据
    module_metadata = {
        "name": module_name,
        "id": module_id,
        "version": module_version,
        "description": description or project_info.get('description', ''),
        "category": category or project_info.get('category', 'other'),
        "maintainer": maintainer or project_info.get('maintainer', ''),
        "source_url": source_repo_url,
        "created_at": None,  # 会在仓库中自动设置
        "updated_at": None,  # 会在仓库中自动设置
    }
    
    # 交互式收集缺失信息
    if not module_metadata['description']:
        if not auto_yes:
            module_metadata['description'] = click.prompt("请输入模块描述", default="")
        else:
            click.echo("⚠️  警告: 缺少模块描述")
    
    if not module_metadata['category']:
        if not auto_yes:
            categories = ['system', 'tools', 'optimization', 'customization', 'security', 'other']
            click.echo(f"可用分类: {', '.join(categories)}")
            module_metadata['category'] = click.prompt("请选择模块分类", default="other")
        else:
            module_metadata['category'] = 'other'
    
    if not module_metadata['maintainer']:
        if not auto_yes:
            module_metadata['maintainer'] = click.prompt("请输入维护者信息（姓名 <邮箱>）", default="")
        else:
            click.echo("⚠️  警告: 缺少维护者信息")
    
    click.echo("✅ 元数据准备完成")
    for key, value in module_metadata.items():
        if value:
            click.echo(f"  {key}: {value}")
    
    # 步骤3：检查GitHub Token
    if not token:
        click.echo("❌ 需要GitHub Token来操作模块集合仓库")
        click.echo("💡 请使用 --token 参数或设置环境变量 GITHUB_TOKEN")
        return
    
    # 步骤4：Fork和操作模块集合仓库
    click.echo(f"\n📋 步骤3: 操作模块集合仓库 {registry}...")
    
    if dry_run:
        click.echo("🔍 模拟运行模式 - 显示将要执行的操作:")
        click.echo(f"  1. Fork 仓库 {registry}")
        click.echo(f"  2. 在 modules/{module_metadata['category']}/{module_id}.json 创建/更新模块信息")
        click.echo(f"  3. 提交 PR: Add/Update {module_name} v{module_version}")
        click.echo("✅ 模拟运行完成")
        return
    
    try:
        # 实际的GitHub API操作
        success = _submit_to_registry(
            registry=registry,
            module_metadata=module_metadata,
            token=token,
            force_update=force_update,
            auto_yes=auto_yes
        )
        
        if success:
            click.echo("🎉 模块提交成功!")
            click.echo(f"📝 请关注 GitHub 上的 PR 状态")
            click.echo(f"🔗 仓库地址: https://github.com/{registry}")
        else:
            click.echo("❌ 模块提交失败")
            
    except Exception as e:
        click.echo(f"❌ 提交过程中发生错误: {e}")


def _submit_to_registry(registry: str, module_metadata: dict[str, Any], 
                       token: str, force_update: bool, auto_yes: bool) -> bool:
    """提交模块到集合仓库"""
    
    # 延迟导入 - 仅在实际使用时导入
    import json
    import requests
    
    # 解析仓库信息
    if '/' not in registry:
        click.echo(f"❌ 无效的仓库格式: {registry}")
        return False
    
    owner, repo_name = registry.split('/', 1)
    module_id = module_metadata['id']
    category = module_metadata['category']
    
    # GitHub API Headers
    headers = {
        'Authorization': f'token {token}',
        'Accept': 'application/vnd.github.v3+json',
        'User-Agent': 'pyrmm-cli'
    }
    
    try:
        # 1. 检查仓库是否存在
        click.echo(f"🔍 检查仓库 {registry}...")
        repo_response = requests.get(f'https://api.github.com/repos/{registry}', headers=headers)
        
        if repo_response.status_code == 404:
            click.echo(f"❌ 仓库 {registry} 不存在")
            return False
        elif repo_response.status_code != 200:
            click.echo(f"❌ 无法访问仓库: {repo_response.status_code}")
            return False
        
        # 2. Fork仓库（如果需要）
        click.echo("🍴 检查是否需要Fork仓库...")
        user_response = requests.get('https://api.github.com/user', headers=headers)
        if user_response.status_code != 200:
            click.echo("❌ 无法获取用户信息，请检查Token权限")
            return False
        
        username = user_response.json()['login']
        
        # 检查用户是否已经Fork了仓库
        fork_response = requests.get(f'https://api.github.com/repos/{username}/{repo_name}', headers=headers)
        
        if fork_response.status_code == 404:
            # 需要Fork
            click.echo(f"🍴 Fork仓库到 {username}/{repo_name}...")
            fork_data = requests.post(f'https://api.github.com/repos/{registry}/forks', headers=headers)
            if fork_data.status_code not in [200, 201, 202]:
                click.echo(f"❌ Fork失败: {fork_data.status_code}")
                return False
            click.echo("✅ Fork成功")
        
        # 3. 检查模块是否已存在
        module_path = f"modules/{category}/{module_id}.json"
        click.echo(f"🔍 检查模块是否已存在: {module_path}")
        
        file_response = requests.get(
            f'https://api.github.com/repos/{username}/{repo_name}/contents/{module_path}',
            headers=headers
        )
        existing_module = None
        file_sha = None
        existing_data = None
        
        if file_response.status_code == 200:
            existing_module = file_response.json()
            file_sha = existing_module['sha']
            
            # 解码现有内容
            import base64
            existing_content = base64.b64decode(existing_module['content']).decode('utf-8')
            existing_data = json.loads(existing_content)
            
            click.echo(f"ℹ️  找到现有模块版本: {existing_data.get('version', 'unknown')}")
            
            if not force_update and not auto_yes:
                if not click.confirm(f"模块 {module_id} 已存在，是否更新？"):
                    click.echo("⏹️  已取消操作")
                    return False
        
        # 4. 准备模块内容
        import datetime
        
        # 设置时间戳
        now = datetime.datetime.now(datetime.timezone.utc).isoformat().replace('+00:00', 'Z')
        if existing_module and existing_data:
            module_metadata['created_at'] = existing_data.get('created_at', now)
            module_metadata['updated_at'] = now
        else:
            module_metadata['created_at'] = now
            module_metadata['updated_at'] = now
        
        module_content = json.dumps(module_metadata, indent=2, ensure_ascii=False)
        
        # 5. 创建或更新文件
        action = "更新" if existing_module else "添加"
        commit_message = f"{action} {module_metadata['name']} v{module_metadata['version']}"
        
        click.echo(f"📝 {action}模块文件...")
        
        # Base64编码内容
        import base64
        encoded_content = base64.b64encode(module_content.encode('utf-8')).decode('utf-8')
        
        update_data = {
            'message': commit_message,
            'content': encoded_content,
            'branch': 'main'
        }
        
        if file_sha:
            update_data['sha'] = file_sha
        
        update_response = requests.put(
            f'https://api.github.com/repos/{username}/{repo_name}/contents/{module_path}',
            headers=headers,
            json=update_data
        )
        
        if update_response.status_code not in [200, 201]:
            click.echo(f"❌ 文件更新失败: {update_response.status_code}")
            click.echo(update_response.text)
            return False
        
        # 6. 创建Pull Request
        click.echo("📨 创建Pull Request...")
        
        pr_title = f"{action} {module_metadata['name']} v{module_metadata['version']}"
        pr_body = f"""
## 模块信息

- **名称**: {module_metadata['name']}
- **版本**: {module_metadata['version']}
- **分类**: {module_metadata['category']}
- **维护者**: {module_metadata['maintainer']}
- **描述**: {module_metadata['description']}

## 源码仓库

{module_metadata['source_url']}

---
*此PR由 pyrmm CLI 自动生成*
        """.strip()
        
        pr_data = {
            'title': pr_title,
            'body': pr_body,
            'head': f'{username}:main',
            'base': 'main'
        }
        
        pr_response = requests.post(
            f'https://api.github.com/repos/{registry}/pulls',
            headers=headers,
            json=pr_data
        )
        
        if pr_response.status_code == 201:
            pr_info = pr_response.json()
            click.echo(f"✅ Pull Request 创建成功!")
            click.echo(f"🔗 PR链接: {pr_info['html_url']}")
            return True
        elif pr_response.status_code == 422:
            # 可能是重复PR或其他冲突
            error_info = pr_response.json()
            if 'A pull request already exists' in str(error_info):
                click.echo("ℹ️  已存在相同的Pull Request")
                return True
            else:
                click.echo(f"❌ Pull Request创建失败: {error_info}")
                return False
        else:
            click.echo(f"❌ Pull Request创建失败: {pr_response.status_code}")
            click.echo(pr_response.text)
            return False
            
    except requests.RequestException as e:
        click.echo(f"❌ 网络请求失败: {e}")
        return False
    except Exception as e:
        click.echo(f"❌ 操作失败: {e}")
        return False
