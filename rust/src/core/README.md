# RmmCore 基础类使用指南

## 📋 概述

RmmCore 是一个功能强大且高效的基础类，专门用于管理 RMM (Root Module Manager) 项目的配置文件和元数据。它提供了统一的接口来读写各种配置文件，包括 `meta.toml`、`rmmproject.toml`、`module.prop` 和 `Rmake.toml`，同时支持 Git 集成和 Python 绑定。

## 🚀 主要特性

- 🚀 **高效缓存机制**: 内存缓存TTL为60秒，显著减少IO操作
- 📁 **多配置文件支持**: 统一管理所有项目相关配置文件
- 🔍 **智能项目扫描**: 递归扫描项目目录，自动发现RMM项目
- 🔄 **双向同步**: 支持配置与文件系统的双向同步
- ⚡ **异步操作**: 支持并发操作，提升性能
- 🛡️ **错误处理**: 完善的错误处理和恢复机制
- 🐍 **Python 绑定**: 完整的 Python API 支持
- 📊 **Git 集成**: 自动检测和分析 Git 仓库信息

## 🎯 快速开始

### 5分钟上手

#### Rust 快速示例

```rust
use rmm::core::RmmCore;

fn main() -> anyhow::Result<()> {
    // 1. 创建 RmmCore 实例
    let core = RmmCore::new();
    
    // 2. 初始化配置
    let meta = core.create_default_meta("user@example.com", "myuser", "1.0.0");
    core.update_meta_config(&meta)?;
    
    // 3. 扫描项目
    let projects = core.scan_projects(std::path::Path::new("."), Some(2))?;
    println!("发现 {} 个项目", projects.len());
    
    // 4. 同步项目到 meta
    core.sync_projects(&[std::path::Path::new(".")], Some(2))?;
    
    println!("✅ RmmCore 初始化完成！");
    Ok(())
}
```

#### Python 快速示例

```python
from pyrmm.cli.rmmcore import RmmCore

# 1. 创建 RmmCore 实例
core = RmmCore()

# 2. 初始化配置
core.create_default_meta_config("user@example.com", "myuser", "1.0.0")

# 3. 扫描项目
projects = core.scan_projects(".", max_depth=2)
print(f"发现 {len(projects)} 个项目")

# 4. 同步项目到 meta
core.sync_projects(["."], max_depth=2)

print("✅ RmmCore 初始化完成！")
```

### 安装指南

#### 环境要求

- **Rust**: 1.70+
- **Python**: 3.8+
- **操作系统**: Windows, Linux, macOS

#### Rust 安装

```toml
# 添加到 Cargo.toml
[dependencies]
rmm = "0.3.0"
```

#### Python 安装

```bash
# 从 PyPI 安装（推荐）
pip install pyrmm

# 或从源代码安装
git clone https://github.com/your-repo/rmm.git
cd rmm
pip install -e .
```

#### 环境变量设置

```bash
# 设置 RMM 根目录（可选）
export RMM_ROOT="$HOME/data/adb/.rmm"

# Windows PowerShell
$env:RMM_ROOT = "$env:USERPROFILE\data\adb\.rmm"
```

## 🔧 核心功能

### 1. RMM_ROOT 路径管理

#### Rust 使用

```rust
use rmm::core::RmmCore;

let core = RmmCore::new();
let rmm_root = core.get_rmm_root();
println!("RMM根目录: {}", rmm_root.display());
```

#### Python 使用

```python
from pyrmm.cli.rmmcore import RmmCore

core = RmmCore()
rmm_root = core.get_rmm_root()
print(f"RMM根目录: {rmm_root}")
```

**环境变量支持**:

- 优先读取 `RMM_ROOT` 环境变量
- 默认路径: `~/data/adb/.rmm/`

### 2. Meta 配置管理

#### 读取 Meta 配置

**Rust:**

```rust
let meta = core.get_meta_config()?;
println!("用户: {}", meta.username);
println!("邮箱: {}", meta.email);
println!("版本: {}", meta.version);
```

**Python:**

```python
meta = core.get_meta_config()
print(f"用户: {meta['username']}")
print(f"邮箱: {meta['email']}")
print(f"版本: {meta['version']}")
```

