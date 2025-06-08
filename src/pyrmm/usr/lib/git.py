import os
import subprocess
from pathlib import Path
import configparser
import re

from typing import Any, TYPE_CHECKING
from dataclasses import dataclass

from .base import RmmBaseMeta, RmmBase

if TYPE_CHECKING:
    from github import Github
    from github.GithubException import GithubException
    from github.Repository import Repository

try:
    from github import Github
    from github.GithubException import GithubException
    from github.Repository import Repository
    _github_available = True
except ImportError:
    _github_available = False
    
@dataclass
class GitRemoteInfo:
    """Git远程仓库信息"""
    name: str
    url: str
    username: str | None = None
    repo_name: str | None = None

@dataclass
class GitBranchInfo:
    """Git分支信息"""
    name: str
    remote: str | None = None
    merge: str | None = None

@dataclass
class GitRepoInfo:
    """Git仓库完整信息"""
    root_path: Path
    remotes: dict[str, GitRemoteInfo]
    branches: dict[str, GitBranchInfo]
    current_branch: str | None = None
    is_clean: bool = True

class RmmGitMeta(RmmBaseMeta):
    """Git操作的元类"""
    
    @property
    def META(cls) -> dict[str, Any]:
        """获取Git配置元数据"""
        return {}
    
    def get_config_key(cls) -> str:
        """获取配置键名"""
        return "git"
    
    def get_reserved_key(cls) -> str:
        """获取保留关键字"""
        return "rmm_git"
    
    def get_item_config(cls, item_name: str) -> dict[str, Any]:
        """获取Git配置"""
        return {}
    
    def _set_item_config(cls, name: str, value: dict[str, Any]) -> None:
        """设置Git配置"""
        pass
    
    def _delete_item_config(cls, name: str) -> None:
        """删除Git配置"""
        pass

