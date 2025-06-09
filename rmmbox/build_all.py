#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Simple Rust module build script
- Traverse rmmbox directory to find projects to compile
- Build wheels for each project
- Extract .pyd files and move to src/pyrmm/usr/lib
"""

import os
import sys
import subprocess
import zipfile
import shutil
from pathlib import Path


def run_command(cmd, cwd=None, check=True):
    """Run command and return result"""
    print(f"Running command: {' '.join(cmd) if isinstance(cmd, list) else cmd}")
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
    """Find all Rust projects that need to be built"""
    projects = []
    
    for item in rmmbox_dir.iterdir():
        if item.is_dir() and not item.name.startswith('.'):
            # Check for pyproject.toml and Cargo.toml
            if (item / "pyproject.toml").exists() and (item / "Cargo.toml").exists():
                projects.append(item)
                print(f"Found Rust project: {item.name}")
    
    return projects


def build_project(project_dir):
    """Build a single project with error handling"""
    print(f"\n=== Building project: {project_dir.name} ===")
    
    try:
        # Create virtual environment
        print("Creating virtual environment...")
        result = run_command("uv venv", cwd=project_dir, check=False)
        if result.returncode != 0:
            print(f" Failed to create virtual environment for {project_dir.name}")
            return False
        
        # Sync dependencies
        print("Syncing dependencies...")
        result = run_command("uv sync", cwd=project_dir, check=False)
        if result.returncode != 0:
            print(f" Failed to sync dependencies for {project_dir.name}")
            return False
        
        # Build wheels
        print("Building wheels...")
        result = run_command("uv build", cwd=project_dir, check=False)
        if result.returncode != 0:
            print(f" Failed to build wheels for {project_dir.name}")
            return False
        
        print(f" Successfully built {project_dir.name}")
        return True
        
    except Exception as e:
        print(f" Exception while building {project_dir.name}: {e}")
        return False


def extract_pyd_files(project_dir, target_lib_dir):
    """Extract .pyd files from dist directory"""
    dist_dir = project_dir / "dist"
    if not dist_dir.exists():
        print(f"Warning: {project_dir.name} has no dist directory")
        return []
    
    extracted_files = []
    
    # Find all .whl files
    for wheel_file in dist_dir.glob("*.whl"):
        print(f"Processing wheel file: {wheel_file.name}")
        
        with zipfile.ZipFile(wheel_file, 'r') as zf:
            # Find .pyd or .so files
            for file_info in zf.filelist:
                if file_info.filename.endswith(('.pyd', '.so')):
                    # Extract to temporary location
                    temp_path = dist_dir / file_info.filename
                    temp_path.parent.mkdir(parents=True, exist_ok=True)
                    
                    with zf.open(file_info.filename) as source, open(temp_path, 'wb') as target:
                        target.write(source.read())
                      # Move to target directory, keeping original filename (with architecture info)
                    filename = Path(file_info.filename).name
                    target_file = target_lib_dir / filename
                    
                    shutil.move(str(temp_path), str(target_file))
                    extracted_files.append(target_file)
                    print(f"Extracted and moved: {file_info.filename} -> {target_file}")
    
    return extracted_files


def main():
    """主函数"""
    script_dir = Path(__file__).parent
    rmmbox_dir = script_dir
    target_lib_dir = script_dir.parent / "src" / "pyrmm" / "usr" / "lib"
    
    print(f"rmmbox path: {rmmbox_dir}")
    print(f"target lib dir: {target_lib_dir}")
    
    # 确保目标目录存在
    target_lib_dir.mkdir(parents=True, exist_ok=True)
    
    # 查找所有Rust项目
    projects = find_rust_projects(rmmbox_dir)
    
    if not projects:

        return
    

    
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
                print(f" {project_dir.name} 构建成功")
            else:
                failed_projects.append(project_dir.name)
                print(f" {project_dir.name} 构建失败")
        except Exception as e:
            failed_projects.append(project_dir.name)
            print(f" {project_dir.name} 构建异常: {e}")

    # 输出结果
    print(f"\n=== 构建总结 ===")
    print(f"成功构建: {success_count}/{len(projects)}")

    if failed_projects:
        print(f"失败的项目: {', '.join(failed_projects)}")
        print("⚠️  注意：某些模块构建失败，但其他模块已成功构建")

    if extracted_files:
        print(f"\n📦 成功提取的文件:")
        for file in extracted_files:
            print(f"  {file}")

    print(f"\n📁 所有文件已移动到: {target_lib_dir}")
    
    # 即使有失败，只要有成功的就返回0 (GitHub Actions会继续)
    # 只有全部失败才返回非0
    if success_count == 0:
        print(" 所有项目都构建失败")
        sys.exit(1)
    else:
        print(f" 构建完成，成功 {success_count} 个，失败 {len(failed_projects)} 个")
        sys.exit(0)


if __name__ == "__main__":
    main()