#!/usr/bin/env python3
"""
简单的Rust模块构建脚本
- 遍历 rmmbox 目录寻找需要编译的项目
- 为每个项目构建wheels
- 提取.pyd文件并移动到 src/pyrmm/usr/lib
"""

import os
import sys
import subprocess
import zipfile
import shutil
from pathlib import Path


def run_command(cmd, cwd=None, check=True):
    """运行命令并返回结果"""
    print(f"运行命令: {' '.join(cmd) if isinstance(cmd, list) else cmd}")
    result = subprocess.run(
        cmd, 
        cwd=cwd, 
        capture_output=True, 
        text=True, 
        shell=True if isinstance(cmd, str) else False,
        check=False
    )
    
    if result.stdout:
        print(result.stdout)
    if result.stderr:
        print(result.stderr, file=sys.stderr)
    
    if check and result.returncode != 0:
        raise subprocess.CalledProcessError(result.returncode, cmd)
    
    return result


def find_rust_projects(rmmbox_dir):
    """查找所有需要构建的Rust项目"""
    projects = []
    
    for item in rmmbox_dir.iterdir():
        if item.is_dir() and not item.name.startswith('.'):
            # 检查是否有 pyproject.toml 和 Cargo.toml
            if (item / "pyproject.toml").exists() and (item / "Cargo.toml").exists():
                projects.append(item)
                print(f"找到Rust项目: {item.name}")
    
    return projects


def build_project(project_dir):
    """构建单个项目"""
    print(f"\n=== 构建项目: {project_dir.name} ===")
    
    # 创建虚拟环境
    print("创建虚拟环境...")
    run_command("uv venv", cwd=project_dir)
    
    # 同步依赖
    print("同步依赖...")
    run_command("uv sync", cwd=project_dir)
    
    # 构建wheels
    print("构建wheels...")
    run_command("uv build", cwd=project_dir)
    
    return True


def extract_pyd_files(project_dir, target_lib_dir):
    """从dist目录提取.pyd文件"""
    dist_dir = project_dir / "dist"
    if not dist_dir.exists():
        print(f"警告: {project_dir.name} 没有dist目录")
        return []
    
    extracted_files = []
    
    # 查找所有.whl文件
    for wheel_file in dist_dir.glob("*.whl"):
        print(f"处理wheel文件: {wheel_file.name}")
        
        with zipfile.ZipFile(wheel_file, 'r') as zf:
            # 查找.pyd或.so文件
            for file_info in zf.filelist:
                if file_info.filename.endswith(('.pyd', '.so')):
                    # 提取到临时位置
                    temp_path = dist_dir / file_info.filename
                    temp_path.parent.mkdir(parents=True, exist_ok=True)
                    
                    with zf.open(file_info.filename) as source, open(temp_path, 'wb') as target:
                        target.write(source.read())
                      # 移动到目标目录，保持原始文件名（包含架构信息）
                    filename = Path(file_info.filename).name
                    target_file = target_lib_dir / filename
                    
                    shutil.move(str(temp_path), str(target_file))
                    extracted_files.append(target_file)
                    print(f"提取并移动: {file_info.filename} -> {target_file}")
    
    return extracted_files


def main():
    """主函数"""
    script_dir = Path(__file__).parent
    rmmbox_dir = script_dir
    target_lib_dir = script_dir.parent / "src" / "pyrmm" / "usr" / "lib"
    
    print(f"rmmbox目录: {rmmbox_dir}")
    print(f"目标lib目录: {target_lib_dir}")
    
    # 确保目标目录存在
    target_lib_dir.mkdir(parents=True, exist_ok=True)
    
    # 查找所有Rust项目
    projects = find_rust_projects(rmmbox_dir)
    
    if not projects:
        print("未找到任何Rust项目")
        return
    
    print(f"找到 {len(projects)} 个项目")
    
    # 构建统计
    success_count = 0
    failed_projects = []
    extracted_files = []
    
    # 逐个构建项目
    for project_dir in projects:
        try:
            if build_project(project_dir):
                # 提取.pyd文件
                files = extract_pyd_files(project_dir, target_lib_dir)
                extracted_files.extend(files)
                success_count += 1
                print(f"✅ {project_dir.name} 构建成功")
            else:
                failed_projects.append(project_dir.name)
                print(f"❌ {project_dir.name} 构建失败")
        except Exception as e:
            failed_projects.append(project_dir.name)
            print(f"❌ {project_dir.name} 构建失败: {e}")
    
    # 输出结果
    print(f"\n=== 构建完成 ===")
    print(f"成功: {success_count}/{len(projects)}")
    
    if failed_projects:
        print(f"失败的项目: {', '.join(failed_projects)}")
    
    if extracted_files:
        print(f"提取的文件:")
        for file in extracted_files:
            print(f"  {file}")
    
    print(f"✅ 所有文件已移动到: {target_lib_dir}")


if __name__ == "__main__":
    main()