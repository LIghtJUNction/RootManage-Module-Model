"""ShellCheck 自动安装器 - 支持跨平台自动下载安装"""
import os
import platform
# import subprocess - 延迟导入
# import tarfile - 延迟导入
from typing import Literal
# import zipfile - 延迟导入
from pathlib import Path
# import requests - 延迟导入
# import tempfile - 延迟导入
import stat

from .proxy import ProxyManager
from .fs import RmmFileSystem


class ShellCheckInstaller:
    """ShellCheck 自动安装器"""
    
    VERSION = "v0.10.0"
    BASE_URL = "https://github.com/koalaman/shellcheck/releases/download"
    
    # 平台架构映射
    PLATFORM_MAPPING = {
        "Windows": {
            "AMD64": "zip",
            "x86_64": "zip",
        },
        "Darwin": {  # macOS
            "arm64": "darwin.aarch64.tar.xz",
            "x86_64": "darwin.x86_64.tar.xz",
        },
        "Linux": {
            "x86_64": "linux.x86_64.tar.xz",
            "aarch64": "linux.aarch64.tar.xz",
            "armv6l": "linux.armv6hf.tar.xz",
            "armv7l": "linux.armv6hf.tar.xz",
            "riscv64": "linux.riscv64.tar.xz",
        }
    }
    
    @classmethod
    def detect_platform_arch(cls) -> tuple[str, str]:
        """检测当前平台和架构"""
        system = platform.system()
        machine = platform.machine().lower()
        
        # 标准化架构名称
        arch_mapping = {
            "amd64": "x86_64",
            "x64": "x86_64",
            "arm64": "aarch64" if system == "Linux" else "arm64",
            "aarch64": "aarch64",
            "armv6l": "armv6l",
            "armv7l": "armv7l",
            "riscv64": "riscv64",
        }
        
        normalized_arch = arch_mapping.get(machine, machine)
        
        return system, normalized_arch
    
    @classmethod
    def get_download_info(cls) -> tuple[str, str]:
        """获取下载文件信息"""
        system, arch = cls.detect_platform_arch()
        
        if system not in cls.PLATFORM_MAPPING:
            raise ValueError(f"不支持的平台: {system}")
        
        platform_map = cls.PLATFORM_MAPPING[system]
        if arch not in platform_map:
            # 尝试回退到 x86_64
            if "x86_64" in platform_map:
                print(f"⚠️ 未找到架构 {arch} 的版本，回退到 x86_64")
                arch = "x86_64"
            else:
                raise ValueError(f"平台 {system} 不支持架构: {arch}")
        
        file_suffix = platform_map[arch]
        
        if file_suffix == "zip":
            filename = f"shellcheck-{cls.VERSION}.zip"
        else:
            filename = f"shellcheck-{cls.VERSION}.{file_suffix}"
        
        download_url = f"{cls.BASE_URL}/{cls.VERSION}/{filename}"
        
        return filename, download_url
    
    @classmethod
    def get_proxy_urls(cls, project_path: Path, download_url: str) -> list[str]:
        """获取代理下载地址列表"""
        try:
            proxies = ProxyManager.load_project_proxies(project_path)
            if not proxies:
                # 如果没有缓存的代理，尝试获取新的
                print("📡 获取代理节点列表...")
                proxies, _ = ProxyManager.get_and_save_proxies(project_path)
            
            # 生成代理URL列表
            proxy_urls: list[str] = []
            for proxy in proxies[:5]:  # 只使用前5个最快的代理
                proxy_url = f"{proxy.url}/{download_url}"
                proxy_urls.append(proxy_url)
            
            return proxy_urls
        except Exception as e:
            print(f"⚠️ 获取代理失败: {e}")
            return []
    @classmethod
    def download_with_proxies(
        cls, 
        download_url: str, 
        proxy_urls: list[str], 
        output_path: Path,
        timeout: int = 120
    ) -> bool:
        """使用代理列表下载文件，自动尝试多个代理"""
        import requests  # 局部导入
        
        urls_to_try = proxy_urls + [download_url]  # 代理 + 原始地址
        url_type = "原始" if not proxy_urls else "代理"
        for i, url in enumerate(urls_to_try):
            try:
                is_proxy = i < len(proxy_urls)
                url_type: Literal['代理','原始'] = "代理" if is_proxy else "原始"
                
                print(f"🌐 尝试{url_type}下载 ({i+1}/{len(urls_to_try)}): {url}")
                
                # 发送请求
                response = requests.get(url, timeout=timeout, stream=True)
                response.raise_for_status()
                
                # 获取文件大小
                total_size = int(response.headers.get('content-length', 0))
                
                # 下载文件
                downloaded_size = 0
                with open(output_path, 'wb') as f:
                    for chunk in response.iter_content(chunk_size=8192):
                        if chunk:
                            f.write(chunk)
                            downloaded_size += len(chunk)
                            
                            # 显示下载进度
                            if total_size > 0:
                                progress = (downloaded_size / total_size) * 100
                                print(f"\r📥 下载进度: {progress:.1f}% ({downloaded_size}/{total_size} bytes)", end="")
                
                print(f"\n✅ 下载成功: {output_path}")
                return True
            except requests.exceptions.RequestException as e:
                print(f"\n❌ {url_type}下载失败: {e}")
                if i < len(urls_to_try) - 1:
                    print("🔄 尝试下一个地址...")
                continue
            except Exception as e:
                print(f"\n❌ 下载出错: {e}")
                continue
        
        print("❌ 所有下载地址都失败了")
        return False
    
    @classmethod
    def extract_archive(cls, archive_path: Path, extract_to: Path) -> bool:
        """解压归档文件"""
        import zipfile  # 局部导入
        import tarfile  # 局部导入
        
        try:
            extract_to.mkdir(parents=True, exist_ok=True)
            
            if archive_path.suffix == '.zip':
                print(f"📦 解压 ZIP 文件: {archive_path}")
                with zipfile.ZipFile(archive_path, 'r') as zip_ref:
                    zip_ref.extractall(extract_to)
            elif archive_path.name.endswith('.tar.xz'):
                print(f"📦 解压 TAR.XZ 文件: {archive_path}")
                with tarfile.open(archive_path, 'r:xz') as tar_ref:
                    tar_ref.extractall(extract_to)
            else:
                raise ValueError(f"不支持的文件格式: {archive_path}")
            
            print(f"✅ 解压完成: {extract_to}")
            return True
            
        except Exception as e:
            print(f"❌ 解压失败: {e}")
            return False
    
    @classmethod
    def find_executable(cls, extract_dir: Path) -> Path | None:
        """在解压目录中查找可执行文件"""
        # 查找 shellcheck 可执行文件
        for item in extract_dir.rglob("*"):
            if item.is_file() and item.name in ["shellcheck", "shellcheck.exe"]:
                return item
        return None
    
    @classmethod
    def install_executable(cls, exe_path: Path, install_dir: Path) -> bool:
        """安装可执行文件到指定目录"""
        try:
            install_dir.mkdir(parents=True, exist_ok=True)
            
            # 确定目标文件名
            exe_name = "shellcheck.exe" if platform.system() == "Windows" else "shellcheck"
            target_path = install_dir / exe_name
            
            # 复制文件
            import shutil
            shutil.copy2(exe_path, target_path)
            
            # 在类 Unix 系统上设置执行权限
            if platform.system() != "Windows":
                target_path.chmod(target_path.stat().st_mode | stat.S_IEXEC)
            
            print(f"✅ 安装完成: {target_path}")
            return True
            
        except Exception as e:
            print(f"❌ 安装失败: {e}")
            return False
    @classmethod
    def verify_installation(cls, install_dir: Path) -> bool:
        """验证安装是否成功"""
        import subprocess  # 局部导入
        
        try:
            exe_name = "shellcheck.exe" if platform.system() == "Windows" else "shellcheck"
            exe_path = install_dir / exe_name
            
            if not exe_path.exists():
                print(f"❌ 可执行文件不存在: {exe_path}")
                return False
            
            # 尝试运行 --version 命令
            result = subprocess.run(
                [str(exe_path), "--version"],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                version_info = result.stdout.strip()
                print(f"✅ ShellCheck 安装验证成功:")
                print(f"   {version_info.split()[0]} {version_info.split()[1]}")
                return True
            else:
                print(f"❌ ShellCheck 运行失败: {result.stderr}")
                return False
                
        except Exception as e:
            print(f"❌ 验证安装失败: {e}")
            return False
    @classmethod
    def install(
        cls,
        install_dir: Path | None = None,
        project_path: Path | None = None,
        use_proxy: bool = True
    ) -> bool:
        """
        安装 ShellCheck
        
        Args:
            install_dir: 安装目录，默认使用 RmmFileSystem.BIN
            project_path: 项目路径，用于获取代理配置，默认使用当前工作目录
            use_proxy: 是否使用代理加速下载
            
        Returns:
            bool: 安装是否成功
        """
        import tempfile  # 局部导入
        
        try:
            # 设置默认值
            if install_dir is None:
                install_dir = RmmFileSystem.BIN
            
            if project_path is None:
                project_path = Path.cwd()
            
            print(f"🔧 开始安装 ShellCheck {cls.VERSION}")
            print(f"📁 安装目录: {install_dir}")
            
            # 检测平台和获取下载信息
            system, arch = cls.detect_platform_arch()
            print(f"🖥️  检测到平台: {system} {arch}")
            
            filename, download_url = cls.get_download_info()
            print(f"📥 下载文件: {filename}")
            print(f"🔗 下载地址: {download_url}")
            
            # 创建临时目录
            with tempfile.TemporaryDirectory() as temp_dir:
                temp_path = Path(temp_dir)
                archive_path = temp_path / filename
                extract_dir = temp_path / "extract"
                
                # 获取代理地址
                proxy_urls = []
                if use_proxy:
                    proxy_urls = cls.get_proxy_urls(project_path, download_url)
                    if proxy_urls:
                        print(f"🚀 找到 {len(proxy_urls)} 个代理节点")
                    else:
                        print("⚠️ 未找到可用代理，将使用原始地址下载")
                
                # 下载文件
                if not cls.download_with_proxies(download_url, proxy_urls, archive_path):
                    return False
                
                # 解压文件
                if not cls.extract_archive(archive_path, extract_dir):
                    return False
                
                # 查找可执行文件
                exe_path = cls.find_executable(extract_dir)
                if not exe_path:
                    print("❌ 未找到 shellcheck 可执行文件")
                    return False
                
                print(f"🔍 找到可执行文件: {exe_path}")
                
                # 安装可执行文件
                if not cls.install_executable(exe_path, install_dir):
                    return False
                
                # 验证安装
                if not cls.verify_installation(install_dir):
                    return False
                
                print(f"🎉 ShellCheck {cls.VERSION} 安装成功!")
                
                # 提示 PATH 配置
                exe_name = "shellcheck.exe" if platform.system() == "Windows" else "shellcheck"
                final_path = install_dir / exe_name
                
                if str(install_dir) not in os.environ.get('PATH', ''):
                    print(f"\n💡 提示: 请将 {install_dir} 添加到系统 PATH 环境变量")
                    print(f"   或者直接使用完整路径: {final_path}")
                else:
                    print(f"\n✅ {install_dir} 已在 PATH 中，可以直接使用 'shellcheck' 命令")
                
                return True
                
        except Exception as e:
            print(f"❌ 安装过程中出错: {e}")
            return False
    
    @classmethod
    def uninstall(cls, install_dir: Path | None = None) -> bool:
        """
        卸载 ShellCheck
        
        Args:
            install_dir: 安装目录，默认使用 RmmFileSystem.BIN
            
        Returns:
            bool: 卸载是否成功
        """
        try:
            if install_dir is None:
                install_dir = RmmFileSystem.BIN
            
            exe_name = "shellcheck.exe" if platform.system() == "Windows" else "shellcheck"
            exe_path = install_dir / exe_name
            
            if exe_path.exists():
                exe_path.unlink()
                print(f"✅ ShellCheck 已卸载: {exe_path}")
                return True
            else:
                print(f"⚠️ ShellCheck 未安装在: {install_dir}")
                return False
                
        except Exception as e:
            print(f"❌ 卸载失败: {e}")
            return False
    
    @classmethod
    def is_installed(cls, install_dir: Path | None = None) -> bool:
        """
        检查 ShellCheck 是否已安装
        
        Args:
            install_dir: 安装目录，默认使用 RmmFileSystem.BIN
            
        Returns:
            bool: 是否已安装
        """
        try:
            if install_dir is None:
                install_dir = RmmFileSystem.BIN
            
            exe_name = "shellcheck.exe" if platform.system() == "Windows" else "shellcheck"
            exe_path = install_dir / exe_name
            
            return exe_path.exists() and exe_path.is_file()
            
        except Exception:
            return False


# 便捷函数
def install_shellcheck(
    install_dir: Path | None = None,
    project_path: Path | None = None,
    use_proxy: bool = True
) -> bool:
    """
    便捷的 ShellCheck 安装函数
    
    Args:
        install_dir: 安装目录，默认使用 RmmFileSystem.BIN
        project_path: 项目路径，用于获取代理配置
        use_proxy: 是否使用代理加速下载
        
    Returns:
        bool: 安装是否成功
    """
    return ShellCheckInstaller.install(install_dir, project_path, use_proxy)


def uninstall_shellcheck(install_dir: Path | None = None) -> bool:
    """
    便捷的 ShellCheck 卸载函数
    
    Args:
        install_dir: 安装目录，默认使用 RmmFileSystem.BIN
        
    Returns:
        bool: 卸载是否成功
    """
    return ShellCheckInstaller.uninstall(install_dir)


def is_shellcheck_installed(install_dir: Path | None = None) -> bool:
    """
    检查 ShellCheck 是否已安装
    
    Args:
        install_dir: 安装目录，默认使用 RmmFileSystem.BIN
        
    Returns:
        bool: 是否已安装
    """
    return ShellCheckInstaller.is_installed(install_dir)


if __name__ == "__main__":
    # 命令行使用示例
    import sys
    
    if len(sys.argv) < 2:
        print("用法:")
        print("  python shellcheck_installer.py install [install_dir]")
        print("  python shellcheck_installer.py uninstall [install_dir]")
        print("  python shellcheck_installer.py check [install_dir]")
        sys.exit(1)
    
    command = sys.argv[1]
    install_dir = Path(sys.argv[2]) if len(sys.argv) > 2 else None
    
    if command == "install":
        success = install_shellcheck(install_dir)
        sys.exit(0 if success else 1)
    elif command == "uninstall":
        success = uninstall_shellcheck(install_dir)
        sys.exit(0 if success else 1)
    elif command == "check":
        installed = is_shellcheck_installed(install_dir)
        print(f"ShellCheck 已安装: {installed}")
        sys.exit(0 if installed else 1)
    else:
        print(f"未知命令: {command}")
        sys.exit(1)