#### 更新 Meta 配置

**Rust:**

```rust
let mut meta = core.create_default_meta("user@example.com", "username", "1.0.0");
meta.projects.insert("MyProject".to_string(), "/path/to/project".to_string());
core.update_meta_config(&meta)?;
```

**Python:**

```python
# 创建新配置
core.create_default_meta_config("user@example.com", "username", "1.0.0")

# 添加项目
core.add_project_to_meta("MyProject", "/path/to/project")
```

#### 获取特定键值

**Rust:**

```rust
let email = core.get_meta_value("email")?;
if let Some(toml::Value::String(email_str)) = email {
    println!("邮箱: {}", email_str);
}
```

**Python:**

```python
email = core.get_meta_value("email")
print(f"邮箱: {email}")
```

### 3. 项目路径管理

**Rust:**

```rust
// 根据项目名获取路径
let project_path = core.get_project_path("MyProject")?;
if let Some(path) = project_path {
    println!("项目路径: {}", path.display());
}
```

**Python:**

```python
# 根据项目名获取路径
project_path = core.get_project_path("MyProject")
if project_path:
    print(f"项目路径: {project_path}")
```

### 4. 项目有效性检查

**Rust:**

```rust
let validity = core.check_projects_validity()?;
for (name, is_valid) in validity {
    println!("项目 {}: {}", name, if is_valid { "有效" } else { "无效" });
}
```

**Python:**

```python
validity = core.check_projects_validity()
for name, is_valid in validity.items():
    status = "有效" if is_valid else "无效"
    print(f"项目 {name}: {status}")
```

### 5. 项目扫描

**Rust:**

```rust
use std::path::Path;

let scan_path = Path::new("/path/to/scan");
let projects = core.scan_projects(scan_path, Some(3))?; // 最大深度3层

for project in projects {
    println!("发现项目: {} at {}", project.name, project.path.display());
}
```

**Python:**

```python
# 扫描项目
projects = core.scan_projects("/path/to/scan", max_depth=3)
for project in projects:
    print(f"发现项目: {project['name']} at {project['path']}")
```

### 6. 项目同步

**Rust:**

```rust
let scan_paths = vec![Path::new("/path1"), Path::new("/path2")];
core.sync_projects(&scan_paths, Some(3))?;
println!("项目同步完成");
```

**Python:**

```python
# 同步项目
scan_paths = ["/path1", "/path2"]
core.sync_projects(scan_paths, max_depth=3)
print("项目同步完成")
```

### 7. 项目配置管理

#### 读取项目配置

**Rust:**

```rust
use std::path::Path;

let project_path = Path::new("/path/to/project");
let project = core.get_project_config(project_path)?;
println!("项目ID: {}", project.project.id);
```

**Python:**

```python
# 读取项目配置
project = core.get_project_config("/path/to/project")
print(f"项目ID: {project['project']['id']}")
```

#### 更新项目配置

**Rust:**

```rust
let project = core.create_default_project("MyProject", "username", "user@example.com");
core.update_project_config(project_path, &project)?;
```

**Python:**

```python
# 创建并更新项目配置
core.create_default_project_config("/path/to/project", "MyProject", "username", "user@example.com")
```

### 8. Module.prop 管理

**Rust:**

```rust
// 读取 module.prop
let module_prop = core.get_module_prop(project_path)?;
println!("模块ID: {}", module_prop.id);

// 更新 module.prop
let mut prop = core.create_default_module_prop("MyModule", "username");
prop.version = "v2.0.0".to_string();
core.update_module_prop(project_path, &prop)?;
```

**Python:**

```python
# 读取 module.prop
module_prop = core.get_module_prop("/path/to/project")
print(f"模块ID: {module_prop['id']}")

# 创建默认 module.prop
core.create_default_module_prop("/path/to/project", "MyModule", "username")
```

### 9. Rmake.toml 管理

**Rust:**

```rust
// 读取 Rmake 配置
let rmake = core.get_rmake_config(project_path)?;
println!("构建包含: {:?}", rmake.build.include);

// 更新 Rmake 配置
let mut rmake = core.create_default_rmake();
rmake.build.exclude.push("*.tmp".to_string());
core.update_rmake_config(project_path, &rmake)?;
```

