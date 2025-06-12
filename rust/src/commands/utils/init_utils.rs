use anyhow::{Result};
use std::fs;
use std::path::Path;
use std::collections::HashMap;
use crate::commands::utils::core::project::{ProjectConfig, Author, Urls, BuildConfig as BuildSettings}; // Renamed BuildConfig to BuildSettings
use crate::commands::utils::core::rmake::RmakeConfig; // Added import for RmakeConfig
use crate::commands::utils::core::common::FileSystemManager;

/// 生成 module.prop 文件内容
pub fn generate_module_prop_content(config: &ProjectConfig) -> String {
    let version = config.version.as_deref().unwrap_or("v1.0.0");
    format!(
        r#"id={}
name={}
version={}
versionCode={}
author={}
description={}
updateJson={}
"#,
        config.id,
        config.name,
        version,
        config.version_code,
        config.authors.first()
            .map(|a| a.name.as_str())
            .unwrap_or("Unknown"),
        config.description.as_deref().unwrap_or(""),
        config.update_json
    )
}

/// 创建 module.prop 文件（用于项目初始化）
pub fn create_module_prop(path: &Path, config: &ProjectConfig) -> Result<()> {
    let module_prop_content = generate_module_prop_content(config);
    let module_prop_path = path.join("module.prop");
    std::fs::write(&module_prop_path, module_prop_content)?;
    println!("✅ 创建文件: module.prop");
    Ok(())
}

/// 生成 module.prop 文件（用于构建时）
pub fn generate_module_prop(config: &ProjectConfig, build_dir: &Path) -> Result<()> {
    let module_prop_content = generate_module_prop_content(config);
    let module_prop_path = build_dir.join("module.prop");
    std::fs::write(module_prop_path, module_prop_content)?;
    println!("📄 生成 module.prop");
    Ok(())
}

pub fn create_project_config(
    project_id: &str,
    author_name: &str,
    author_email: &str,
    rmm_version: &str,
    git_info_tuple: Option<(String, String)>, // Changed type from Option<GitInfo>
) -> Result<ProjectConfig> {    let path = std::env::current_dir()?.join(project_id);
    FileSystemManager::ensure_dir_exists(&path)?;
    
    let git_info_struct: Option<crate::commands::utils::core::project::GitInfo> = git_info_tuple.as_ref().map(|(username, repo_name)| crate::commands::utils::core::project::GitInfo {
        url: format!("https://github.com/{}/{}", username, repo_name),
        branch: "main".to_string(),
        commit: "".to_string(),
        git_root: path.to_string_lossy().into_owned(),
        remote_url: format!("https://github.com/{}/{}.git", username, repo_name),
        username: username.clone(), // Clone if username is &String, or ensure it's String
        repo_name: repo_name.clone(), // Clone if repo_name is &String, or ensure it's String
        is_in_repo_root: true,
    });

    let mut authors = vec![Author {
        name: author_name.to_string(),
        email: author_email.to_string(),
    }];

    // If git information is available and it indicates being in a repo root,
    // try to add git user as author if not already present.
    if let Some(git_details) = &git_info_struct { // Changed from git_info to git_info_struct
        if git_details.is_in_repo_root { // Now correctly refers to the field of GitInfo
            // This is a simplified check. A more robust check would compare emails or full names.
            if !authors.iter().any(|a| a.name == git_details.username) {
                authors.push(Author {
                    name: git_details.username.clone(),
                    email: "".to_string(), // Placeholder for git email if available
                });
            }
        }
    }

    let (github_url, update_json) = if let Some(ref git_tuple_data) = git_info_tuple { // Renamed git to git_tuple_data for clarity
        // git_tuple_data is (username, repo_name_or_url)
        // git_info_struct is Option<GitInfo> which has is_in_repo_root
        if git_tuple_data.1.contains("github.com") { // Assuming git_tuple_data.1 is the repo URL or name part
            // 在GitHub仓库中，生成真实URL
            let github_url = format!("https://github.com/{}/{}", git_tuple_data.0, git_tuple_data.1); // Use tuple data for URL parts

            let in_repo_root = git_info_struct.as_ref().map_or(false, |gis| gis.is_in_repo_root);

            let update_json_url_path = if in_repo_root {
                format!("https://raw.githubusercontent.com/{}/{}/main/update.json", git_tuple_data.0, git_tuple_data.1)
            } else {
                // 如果不在仓库根目录，需要计算相对路径
                format!("https://raw.githubusercontent.com/{}/{}/main/{}/update.json", git_tuple_data.0, git_tuple_data.1, project_id)
            };
            (github_url, update_json_url_path)
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
    };    Ok(ProjectConfig {
        id: project_id.to_string(),
        name: project_id.to_string(),
        description: Some(format!("RMM项目 {}", project_id)),
        requires_rmm: format!(">={}", rmm_version),
        version: Some("v0.1.0".to_string()),
        version_code: "1000000".to_string(), // 使用合理的初始版本代码
        update_json,
        readme: "README.MD".to_string(),
        changelog: "CHANGELOG.MD".to_string(),
        license: "LICENSE".to_string(),
        dependencies: vec![],
        authors: vec![Author {
            name: author_name.to_string(),
            email: author_email.to_string(),
        }],
        scripts: {
            let mut scripts = HashMap::new();
            scripts.insert("build".to_string(), "rmm build".to_string());
            scripts
        },        urls: Urls {
            github: github_url,
        },
        build: Some(BuildSettings {
            target: None,
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
        git: git_info_struct,
    })
}

pub fn create_basic_structure(path: &Path) -> Result<()> {
    FileSystemManager::ensure_dir_exists(&path.join("system"))?;
    FileSystemManager::ensure_dir_exists(&path.join(".rmmp"))?;
    Ok(())
}

pub fn create_library_structure(path: &Path) -> Result<()> {
    FileSystemManager::ensure_dir_exists(&path.join("lib"))?;
    FileSystemManager::ensure_dir_exists(&path.join(".rmmp"))?;
    Ok(())
}

pub fn create_ravd_structure(path: &Path) -> Result<()> {
    FileSystemManager::ensure_dir_exists(&path.join("assets"))?;
    FileSystemManager::ensure_dir_exists(&path.join("scripts"))?;
    FileSystemManager::ensure_dir_exists(&path.join(".rmmp"))?;
    Ok(())
}

pub fn create_basic_files(path: &Path, project_name: &str, author: &str) -> Result<()> {
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
    }    Ok(())
}

pub fn create_rmake_toml(path: &Path, _project_name: &str) -> Result<()> {
    // 使用默认的 RmakeConfig 生成 Rmake.toml
    let default_config = RmakeConfig::default(); 
    // save_to_dir 会创建 .rmmp 目录并写入 Rmake.toml
    default_config.save_to_dir(path)?;
    println!("✅ 创建默认 .rmmp/Rmake.toml");
    Ok(())
}
