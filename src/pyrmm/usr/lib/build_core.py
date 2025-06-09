from pathlib import Path
from collections.abc import Callable
from typing import Any
import importlib.util
import shutil
import zipfile
import tarfile
import os
import subprocess
import stat
import platform
import socket

class RmmBuildCore:
    """RMM Build Core - 核心构建逻辑实现"""
    
    @classmethod
    def default_build(cls, project_path: Path, output_dir: Path) -> None | Path:
        """默认构建逻辑：压缩整个项目为zip文件"""
        
        print(f"🏗️  执行默认构建逻辑: {project_path}\n")
        
        # 检查是否是RMM项目
        from .project import RmmProject
        if not RmmProject.is_rmmproject(project_path):
            print("⚠️  警告: 这不是一个RMM项目，跳过构建")
            return
        
        # 读取项目信息
        project_info = RmmProject.project_info(project_path)
        project_name = project_info.get("name", project_path.name)
        version = project_info.get("version", "1.0.0")
        
        # 检查是否有module.prop文件
        module_prop = project_path / "module.prop"
        # 初始化 module_zip 变量
        module_zip: Path | None = None
        
        if module_prop.exists():
            # 创建基本的zip包
            output_file = output_dir / f"{project_name}-{version}.zip"
        
            print(f"📦 正在创建模块包: {output_file}\n")
            with zipfile.ZipFile(output_file, 'w', zipfile.ZIP_DEFLATED) as zf:
                # 遍历项目目录，添加所有文件到zip
                for root, dirs, files in os.walk(project_path):
                    # 跳过隐藏目录、dist目录、__pycache__目录
                    dirs[:] = [d for d in dirs if not d.startswith('.') and d != 'dist' and d != '__pycache__']
                    for file in files:
                        # 跳过隐藏文件、Rmake.py文件、Python缓存文件
                        if file.startswith('.') or file == 'Rmake.py' or file.endswith('.pyc'):
                            continue
                        
                        file_path = Path(root) / file
                        # 计算相对路径
                        arcname = file_path.relative_to(project_path)

                        print(f"  [+]📄 添加文件: {arcname}")

                        zf.write(file_path, arcname)

            print(f"✅ 模块包创建完成: {output_file}\n")
            module_zip = output_file

        else:
            print("⚠️  未找到module.prop文件，跳过模块打包")

        source_output_file = output_dir / f"{project_name}-{version}.tar.gz"
  
        print(f"📦 正在创建源代码包: {source_output_file}")
        
        with tarfile.open(source_output_file, 'w:gz') as tf:
            # 遍历项目目录，添加所有文件到tar.gz
            for root, dirs, files in os.walk(project_path):
                # 跳过隐藏目录、dist目录、__pycache__目录、
                dirs[:] = [d for d in dirs if not d.startswith('_') and d != '__pycache__' and d != 'dist']

                for file in files:
                    if file.endswith('.pyc') or file.startswith('_'):
                        continue
                    
                    file_path = Path(root) / file
                    # 计算相对路径
                    arcname = file_path.relative_to(project_path)
          
                    print(f"  [+]📄 添加源文件: {arcname}")
                    
                    tf.add(file_path, arcname)

        print(f"✅ 源代码包创建完成: {source_output_file}\n")
        
        return module_zip
    
    @classmethod
    def update_gitignore(cls, project_path: Path) -> None:
        """更新 .gitignore 文件，确保 .rmmp/dist 被忽略"""
        gitignore_path = project_path / ".gitignore"
        ignore_entry = ".rmmp/dist"
        
        # 读取现有的 .gitignore 内容
        existing_lines = []
        if gitignore_path.exists():
            try:
                with open(gitignore_path, 'r', encoding='utf-8') as f:
                    existing_lines = [line.rstrip() for line in f.readlines()]
            except Exception:
                # 如果读取失败，忽略错误，继续处理
                pass
        
        # 检查是否已经包含 .rmmp/dist 或相关条目
        has_rmmp_dist = any(
            line.strip() in [ignore_entry, ".rmmp/", ".rmmp/*", "**/.rmmp/dist", "**/.rmmp/*"]
            for line in existing_lines
        )
        
        if not has_rmmp_dist:
            # 添加 .rmmp/dist 到 .gitignore
            if existing_lines and not existing_lines[-1] == "":
                existing_lines.append("")  # 添加空行分隔
            
            existing_lines.extend([
                "# RMM 构建输出目录",
                ignore_entry
            ])
            
            try:
                with open(gitignore_path, 'w', encoding='utf-8') as f:
                    f.write('\n'.join(existing_lines) + '\n')
            except Exception as e:
                # 如果写入失败，只是打印警告，不影响构建过程
                print(f"⚠️  警告: 无法更新 .gitignore 文件: {e}")

    @classmethod
    def execute_script(
        cls, 
        script_config: str, 
        script_type: str, 
        project_path: Path
    ) -> bool:
        """
        执行脚本配置
        
        Args:
            script_config: 脚本配置 ("default", "Rmake", 或可执行文件路径)
            script_type: 脚本类型 ("prebuild", "build", "postbuild")
            project_path: 项目路径
            
        Returns:
            bool: 执行是否成功
        """
        if script_config == "default":
            print(f"  ➤ 使用默认 {script_type} 逻辑")
            return True
        elif script_config == "Rmake":
            print(f"  ➤ 使用 Rmake.py 中的 {script_type} 函数")
            # 这个逻辑在后面的构建过程中处理
            return True
        else:
            # 自定义可执行文件路径
            executable_path = Path(script_config)
            
            # 如果是相对路径，相对于项目目录
            if not executable_path.is_absolute():
                executable_path = project_path / executable_path
            
            if not executable_path.exists():
                print(f"  ❌ {script_type} 可执行文件不存在: {executable_path}")
                return False

            print(f"  ➤ 执行自定义 {script_type} 脚本: {executable_path}")

            try:
                # 设置工作目录为项目目录
                result = subprocess.run(
                    [str(executable_path)],
                    cwd=str(project_path),
                    capture_output=True,
                    text=True,
                    timeout=300  # 5分钟超时
                )
                
                print(f"    输出: {result.stdout}")
                
                if result.returncode != 0:
                    error_msg = result.stderr or f"脚本退出码: {result.returncode}"
                    print(f"    ❌ {script_type} 脚本执行失败: {error_msg}")
                    return False

                print(f"    ✅ {script_type} 脚本执行成功")
                return True

            except subprocess.TimeoutExpired:
                print(f"    ❌ {script_type} 脚本执行超时 (5分钟)")
                return False
            except Exception as e:
                print(f"    ❌ {script_type} 脚本执行出错: {e}")
                return False

    @classmethod
    def load_rmake_script(
        cls, 
        project_path: Path, 
        build_cache: dict[str, Any], 
        build_mtime: dict[str, float]
    ) -> tuple[bool, Any]:
        """
        加载 .rmmp/Rmake.py 文件（如果存在）
        
        Returns:
            tuple[bool, Any]: (是否成功加载, 模块对象或None)
        """
        # 构建文件放在 .rmmp 目录下
        BUILD_FILE = project_path / ".rmmp" / "Rmake.py"
        
        if not BUILD_FILE.exists():
            return False, None
        
        # 使用文件路径作为缓存键
        cache_key = str(BUILD_FILE.resolve())
        
        # 检查文件修改时间
        current_mtime = BUILD_FILE.stat().st_mtime
        
        # 检查是否需要重新加载
        should_reload = (
            cache_key not in build_cache or 
            cache_key not in build_mtime or 
            build_mtime[cache_key] != current_mtime
        )
        
        if should_reload:
            print(f"📝 检测到 Rmake.py 文件更新，重新加载构建脚本...")
            
            # 清除可能存在的 Python 模块缓存
            import sys
            module_name = f"Rmake_{hash(cache_key)}"
            if module_name in sys.modules:
                del sys.modules[module_name]
            
            # 重新加载模块
            try:
                spec = importlib.util.spec_from_file_location(module_name, BUILD_FILE)
                if spec is None or spec.loader is None:
                    return False, None
                
                module = importlib.util.module_from_spec(spec)
                spec.loader.exec_module(module)
                
                # 更新缓存
                build_cache[cache_key] = module
                build_mtime[cache_key] = current_mtime
                
                print(f"✅ Rmake.py 构建脚本已重新加载")
                
            except Exception as e:
                print(f"❌ 加载 .rmmp/Rmake.py 时出错: {e}")
                return False, None
        else:
            # 从缓存恢复模块
            module = build_cache[cache_key]
            print(f"📋 使用缓存的 Rmake.py 构建脚本")
        
        return True, module

    @classmethod
    def check_network_connection(cls, url: str) -> bool:
        """检查网络连接"""
        try:
            # 从URL中提取主机名
            if url.startswith("https://github.com"):
                host = "github.com"
                port = 443
            elif url.startswith("http://"):
                host = url.split("//")[1].split("/")[0]
                port = 80
            else:
                # SSH格式或其他，尝试解析
                if "@" in url:
                    host = url.split("@")[1].split(":")[0]
                    port = 22
                else:
                    return True  # 无法解析，假设可连接
            
            # 测试连接
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(10)
            result = sock.connect_ex((host, port))
            sock.close()
            return result == 0
        except Exception:
            return False

    @classmethod
    def handle_remove_readonly(cls, func: Callable[[str], None], path: str, exc: BaseException) -> None:
        """处理只读文件删除错误"""
        try:
            if platform.system() == "Windows" and hasattr(exc, 'errno') and exc.errno in (13, 5):  # type: ignore
                os.chmod(path, stat.S_IWRITE)
                func(path)
            else:
                try:
                    os.chmod(path, 0o777)
                    func(path)
                except:
                    print(f"  ⚠️ 无法删除文件: {path}")
        except Exception as e2:
            print(f"  ⚠️ 处理只读文件失败: {path} - {e2}")

    @classmethod
    def cleanup_directory(cls, directory: Path) -> None:
        """清理目录，处理只读文件"""
        try:
            # 先尝试修改整个目录的权限
            if platform.system() == "Windows":
                for root, dirs, files in os.walk(directory):
                    for d in dirs:
                        try:
                            os.chmod(os.path.join(root, d), stat.S_IWRITE)
                        except:
                            pass
                    for f in files:
                        try:
                            os.chmod(os.path.join(root, f), stat.S_IWRITE)
                        except:
                            pass
            
            shutil.rmtree(directory, onexc=cls.handle_remove_readonly)
            print(f"✅ 已清理目录: {directory}")
        except Exception as e:
            print(f"⚠️ 清理目录失败: {e}")

    @classmethod
    def manage_temp_directory_size(cls, tmp_base: Path, repo_name: str) -> None:
        """管理临时目录大小，清理过大的目录"""
        if tmp_base.exists():
            try:
                total_size = sum(f.stat().st_size for f in tmp_base.rglob('*') if f.is_file())
                if total_size > 1 * 1024 * 1024 * 1024:  # 1GB
                    print(f"⚠️ 临时构建目录 {tmp_base} 太大（{total_size / (1024*1024*1024):.1f}GB），清理旧文件...")
                    # 只清理其他项目的构建目录，保留当前项目
                    for item in tmp_base.iterdir():
                        if item.is_dir() and item.name != repo_name:
                            try:
                                shutil.rmtree(item)
                                print(f"🧹 已清理: {item.name}")
                            except Exception as e:
                                print(f"⚠️ 清理失败 {item.name}: {e}")
            except Exception as e:
                print(f"⚠️ 检查临时目录大小失败: {e}")

    @classmethod
    def execute_build_hooks(
        cls, 
        hooks: list[tuple[str, Callable[..., Any]]], 
        hook_type: str
    ) -> None:
        """执行构建钩子函数"""
        if hooks:
            print(f"🔧 执行 {len(hooks)} 个{hook_type}钩子...")
            for hook_name, hook_func in hooks:
                print(f"  ➤ 执行{hook_type}钩子: {hook_name}")
                hook_func()