**Python:**

```python
# 读取 Rmake 配置
rmake = core.get_rmake_config("/path/to/project")
print(f"构建包含: {rmake['build']['include']}")

# 创建默认 Rmake 配置
core.create_default_rmake_config("/path/to/project")
```

### 10. Git 集成功能

#### 获取 Git 信息

**Rust:**

```rust
let git_info = core.get_git_info(&project_path)?;
println!("Git 仓库根目录: {}", git_info.repo_root.display());
println!("相对路径: {}", git_info.relative_path.display());
println!("远程URL: {:?}", git_info.remote_url);
println!("当前分支: {}", git_info.branch);
```

**Python:**

```python
# 获取项目的 Git 信息
git_info = core.get_git_info("/path/to/project")
print(f"Git 仓库根目录: {git_info['repo_root']}")
print(f"相对路径: {git_info['relative_path']}")
print(f"远程URL: {git_info['remote_url']}")
print(f"当前分支: {git_info['branch']}")
```

#### 检查项目是否在 Git 仓库中

**Python:**

```python
is_in_git = core.is_project_in_git("MyProject")
print(f"项目是否在 Git 仓库中: {is_in_git}")
```

### 11. 配置移除功能

#### 移除项目

**Rust:**

```rust
// 移除单个项目
let removed = core.remove_project_from_meta("old_project")?;
println!("项目已移除: {}", removed);

// 移除多个项目
let removed_projects = core.remove_projects_from_meta(&["project1", "project2"])?;
println!("已移除项目: {:?}", removed_projects);
```

**Python:**

```python
# 移除单个项目
removed = core.remove_project_from_meta("old_project")
print(f"项目已移除: {removed}")

# 移除多个项目
removed_projects = core.remove_projects_from_meta(["project1", "project2"])
print(f"已移除项目: {removed_projects}")
```

#### 清理无效项目

**Rust:**

```rust
// 移除无效项目
let invalid_projects = core.remove_invalid_projects()?;
println!("已移除无效项目: {:?}", invalid_projects);
```

**Python:**

```python
# 清理无效项目
invalid_projects = core.remove_invalid_projects()
print(f"已移除无效项目: {invalid_projects}")
```

## 📊 缓存管理

### 缓存统计

**Rust:**

```rust
let (meta_cached, project_count) = core.get_cache_stats();
println!("Meta缓存状态: {}", meta_cached);
println!("项目缓存数量: {}", project_count);
```

**Python:**

```python
cache_stats = core.get_cache_stats()
print(f"Meta缓存状态: {cache_stats['meta_cached']}")
print(f"项目缓存数量: {cache_stats['project_count']}")
```

### 清理过期缓存

**Rust:**

```rust
core.cleanup_expired_cache();
```

**Python:**

```python
# 清理过期缓存
core.cleanup_expired_cache()

# 清理所有缓存
core.clear_all_cache()
```

## 📁 配置文件结构

### meta.toml

```toml
email = "user@example.com"
username = "username"
version = "1.0.0"

[projects]
ProjectName = "/path/to/project"
```

### rmmproject.toml

```toml
[project]
id = "MyProject"
description = "我的RMM项目"
updateJson = "https://example.com/update.json"
readme = "README.md"
changelog = "CHANGELOG.md"
license = "LICENSE"
dependencies = []

[[authors]]
name = "username"
email = "user@example.com"

[project.scripts]
build = "rmm build"

[urls]
github = "https://github.com/user/repo"

[build-system]
requires = ["rmm>=0.3.0"]
build-backend = "rmm"
```

### module.prop

```toml
id = "MyModule"
name = "My Module"
version = "v1.0.0"
versionCode = "1000000"
author = "username"
description = "模块描述"
updateJson = "https://example.com/update.json"
```

### Rmake.toml

```toml
[build]
include = ["rmm"]
exclude = [".git", ".rmmp"]
prebuild = ["echo 'Pre-build'"]
build = ["rmm"]
postbuild = []

[build.src]
include = []
exclude = []

[build.scripts]
release = "rmm build --release"
```

## 🧪 Python 完整示例

