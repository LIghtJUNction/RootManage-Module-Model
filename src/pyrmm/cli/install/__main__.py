import click
from pathlib import Path

@click.group()
def install():
    """直接安装RMM项目/任意模块到手机 或者安装二进制程序到本地"""
    pass

@install.command()
@click.argument('binary_name', required=True)
@click.option('--install-dir', '-d', type=click.Path(path_type=Path), help='安装目录')
@click.option('--no-proxy', is_flag=True, help='不使用代理加速下载')
def bin(binary_name: str, install_dir: Path | None, no_proxy: bool):
    """安装二进制程序到本地"""
    import os
    import platform
    click.echo(f"🔧 安装二进制程序: {binary_name}")
    click.echo(f"🔧 当前系统: {os.name}")
    click.echo(f"🔧 当前系统架构: {platform.machine()}")
    click.echo(f"🔧 当前系统版本: {platform.release()}")
    click.echo(f"🔧 当前系统名称: {platform.system()}")

    from pyrmm.usr.lib.installer import RmmInstaller
    from pyrmm.usr.lib.fs import RmmFileSystem

    # 设置安装目录
    if install_dir is None:
        install_dir = RmmFileSystem.BIN
    
    click.echo(f"🔧 二进制安装路径: {install_dir}")
    
    try:
        # 安装二进制程序
        success = RmmInstaller.install_bin(
            name=binary_name,
            install_dir=install_dir,
            project_path=Path.cwd(),
            use_proxy=not no_proxy
        )
        
        if success:
            click.echo(f"✅ {binary_name} 安装成功!")
            click.echo(f"🔧 安装路径: {install_dir}")
            click.echo(f"请注意，仅限rmm使用此工具。如果需要，请将此路径添加到PATH环境变量中。")
        else:
            click.echo(f"❌ {binary_name} 安装失败!")
            exit(1)
            
    except ValueError as e:
        click.echo(f"❌ 错误: {e}")
        exit(1)
    except Exception as e:
        click.echo(f"❌ 安装过程中出错: {e}")
        exit(1)

@install.command()
@click.option('--from-release', is_flag=True, help='从最新的Release版本安装')
def git(from_release: bool):
    """从Git仓库下载并安装RMM"""
    click.echo("🔧 正在从Git仓库下载RMM...")
    
    from pyrmm.usr.lib.build import RmmBuilder
    from pyrmm.usr.lib.installer import RmmInstaller
    
    try:
        zip_path: Path | None = RmmBuilder.build_from_git("git", from_release=from_release)
        if zip_path and zip_path.exists():
            RmmInstaller.install(zip_path)
            click.echo("✅ RMM 安装成功!")
        else:
            click.echo("❌ 构建失败，未找到生成的安装包")
            exit(1)
    except Exception as e:
        click.echo(f"❌ 安装过程中出错: {e}")
        exit(1)

@install.command()
@click.argument('zip_file', type=click.Path(exists=True, path_type=Path))
def local(zip_file: Path):
    """从本地zip文件安装RMM模块"""
    click.echo(f"🔧 正在从本地文件安装: {zip_file}")
    
    from pyrmm.usr.lib.installer import RmmInstaller
    
    try:
        RmmInstaller.install(zip_file)
        click.echo("✅ 模块安装成功!")
    except Exception as e:
        click.echo(f"❌ 安装过程中出错: {e}")
        exit(1)

@install.command()
@click.argument('src_file', type=click.Path(exists=True, path_type=Path))
def src(src_file: Path):
    """从源码文件(*.tar.gz)构建并安装"""
    click.echo(f"🔧 正在从源码文件构建并安装: {src_file}")
    
    from pyrmm.usr.lib.build import RmmBuilder
    from pyrmm.usr.lib.installer import RmmInstaller
    
    try:
        result = RmmBuilder.build(str(src_file))
        zip_path: Path | None = result.get("zip_path", None)
        if zip_path is None:
            click.echo("❌ 错误: 无法从源码构建，未找到生成的zip文件。")
            exit(1)
        
        RmmInstaller.install(zip_path)
        click.echo("✅ 源码构建并安装成功!")
    except Exception as e:
        click.echo(f"❌ 安装过程中出错: {e}")
        exit(1)