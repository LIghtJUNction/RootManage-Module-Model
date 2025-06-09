import click
from pathlib import Path

# 延迟导入 - 只在实际执行时才导入耗时的模块
@click.command()
@click.argument("project_name", required=False)
@click.option("--path", "-p", type=click.Path(exists=True, path_type=Path), help="指定项目路径")
@click.option("--output", "-o", type=click.Path(path_type=Path), help="指定输出目录")
@click.option("--clean", "-c", is_flag=True, help="构建前清理输出目录")
@click.option("--verbose", "-v", is_flag=True, help="显示详细构建信息")
@click.option("--debug", "-d", is_flag=True, help="启用调试模式")
def build(project_name: str | None, path: Path | None, output: Path | None, clean: bool, verbose: bool, debug: bool) -> None:
    """构建RMM项目
    
    PROJECT_NAME: 要构建的项目名称 (可选，如果不指定则构建当前目录的项目)
    """
    # 延迟导入 - 显著减少模块加载时间
    from pyrmm.usr.lib.build import RmmBuilder
    from pyrmm.usr.lib.project import RmmProject
    from pyrmm.usr.lib.version import VersionGenerator
    try:
        # 确定项目路径
        if path:
            project_path = path
            project_name = project_path.name
        elif project_name:
            # 通过项目名称获取路径
            project_path = RmmProject.project_path(project_name)
        else:
            # 使用当前目录
            project_path = Path.cwd()
            project_name = project_path.name
        
        # 检查是否是有效的RMM项目
        if not RmmProject.is_rmmproject(project_path):
            click.echo(f"❌ 错误: '{project_path}' 不是一个有效的RMM项目。")
            click.echo("请确保项目目录包含 rmmproject.toml 文件。")
            return
        
        click.echo(f"🔨 正在构建项目: {project_name}")
        click.echo(f"📁 项目路径: {project_path}")        # 生成新版本
        click.echo(f"📝 正在为项目 {project_name} 生成新版本...")
        try:
            # 获取项目的当前版本信息
            project_info = RmmProject.project_info(project_path)
            version_value = project_info.get("version", "1.0.0")
            
            # 确保版本是字符串类型
            if isinstance(version_value, str):
                old_version = version_value
            else:
                old_version = "1.0.0"  # 使用默认版本，如果不是字符串类型
            
            click.echo(f"🔄 当前版本: {old_version}")
            
            # 使用当前版本生成新版本
            version_info = VersionGenerator.generate(old_version, project_path)
            click.echo(f"📋 新版本信息: {version_info['version']} (版本代码: {version_info['versionCode']})")
        except Exception as e:
            click.echo(f"⚠️  版本生成警告: {e}")
            click.echo("继续构建...")
        
        if verbose:
            click.echo(f"🔍 详细模式已启用")
        if debug:
            click.echo(f"🐛 调试模式已启用")
        if clean:
            click.echo(f"🧹 清理模式已启用")
          # 设置输出目录到 .rmmp/dist
        if not output:
            output = project_path / ".rmmp" / "dist"
        
        click.echo(f"📦 输出目录: {output}")
        
        # 执行构建
        result = RmmBuilder.build(
            project_name=project_name,
            project_path=project_path,
            output_dir=output,
            clean=clean,
            verbose=verbose,
            debug=debug        )
        
        if result.get("success", False):
            click.echo(f"✅ 项目 '{project_name}' 构建成功！")
            
            # 显示所有输出文件
            if "output_files" in result:
                click.echo("📦 生成的文件:")
                for output_file in result["output_files"]:
                    file_path = Path(output_file)
                    if file_path.suffix == ".zip":
                        click.echo(f"  🗜️  模块包: {output_file}")
                    elif file_path.name.endswith(".tar.gz"):
                        click.echo(f"  📄 源代码包: {output_file}")
                    else:
                        click.echo(f"  📦 文件: {output_file}")
            elif "output_file" in result:
                # 向后兼容
                click.echo(f"📦 输出文件: {result['output_file']}")
                
            if "build_time" in result:
                click.echo(f"⏱️  构建时间: {result['build_time']:.2f}秒")
        else:
            click.echo(f"❌ 项目 '{project_name}' 构建失败。")
            if "error" in result:
                click.echo(f"错误: {result['error']}")
            
    except FileNotFoundError as e:
        click.echo(f"❌ 文件未找到: {e}")
    except KeyError as e:
        click.echo(f"❌ 项目未找到: {e}")
    except Exception as e:
        click.echo(f"❌ 构建过程中发生错误: {e}")
        if debug:
            import traceback
            click.echo(f"详细错误信息:\n{traceback.format_exc()}")