```python
#!/usr/bin/env python3
"""
RmmCore Python 使用示例
"""

from pyrmm.cli.rmmcore import RmmCore
import json

def main():
    # 创建 RmmCore 实例
    core = RmmCore()
    
    print("🚀 RmmCore Python 示例开始")
    print(f"📁 RMM_ROOT 路径: {core.get_rmm_root()}")
    
    # 创建默认配置
    print("\n📝 创建默认 Meta 配置...")
    core.create_default_meta_config(
        "example@gmail.com", 
        "example_user", 
        "0.1.0"
    )
    
    # 读取配置
    print("\n📖 读取 Meta 配置...")
    try:
        meta = core.get_meta_config()
        print(f"   📧 Email: {meta.get('email', 'N/A')}")
        print(f"   👤 Username: {meta.get('username', 'N/A')}")
        print(f"   🔢 Version: {meta.get('version', 'N/A')}")
    except Exception as e:
        print(f"❌ 读取配置失败: {e}")
    
    # 项目扫描
    print("\n🔍 扫描当前目录的项目...")
    try:
        projects = core.scan_projects(".", max_depth=2)
        print(f"📊 找到 {len(projects)} 个项目")
        for project in projects[:3]:  # 只显示前3个
            print(f"   📁 {project}")
    except Exception as e:
        print(f"❌ 项目扫描失败: {e}")
    
    # 缓存统计
    print("\n📈 缓存统计:")
    cache_stats = core.get_cache_stats()
    print(f"   🗂️  Meta 缓存: {'已缓存' if cache_stats.get('meta_cached') else '未缓存'}")
    print(f"   📁 项目缓存: {cache_stats.get('project_count', 0)} 个")
    
    print("\n🎉 RmmCore Python 示例完成！")

if __name__ == "__main__":
    main()
```

## 💡 实际使用场景

### 场景1：项目初始化脚本

创建一个完整的项目初始化脚本：

#### Rust 版本

```rust
use rmm::core::RmmCore;
use std::path::Path;

fn initialize_rmm_workspace(workspace_path: &Path) -> anyhow::Result<()> {
    let core = RmmCore::new();
    
    // 1. 创建基础配置
    println!("🔧 初始化 RMM 配置...");
    let meta = core.create_default_meta(
        "developer@example.com", 
        "developer", 
        "1.0.0"
    );
    core.update_meta_config(&meta)?;
    
    // 2. 扫描现有项目
    println!("🔍 扫描工作空间项目...");
    let projects = core.scan_projects(workspace_path, Some(3))?;
    println!("发现 {} 个项目", projects.len());
    
    // 3. 同步到 meta 配置
    println!("🔄 同步项目配置...");
    core.sync_projects(&[workspace_path], Some(3))?;
    
    // 4. 验证项目有效性
    println!("✅ 验证项目...");
    let validity = core.check_projects_validity()?;
    let invalid_count = validity.values().filter(|&&v| !v).count();
    
    if invalid_count > 0 {
        println!("⚠️  发现 {} 个无效项目，建议清理", invalid_count);
        let cleaned = core.remove_invalid_projects()?;
        println!("🧹 已清理项目: {:?}", cleaned);
    }
    
    println!("🎉 RMM 工作空间初始化完成！");
    Ok(())
}
```

#### Python 版本

```python
from pyrmm.cli.rmmcore import RmmCore
import os

def initialize_rmm_workspace(workspace_path: str):
    """初始化 RMM 工作空间"""
    core = RmmCore()
    
    # 1. 创建基础配置
    print("🔧 初始化 RMM 配置...")
    core.create_default_meta_config(
        "developer@example.com", 
        "developer", 
        "1.0.0"
    )
    
    # 2. 扫描现有项目
    print("🔍 扫描工作空间项目...")
    projects = core.scan_projects(workspace_path, max_depth=3)
    print(f"发现 {len(projects)} 个项目")
    
    # 3. 同步到 meta 配置
    print("🔄 同步项目配置...")
    core.sync_projects([workspace_path], max_depth=3)
    
    # 4. 验证项目有效性
    print("✅ 验证项目...")
    validity = core.check_projects_validity()
    invalid_count = sum(1 for v in validity.values() if not v)
    
    if invalid_count > 0:
        print(f"⚠️  发现 {invalid_count} 个无效项目，建议清理")
        cleaned = core.remove_invalid_projects()
        print(f"🧹 已清理项目: {cleaned}")
    
    print("🎉 RMM 工作空间初始化完成！")

# 使用示例
if __name__ == "__main__":
    initialize_rmm_workspace("~/Projects")
```

