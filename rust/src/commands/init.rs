use clap::{Arg, ArgAction, ArgMatches, Command};
use anyhow::Result;
use std::path::Path;
use std::collections::HashMap;
use crate::config::{RmmConfig, ProjectConfig};
use crate::utils::{ensure_dir_exists, get_git_info};
use std::fs;

pub fn build_command() -> Command {
    Command::new("init")
        .about("初始化新的 RMM 项目")
        .arg(
            Arg::new("path")
                .help("项目路径")
                .value_name("PATH")
                .default_value(".")
        )
        .arg(
            Arg::new("yes")
                .short('y')
                .long("yes")
                .action(ArgAction::SetTrue)
                .help("自动确认所有选项")
        )
        .arg(
            Arg::new("basic")
                .long("basic")
                .action(ArgAction::SetTrue)
                .help("创建基础项目（默认）")
        )
        .arg(
            Arg::new("lib")
                .long("lib")
                .action(ArgAction::SetTrue)
                .help("创建库项目")
        )
        .arg(
            Arg::new("ravd")
                .long("ravd")
                .action(ArgAction::SetTrue)
                .help("创建 RAVD 项目")
        )
}

pub fn handle_init(config: &RmmConfig, matches: &ArgMatches) -> Result<()> {
    let project_path = matches.get_one::<String>("path").unwrap();
    let yes = matches.get_flag("yes");
    let is_lib = matches.get_flag("lib");
    let is_ravd = matches.get_flag("ravd");
      let path = Path::new(project_path);    // 获取项目名称，正确处理当前目录的情况
    let project_name = if project_path == "." {
        // 如果是当前目录，获取当前目录的名称并存储为 String
        std::env::current_dir()?
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unnamed_project".to_string())
    } else {
        // 如果是其他路径，获取路径的最后一部分
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unnamed_project".to_string())
    };
    
    println!("🚀 正在初始化 RMM 项目: {}", project_name);
    println!("📁 项目路径: {}", path.display());
    
    // 确保项目目录存在
    ensure_dir_exists(path)?;
      // 检测 Git 信息
    let git_info = get_git_info(path);
    
    // 使用RMM配置中的用户信息作为默认值
    let author_name = &config.username;
    let author_email = &config.email;
      // 创建项目配置
    let project_config = create_project_config(&project_name, author_name, author_email, &config.version, git_info)?;
    
    // 保存项目配置
    project_config.save_to_dir(path)?;
    
    // 创建项目结构
    if is_lib {
        create_library_structure(path)?;
        println!("📚 已创建库项目结构");
    } else if is_ravd {
        create_ravd_structure(path)?;
        println!("🎮 已创建 RAVD 项目结构");
    } else {
        create_basic_structure(path)?;
        println!("📦 已创建基础项目结构");
    }    // 创建基础文件
    create_basic_files(path, &project_name, author_name)?;
      // 创建 module.prop
    create_module_prop(path, &project_config)?;
      // 将新创建的项目添加到全局元数据
    let mut rmm_config = RmmConfig::load()?;
    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    rmm_config.add_current_project(&project_name, &canonical_path)?;
    
    println!("✅ 项目 '{}' 初始化完成！", project_name);
    
    if !yes {
        println!("\n💡 提示:");
        println!("  - 使用 'rmm build' 构建项目");
        println!("  - 使用 'rmm sync' 同步项目");
        println!("  - 编辑 'rmmproject.toml' 配置项目信息");
    }
    
    Ok(())
}

fn create_project_config(
    name: &str,
    username: &str,
    email: &str,
    rmm_version: &str,
    git_info: Option<crate::utils::GitInfo>,
) -> Result<ProjectConfig> {
    // 只有当项目在GitHub仓库中时才生成真实的GitHub URL
    let (github_url, update_json) = if let Some(ref git) = git_info {
        if git.remote_url.contains("github.com") {
            // 在GitHub仓库中，生成真实URL
            let github_url = format!("https://github.com/{}/{}", git.username, git.repo_name);
            let update_json = if git.is_in_repo_root {
                format!("https://raw.githubusercontent.com/{}/{}/main/update.json", git.username, git.repo_name)
            } else {
                // 如果不在仓库根目录，需要计算相对路径
                format!("https://raw.githubusercontent.com/{}/{}/main/{}/update.json", git.username, git.repo_name, name)
            };
            (github_url, update_json)
        } else {
            // 非GitHub仓库，使用占位符
            (
                "https://github.com/YOUR_USERNAME/YOUR_REPOSITORY".to_string(),
                "https://raw.githubusercontent.com/YOUR_USERNAME/YOUR_REPOSITORY/main/update.json".to_string()
            )
        }
    } else {
        // 没有Git仓库，使用占位符
        (
            "https://github.com/YOUR_USERNAME/YOUR_REPOSITORY".to_string(),
            "https://raw.githubusercontent.com/YOUR_USERNAME/YOUR_REPOSITORY/main/update.json".to_string()
        )
    };
      Ok(ProjectConfig {
        id: name.to_string(),
        name: name.to_string(),
        description: Some(format!("RMM项目 {}", name)),        requires_rmm: format!(">={}", rmm_version),
        version: Some("v0.1.0".to_string()),
        version_code: "1000000".to_string(), // 使用合理的初始版本代码
        update_json,
        readme: "README.MD".to_string(),
        changelog: "CHANGELOG.MD".to_string(),
        license: "LICENSE".to_string(),
        dependencies: vec![],
        authors: vec![crate::config::Author {
            name: username.to_string(),
            email: email.to_string(),
        }],        
        scripts: {
            let mut scripts = HashMap::new();
            scripts.insert("build".to_string(), "rmm build".to_string());
            scripts
        },
        urls: crate::config::Urls {
            github: github_url,
        },        build: Some(crate::config::BuildConfig {
            prebuild: Some(vec!["Rmake".to_string()]),
            build: Some(vec!["default".to_string()]),
            postbuild: Some(vec!["Rmake".to_string()]),
            exclude: Some(vec![
                ".git".to_string(),
                "target".to_string(),
                "*.log".to_string(),
                ".vscode".to_string(),
                ".idea".to_string(),
            ]),
        }),
        git: git_info.map(|gi| crate::config::GitInfo {
            git_root: gi.git_root,
            remote_url: gi.remote_url,
            username: gi.username,
            repo_name: gi.repo_name,
            is_in_repo_root: gi.is_in_repo_root,
        }),
    })
}

