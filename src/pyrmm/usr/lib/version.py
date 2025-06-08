from pathlib import Path
import subprocess
import re
from datetime import datetime
import json
from typing import Any

# 类型别名
GitInfo = tuple[str | None, str | None, str | None]
VersionDict = dict[str, str | dict[str, str | None]]
CacheEntry = dict[str, int | str]

class VersionGeneratorMeta(type):
    """版本生成器元类"""
    
    # 版本缓存，避免同一天重复生成
    _version_cache: dict[str, dict[str, int | str]] = {}
    _cache_file: Path = Path.home() / ".rmm" / "version_cache.json"
    
    def __init__(cls, name: str, bases: tuple[type, ...], attrs: dict[str, Any]) -> None:
        super().__init__(name, bases, attrs)
        cls._load_cache()
    
    def _load_cache(cls) -> None:
        """加载版本缓存"""
        if cls._cache_file.exists():
            try:
                with open(cls._cache_file, 'r', encoding='utf-8') as f:
                    cls._version_cache = json.load(f)
            except Exception:
                cls._version_cache = {}
    
    def _save_cache(cls) -> None:
        """保存版本缓存"""
        cls._cache_file.parent.mkdir(parents=True, exist_ok=True)
        try:
            with open(cls._cache_file, 'w', encoding='utf-8') as f:
                json.dump(cls._version_cache, f, indent=2, ensure_ascii=False)
        except Exception:
            pass

    def _get_git_info(cls, project_path: Path) -> GitInfo:
        """获取 Git 信息：(分支名, 提交哈希, 标签)"""
        try:
            # 查找 Git 根目录
            git_root = project_path
            while git_root != git_root.parent:
                if (git_root / ".git").exists():
                    break
                git_root = git_root.parent
            else:
                return None, None, None
            
            # 获取当前分支
            try:
                result = subprocess.run(
                    ["git", "branch", "--show-current"],
                    cwd=git_root,
                    capture_output=True,
                    text=True,
                    timeout=5
                )
                branch = result.stdout.strip() if result.returncode == 0 else None
            except Exception:
                branch = None
            
            # 获取当前提交哈希
            try:
                result = subprocess.run(
                    ["git", "rev-parse", "--short", "HEAD"],
                    cwd=git_root,
                    capture_output=True,
                    text=True,
                    timeout=5
                )
                commit = result.stdout.strip() if result.returncode == 0 else None
            except Exception:
                commit = None
            
            # 获取最近的标签
            try:
                result = subprocess.run(
                    ["git", "describe", "--tags", "--abbrev=0"],
                    cwd=git_root,
                    capture_output=True,
                    text=True,
                    timeout=5
                )
                tag = result.stdout.strip() if result.returncode == 0 else None
            except Exception:
                tag = None
            
            return branch, commit, tag
            
        except Exception:
            return None, None, None

    def _parse_version(cls, version_str: str) -> tuple[int, int, int]:
        """解析版本号 vX.Y.Z -> (X, Y, Z)"""
        if not version_str:
            return 1, 0, 0
        
        # 移除 'v' 前缀并解析
        clean_version = version_str.lstrip('v')
        match = re.match(r'^(\d+)\.(\d+)\.(\d+)', clean_version)
        if match:
            return int(match.group(1)), int(match.group(2)), int(match.group(3))
        return 1, 0, 0
    
    def _generate_version_code(cls, project_path: str) -> str:
        """生成版本代码：YYYYMMDDNN"""
        now = datetime.now()
        date_prefix = now.strftime("%Y%m%d")
        
        # 从缓存中获取今天的序列号
        cache_key = f"{project_path}_{date_prefix}"
        
        if cache_key in cls._version_cache:
            cache_data = cls._version_cache[cache_key]
            sequence = int(cache_data.get('sequence', 0)) + 1
        else:
            sequence = 1
        
        # 确保序列号不超过 99
        if sequence > 99:
            sequence = 99
        
        # 更新缓存
        cls._version_cache[cache_key] = {
            'sequence': sequence,
            'timestamp': now.isoformat()
        }
        cls._save_cache()
        version_code = f"{date_prefix}{sequence:02d}"
        
        return version_code
    
    def generate(cls, old_version: str = "", project_path: Path | None = None, bump_type: str = "patch") -> VersionDict:
        """
        生成新版本
        
        Args:
            old_version: 旧版本号，如 "v1.2.3"
            project_path: 项目路径，用于获取 Git 信息
            bump_type: 版本升级类型 ("major", "minor", "patch")
        
        Returns:
            包含 version 和 versionCode 的字典
        """
        if project_path is None:
            project_path = Path.cwd()
        
        # 解析旧版本
        major, minor, patch = cls._parse_version(old_version)
        
        # 根据升级类型计算新版本
        if bump_type == "major":
            major += 1
            minor = 0
            patch = 0
        elif bump_type == "minor":
            minor += 1
            patch = 0
        else:  # patch
            patch += 1
        
        # 获取 Git 信息
        branch, commit, tag = cls._get_git_info(project_path)
        
        # 构建版本标签
        version_tags: list[str] = []
        if branch and branch != "main" and branch != "master":
            version_tags.append(branch)
        if commit:
            version_tags.append(commit)
        
        # 构建完整版本号
        version = f"v{major}.{minor}.{patch}"
        if version_tags:
            version += f"-{'.'.join(version_tags)}"
        
        # 生成版本代码
        version_code = cls._generate_version_code(str(project_path.resolve()))

        version_info: VersionDict = {
            "version": version,
            "versionCode": version_code,
            "git_info": {
                "branch": branch,
                "commit": commit,
                "tag": tag
            }
        }

        return version_info

    def auto_bump(cls, old_version: str = "", project_path: Path | None = None) -> VersionDict:
        """
        自动确定版本升级类型并生成新版本
        基于 Git 提交信息判断升级类型
        """
        if project_path is None:
            project_path = Path.cwd()
        
        # 获取最近的提交信息
        try:
            git_root = project_path
            while git_root != git_root.parent:
                if (git_root / ".git").exists():
                    break
                git_root = git_root.parent
            
            result = subprocess.run(
                ["git", "log", "--oneline", "-10"],
                cwd=git_root,
                capture_output=True,
                text=True,
                timeout=5
            )
            
            if result.returncode == 0:
                commits = result.stdout.lower()
                
                # 判断升级类型
                if any(keyword in commits for keyword in ["breaking", "major", "!:"]):
                    bump_type = "major"
                elif any(keyword in commits for keyword in ["feat", "feature", "add", "new"]):
                    bump_type = "minor"
                else:
                    bump_type = "patch"
            else:
                bump_type = "patch"
                
        except Exception:
            bump_type = "patch"
        
        # 调用 generate 方法生成新版本
        new_version_info = cls.generate(old_version, project_path, bump_type)

        return new_version_info


