"""
GitHub 发布模块
处理 RMM 项目的 GitHub Release 发布功能
"""
import os
import sys
import json
import requests
from pathlib import Path
from typing import Optional, Dict, Any, List
from github import Github, GithubException
from github.Repository import Repository
from github.GitRelease import GitRelease

class GitHubPublisher:
    def __init__(self, token: str, repo_name: str):
        """
        初始化 GitHub 发布器
        
        Args:
            token: GitHub personal access token
            repo_name: 仓库名称 (owner/repo)
        """
        self.github = Github(token)
        self.repo_name = repo_name
        self.repo: Repository = None
        
    def initialize_repo(self) -> bool:
        """初始化仓库连接"""
        try:
            self.repo = self.github.get_repo(self.repo_name)
            return True
        except GithubException as e:
            print(f"❌ 无法连接到仓库 {self.repo_name}: {e}")
            return False
    
    def get_fastest_proxy(self) -> Optional[str]:
        """获取最快的 GitHub 代理"""
        try:
            response = requests.get("https://api.akams.cn/github", timeout=10)
            response.raise_for_status()
            data = response.json()
            
            if data.get("code") == 200 and "data" in data:
                proxies = data["data"]
                if proxies:
                    # 返回第一个代理（API 已按速度排序）
                    return proxies[0]["proxy"]
        except Exception as e:
            print(f"⚠️  获取代理失败: {e}")
        
        return None
    
    def apply_proxy_to_url(self, url: str, proxy: str) -> str:
        """将代理应用到 URL"""
        if not proxy or not url:
            return url
        
        # 移除代理 URL 末尾的斜杠
        proxy = proxy.rstrip('/')
        
        # 如果 URL 是 GitHub 相关的，应用代理
        if "github.com" in url or "githubusercontent.com" in url:
            return f"{proxy}/{url}"
        
        return url
    
    def create_release(self, 
                      version: str, 
                      name: str,
                      body: str,
                      draft: bool = False,
                      prerelease: bool = False) -> Optional[GitRelease]:
        """
        创建 GitHub Release
        
        Args:
            version: 版本标签 (如 v0.1.0-abc123)
            name: Release 名称
            body: Release 描述
            draft: 是否为草稿
            prerelease: 是否为预发布版本
        
        Returns:
            GitRelease 对象或 None
        """
        try:
            print(f"📦 创建 Release: {version}")
            release = self.repo.create_git_release(
                tag=version,
                name=name,
                message=body,
                draft=draft,
                prerelease=prerelease
            )
            print(f"✅ Release 创建成功: {release.html_url}")
            return release
        except GithubException as e:
            if e.status == 422 and "already_exists" in str(e):
                print(f"⚠️  Release {version} 已存在，尝试获取现有 Release")
                try:
                    release = self.repo.get_release(version)
                    print(f"✅ 获取到现有 Release: {release.html_url}")
                    return release
                except GithubException:
                    print(f"❌ 无法获取现有 Release {version}")
                    return None
            else:
                print(f"❌ 创建 Release 失败: {e}")
                return None
    
    def upload_asset(self, release: GitRelease, file_path: Path, name: Optional[str] = None) -> bool:
        """
        上传文件到 Release
        
        Args:
            release: GitRelease 对象
            file_path: 要上传的文件路径
            name: 文件名（可选，默认使用文件名）
        
        Returns:
            是否上传成功
        """
        if not file_path.exists():
            print(f"❌ 文件不存在: {file_path}")
            return False
        
        asset_name = name or file_path.name
        
        try:
            print(f"📤 上传文件: {asset_name}")
            
            # 检查是否已存在同名文件
            for asset in release.get_assets():
                if asset.name == asset_name:
                    print(f"⚠️  文件 {asset_name} 已存在，删除旧文件")
                    asset.delete_asset()
                    break
            
            # 上传新文件
            with open(file_path, 'rb') as f:
                asset = release.upload_asset(f, name=asset_name)
            
            print(f"✅ 文件上传成功: {asset.browser_download_url}")
            return True
        except GithubException as e:
            print(f"❌ 文件上传失败: {e}")
            return False
        except Exception as e:
            print(f"❌ 上传过程中出错: {e}")
            return False
    
    def generate_proxy_links(self, release: GitRelease, proxy: str) -> str:
        """
        生成代理加速链接文本
        
        Args:
            release: GitRelease 对象
            proxy: 代理 URL
        
        Returns:
            代理链接文本
        """
        if not proxy:
            return ""
        
        links_text = "\n\n## 🚀 加速下载链接\n\n"
        links_text += "如果直接下载较慢，可以使用以下代理加速链接：\n\n"
        
        # 获取所有资产
        assets = list(release.get_assets())
        
        for asset in assets:
            original_url = asset.browser_download_url
            proxy_url = self.apply_proxy_to_url(original_url, proxy)
            
            if asset.name.endswith('.zip'):
                emoji = "📦"
                desc = "模块包"
            elif asset.name.endswith('.tar.gz'):
                emoji = "📋"
                desc = "源码包"
            else:
                emoji = "📄"
                desc = "文件"
            
            links_text += f"- {emoji} **{desc}**: [{asset.name}]({proxy_url})\n"
        
        # 添加源码下载链接
        source_zip_url = f"https://github.com/{self.repo_name}/archive/refs/tags/{release.tag_name}.zip"
        source_tar_url = f"https://github.com/{self.repo_name}/archive/refs/tags/{release.tag_name}.tar.gz"
        
        proxy_zip_url = self.apply_proxy_to_url(source_zip_url, proxy)
        proxy_tar_url = self.apply_proxy_to_url(source_tar_url, proxy)
        
        links_text += f"- 📁 **源码 (ZIP)**: [源码包.zip]({proxy_zip_url})\n"
        links_text += f"- 📁 **源码 (TAR.GZ)**: [源码包.tar.gz]({proxy_tar_url})\n"
        
        links_text += f"\n> 代理服务: `{proxy.replace('https://', '').replace('http://', '')}`\n"
        
        return links_text
    
    def update_release_body(self, release: GitRelease, new_body: str) -> bool:
        """
        更新 Release 描述
        
        Args:
            release: GitRelease 对象
            new_body: 新的描述内容
        
        Returns:
            是否更新成功
        """
        try:
            release.update_release(
                name=release.title,
                message=new_body,
                draft=release.draft,
                prerelease=release.prerelease
            )
            print("✅ Release 描述更新成功")
            return True
        except GithubException as e:
            print(f"❌ 更新 Release 描述失败: {e}")
            return False