class RmmGit(RmmBase, metaclass=RmmGitMeta):
    """RMM Git 操作类"""
    # Git仓库信息缓存
    _git_cache: dict[str, GitRepoInfo] = {}
    _git_mtime: dict[str, float] = {}
    
    @classmethod
    def find_git_root(cls, start_path: Path, max_levels: int = 5) -> Path | None:
        """向上查找 Git 仓库根目录
        
        Args:
            start_path: 开始搜索的路径
            max_levels: 最大向上搜索层数
            
        Returns:
            Git仓库根目录路径，如果没找到返回None
        """
        current_path = start_path.resolve()
        level = 0
        
        while current_path != current_path.parent and level <= max_levels:
            git_dir = current_path / ".git"
            if git_dir.exists():
                return current_path
            current_path = current_path.parent
            level += 1
        
        return None
    
    @classmethod
    def parse_git_config(cls, git_root: Path) -> dict[str, Any]:
        """解析 Git 配置文件
        
        Args:
            git_root: Git仓库根目录
            
        Returns:
            包含远程仓库和分支信息的字典
        """
        git_config_path = git_root / ".git" / "config"
        if not git_config_path.exists():
            return {"remotes": {}, "branches": {}}
        
        config = configparser.ConfigParser(allow_no_value=True, strict=False)
        try:
            config.read(git_config_path, encoding='utf-8')
            
            # 提取远程仓库信息
            remote_info = {}
            for section_name in config.sections():
                if section_name.startswith('remote "'):
                    remote_name = section_name.split('"')[1]
                    if 'url' in config[section_name]:
                        url = config[section_name]['url']
                        username, repo_name = cls.extract_repo_info(url)
                        remote_info[remote_name] = GitRemoteInfo(
                            name=remote_name,
                            url=url,
                            username=username,
                            repo_name=repo_name
                        )
            
            # 提取分支信息
            branch_info = {}
            for section_name in config.sections():
                if section_name.startswith('branch "'):
                    branch_name = section_name.split('"')[1]
                    branch_data = dict(config[section_name])
                    branch_info[branch_name] = GitBranchInfo(
                        name=branch_name,
                        remote=branch_data.get('remote'),
                        merge=branch_data.get('merge')
                    )
            
            return {
                'remotes': remote_info,
                'branches': branch_info
            }
        except Exception as e:
            print(f"解析 Git 配置时出错: {e}")
            return {"remotes": {}, "branches": {}}
    
    @classmethod
    def extract_repo_info(cls, remote_url: str) -> tuple[str | None, str | None]:
        """从远程 URL 中提取用户名和仓库名
        
        Args:
            remote_url: Git远程仓库URL
            
        Returns:
            (username, repo_name) 元组，如果解析失败返回 (None, None)
        """
        if not remote_url:
            return None, None
        
        # 支持 HTTPS 和 SSH 格式
        # HTTPS: https://github.com/username/repo.git
        # SSH: git@github.com:username/repo.git
        
        # HTTPS 格式
        https_pattern = r'https://github\.com/([^/]+)/([^/]+?)(?:\.git)?/?$'
        match = re.match(https_pattern, remote_url)
        if match:
            return match.group(1), match.group(2)
        
        # SSH 格式
        ssh_pattern = r'git@github\.com:([^/]+)/([^/]+?)(?:\.git)?/?$'
        match = re.match(ssh_pattern, remote_url)
        if match:
            return match.group(1), match.group(2)
        
        # 支持其他Git托管平台的URL格式
        # GitLab HTTPS: https://gitlab.com/username/repo.git
        gitlab_https_pattern = r'https://gitlab\.com/([^/]+)/([^/]+?)(?:\.git)?/?$'
        match = re.match(gitlab_https_pattern, remote_url)
        if match:
            return match.group(1), match.group(2)
        
        # GitLab SSH: git@gitlab.com:username/repo.git
        gitlab_ssh_pattern = r'git@gitlab\.com:([^/]+)/([^/]+?)(?:\.git)?/?$'
        match = re.match(gitlab_ssh_pattern, remote_url)
        if match:
            return match.group(1), match.group(2)
        
        return None, None
    
    @classmethod
    def get_repo_info(cls, project_path: Path, use_cache: bool = True) -> GitRepoInfo | None:
        """获取完整的Git仓库信息
        
        Args:
            project_path: 项目路径
            use_cache: 是否使用缓存
            
        Returns:
            GitRepoInfo对象或None
        """
        git_root = cls.find_git_root(project_path)
        if not git_root:
            return None
        
        cache_key = str(git_root.resolve())
        
        if use_cache and cache_key in cls._git_cache:
            return cls._git_cache[cache_key]
        
        try:
            git_config = cls.parse_git_config(git_root)
            
            repo_info = GitRepoInfo(
                root_path=git_root,
                remotes=git_config.get('remotes', {}),
                branches=git_config.get('branches', {})
            )
            
            # 尝试获取当前分支
            try:
                head_file = git_root / ".git" / "HEAD"
                if head_file.exists():
                    with open(head_file, 'r', encoding='utf-8') as f:
                        head_content = f.read().strip()
                        if head_content.startswith('ref: refs/heads/'):
                            repo_info.current_branch = head_content.replace('ref: refs/heads/', '')
            except Exception:
                pass
            
            # 检查仓库状态
            repo_info.is_clean = cls.is_repo_clean(project_path)
            
            # 缓存结果
            if use_cache:
                cls._git_cache[cache_key] = repo_info
            
            return repo_info
            
        except Exception as e:
            print(f"获取Git仓库信息时出错: {e}")
            return None
        
    @classmethod
    def get_github_repo(cls, username: str, repo_name: str, token: str | None = None) -> Any | None:
        """获取GitHub仓库对象
        
        Args:
            username: GitHub用户名
            repo_name: 仓库名
            token: GitHub API token (可选)
            
        Returns:
            Repository对象或None
        """
        if not _github_available:
            print("⚠️  警告: PyGithub库未安装，无法使用GitHub API功能")
            return None
        
        try:
            # 使用token或者匿名访问
            if token:
                g: Github = Github(token)
            else:
                # 尝试从环境变量获取token
                env_token = os.getenv('GITHUB_ACCESS_TOKEN')
                if env_token:
                    g = Github(env_token)
                else:
                    g = Github()  # 匿名访问，有API限制
            
            repo: Repository = g.get_repo(f"{username}/{repo_name}")
            return repo
            
        except GithubException as e:
            print(f"获取GitHub仓库失败: {e}")
            return None
        except Exception as e:
            print(f"GitHub API调用出错: {e}")
            return None
        
    @classmethod
    def get_repo_latest_release(cls, username: str, repo_name: str, token: str | None = None) -> dict[str, Any] | None:
        """获取仓库的最新release信息
        
        Args:
            username: GitHub用户名
            repo_name: 仓库名
            token: GitHub API token (可选)
            
        Returns:
            包含release信息的字典或None
        """
        repo: Any | None = cls.get_github_repo(username, repo_name, token)
        if not repo:
            return None
        
        try:
            latest_release: Any = repo.get_latest_release()
            return {
                'tag_name': str(latest_release.tag_name),
                'name': str(latest_release.title),
                'body': str(latest_release.body),
                'published_at': latest_release.published_at,
                'html_url': str(latest_release.html_url),
                'download_url': str(latest_release.tarball_url),
                'assets': [
                    {
                        'name': str(asset.name),
                        'download_url': str(asset.browser_download_url),
                        'size': int(asset.size)
                    }
                    for asset in latest_release.get_assets()
                ]
            }
        except GithubException as e:
            print(f"获取最新release失败: {e}")
            return None

    @classmethod
    def check_repo_exists(cls, username: str, repo_name: str, token: str | None = None) -> bool:
        """检查GitHub仓库是否存在
        
        Args:
            username: GitHub用户名
            repo_name: 仓库名
            token: GitHub API token (可选)
            
        Returns:
            仓库是否存在
        """
        repo: Any | None = cls.get_github_repo(username, repo_name, token)
        return repo is not None

    @classmethod
    def create_release(cls, username: str, repo_name: str, tag_name: str, 
                      release_name: str, body: str = "", draft: bool = False, 
                      prerelease: bool = False, token: str | None = None) -> dict[str, Any] | None:
        """创建GitHub release
        
        Args:
            username: GitHub用户名
            repo_name: 仓库名
            tag_name: 标签名
            release_name: Release名称
            body: Release描述
            draft: 是否为草稿
            prerelease: 是否为预发布版本
            token: GitHub API token (可选)
            
        Returns:
            包含release信息的字典或None        """
        repo: Any | None = cls.get_github_repo(username, repo_name, token)
        if not repo:
            return None
        try:
            # 获取当前分支的最新 commit SHA
            default_branch = repo.default_branch
            branch = repo.get_branch(default_branch)
            target_commitish = branch.commit.sha
            
            # 创建 Git release，这会自动创建对应的 tag
            release: Any = repo.create_git_release(
                tag=tag_name,
                name=release_name,
                message=body,
                draft=draft,
                prerelease=prerelease,
                target_commitish=target_commitish
            )
            
            return {
                'id': release.id,
                'tag_name': str(release.tag_name),
                'name': str(release.title),
                'body': str(release.body),
                'html_url': str(release.html_url),
                'upload_url': str(release.upload_url),
                'draft': release.draft,
                'prerelease': release.prerelease
            }
        except GithubException as e:
            if e.status == 403:
                print(f"创建Release失败: GitHub token权限不足")
                print(f"错误详情: {e}")
                print("💡 请确保GitHub token具有以下权限:")
                print("   - repo (完整仓库权限)")
                print("   - contents:write (写入内容)")
                print("🔗 更新token权限: https://github.com/settings/tokens")
            elif e.status == 422:
                print(f"创建Release失败: 标签 '{tag_name}' 可能已存在")
                print(f"错误详情: {e}")
            else:
                print(f"创建Release失败: {e}")
            return None
        except Exception as e:
            print(f"GitHub API调用出错: {e}")
            return None

    @classmethod
    def upload_release_assets(cls, username: str, repo_name: str, tag_name: str, 
                             assets: list[Path], token: str | None = None) -> bool:
        """上传文件到GitHub release
        
        Args:
            username: GitHub用户名
            repo_name: 仓库名
            tag_name: 标签名
            assets: 要上传的文件路径列表
            token: GitHub API token (可选)
            
        Returns:
            是否上传成功
        """
        repo: Any | None = cls.get_github_repo(username, repo_name, token)
        if not repo:
            return False
        
        try:
            # 获取指定的release
            release: Any = repo.get_release(tag_name)
            
            success_count = 0
            for asset_path in assets:
                if not asset_path.exists():
                    print(f"⚠️  文件不存在: {asset_path}")
                    continue
                
                try:
                    print(f"📤 上传文件: {asset_path.name}")
                    release.upload_asset(str(asset_path))
                    success_count += 1
                    print(f"✅ 成功上传: {asset_path.name}")
                except Exception as e:
                    print(f"❌ 上传失败 {asset_path.name}: {e}")
            
            return success_count > 0
            
        except GithubException as e:
            print(f"获取Release失败: {e}")
            return False
        except Exception as e:
            print(f"上传资源失败: {e}")
            return False

    @classmethod
    def get_release_by_tag(cls, username: str, repo_name: str, tag_name: str, 
                          token: str | None = None) -> dict[str, Any] | None:
        """根据标签获取release信息
        
        Args:
            username: GitHub用户名
            repo_name: 仓库名
            tag_name: 标签名
            token: GitHub API token (可选)
            
        Returns:
            包含release信息的字典或None
        """
        repo: Any | None = cls.get_github_repo(username, repo_name, token)
        if not repo:
            return None
        try:
            release: Any = repo.get_release(tag_name)
            return {
                'id': release.id,
                'tag_name': str(release.tag_name),
                'name': str(release.title),
                'body': str(release.body),
                'published_at': release.published_at,
                'html_url': str(release.html_url),
                'draft': release.draft,
                'prerelease': release.prerelease,
                'assets': [
                    {
                        'name': str(asset.name),
                        'download_url': str(asset.browser_download_url),
                        'size': int(asset.size)
                    }
                    for asset in release.get_assets()
                ]
            }
        except GithubException as e:
            # 404错误是正常情况（release不存在），不需要显示错误信息
            if e.status != 404:
                print(f"获取Release失败: {e}")
            return None
    
    @classmethod
    def get_commit_info(cls, project_path: Path, commit_hash: str | None = None) -> dict[str, Any] | None:
        """获取Git提交信息
        
        Args:
            project_path: 项目路径
            commit_hash: 提交哈希值，None表示获取最新提交
            
        Returns:
            包含提交信息的字典或None
        """
        git_root = cls.find_git_root(project_path)
        if not git_root:
            return None
        
        try:
            cmd = ['git', 'log', '-1', '--format=%H|%an|%ae|%ad|%s']
            if commit_hash:
                cmd.append(commit_hash)
            
            result = subprocess.run(
                cmd,
                cwd=git_root,
                capture_output=True,
                text=True,
                encoding='utf-8'
            )
            
            if result.returncode == 0 and result.stdout.strip():
                parts = result.stdout.strip().split('|', 4)
                if len(parts) == 5:
                    return {
                        'hash': parts[0],
                        'author_name': parts[1],
                        'author_email': parts[2],
                        'date': parts[3],
                        'message': parts[4]
                    }
        except Exception as e:
            print(f"获取Git提交信息失败: {e}")
        
        return None
    
    @classmethod
    def is_repo_clean(cls, project_path: Path) -> bool:
        """检查Git仓库是否为干净状态（无未提交的更改）
        
        Args:
            project_path: 项目路径
            
        Returns:
            仓库是否为干净状态
        """
        git_root = cls.find_git_root(project_path)
        if not git_root:
            return True  # 非Git仓库视为干净状态
        
        try:
            result = subprocess.run(
                ['git', 'status', '--porcelain'],
                cwd=git_root,
                capture_output=True,
                text=True,
                encoding='utf-8'
            )
            
            return result.returncode == 0 and not result.stdout.strip()
        except Exception:
            return True  # 出错时假设为干净状态
    
    @classmethod
    def get_repo_url_from_path(cls, project_path: Path, remote_name: str = 'origin') -> str | None:
        """从项目路径获取远程仓库URL
        
        Args:
            project_path: 项目路径
            remote_name: 远程仓库名，默认为'origin'
            
        Returns:
            远程仓库URL或None
        """
        repo_info = cls.get_repo_info(project_path)
        if not repo_info or remote_name not in repo_info.remotes:
            return None
        
        return repo_info.remotes[remote_name].url
    
    @classmethod
    def is_valid_item(cls, item_name: str) -> bool:
        """检查是否是有效的Git项目"""
        return True
    @classmethod
    def get_sync_prompt(cls, item_name: str) -> str:
        """获取同步提示信息"""
        return f"Git仓库 '{item_name}' 信息"

    @classmethod
    def get_local_tags(cls, project_path: Path) -> list[str]:
        """获取本地Git标签列表
        
        Args:
            project_path: 项目路径
            
        Returns:
            本地标签列表
        """
        git_root = cls.find_git_root(project_path)
        if not git_root:
            return []
        
        try:
            result = subprocess.run(
                ['git', 'tag', '--list'],
                cwd=git_root,
                capture_output=True,
                text=True,
                encoding='utf-8'
            )
            
            if result.returncode == 0:
                tags = [tag.strip() for tag in result.stdout.split('\n') if tag.strip()]
                return sorted(tags)
        except Exception as e:
            print(f"获取本地标签失败: {e}")
        
        return []
    
    @classmethod
    def get_github_releases(cls, username: str, repo_name: str, token: str | None = None) -> list[str]:
        """获取GitHub仓库的所有release标签
        
        Args:
            username: GitHub用户名
            repo_name: 仓库名
            token: GitHub API token (可选)
            
        Returns:
            release标签列表
        """
        repo: Any | None = cls.get_github_repo(username, repo_name, token)
        if not repo:
            return []
        
        try:
            releases: Any = repo.get_releases()
            release_tags = [str(release.tag_name) for release in releases]
            return sorted(release_tags)
        except GithubException as e:
            print(f"获取GitHub releases失败: {e}")
            return []
        except Exception as e:
            print(f"GitHub API调用出错: {e}")
            return []
    
    @classmethod
    def find_orphan_tags(cls, project_path: Path, remote_name: str = 'origin', 
                        token: str | None = None) -> list[str]:
        """查找孤立标签（本地存在但GitHub上没有对应release的标签）
        
        Args:
            project_path: 项目路径
            remote_name: 远程仓库名，默认为'origin'
            token: GitHub API token (可选)
            
        Returns:
            孤立标签列表
        """
        # 获取仓库信息
        repo_info = cls.get_repo_info(project_path)
        if not repo_info or remote_name not in repo_info.remotes:
            return []
        
        remote_info = repo_info.remotes[remote_name]
        if not remote_info.username or not remote_info.repo_name:
            return []
        
        # 获取本地标签
        local_tags = cls.get_local_tags(project_path)
        if not local_tags:
            return []
        
        # 获取GitHub releases
        release_tags = cls.get_github_releases(remote_info.username, remote_info.repo_name, token)
        
        # 找出本地存在但GitHub上没有release的标签
        orphan_tags = [tag for tag in local_tags if tag not in release_tags]
        
        return orphan_tags
    
    @classmethod
    def delete_local_tag(cls, project_path: Path, tag_name: str) -> bool:
        """删除本地Git标签
        
        Args:
            project_path: 项目路径
            tag_name: 标签名
            
        Returns:
            是否删除成功
        """
        git_root = cls.find_git_root(project_path)
        if not git_root:
            return False
        
        try:
            result = subprocess.run(
                ['git', 'tag', '-d', tag_name],
                cwd=git_root,
                capture_output=True,
                text=True,
                encoding='utf-8'
            )
            
            return result.returncode == 0
        except Exception as e:
            print(f"删除本地标签 {tag_name} 失败: {e}")
            return False
    
    @classmethod
    def clean_orphan_tags(cls, project_path: Path, remote_name: str = 'origin', 
                         token: str | None = None, dry_run: bool = False) -> tuple[list[str], list[str]]:
        """清理孤立标签
        
        Args:
            project_path: 项目路径
            remote_name: 远程仓库名，默认为'origin'
            token: GitHub API token (可选)
            dry_run: 是否为干运行（只查找不删除）
            
        Returns:
            (成功删除的标签, 删除失败的标签) 元组
        """
        orphan_tags = cls.find_orphan_tags(project_path, remote_name, token)
        
        if not orphan_tags:
            return [], []
        
        if dry_run:
            return orphan_tags, []
        
        success_tags: list[str] = []
        failed_tags: list[str] = []
        
        for tag in orphan_tags:
            # 删除本地标签
            if cls.delete_local_tag(project_path, tag):
                success_tags.append(tag)
                print(f"✅ 已删除本地标签: {tag}")
            else:
                failed_tags.append(tag)
                print(f"❌ 删除本地标签失败: {tag}")
        
        return success_tags, failed_tags


# 导出类型和常用函数
__all__ = [
    'RmmGit',
    'GitRemoteInfo', 
    'GitBranchInfo',
    'GitRepoInfo',
    'RmmGitMeta'
]