class VersionGenerator(metaclass=VersionGeneratorMeta):
    """版本生成器
    
    使用示例:
        # 基本使用
        result = VersionGenerator.generate("v1.2.3")
        print(result["version"])     # v1.2.4-main.abc123
        print(result["versionCode"]) # 2025060801
        
        # 指定升级类型
        result = VersionGenerator.generate("v1.2.3", bump_type="minor")
        print(result["version"])     # v1.3.0-main.abc123
        
        # 自动判断升级类型
        result = VersionGenerator.auto_bump("v1.2.3")
        
        # 更新项目文件
        VersionGenerator.update_project_files(project_path, version_info)
    """
    
    @classmethod
    def update_project_files(cls, project_path: Path, version_info: VersionDict | None = None) -> bool:
        """
        更新项目文件中的版本信息
        
        Args:
            project_path: 项目路径
            version_info: 版本信息字典，如果为None则生成新版本
            
        Returns:
            bool: 更新是否成功
        """
        try:
            import toml
            
            # 如果没有提供版本信息，则生成新版本
            if version_info is None:
                version_info = cls.generate("", project_path)
            # 更新 module.prop 文件
            module_prop_path = project_path / "module.prop"
            if module_prop_path.exists():
                # 读取现有内容
                module_prop_content: dict[str, str] = {}
                with open(module_prop_path, 'r', encoding='utf-8') as f:
                    for line in f:
                        line = line.strip()
                        if line and '=' in line and not line.startswith('#'):
                            key, value = line.split('=', 1)
                            module_prop_content[key] = value
                
                # 更新版本信息
                module_prop_content['version'] = str(version_info['version'])
                module_prop_content['versionCode'] = str(version_info['versionCode'])
                
                # 写回文件
                with open(module_prop_path, 'w', encoding='utf-8') as f:
                    for key, value in module_prop_content.items():
                        f.write(f"{key}={value}\n")
                
                print(f"✅ 已更新 module.prop: {version_info['version']} (代码: {version_info['versionCode']})")
              # 更新 rmmproject.toml 文件
            rmmproject_path = project_path / "rmmproject.toml"
            if rmmproject_path.exists():
                # 读取现有配置
                with open(rmmproject_path, 'r', encoding='utf-8') as f:
                    project_config: dict[str, Any] = toml.load(f)
                
                # 更新版本信息
                project_config['version'] = str(version_info['version'])
                project_config['versionCode'] = str(version_info['versionCode'])
                
                # 写回文件
                with open(rmmproject_path, 'w', encoding='utf-8') as f:
                    toml.dump(project_config, f)
                
                print(f"✅ 已更新 rmmproject.toml: {version_info['version']} (代码: {version_info['versionCode']})")
            
            return True
            
        except Exception as e:
            print(f"❌ 更新项目文件版本信息失败: {e}")
            return False
    
    @classmethod
    def sync_and_update(cls, project_path: Path, old_version: str = "", bump_type: str = "patch") -> VersionDict:
        """
        生成新版本并同步更新到项目文件
        
        Args:
            project_path: 项目路径
            old_version: 旧版本号
            bump_type: 升级类型 ("major", "minor", "patch")
            
        Returns:
            VersionDict: 生成的版本信息
        """
        # 生成新版本
        version_info = cls.generate(old_version, project_path, bump_type)
        
        # 更新项目文件
        cls.update_project_files(project_path, version_info)
        
        return version_info