def publish_to_github(config_data: Dict[str, Any]) -> bool:
    """
    发布到 GitHub 的主函数
    
    Args:
        config_data: 包含发布配置的字典
    
    Returns:
        是否发布成功
    """
    # 获取 GitHub token (优先级: GITHUB_TOKEN > GITHUB_ACCESS_TOKEN)
    token = os.getenv('GITHUB_TOKEN') or os.getenv('GITHUB_ACCESS_TOKEN')
    if not token:
        print("❌ 未找到 GitHub Token")
        print("请设置以下环境变量之一:")
        print("  export GITHUB_TOKEN=your_token_here")
        print("  export GITHUB_ACCESS_TOKEN=your_token_here")
        return False
    
    # 解析配置
    repo_name = config_data.get('repo_name')
    version = config_data.get('version')
    release_name = config_data.get('release_name')
    release_body = config_data.get('release_body', '')
    module_zip_path = config_data.get('module_zip_path')
    source_tar_path = config_data.get('source_tar_path')
    enable_proxy = config_data.get('enable_proxy', False)
    draft = config_data.get('draft', False)
    prerelease = config_data.get('prerelease', False)
    
    if not all([repo_name, version, release_name]):
        print("❌ 缺少必要的配置参数")
        return False
    
    # 初始化发布器
    publisher = GitHubPublisher(token, repo_name)
    if not publisher.initialize_repo():
        return False
    
    # 创建 Release
    release = publisher.create_release(
        version=version,
        name=release_name,
        body=release_body,
        draft=draft,
        prerelease=prerelease
    )
    
    if not release:
        return False
    
    # 上传文件
    upload_success = True
    
    if module_zip_path and Path(module_zip_path).exists():
        if not publisher.upload_asset(release, Path(module_zip_path)):
            upload_success = False
    
    if source_tar_path and Path(source_tar_path).exists():
        if not publisher.upload_asset(release, Path(source_tar_path)):
            upload_success = False
    
    # 如果启用代理功能，添加代理链接
    if enable_proxy and upload_success:
        print("🔍 正在获取最快的 GitHub 代理...")
        proxy = publisher.get_fastest_proxy()
        
        if proxy:
            print(f"✅ 选择代理: {proxy}")
            proxy_links = publisher.generate_proxy_links(release, proxy)
            updated_body = release_body + proxy_links
            publisher.update_release_body(release, updated_body)
        else:
            print("⚠️  未找到可用代理，跳过代理链接添加")
    
    if upload_success:
        print(f"🎉 发布完成! Release URL: {release.html_url}")
        return True
    else:
        print("❌ 部分文件上传失败")
        return False

def main():
    """命令行入口"""
    if len(sys.argv) != 2:
        print("用法: python publisher.py <config_json>")
        sys.exit(1)
    
    config_json = sys.argv[1]
    
    try:
        config_data = json.loads(config_json)
    except json.JSONDecodeError as e:
        print(f"❌ 配置 JSON 解析失败: {e}")
        sys.exit(1)
    
    success = publish_to_github(config_data)
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