fn create_basic_structure(path: &Path) -> Result<()> {
    ensure_dir_exists(&path.join("system"))?;
    ensure_dir_exists(&path.join(".rmmp"))?;
    Ok(())
}

fn create_library_structure(path: &Path) -> Result<()> {
    ensure_dir_exists(&path.join("lib"))?;
    ensure_dir_exists(&path.join(".rmmp"))?;
    Ok(())
}

fn create_ravd_structure(path: &Path) -> Result<()> {
    ensure_dir_exists(&path.join("assets"))?;
    ensure_dir_exists(&path.join("scripts"))?;
    ensure_dir_exists(&path.join(".rmmp"))?;
    Ok(())
}

fn create_basic_files(path: &Path, project_name: &str, author: &str) -> Result<()> {
    // README.MD
    let readme_content = format!(r#"# {}

一个基于 RMM (Root Module Manager) 的模块项目。

## 功能特性

- 支持 Magisk、APatch、KernelSU
- 自动版本管理
- 构建输出优化
- GitHub 集成

## 安装方法

1. 下载最新的 release 文件
2. 通过 Magisk/APatch/KernelSU 安装模块
3. 重启设备

## 构建

```bash
# 构建模块
rmm build

# 发布到 GitHub
rmm publish
```

## 开发

```bash
# 安装开发依赖
uv tool install pyrmm

# 初始化项目
rmm init .

# 构建并测试
rmm build && rmm test
```

## 许可证

MIT License - 查看 [LICENSE](LICENSE) 文件了解详情。

## 作者

- {}

---

使用 [RMM](https://github.com/LIghtJUNction/RootManage-Module-Model) 构建
"#, project_name, author);

    // CHANGELOG.MD
    let changelog_content = format!(r#"# 更新日志

所有对该项目的重要更改都会记录在此文件中。

## [未发布]

### 新增
- 初始项目设置
- 基本模块结构

### 变更
- 无

### 修复
- 无

## [1.0.0] - {}

### 新增
- 项目初始版本
- 基本功能实现

---

## 版本格式说明

- **[未发布]** - 即将发布的更改
- **[版本号]** - 已发布的版本及发布日期

### 更改类型

- **新增** - 新功能
- **变更** - 现有功能的更改
- **弃用** - 即将移除的功能
- **移除** - 已移除的功能
- **修复** - Bug 修复
- **安全** - 安全相关的修复
"#, chrono::Utc::now().format("%Y-%m-%d"));

    // LICENSE
    let license_content = r#"
# LICENSES


# RMM License
MIT License

Copyright (c) 2025 LIghtJUNction

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"#;

    // customize.sh
    let customize_content = r#"#!/system/bin/sh

# RMM 模块自定义脚本
# 此脚本在模块安装时执行，用于进行必要的设置和配置

MODDIR=${0%/*}

# 打印安装信息
ui_print "- 正在安装 RMM 模块..."
ui_print "- 模块目录: $MODDIR"

# 设置权限
set_perm_recursive "$MODDIR" 0 0 0755 0644

# 自定义安装逻辑
# 在这里添加您的安装步骤

ui_print "- 模块安装完成"
"#;

    let files = vec![
        ("README.MD", readme_content),
        ("CHANGELOG.MD", changelog_content),
        ("LICENSE", license_content.to_string()),
        ("customize.sh", customize_content.to_string()),
    ];

    for (filename, content) in files {
        let file_path = path.join(filename);
        if !file_path.exists() {
            fs::write(&file_path, content)?;
            println!("✅ 创建文件: {}", filename);
        }
    }

    Ok(())
}

fn create_module_prop(path: &Path, config: &ProjectConfig) -> Result<()> {
    let module_prop_content = format!(
        "id={}\nname={}\nversion={}\nversionCode={}\nauthor={}\ndescription={}\nupdateJson={}\n",
        config.id,
        config.name,
        config.version.as_ref().unwrap_or(&"v0.1.0".to_string()),
        config.version_code,
        config.authors.first().map(|a| &a.name).unwrap_or(&config.id),
        config.description.as_ref().unwrap_or(&config.name),
        config.update_json
    );

    let module_prop_path = path.join("module.prop");
    fs::write(&module_prop_path, module_prop_content)?;
    println!("✅ 创建文件: module.prop");

    Ok(())
}
