from pathlib import Path
from .shellcheck_installer import ShellCheckInstaller
class RmmInstallMeta(type):
    pass


class RmmInstaller(metaclass=RmmInstallMeta):
    """RMM安装器类，负责处理RMM项目的安装逻辑"""
    
    @classmethod
    def install(cls, zip_path: Path):
        """安装RMM模块"""
        if not zip_path.exists():
            raise FileNotFoundError(f"指定的安装包不存在: {zip_path}")
        
        # TODO: 实现RMM模块安装逻辑
        pass

    @classmethod
    def install_bin(
        cls, 
        name: str, 
        install_dir: Path | None = None,
        project_path: Path | None = None,
        use_proxy: bool = True
    ) -> bool:
        """
        安装二进制程序到本地
        
        Args:
            name: 二进制程序名称（如 "shellcheck"）
            install_dir: 安装目录，默认使用 RmmFileSystem.BIN
            project_path: 项目路径，用于获取代理配置
            use_proxy: 是否使用代理加速下载
            
        Returns:
            bool: 安装是否成功
        """
        if name.lower() == "shellcheck":
            return ShellCheckInstaller.install(install_dir, project_path, use_proxy)
        else:
            raise ValueError(f"不支持的二进制程序: {name}")
    
    @classmethod
    def uninstall_bin(cls, name: str, install_dir: Path | None = None) -> bool:
        """
        卸载二进制程序
        
        Args:
            name: 二进制程序名称
            install_dir: 安装目录，默认使用 RmmFileSystem.BIN
            
        Returns:
            bool: 卸载是否成功
        """
        if name.lower() == "shellcheck":
            return ShellCheckInstaller.uninstall(install_dir)
        else:
            raise ValueError(f"不支持的二进制程序: {name}")
    
    @classmethod
    def is_bin_installed(cls, name: str, install_dir: Path | None = None) -> bool:
        """
        检查二进制程序是否已安装
        
        Args:
            name: 二进制程序名称
            install_dir: 安装目录，默认使用 RmmFileSystem.BIN
            
        Returns:
            bool: 是否已安装
        """
        if name.lower() == "shellcheck":
            return ShellCheckInstaller.is_installed(install_dir)
        else:
            raise ValueError(f"不支持的二进制程序: {name}")


# 便捷函数 - 重新导出原有的便捷函数
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
    return RmmInstaller.install_bin("shellcheck", install_dir, project_path, use_proxy)


def uninstall_shellcheck(install_dir: Path | None = None) -> bool:
    """
    便捷的 ShellCheck 卸载函数
    
    Args:
        install_dir: 安装目录，默认使用 RmmFileSystem.BIN
        
    Returns:
        bool: 卸载是否成功
    """
    return RmmInstaller.uninstall_bin("shellcheck", install_dir)


def is_shellcheck_installed(install_dir: Path | None = None) -> bool:
    """
    检查 ShellCheck 是否已安装
    
    Args:
        install_dir: 安装目录，默认使用 RmmFileSystem.BIN
        
    Returns:
        bool: 是否已安装
    """
    return RmmInstaller.is_bin_installed("shellcheck", install_dir)
