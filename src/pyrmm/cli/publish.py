# rmm publish python 拓展命令
import os
from pathlib import Path
import platform
from typing import Any
import subprocess
import re
import json
from rich.console import Console
from rich.panel import Panel
from rich.table import Table
from rich.text import Text
from rich.tree import Tree
from rich.align import Align
from typing import Any

# 初始化 rich console
console = Console()

def success(message: str) -> None:
    """打印成功消息"""
    console.print(f"[bold green]✅ {message}[/bold green]")

def warning(message: str) -> None:
    """打印警告消息"""
    console.print(f"[bold yellow]⚠️  {message}[/bold yellow]")

def error(message: str) -> None:
    """打印错误消息"""
    console.print(f"[bold red]❌ {message}[/bold red]")

def info(message: str) -> None:
    """打印信息消息"""
    console.print(f"[cyan]ℹ️  {message}[/cyan]")

def step(message: str) -> None:
    """打印步骤消息"""
    console.print(f"[bold magenta]🚀 {message}[/bold magenta]")

def print_banner(title: str, subtitle: str = "") -> None:
    """打印美化的横幅"""
    text = Text(title, style="bold white")
    panel = Panel(
        Align.center(text),
        style="bold blue",
        border_style="bright_blue",
        subtitle=subtitle,
        subtitle_align="center"
    )
    console.print(panel)

def print_table(title: str, data: dict[str, Any]) -> None:
    """打印美化的表格"""
    table = Table(title=title, style="cyan")
    table.add_column("属性", style="bold yellow", no_wrap=True)
    table.add_column("值", style="green")
    
    for key, value in data.items():
        table.add_row(key, str(value))
    
    console.print(table)

def print_file_tree(files: list[Path], title: str = "目标文件") -> None:
    """打印文件树"""
    tree = Tree(f"[bold blue]{title}[/bold blue]")
    for file in files:
        tree.add(f"[green]{file.name}[/green]")
    console.print(tree)

def is_rmmp(project_path: Path = Path.cwd()) -> bool:
    """
    检查给定路径是否为 RMM 项目目录。

    参数:
        project_path (Path): 要检查的路径。

    返回:
        bool: 如果路径是 RMM 项目目录，则返回 True；否则返回 False。
    """
    return (project_path / "rmmproject.toml").exists()

def get_repo_name(project_path: Path) -> str | None:
    """
    从 rmmproject.toml 或 .git 文件夹获取 GitHub 仓库名。
    
    参数:
        project_path (Path): 项目路径
        
    返回:
        str | None: 仓库名 (格式: owner/repo) 或 None
    """
    # 首先尝试从 rmmproject.toml 读取
    rmmproject_file = project_path / "rmmproject.toml"
    if rmmproject_file.exists():
        try:
            # 读取 TOML 文件内容
            content = rmmproject_file.read_text(encoding="utf-8")
            
            # 简单解析 [urls] 部分的 github 字段
            github_match = re.search(r'\[urls\].*?github\s*=\s*"([^"]+)"', content, re.DOTALL)
            if github_match:
                github_url = github_match.group(1)                # 解析 GitHub URL
                if "github.com" in github_url:
                    # HTTPS: https://github.com/owner/repo
                    match = re.search(r"github.com/([^/]+/[^/]+?)/?$", github_url)
                    if match:
                        repo_name = match.group(1)
                        success(f"从 rmmproject.toml 获取到仓库名: {repo_name}")
                        return repo_name
        except Exception as e:
            warning(f"读取 rmmproject.toml 失败: {e}")
    
    # 如果从 rmmproject.toml 获取失败，尝试从 git 获取
    info("尝试从 git 获取仓库名...")
    try:
        git_dir = project_path / ".git"
        if not git_dir.exists():
            # 查找上级目录中的 .git
            current = project_path
            while current.parent != current:
                git_dir = current / ".git"
                if git_dir.exists():
                    break
                current = current.parent
            else:
                return None
        
        # 尝试使用 git remote get-url origin
        result = subprocess.run(
            ["git", "remote", "get-url", "origin"],
            cwd=project_path,
            capture_output=True,
            text=True,
            check=True
        )
        
        remote_url = result.stdout.strip()
        
        # 解析 GitHub URL
        # 支持 HTTPS 和 SSH 格式
        if "github.com" in remote_url:
            # HTTPS: https://github.com/owner/repo.git
            # SSH: git@github.com:owner/repo.git
            if remote_url.startswith("https://github.com/"):
                match = re.search(r"https://github.com/([^/]+/[^/]+?)(?:\.git)?/?$", remote_url)
            elif remote_url.startswith("git@github.com:"):
                match = re.search(r"git@github.com:([^/]+/[^/]+?)(?:\.git)?/?$", remote_url)
            else:
                return None
            if match:
                repo_name = match.group(1)
                success(f"从 git 获取到仓库名: {repo_name}")
                
                # 将获取到的仓库名同步回 rmmproject.toml
                try:
                    sync_repo_to_toml(project_path, f"https://github.com/{repo_name}")
                except Exception as e:
                    warning(f"同步仓库名到 rmmproject.toml 失败: {e}")
                
                return repo_name
    
    except (subprocess.CalledProcessError, FileNotFoundError):
        pass
    
    return None

