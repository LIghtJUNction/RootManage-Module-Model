from pathlib import Path
from collections.abc import Callable
from typing import Any, TypeVar
import importlib.util
import time
import traceback
import shutil
import zipfile
import os
from .config import Config
from .base import RmmBaseMeta, RmmBase

F = TypeVar('F', bound=Callable[..., Any])

class RmmBuilderMeta(RmmBaseMeta):
    """Meta class for RMM Builder"""
    
    @property
    def META(cls) -> dict[str, Any]:
        """Get the build metadata."""
        meta: dict[str, str | dict[str, str]] = Config.META
        build_config: str | dict[str, str] = meta.get("build", {"default": "basic"})
        if isinstance(build_config, str):
            raise AttributeError(f"构建配置错误!： '{build_config}' 请检查：{Config.META}")
        return build_config
    
    def get_config_key(cls) -> str:
        """获取配置键名"""
        return "build"
    
    def get_reserved_key(cls) -> str:
        """获取保留关键字"""
        return "default"

class RmmBuilder(RmmBase, metaclass=RmmBuilderMeta):
    """RMM Builder class - 简化版本，只使用一个 Rmake.py 文件"""
    
    # 构建脚本缓存
    _build_cache: dict[str, Any] = {}
    _build_mtime: dict[str, float] = {}
    
    # 存储钩子函数
    _prebuilds: list[tuple[str, Callable[..., Any]]] = []
    _postbuilds: list[tuple[str, Callable[..., Any]]] = []
    _custom_build: Callable[..., Any] | None = None
    _build_context: dict[str, Any] = {}
    
    @classmethod
    def reset_hooks(cls):
        """清空所有钩子函数，用于重新构建时清理状态"""
        cls._prebuilds.clear()
        cls._postbuilds.clear()
        cls._custom_build = None
        cls._build_context.clear()    
    @classmethod
    def load(cls, project_path: Path) -> bool:
        """加载 .rmmp/Rmake.py 文件（如果存在）"""
        # 清空之前的钩子函数
        cls.reset_hooks()
        
        # 构建文件放在 .rmmp 目录下
        BUILD_FILE = project_path / ".rmmp" / "Rmake.py"
        
        if not BUILD_FILE.exists():
            return False
        
        # 使用文件路径作为缓存键
        cache_key = str(BUILD_FILE.resolve())
        
        # 检查文件修改时间
        current_mtime = BUILD_FILE.stat().st_mtime
          # 如果缓存中有数据且文件未修改，使用缓存
        if (cache_key in cls._build_cache and 
            cache_key in cls._build_mtime and 
            cls._build_mtime[cache_key] == current_mtime):
            # 从缓存恢复模块
            module = cls._build_cache[cache_key]
        else:
            # 重新加载模块
            try:
                spec = importlib.util.spec_from_file_location("Rmake", BUILD_FILE)
                if spec is None or spec.loader is None:
                    return False
                
                module = importlib.util.module_from_spec(spec)
                spec.loader.exec_module(module)
                
                # 更新缓存
                cls._build_cache[cache_key] = module
                cls._build_mtime[cache_key] = current_mtime
                
            except Exception as e:
                print(f"加载 .rmmp/Rmake.py 时出错: {e}")
                return False
        
        # 尝试从模块中获取标准函数
        if hasattr(module, 'prebuild') and callable(getattr(module, 'prebuild')):
            cls._prebuilds.append(('prebuild', getattr(module, 'prebuild')))
        if hasattr(module, 'postbuild') and callable(getattr(module, 'postbuild')):
            cls._postbuilds.append(('postbuild', getattr(module, 'postbuild')))
        if hasattr(module, 'build') and callable(getattr(module, 'build')):
            cls._custom_build = getattr(module, 'build')
        
        return True
    
    @classmethod
    def build(
        cls, 
        project_name: str | None = None,
        project_path: Path | None = None, 
        output_dir: Path | None = None,
        clean: bool = False,
        verbose: bool = False,
        debug: bool = False
    ) -> dict[str, Any]:
        """执行构建过程"""
        
        start_time = time.time()
        
        try:
            # 如果没有提供project_path但提供了project_name，从配置获取路径
            if project_path is None and project_name:
                from .project import RmmProject
                project_path = RmmProject.project_path(project_name)
            elif project_path is None:
                project_path = Path.cwd()
            if verbose:
              print(f"🔨 开始构建项目: {project_path}")
            
            # 设置默认输出目录到 .rmmp/dist
            if output_dir is None:
                output_dir = project_path / ".rmmp" / "dist"
            
            # 确保 .rmmp 目录存在
            rmmp_dir = project_path / ".rmmp"
            rmmp_dir.mkdir(exist_ok=True)
            
            # 更新 .gitignore 文件
            cls._update_gitignore(project_path)
            
            # 清理输出目录
            if clean and output_dir.exists():
                if verbose:
                    print(f"🧹 清理输出目录: {output_dir}")
                shutil.rmtree(output_dir)
            
            # 确保输出目录存在
            output_dir.mkdir(parents=True, exist_ok=True)
            
            # 设置构建上下文
            cls._build_context = {
                "project_name": project_name or project_path.name,
                "project_path": project_path,
                "output_dir": output_dir,
                "clean": clean,
                "verbose": verbose,
                "debug": debug
            }
            
            # 加载构建脚本
            script_loaded = cls.load(project_path)
            
            if verbose:
                if script_loaded:
                    print(f"✅ 找到 Rmake.py，已加载自定义构建逻辑")
                else:
                    print(f"ℹ️  未找到 Rmake.py，使用默认构建逻辑")
            
            # 执行 prebuild 钩子
            if cls._prebuilds:
                if verbose:
                    print(f"🔧 执行 {len(cls._prebuilds)} 个预构建钩子...")
                for hook_name, hook_func in cls._prebuilds:
                    if verbose:
                        print(f"  ➤ 执行预构建钩子: {hook_name}")
                    hook_func()
            
            # 执行构建逻辑：自定义构建函数覆盖默认构建
            if cls._custom_build:
                if verbose:
                    print(f"🎯 执行自定义构建逻辑...")
                cls._custom_build()
            else:
                if verbose:
                    print(f"🏗️  执行默认构建逻辑...")
                cls._default_build(project_path, output_dir, verbose)
            
            # 执行 postbuild 钩子
            if cls._postbuilds:
                if verbose:
                    print(f"🔧 执行 {len(cls._postbuilds)} 个后构建钩子...")
                for hook_name, hook_func in cls._postbuilds:
                    if verbose:
                        print(f"  ➤ 执行后构建钩子: {hook_name}")
                    hook_func()
            
            # 计算构建时间
            build_time = time.time() - start_time
            
            # 查找输出文件
            output_files = list(output_dir.glob("*.zip"))
            output_file = str(output_files[0]) if output_files else None
            
            result: dict[str, Any] = {
                "success": True,
                "build_time": build_time
            }
            
            if output_file:
                result["output_file"] = output_file
            
            if verbose:
                print(f"✅ 构建完成，耗时 {build_time:.2f} 秒")
            
            return result
            
        except Exception as e:
            build_time = time.time() - start_time
            error_msg = str(e)
            
            if debug:
                error_msg = f"{error_msg}\n详细错误信息:\n{traceback.format_exc()}"
            
            if verbose:
                print(f"❌ 构建失败，耗时 {build_time:.2f} 秒")
                print(f"错误: {error_msg}")
            
            return {
                "success": False,
                "build_time": build_time,
                "error": error_msg
            }
    
    @classmethod
    def _default_build(cls, project_path: Path, output_dir: Path, verbose: bool = False) -> None:
        """默认构建逻辑：压缩整个项目为zip文件"""
        if verbose:
            print(f"🏗️  执行默认构建逻辑: {project_path}")
        
        # 检查是否是RMM项目
        from .project import RmmProject
        if not RmmProject.is_rmmproject(project_path):
            if verbose:
                print("⚠️  警告: 这不是一个RMM项目，跳过构建")
            return
        
        # 读取项目信息
        project_info = RmmProject.project_info(project_path)
        project_name = project_info.get("name", project_path.name)
        version = project_info.get("version", "1.0.0")
        
        # 检查是否有module.prop文件
        module_prop = project_path / "module.prop"
        if module_prop.exists():
            # 创建基本的zip包
            output_file = output_dir / f"{project_name}-{version}.zip"
            if verbose:
                print(f"📦 正在创建模块包: {output_file}")
            
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
                        
                        if verbose:
                            print(f"  📄 添加文件: {arcname}")
                        
                        zf.write(file_path, arcname)
            
            if verbose:
                print(f"✅ 模块包创建完成: {output_file}")
        else:
            if verbose:
                print("⚠️  未找到module.prop文件，跳过模块打包")
    
    @classmethod
    def is_valid_item(cls, item_name: str) -> bool:
        """检查是否是有效项目"""
        return True  # 简化版本，总是返回True
    
    @classmethod
    def get_sync_prompt(cls, item_name: str) -> str:
        """获取同步提示信息"""
        return f"构建器 '{item_name}' 配置。"
    
    @classmethod
    def _update_gitignore(cls, project_path: Path) -> None:
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
