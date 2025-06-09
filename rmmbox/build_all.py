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
    """Build a single project"""
    print(f"\n=== Building project: {project_dir.name} ===")
    
    # Create virtual environment
    print("Creating virtual environment...")
    run_command("uv venv", cwd=project_dir)
    
    # Sync dependencies
    print("Syncing dependencies...")
    run_command("uv sync", cwd=project_dir)
    
    # Build wheels
    print("Building wheels...")
    run_command("uv build", cwd=project_dir)
    
    return True


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
                print(f"✅ {project_dir.name} ")
            else:
                failed_projects.append(project_dir.name)
                print(f"❌ {project_dir.name} ")
        except Exception as e:
            failed_projects.append(project_dir.name)
            print(f"❌ {project_dir.name} {e}")

    # 输出结果
    print(f"\n=== summary ===")
    print(f"success: {success_count}/{len(projects)}")

    if failed_projects:
        print(f"failed projects: {', '.join(failed_projects)}")

    if extracted_files:
        print(f"extracted files:")
        for file in extracted_files:
            print(f"  {file}")

    print(f"✅ all files have been moved to: {target_lib_dir}")


if __name__ == "__main__":
    main()