def sync_repo_to_toml(project_path: Path, github_url: str) -> None:
    """
    将 GitHub 仓库地址同步到 rmmproject.toml 文件。
    
    参数:
        project_path (Path): 项目路径
        github_url (str): GitHub 仓库地址
    """
    rmmproject_file = project_path / "rmmproject.toml"
    if not rmmproject_file.exists():
        return
    
    # 读取现有内容
    content = rmmproject_file.read_text(encoding="utf-8")
    
    # 检查是否已存在 [urls] 部分
    if "[urls]" in content:
        # 更新现有的 github 字段
        if "github =" in content:
            # 替换现有的 github 行
            content = re.sub(
                r'github = ".*?"',
                f'github = "{github_url}"',
                content
            )
        else:
            # 在 [urls] 部分添加 github 字段
            content = re.sub(
                r'(\[urls\])',
                f'\\1\ngithub = "{github_url}"',
                content
            )
    else:
        # 添加整个 [urls] 部分
        content += f"\n[urls]\ngithub = \"{github_url}\"\n"
      # 写回文件
    rmmproject_file.write_text(content, encoding="utf-8")
    success(f"已将仓库地址同步到 rmmproject.toml: {github_url}")

# rmmcore会调用这里
def publish(args: list[Any]) -> None:
    """
    发布 RMM 项目。

    参数:
        project_path (Path): 要发布的项目路径，默认为当前工作目录。
    """    
    if len(args) == 0:
        project_path = Path.cwd()        
    elif len(args) == 1:
        project_path = Path(args[0])
    else:
        error("使用方法: rmm publish [project_path]")
        return

    if not is_rmmp(project_path):
        error(f"路径 {project_path} 不是一个有效的 RMM 项目目录。")
        return    # 显示发布标题
    print_banner("🚀 RMM 项目发布工具", f"项目路径: {project_path}")
    from github import Github
    GITHUB_TOKEN = os.getenv("GITHUB_ACCESS_TOKEN",os.getenv("GITHUB_TOKEN","")) 
    if not GITHUB_TOKEN:
        info("请设置环境变量 GITHUB_ACCESS_TOKEN 或 GITHUB_TOKEN。")
        
        if platform.system() == "Windows":
            info("在 Windows 上，您可以通过以下命令设置环境变量：")
            info("set GITHUB_ACCESS_TOKEN=your_token_here")
        else:
            info("在 Linux 或 macOS 上，您可以通过以下命令设置环境变量：")
            info("export GITHUB_ACCESS_TOKEN=your_token_here")
        return
    try:
        g = Github(GITHUB_TOKEN)
        user = g.get_user()
        success(f"已连接到 GitHub 用户: {user.login}")        
        updateJson = project_path / ".rmmp" / "dist" /"update.json"
        if not updateJson.exists():
            error(f"文件不存在: {updateJson}")
            return
            
        from json import load as json_load
        with open(updateJson, "r", encoding="utf-8") as f:
            update_data = json_load(f)
        
        # 美化显示更新数据
        print_table("📦 Release 信息", {
            "版本": update_data.get('version', '未知'),
            "版本代码": update_data.get('versionCode', '未知'),
            "变更日志": update_data.get('changelog', '无'),
            "下载链接": update_data.get('zipUrl', '无')
        })        # 依据 versionCode 找到目标文件 （匹配包含versionCode的文件名）
        version_code = update_data.get('versionCode', '')
        if not version_code:
            error("❌ 无法找到版本代码")
            return

        # 将 version_code 转换为字符串以便进行字符串匹配
        version_code_str = str(version_code)
        target_files: list[Path] = []
        for file in (project_path / ".rmmp" / "dist").glob("*"):
            if version_code_str in file.name:
                target_files.append(file)
        
        # 🔥 重要修复：确保 update.json 文件也会被上传
        if updateJson not in target_files:
            target_files.append(updateJson)
            info("✅ 已添加 update.json 到上传文件列表")# 验证
        module_prop : Path = project_path / "module.prop"
        module_info: dict[str, str] = {}
        with open(module_prop, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if line and '=' in line and not line.startswith('#'):
                    key, value = line.split('=', 1)
                    module_info[key.strip()] = value.strip()
        
        verify_versionCode = module_info.get("versionCode", "")

        if verify_versionCode != version_code_str:
            error(f"❌ 将要上传的版本代号与module.prop定义的版本代号不匹配: {version_code_str} != {verify_versionCode}")
            return

        info(f"验证通过：将要上传的版本代号: {version_code_str} 与 module.prop 中定义的版本代号匹配")

        # 如果匹配 获取version 作为标签tag
        tag = module_info.get("version", "v?.?.?")

        if not tag:
            error("❌ 无法找到版本号，请在 module.prop 中定义版本号")
            return

        info(f"将要上传的版本号: {tag}")

        if not target_files:
            error("❌ 无法找到目标文件")
            return

        info(f"找到目标文件: {target_files}")

        # 获取仓库名
        repo_name = get_repo_name(project_path)
        if not repo_name:
            error("❌ 无法获取 GitHub 仓库名，请确保项目在 Git 仓库中且有 GitHub 远程源")
            return

        info(f"仓库名: {repo_name}")

        # 获取仓库对象
        try:
            repo = g.get_repo(repo_name)
            success(f"✅ 已找到仓库: {repo.full_name}")
        except Exception as e:
            error(f"❌ 无法找到仓库 {repo_name}: {e}")
            return
        
        # 创建 Release
        tag_name = tag if tag.startswith("v") else f"v{tag}"
        release_name = f"Release {update_data.get('version', version_code)}"
        release_body = update_data.get('changelog', '无变更日志')
        
        try:
            # 检查是否已存在该标签的 Release
            try:
                existing_release = repo.get_release(tag_name)
                print(f"⚠️  Release {tag_name} 已存在，将更新现有 Release")
                release = existing_release
                # 更新 Release 信息
                release.update_release(
                    name=release_name,
                    message=release_body,
                    draft=False,
                    prerelease=False
                )
            except:
                # 创建新 Release
                step(f"正在创建 Release: {tag_name}")                #region proxy

                release_body = proxy_handler(project_path, target_files=target_files, release_body=release_body, repo_name=repo_name, tag_name=tag_name, version_code_str=version_code_str)

                release = repo.create_git_release(
                    tag=tag_name,
                    name=release_name,
                    message=release_body,
                    draft=False,
                    prerelease=False
                )
                success(f"✅ 已创建 Release: {release.html_url}")
              # 上传文件到 Release
            print("正在上传文件...")
            for target_file in target_files:
                try:
                    # 检查是否已存在同名文件，如果存在则删除
                    existing_assets = release.get_assets()
                    for asset in existing_assets:
                        if asset.name == target_file.name:
                            print(f"🔄 删除已存在的文件: {asset.name}")
                            asset.delete_asset()
                            break
                      # 上传新文件
                    with open(target_file, "rb") as f:
                        asset = release.upload_asset(
                            path=str(target_file),
                            label=target_file.name
                        )
                    info(f"✅ 已上传文件: {target_file.name}")
                    info(f"   下载链接: {asset.browser_download_url}")
                except Exception as e:
                    error(f"❌ 上传文件 {target_file.name} 失败: {e}")
            success(f"🎉 发布完成！")
            info(f"Release 链接: {release.html_url}")

        except Exception as e:
            error(f"❌ 创建 Release 失败: {e}")
            return
    except Exception as e:
        error(f"连接到 GitHub 失败: {e}")
        return
    

def proxy_handler(path: Path, target_files: list[Path], release_body: str, repo_name: str, tag_name: str, version_code_str: str) -> str:
    """
    处理代理加速链接
    
    参数:
        path: 项目路径
        target_files: 目标文件列表
        release_body: Release 描述内容
        repo_name: 仓库名 (owner/repo)
        tag_name: 标签名 (如 v1.0.0)
    
    返回:
        str: 处理后的 Release 描述内容
    """
    try:
        from ..utils.proxy import get_github_proxies, get_best_github_proxy
        
        # 获取代理列表和最佳代理
        proxies = get_github_proxies()
        best_proxy = get_best_github_proxy()
        
        info(f"使用最佳代理: {best_proxy}")
        
    except ImportError:
        warning("代理模块未找到，使用默认代理")
        # 回退到默认代理列表
        proxies = [
            "https://ghproxy.com/",
            "https://mirror.ghproxy.com/", 
            "https://ghps.cc/",
            "https://gh-proxy.com/",
            "https://ghproxy.net/",
            "https://hub.gitmirror.com/"        ]
        best_proxy = proxies[0]
    
    module_prop: Path = path / "module.prop"
    # 处理每个目标文件
    proxy_links: list[str] = []
    
    for target_file in target_files:
        if target_file.suffix == ".json":
            # 处理 update.json 文件
            try:
                with open(target_file, 'r', encoding='utf-8') as f:
                    update_data = json.load(f)
                  # 🔥 关键修复：将 zipUrl 中的 latest 替换为具体的 tag
                if 'zipUrl' in update_data:
                    original_url = update_data['zipUrl']
                    if original_url.startswith('https://github.com/'):                        # 1. 先将 latest 替换为具体的 tag，并确保使用正确的文件名
                        if '/releases/latest/download/' in original_url:
                            # 🔥 修复：使用当前版本代码匹配的文件名
                            # 从原始URL中提取基础文件名模式
                            filename_match = re.search(r'/([^/]+)\.(zip|tar\.gz)$', original_url)
                            if filename_match:
                                # 生成新的文件名，使用当前的版本代码
                                extension = filename_match.group(2)
                                new_filename = f"TEST-{version_code_str}.{extension}"
                                tag_url = f"https://github.com/{repo_name}/releases/download/{tag_name}/{new_filename}"
                            else:
                                # 回退到原始逻辑
                                tag_url = original_url.replace('/releases/latest/download/', f'/releases/download/{tag_name}/')
                        else:
                            tag_url = original_url
                        
                        # 2. 再添加代理前缀 - 🔥 修复URL拼接
                        best_proxy_str = str(best_proxy)
                        if not best_proxy_str.endswith('/'):
                            best_proxy_str += '/'
                        
                        # 确保代理URL格式正确
                        if best_proxy_str.startswith('http'):
                            proxied_url = best_proxy_str + tag_url
                        else:
                            proxied_url = f"https://{best_proxy_str}" + tag_url
                        
                        update_data['zipUrl'] = proxied_url
                        
                        # 保存修改后的文件
                        with open(target_file, 'w', encoding='utf-8') as f:
                            json.dump(update_data, f, indent=2, ensure_ascii=False)
                        
                        success(f"已更新 {target_file.name} 中的 zipUrl:")
                        info(f"  原始: {original_url}")
                        info(f"  修改: {proxied_url}")
                        info(f"  ✅ latest → {tag_name}")
                
                # 🔥 为 update.json 添加到 proxy_links 中
                proxy_links.append(f"\n### 📥 {target_file.name} (更新配置文件)")
                proxy_links.append("\n**🔗 下载链接:**")
                
                # 生成 update.json 的官方链接
                update_json_url = f"https://github.com/{repo_name}/releases/download/{tag_name}/{target_file.name}"
                proxy_links.append(f"- [📦 官方下载]({update_json_url})")
                
                # 生成 update.json 的代理下载链接
                for proxy in proxies[:2]:  # 为 update.json 显示前2个代理
                    try:
                        if isinstance(proxy, dict) and 'url' in proxy:
                            proxy_url = str(proxy['url'])
                            if not proxy_url.startswith('http'):
                                proxy_url = f"https://{proxy_url}"
                            
                            full_proxy_url = f"{proxy_url}/{update_json_url}"
                            proxy_name = str(proxy['url']).replace('https://', '').replace('http://', '')
                            proxy_links.append(f"- [🚀 {proxy_name}]({full_proxy_url})")
                        elif isinstance(proxy, str):
                            proxy_url = proxy
                            if not proxy_url.startswith('http'):
                                proxy_url = f"https://{proxy_url}"
                            
                            full_proxy_url = f"{proxy_url}/{update_json_url}"
                            proxy_name = proxy_url.replace('https://', '').replace('http://', '').replace('/', '')
                            proxy_links.append(f"- [🚀 {proxy_name}]({full_proxy_url})")
                    except Exception as e:
                        warning(f"处理 {target_file.name} 代理失败: {e}")
                        continue
                
            except Exception as e:
                warning(f"处理 {target_file.name} 失败: {e}")
        else:
            # 处理其他文件，生成多个代理加速链接
            file_name = target_file.name
            
            # 添加文件下载部分到 release_body
            proxy_links.append(f"\n### 📥 {file_name}")
            proxy_links.append("\n**🔗 下载链接:**")
            
            # ⚠️ 重要：其他文件使用具体的 tag，不使用 latest！
            original_url = f"https://github.com/{repo_name}/releases/download/{tag_name}/{file_name}"
            proxy_links.append(f"- [📦 官方下载]({original_url})")
            
            # 生成代理下载链接
            for proxy in proxies[:4]:  # 显示前4个代理
                try:
                    # 从代理字典中提取URL
                    if isinstance(proxy, dict) and 'url' in proxy:
                        proxy_url = str(proxy['url'])
                        if not proxy_url.startswith('http'):
                            proxy_url = f"https://{proxy_url}"
                        
                        # 生成完整的代理链接
                        full_proxy_url = f"{proxy_url}/{original_url}"
                        
                        # 提取代理名称用于显示
                        proxy_name = str(proxy['url']).replace('https://', '').replace('http://', '')
                        location = str(proxy.get('location', '')).strip()
                        speed_val = proxy.get('speed', 0)
                          # 安全转换 speed 值
                        try:
                            speed = float(str(speed_val)) if speed_val else 0
                        except (ValueError, TypeError):
                            speed = 0
                        
                        # 生成显示名称
                        display_name = f"🚀 {proxy_name}"
                        if location:
                            display_name += f" ({location})"
                        if speed > 0:
                            display_name += f" - {speed:.1f}MB/s"
                        
                        proxy_links.append(f"- [{display_name}]({full_proxy_url})")
                    elif isinstance(proxy, str):
                        # 如果是字符串格式
                        proxy_url = proxy
                        if not proxy_url.startswith('http'):
                            proxy_url = f"https://{proxy_url}"
                        
                        full_proxy_url = f"{proxy_url}/{original_url}"
                        proxy_name = proxy_url.replace('https://', '').replace('http://', '').replace('/', '')
                        proxy_links.append(f"- [🚀 {proxy_name}]({full_proxy_url})")
                except Exception as e:
                    warning(f"处理代理失败: {e}")
                    continue
      # 处理 module.prop 文件中的 updateJson 链接
    if module_prop.exists():
        try:
            content = module_prop.read_text(encoding='utf-8')
            
            # 查找并替换 updateJson 链接
            update_json_pattern = r'updateJson=(https://github\.com/[^\s]+)'
            match = re.search(update_json_pattern, content)
            
            if match:
                original_update_url = match.group(1)
                
                # 🔥 修复：正确拼接代理URL
                best_proxy_str = str(best_proxy)
                if not best_proxy_str.endswith('/'):
                    best_proxy_str += '/'
                
                # 确保代理URL格式正确
                if best_proxy_str.startswith('http'):
                    proxied_update_url = best_proxy_str + original_update_url
                else:
                    proxied_update_url = f"https://{best_proxy_str}" + original_update_url
                
                # 替换链接
                new_content = content.replace(original_update_url, proxied_update_url)
                module_prop.write_text(new_content, encoding='utf-8')
                
                success("已更新 module.prop 中的 updateJson 链接:")
                info(f"  原始: {original_update_url}")
                info(f"  修改: {proxied_update_url}")
                info("  ✅ module.prop 使用 latest (正确)")
            
        except Exception as e:
            warning(f"处理 module.prop 失败: {e}")
            
    else:
        warning(f"未找到 module.prop 文件: {module_prop}")
    
    # 将代理链接添加到 release_body
    if proxy_links:
        proxy_section = "\n\n---\n## 🚀 加速下载\n" + "\n".join(proxy_links)
        release_body += proxy_section
    
    return release_body