### 场景2：CI/CD 集成

在 CI/CD 管道中使用 RmmCore：

```python
#!/usr/bin/env python3
"""
CI/CD 管道中的 RMM 项目验证脚本
"""

from pyrmm.cli.rmmcore import RmmCore
import sys
import os

def validate_rmm_project():
    """验证 RMM 项目的完整性"""
    core = RmmCore()
    current_dir = os.getcwd()
    
    try:
        # 检查项目配置
        print("📋 检查项目配置...")
        project_config = core.get_project_config(current_dir)
        print(f"✅ 项目 ID: {project_config['project']['id']}")
        
        # 检查 module.prop
        print("🔧 检查 module.prop...")
        module_prop = core.get_module_prop(current_dir)
        print(f"✅ 模块 ID: {module_prop['id']}")
        
        # 检查 Git 状态
        print("📊 检查 Git 状态...")
        git_info = core.get_git_info(current_dir)
        print(f"✅ Git 分支: {git_info['branch']}")
        print(f"✅ 远程URL: {git_info.get('remote_url', 'N/A')}")
        
        # 验证构建配置
        print("🏗️  检查构建配置...")
        rmake_config = core.get_rmake_config(current_dir)
        print(f"✅ 构建包含: {rmake_config['build']['include']}")
        
        print("🎉 项目验证成功！")
        return 0
        
    except Exception as e:
        print(f"❌ 项目验证失败: {e}")
        return 1

if __name__ == "__main__":
    sys.exit(validate_rmm_project())
```

### 场景3：批量项目管理

管理多个 RMM 项目的脚本：

```python
from pyrmm.cli.rmmcore import RmmCore
import os
from pathlib import Path

def manage_multiple_projects(project_dirs: list):
    """批量管理多个 RMM 项目"""
    core = RmmCore()
    
    print(f"🔧 管理 {len(project_dirs)} 个项目...")
    
    for project_dir in project_dirs:
        print(f"\n📁 处理项目: {project_dir}")
        
        try:
            # 获取项目信息
            project_config = core.get_project_config(project_dir)
            project_name = project_config['project']['id']
            
            # 检查 Git 状态
            git_info = core.get_git_info(project_dir)
            
            # 更新项目元数据
            core.add_project_to_meta(project_name, project_dir)
            
            print(f"  ✅ {project_name} - 分支: {git_info['branch']}")
            
        except Exception as e:
            print(f"  ❌ 处理失败: {e}")
    
    # 清理无效项目
    print("\n🧹 清理无效项目...")
    cleaned = core.remove_invalid_projects()
    if cleaned:
        print(f"已清理: {cleaned}")
    else:
        print("无需清理")
    
    print("🎉 批量管理完成！")

# 使用示例
project_paths = [
    "~/Projects/MyModule1",
    "~/Projects/MyModule2", 
    "~/Projects/MyModule3"
]
manage_multiple_projects(project_paths)
```

## 🎛️ 高级配置

### 自定义缓存策略

```rust
use rmm::core::RmmCore;
use std::time::Duration;

// 创建自定义缓存配置的 RmmCore
let mut core = RmmCore::new();

// 手动清理缓存（适用于内存敏感场景）
core.clear_all_cache();

// 定期清理过期缓存
core.cleanup_expired_cache();

// 获取缓存统计
let (meta_cached, project_count) = core.get_cache_stats();
println!("缓存状态 - Meta: {}, 项目: {}", meta_cached, project_count);
```

### 环境变量配置

```bash
# 设置自定义 RMM 根目录
export RMM_ROOT="/custom/path/to/rmm"

# 设置调试模式
export RMM_DEBUG=1

# 设置日志级别
export RUST_LOG=rmm=debug
```

### 性能调优参数

```python
from pyrmm.cli.rmmcore import RmmCore

# 创建实例
core = RmmCore()

# 针对大型项目的优化设置
projects = core.scan_projects(
    ".", 
    max_depth=2  # 限制扫描深度，提高性能
)

# 批量同步（比单个同步更高效）
core.sync_projects([
    "./projects/batch1",
    "./projects/batch2", 
    "./projects/batch3"
], max_depth=2)
```

## 🚨 故障排除

### 常见问题及解决方案

#### 1. 配置文件损坏

**问题**: `meta.toml` 文件格式错误

```python
# 解决方案：重新创建默认配置
core = RmmCore()
try:
    meta = core.get_meta_config()
except Exception:
    print("配置文件损坏，重新创建...")
    core.create_default_meta_config(
        "your-email@example.com",
        "your-username", 
        "1.0.0"
    )
```

#### 2. 权限问题

**问题**: 无法写入配置文件

```bash
# 检查 RMM_ROOT 目录权限
ls -la $RMM_ROOT
chmod 755 $RMM_ROOT

# Windows
icacls %RMM_ROOT% /grant:r %USERNAME%:(OI)(CI)F
```

#### 3. Git 仓库检测失败

**问题**: Git 信息获取失败

```python
# 调试 Git 状态
try:
    git_info = core.get_git_info("/path/to/project")
    print(f"Git 信息: {git_info}")
except Exception as e:
    print(f"Git 检测失败: {e}")
    # 可能原因：
    # - 目录不是 Git 仓库
    # - Git 未安装
    # - 权限问题
```

#### 4. 缓存问题

**问题**: 数据不一致

```python
# 强制刷新缓存
core.clear_all_cache()
core.cleanup_expired_cache();

# 重新加载配置
meta = core.get_meta_config()
```

#### 5. 项目扫描超时

**问题**: 大型目录扫描耗时过长

```python
# 限制扫描深度
projects = core.scan_projects(".", max_depth=1)

# 分批扫描
import os
for subdir in os.listdir("."):
    if os.path.isdir(subdir):
        projects = core.scan_projects(subdir, max_depth=2)
        # 处理每批结果...
```

### 调试技巧

#### 启用调试日志

```bash
# Rust 调试
export RUST_LOG=rmm=debug

# Python 调试
import logging
logging.basicConfig(level=logging.DEBUG)
```

#### 性能分析

```python
import time
from pyrmm.cli.rmmcore import RmmCore

# 测试操作耗时
start_time = time.time()
core = RmmCore()
projects = core.scan_projects(".", max_depth=3)
end_time = time.time()

print(f"扫描耗时: {end_time - start_time:.2f}s")
print(f"项目数量: {len(projects)}")
```

## 🔧 最佳实践

### 1. 实例管理

```python
# ✅ 推荐：复用实例
class RMMManager:
    def __init__(self):
        self._core = RmmCore()
    
    def get_core(self):
        return self._core

# ❌ 避免：频繁创建实例
def bad_example():
    for i in range(100):
        core = RmmCore()  # 每次都创建新实例
        # ... 操作
```

### 2. 错误处理

```python
# ✅ 推荐：完整的错误处理
def safe_get_project_config(project_path):
    core = RmmCore()
    try:
        return core.get_project_config(project_path)
    except FileNotFoundError:
        print(f"项目配置文件不存在: {project_path}")
        return None
    except Exception as e:
        print(f"读取配置失败: {e}")
        return None
```

### 3. 缓存管理

```python
# ✅ 推荐：定期清理缓存
import threading
import time

def cache_cleanup_worker(core):
    while True:
        time.sleep(300)  # 5分钟
        core.cleanup_expired_cache()

# 启动后台清理线程
core = RmmCore()
cleanup_thread = threading.Thread(
    target=cache_cleanup_worker, 
    args=(core,), 
    daemon=True
)
cleanup_thread.start()
```

### 4. 批量操作

```python
# ✅ 推荐：批量同步
core.sync_projects([
    "/path/to/projects1",
    "/path/to/projects2"
], max_depth=2)

# ❌ 避免：逐个同步
for path in project_paths:
    core.sync_projects([path], max_depth=2)  # 效率低
```
