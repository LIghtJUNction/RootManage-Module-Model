use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml;
use colored::*;
use chrono::{Utc, Datelike};
use serde_json;
use git2::{Repository, Config};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::core::rmm_core::{
    Author, BuildConfig, BuildSystem, ModuleProp, ProjectInfo, 
    RmakeConfig, RmmProject, SrcConfig, UrlsInfo, GitAnalyzer, GitInfo
};

/// 初始化新的模块项目
pub fn init_project(project_path: &Path, project_id: &str, author: &str, email: &str) -> Result<()> {
    let project_path = project_path.canonicalize().unwrap_or_else(|_| project_path.to_path_buf());
      // 确保项目目录存在
    if !project_path.exists() {
        anyhow::bail!("项目目录不存在: {}", project_path.display());
    }    // 检查是否已经是一个项目，如果是，则打印警告而不是直接退出
    if project_path.join("module.prop").exists() || project_path.join(".rmmp").exists() {
        println!("{} 检测到目录已包含项目文件，将跳过已存在的文件和目录。", "⚠️ ".yellow().bold());
    } else {
        println!("{} 正在初始化模块项目: {}", "🚀".green().bold(), project_id.cyan().bold());
    }// 验证项目ID格式（符合KernelSU要求）
    // ID必须与这个正则表达式匹配：^[a-zA-Z][a-zA-Z0-9._-]+$
    // 例如：✓ a_module，✓ a.module，✓ module-101，✗ a module，✗ 1_module，✗ -a-module
    let id_regex = regex::Regex::new(r"^[a-zA-Z][a-zA-Z0-9._-]+$").unwrap();
    if !id_regex.is_match(project_id) {
        anyhow::bail!("项目ID格式无效。必须以字母开头，只能包含字母、数字、点、下划线和连字符，且至少2个字符");
    }

    // 获取智能用户信息
    let (smart_author, smart_email) = get_smart_user_info(author, email, &project_path)?;

    // 检测 Git 信息
    let git_info = GitAnalyzer::analyze_git_info(&project_path)?;
    
    if let Some(ref git) = git_info {
        println!("{} 检测到 Git 仓库", "🔍".yellow().bold());
        println!("  {}: {}", 
            "分支".cyan().bold(), 
            git.branch.green().bold()
        );
        if let Some(ref remote_url) = git.remote_url {
            println!("  {}: {}", 
                "远程仓库".cyan().bold(), 
                remote_url.green()
            );
        }
        if git.has_uncommitted_changes {
            println!("  {}: {}", 
                "状态".cyan().bold(), 
                "有未提交的更改".yellow()
            );
        } else {
            println!("  {}: {}", 
                "状态".cyan().bold(), 
                "工作目录清洁".green()
            );
        }
        println!();
    }

    println!("{} 正在初始化模块项目: {}", 
        "🚀".green().bold(), 
        project_id.cyan().bold()
    );

    // 1. 创建.rmmp目录结构
    create_rmmp_structure(&project_path)?;

    // 2. 创建Rmake.toml
    create_rmake_config(&project_path)?;    // 3. 创建rmmproject.toml
    create_project_config(&project_path, project_id, &smart_author, &smart_email, &git_info)?;

    // 4. 创建module.prop
    create_module_prop(&project_path, project_id, &smart_author, &git_info)?;

    // 5. 创建system目录
    create_system_structure(&project_path)?;

    // 6. 创建customize.sh
    create_customize_script(&project_path)?;

    // 7. 创建update.json
    create_update_json(&project_path, project_id, &git_info)?;

    // 8. 创建其他推荐文件
    create_documentation_files(&project_path, project_id)?;println!();
    println!("{} 模块项目初始化完成！", "🎉".green().bold());
    println!("{} 项目路径: {}", 
        "📁".cyan().bold(), 
        project_path.display().to_string().green()
    );
    println!("{} 项目ID: {}", 
        "🔧".cyan().bold(), 
        project_id.green().bold()
    );
    println!();
    println!("{}:", "下一步".yellow().bold());
    println!("  {}. 编辑 {} 目录，添加你要修改的系统文件", 
        "1".cyan().bold(), 
        "system/".green().bold()
    );
    println!("  {}. 根据需要修改 {} 安装脚本", 
        "2".cyan().bold(), 
        "customize.sh".green().bold()
    );
    println!("  {}. 运行 {} 构建模块", 
        "3".cyan().bold(), 
        "'rmm build'".green().bold()
    );    println!("  {}. 运行 {} 安装到设备测试", 
        "4".cyan().bold(), 
        "'rmm device install'".green().bold()
    );
    println!();

    Ok(())
}

/// 获取智能用户信息，优先使用Git配置
fn get_smart_user_info(author: &str, email: &str, project_path: &Path) -> Result<(String, String)> {
    // 如果用户提供的是默认值，尝试从Git获取
    let mut final_author = author.to_string();
    let mut final_email = email.to_string();
    
    // 检查是否需要从Git获取信息
    let should_get_git_author = author == "Your Name" || author.is_empty();
    let should_get_git_email = email == "your.email@example.com" || email.is_empty();
    
    if should_get_git_author || should_get_git_email {
        if let Ok((git_author, git_email)) = get_git_user_config(project_path) {
            if should_get_git_author && !git_author.is_empty() {
                final_author = git_author;
            }
            if should_get_git_email && !git_email.is_empty() {
                final_email = git_email;
            }
        }
    }
    
    Ok((final_author, final_email))
}

/// 从Git配置获取用户信息
fn get_git_user_config(project_path: &Path) -> Result<(String, String)> {
    // 尝试从项目级Git配置获取
    if let Ok(repo) = Repository::open(project_path) {
        if let Ok(config) = repo.config() {
            let name = config.get_string("user.name").unwrap_or_default();
            let email = config.get_string("user.email").unwrap_or_default();
            return Ok((name, email));
        }
    }
    
    // 如果项目级配置不可用，尝试全局配置
    if let Ok(config) = Config::open_default() {
        let name = config.get_string("user.name").unwrap_or_default();
        let email = config.get_string("user.email").unwrap_or_default();
        return Ok((name, email));
    }
    
    Ok((String::new(), String::new()))
}

/// 创建.rmmp目录结构
fn create_rmmp_structure(project_path: &Path) -> Result<()> {
    let rmmp_dir = project_path.join(".rmmp");
    let build_dir = rmmp_dir.join("build");
    let dist_dir = rmmp_dir.join("dist");

    if rmmp_dir.exists() {
        println!("{} 目录 {} 已存在，跳过创建。", "[!]".yellow().bold(), ".rmmp".cyan().bold());
    } else {
        fs::create_dir_all(&rmmp_dir)?;
        println!("{} 创建 {} 目录结构", "[+]".green().bold(), ".rmmp".cyan().bold());
    }

    if build_dir.exists() {
        println!("{} 目录 {} 已存在，跳过创建。", "[!]".yellow().bold(), ".rmmp/build".cyan().bold());
    } else {
        fs::create_dir_all(&build_dir)?;
        println!("{} 创建 {} 目录", "[+]".green().bold(), ".rmmp/build".cyan().bold());
    }

    if dist_dir.exists() {
        println!("{} 目录 {} 已存在，跳过创建。", "[!]".yellow().bold(), ".rmmp/dist".cyan().bold());
    } else {
        fs::create_dir_all(&dist_dir)?;
        println!("{} 创建 {} 目录", "[+]".green().bold(), ".rmmp/dist".cyan().bold());
    }
    Ok(())
}

/// 作者注：重复实现，主要是为了稳定性 这个是内部调用的办法。 rmmcore主要是设计给给外部调用的
/// 创建Rmake.toml配置文件
fn create_rmake_config(project_path: &Path) -> Result<()> {
    let rmake_path = project_path.join(".rmmp").join("Rmake.toml");
    
    if rmake_path.exists() {
        println!("{} 文件 {} 已存在，跳过创建。", "[!]".yellow().bold(), ".rmmp/Rmake.toml".cyan().bold());
        return Ok(());
    }

    // 确保父目录存在
    if let Some(parent_dir) = rmake_path.parent() {
        fs::create_dir_all(parent_dir)?;
    }

    let rmake_config = RmakeConfig {
        build: BuildConfig {
            include: vec!["# 额外包含的文件或目录，如：\"extra/\"".to_string()],
            exclude: vec![
                ".git".to_string(), 
                ".rmmp".to_string(), 
                "*.tmp".to_string(), 
                "*.log".to_string()
            ],
            prebuild: vec!["echo 'Starting build'".to_string()],
            build: vec!["rmm".to_string()],
            postbuild: vec!["echo 'Build completed'".to_string()],
            src: Some(SrcConfig {
                include: vec!["# 源代码额外包含的文件，如：\"docs/\"".to_string()],
                exclude: vec![
                    ".git".to_string(),
                    "*.tmp".to_string(),
                    "*.log".to_string(),
                    "node_modules".to_string()
                ],
            }),            
            scripts: Some({
                let mut scripts = HashMap::new();
                // 使用跨平台兼容的clean命令
                let clean_cmd = if cfg!(target_os = "windows") {
                    "Remove-Item '.rmmp\\build' -Recurse -Force -ErrorAction SilentlyContinue; Remove-Item '.rmmp\\dist' -Recurse -Force -ErrorAction SilentlyContinue; New-Item -Path '.rmmp\\build' -ItemType Directory -Force; New-Item -Path '.rmmp\\dist' -ItemType Directory -Force"
                } else {
                    "rm -rf .rmmp/build/* .rmmp/dist/*"
                };
                scripts.insert("clean".to_string(), clean_cmd.to_string());
                // 安装模块的手动方式参考：
                // /data/adb/magisk --install-module xxx
                // /data/adb/ksud module install xxx
                // /data/adb/apd module install xxx
                scripts
            }),
        },
    };
    
    let rmake_content = toml::to_string_pretty(&rmake_config)?;
    // 保存到 .rmmp/Rmake.toml
    fs::write(&rmake_path, rmake_content)?;
    println!("{} 创建 {}", 
        "[+]".green().bold(), 
        ".rmmp/Rmake.toml".cyan().bold()
    );
    Ok(())
}

/// 创建项目配置文件
fn create_project_config(project_path: &Path, project_id: &str, author: &str, email: &str, git_info: &Option<GitInfo>) -> Result<()> {
    let project_config_path = project_path.join("rmmproject.toml");
    
    if project_config_path.exists() {
        println!("{} 文件 {} 已存在，跳过创建。", "[!]".yellow().bold(), "rmmproject.toml".cyan().bold());
        return Ok(());
    }

    // 生成智能的update_json URL
    let update_json_url = if let Some(git) = git_info {
        if let Some(remote_url) = &git.remote_url {
            generate_update_json_url(remote_url, project_id)
        } else {
            format!("https://github.com/{}/releases/latest/download/update.json", project_id)
        }
    } else {
        format!("https://github.com/{}/releases/latest/download/update.json", project_id)
    };

    // 生成GitHub URL
    let github_url = if let Some(git) = git_info {
        if let Some(remote_url) = &git.remote_url {
            if let Some((owner, repo)) = parse_github_url(remote_url) {
                format!("https://github.com/{}/{}", owner, repo)
            } else {
                format!("https://github.com/{}/{}", author, project_id)
            }
        } else {
            format!("https://github.com/{}/{}", author, project_id)
        }
    } else {
        format!("https://github.com/{}/{}", author, project_id)
    };

    let project_config = RmmProject {
        project: ProjectInfo {
            id: project_id.to_string(),
            description: format!("A Rmm project: {}", project_id),
            readme: "README.md".to_string(),
            changelog: "CHANGELOG.md".to_string(),
            license: "LICENSE".to_string(),
            dependencies: Vec::new(),
            scripts: Some({
                let mut scripts = HashMap::new();
                scripts.insert("build".to_string(), "rmm build".to_string());
                scripts.insert("install".to_string(), "rmm device install".to_string());
                scripts.insert("test".to_string(), "rmm test".to_string());
                scripts
            }),
        },
        authors: vec![Author {
            name: author.to_string(),
            email: email.to_string(),
        }],
        urls: Some(UrlsInfo {
            github: github_url,
        }),
        build_system: Some(BuildSystem {
            requires: vec!["rmm>=0.3.0".to_string()],
            build_backend: "rmm".to_string(),
        }),
        tool: None,
    };

    let project_content = toml::to_string_pretty(&project_config)?;
    fs::write(&project_config_path, project_content)?;
    println!("{} 创建 {}", 
        "[+]".green().bold(), 
        "rmmproject.toml".cyan().bold()
    );
    Ok(())
}

/// 创建module.prop文件
fn create_module_prop(project_path: &Path, project_id: &str, author: &str, git_info: &Option<GitInfo>) -> Result<()> {
    let module_prop_path = project_path.join("module.prop");
    
    if module_prop_path.exists() {
        println!("{} 文件 {} 已存在，跳过创建。", "[!]".yellow().bold(), "module.prop".cyan().bold());
        return Ok(());
    }

    // 生成智能的update_json URL
    let update_json_url = if let Some(git) = git_info {
        if let Some(remote_url) = &git.remote_url {
            generate_update_json_url(remote_url, project_id)
        } else {
            format!("https://github.com/{}/releases/latest/download/update.json", project_id)
        }
    } else {
        format!("https://github.com/{}/releases/latest/download/update.json", project_id)
    };

    // 生成基于当前日期的 versionCode（整数）
    let now = Utc::now();
    let version_code: i64 = format!("{:04}{:02}{:02}{:02}", 
        now.year(), now.month(), now.day(), 1).parse().unwrap_or(2025061301);

    let module_prop = ModuleProp {
        id: project_id.to_string(),
        name: format!("{} Module", 
            project_id.chars().next().unwrap().to_uppercase().to_string() + &project_id[1..]),
        version: env!("CARGO_PKG_VERSION").to_string(),
        version_code: version_code.to_string(),
        author: author.to_string(),
        description: format!("A rmm project: {}", project_id),
        update_json: update_json_url,
    };

    // 使用 UNIX 换行符 (LF) 构建内容
    let mut prop_content = String::new();
    prop_content.push_str(&format!("id={}\n", module_prop.id));
    prop_content.push_str(&format!("name={}\n", module_prop.name));
    prop_content.push_str(&format!("version={}\n", module_prop.version));
    prop_content.push_str(&format!("versionCode={}\n", module_prop.version_code));
    prop_content.push_str(&format!("author={}\n", module_prop.author));
    prop_content.push_str(&format!("description={}\n", module_prop.description));
    prop_content.push_str(&format!("updateJson={}\n", module_prop.update_json));

    // 确保使用 UNIX 换行符写入文件
    let prop_content_bytes = prop_content.replace("\r\n", "\n").replace("\r", "\n");
    fs::write(&module_prop_path, prop_content_bytes)?;
    println!("{} 创建 {}", 
        "[+]".green().bold(), 
        "module.prop".cyan().bold()
    );
    Ok(())
}

/// 创建system目录结构
fn create_system_structure(project_path: &Path) -> Result<()> {
    let system_dir = project_path.join("system");
    let system_etc_dir = system_dir.join("etc");
    let example_conf_path = system_etc_dir.join("example.conf");

    if system_dir.exists() {
        println!("{} 目录 {} 已存在，跳过创建。", "[!]".yellow().bold(), "system".cyan().bold());
    } else {
        fs::create_dir_all(&system_dir)?;
        println!("{} 创建 {} 目录", "[+]".green().bold(), "system".cyan().bold());
    }
    
    // 创建一个示例目录和文件
    if system_etc_dir.exists() {
        println!("{} 目录 {} 已存在，跳过创建。", "[!]".yellow().bold(), "system/etc".cyan().bold());
    } else {
        fs::create_dir_all(&system_etc_dir)?;
        println!("{} 创建 {} 目录", "[+]".green().bold(), "system/etc".cyan().bold());
    }

    if example_conf_path.exists() {
        println!("{} 文件 {} 已存在，跳过创建。", "[!]".yellow().bold(), "system/etc/example.conf".cyan().bold());
    } else {
        fs::write(
            &example_conf_path,
            "# 这是一个示例配置文件\n# 将此文件放置在system目录中，它会被挂载到 /system/etc/example.conf\n"
        )?;
        println!("{} 创建 {} 文件", "[+]".green().bold(), "system/etc/example.conf".cyan().bold());
    }

    Ok(())
}

/// 创建customize.sh安装脚本
fn create_customize_script(project_path: &Path) -> Result<()> {
    let customize_script_path = project_path.join("customize.sh");
    
    if customize_script_path.exists() {
        println!("{} 文件 {} 已存在，跳过创建。", "[!]".yellow().bold(), "customize.sh".cyan().bold());
        return Ok(());
    }

    let customize_script = r#"#!/system/bin/sh
# KernelSU 模块自定义安装脚本

# 检查设备信息
ui_print "- 设备架构: $ARCH"
ui_print "- Android API: $API"
ui_print "- KernelSU 版本: $KSU_VER"

# 根据设备架构进行不同的处理
case $ARCH in
    arm64)
        ui_print "- 64位ARM设备"
        ;;
    arm)
        ui_print "- 32位ARM设备"
        ;;
    x64)
        ui_print "- x86_64设备"
        ;;
    x86)
        ui_print "- x86设备"
        ;;
esac

# 根据Android版本进行处理
# 示例shellcheck 自动修复 $API -> "$API"
if [ $API -lt 29 ]; then
    ui_print "- Android 10以下版本"
else
    ui_print "- Android 10及以上版本"
fi

# 设置权限（如果需要）
# set_perm_recursive $MODPATH/system/bin 0 0 0755 0755
# set_perm $MODPATH/system/etc/example.conf 0 0 0644

# 示例：删除系统文件（取消注释以使用）
# REMOVE="
# /system/app/SomeSystemApp
# /system/etc/some_config_file
# "

# 示例：替换系统目录（取消注释以使用）
# REPLACE="
# /system/app/SomeSystemApp
# "

ui_print "- 模块安装完成"
"#;

    fs::write(&customize_script_path, customize_script)?;
    
    // 设置可执行权限（仅在Unix系统上）
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&customize_script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&customize_script_path, perms)?;
    }
    
    println!("{} 创建 {}", 
        "[+]".green().bold(), 
        "customize.sh".cyan().bold()
    );
    Ok(())
}

/// 创建文档文件
fn create_documentation_files(project_path: &Path, project_id: &str) -> Result<()> {
    create_readme(project_path, project_id)?;
    create_changelog(project_path)?;
    create_license(project_path)?;
    Ok(())
}

/// 创建README.md文件
fn create_readme(project_path: &Path, project_id: &str) -> Result<()> {
    let readme_path = project_path.join("README.md");
    
    if readme_path.exists() {
        println!("{} 文件 {} 已存在，跳过创建。", "[!]".yellow().bold(), "README.md".cyan().bold());
        return Ok(());
    }

    let readme_content = format!(r#"# {} Module

这是一个 rmm 模块项目。

## 说明

RMMP ID: {}

## 安装

1. 使用 ROOT 管理器安装此模块
2. 重启设备

## 开发

```bash
# 构建模块
rmm build

# 安装到设备
rmm device install

# 运行测试
rmm test
```

## 文件结构

```
{}
├── .rmmp/              # RMM 项目文件
│   ├── Rmake.toml     # 构建配置
│   ├── build/         # 构建输出
│   └── dist/          # 发布文件
├── system/            # 系统文件覆盖
├── module.prop        # 模块属性
├── customize.sh       # 安装脚本
├── rmmproject.toml    # 项目配置
└── README.md          # 说明文档
```

## 许可证

见 LICENSE 文件。
"#, 
        project_id.chars().next().unwrap().to_uppercase().to_string() + &project_id[1..],
        project_id, 
        project_id
    );

    fs::write(&readme_path, readme_content)?;
    println!("{} 创建 {}", 
        "[+]".green().bold(), 
        "README.md".cyan().bold()
    );
    Ok(())
}

/// 创建CHANGELOG.md文件
fn create_changelog(project_path: &Path) -> Result<()> {
    let changelog_path = project_path.join("CHANGELOG.md");
    
    if changelog_path.exists() {
        println!("{} 文件 {} 已存在，跳过创建。", "[!]".yellow().bold(), "CHANGELOG.md".cyan().bold());
        return Ok(());
    }

    let changelog_content = r#"# 更新日志

### 新增
- 初始版本
- 基本模块功能

### 修复
- 无

### 更改
- 无
"#;

    fs::write(&changelog_path, changelog_content)?;
    println!("{} 创建 {}", 
        "[+]".green().bold(), 
        "CHANGELOG.md".cyan().bold()
    );
    Ok(())
}

/// 创建LICENSE文件
fn create_license(project_path: &Path) -> Result<()> {
    let license_path = project_path.join("LICENSE");
    
    if license_path.exists() {
        println!("{} 文件 {} 已存在，跳过创建。", "[!]".yellow().bold(), "LICENSE".cyan().bold());
        return Ok(());
    }

    let license_content = r#"#在此处添加你的许可证
    
# 请不要移除以下许可信息
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

    fs::write(&license_path, license_content)?;
    println!("{} 创建 {}", 
        "[+]".green().bold(), 
        "LICENSE".cyan().bold()
    );
    Ok(())
}


/// 生成 update.json URL
fn generate_update_json_url(remote_url: &str, project_id: &str) -> String {
    // 解析GitHub URL
    if let Some(github_info) = parse_github_url(remote_url) {
        format!("https://github.com/{}/{}/releases/latest/download/update.json", 
                github_info.0, github_info.1)
    } else {
        // 非GitHub仓库，使用通用格式
        format!("https://github.com/USER/{}/releases/latest/download/update.json", project_id)
    }
}

/// 解析GitHub URL，返回 (owner, repo)
fn parse_github_url(url: &str) -> Option<(String, String)> {
    let patterns = [
        r"github\.com[:/]([^/]+)/([^/\.]+)(?:\.git)?",
        r"github\.com/([^/]+)/([^/\.]+)",
    ];
    
    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(url) {
                if caps.len() >= 3 {
                    return Some((caps[1].to_string(), caps[2].to_string()));
                }
            }
        }
    }
    None
}

/// 创建 update.json 文件
fn create_update_json(
    project_path: &Path, 
    project_id: &str, 
    git_info: &Option<GitInfo>
) -> Result<()> {
    let update_json_path = project_path.join("update.json");
    
    if update_json_path.exists() {
        println!("{} 文件 {} 已存在，跳过创建。", "[!]".yellow().bold(), "update.json".cyan().bold());
        return Ok(());
    }

    use serde_json::json;
    use chrono::{Utc, Datelike};
      // 生成版本代码（基于当前日期，与 module.prop 保持一致）
    let now = Utc::now();
    let version_code_int: i64 = format!("{:04}{:02}{:02}{:02}", 
        now.year(), now.month(), now.day(), 1).parse().unwrap_or(2025061301);
    
    // 生成版本号
    let version = if let Some(git) = git_info {
        if let Some(commit_hash) = &git.last_commit_hash {
            format!("v0.1.0-{}", &commit_hash[..8])
        } else {
            "v0.1.0".to_string()
        }
    } else {
        "v0.1.0".to_string()
    };
    
    // 生成发布包 URL
    let zip_url = if let Some(git) = git_info {
        if let Some(remote_url) = &git.remote_url {            if let Some((owner, repo)) = parse_github_url(remote_url) {
                format!("https://github.com/{}/{}/releases/latest/download/{}-{}.zip", 
                        owner, repo, project_id, version_code_int)
            } else {
                format!("https://github.com/USER/{}/releases/latest/download/{}-{}.zip", 
                        project_id, project_id, version_code_int)
            }
        } else {
            format!("https://github.com/USER/{}/releases/latest/download/{}-{}.zip", 
                    project_id, project_id, version_code_int)
        }
    } else {
        format!("https://github.com/USER/{}/releases/latest/download/{}-{}.zip", 
                project_id, project_id, version_code_int)
    };    // 生成 changelog URL，需要考虑项目的相对路径
    let changelog_url = if let Some(git) = git_info {
        if let Some(remote_url) = &git.remote_url {            if let Some((owner, repo)) = parse_github_url(remote_url) {
                // 计算项目相对于 Git 仓库根目录的路径
                let project_relative_path = if let Ok(repo_root) = get_git_repo_root(project_path) {
                    // 规范化项目路径
                    let normalized_project_path = project_path.canonicalize().unwrap_or_else(|_| project_path.to_path_buf());
                    
                    if let Ok(relative_path) = normalized_project_path.strip_prefix(&repo_root) {
                        if relative_path.as_os_str().is_empty() {
                            "CHANGELOG.md".to_string()
                        } else {
                            // 将 Windows 路径分隔符转换为 URL 分隔符
                            let relative_path_str = relative_path.display().to_string().replace("\\", "/");
                            format!("{}/CHANGELOG.md", relative_path_str)
                        }
                    } else {
                        "CHANGELOG.md".to_string()
                    }
                } else {
                    "CHANGELOG.md".to_string()
                };
                
                format!("https://raw.githubusercontent.com/{}/{}/{}/{}", 
                        owner, repo, git.branch, project_relative_path)
            } else {
                format!("https://github.com/USER/REPO/raw/{}/CHANGELOG.md", git.branch)
            }
        } else {
            format!("https://github.com/USER/REPO/raw/{}/CHANGELOG.md", git.branch)
        }
    } else {
        "https://github.com/USER/REPO/raw/main/CHANGELOG.md".to_string()
    };
      let update_json = json!({
        "changelog": changelog_url,
        "version": version,
        "versionCode": version_code_int,
        "zipUrl": zip_url
    });
    
    let update_json_content = serde_json::to_string_pretty(&update_json)?;
    fs::write(project_path.join("update.json"), update_json_content)?;
    
    println!("{} 创建 {}", 
        "[+]".green().bold(), 
        "update.json".cyan().bold()
    );
    Ok(())
}

/// 获取 Git 仓库的根目录
fn get_git_repo_root(path: &Path) -> Result<PathBuf> {
    let repo = git2::Repository::discover(path)
        .map_err(|e| anyhow::anyhow!("无法找到 Git 仓库: {}", e))?;
    
    let workdir = repo.workdir()
        .ok_or_else(|| anyhow::anyhow!("无法获取 Git 工作目录"))?;
    
    // 规范化路径，确保路径格式一致
    Ok(workdir.canonicalize()?)
}

/// 初始化新的脚本项目
pub fn init_script_project(project_path: &Path, script_id: &str, script_type: &str, author: &str, email: &str) -> Result<()> {
    let project_path = project_path.canonicalize().unwrap_or_else(|_| project_path.to_path_buf());
    
    // 确保项目目录存在
    if !project_path.exists() {
        anyhow::bail!("项目目录不存在: {}", project_path.display());
    }    // 检查是否已经是一个项目，如果是，则打印警告而不是直接退出
    if project_path.join(".rmms").exists() || project_path.join(".rmmp").exists() {
        println!("{} 检测到目录已包含项目文件，将跳过已存在的文件和目录。", "⚠️ ".yellow().bold());
    } else {
        println!("{} 正在初始化 RMM 脚本项目: {}", "🚀".blue().bold(), script_id.cyan().bold());
    }

    // 验证脚本ID格式（与模块ID相同）
    let id_regex = regex::Regex::new(r"^[a-zA-Z][a-zA-Z0-9._-]+$").unwrap();
    if !id_regex.is_match(script_id) {
        anyhow::bail!("脚本ID格式无效。必须以字母开头，只能包含字母、数字、点、下划线和连字符，且至少2个字符");
    }    // 验证脚本类型
    let valid_types = ["prebuild", "build", "postbuild"];
    if !valid_types.contains(&script_type) {
        anyhow::bail!("无效的脚本类型。支持的类型: {}", valid_types.join(", "));
    }

    // 获取智能用户信息
    let (smart_author, smart_email) = get_smart_user_info(author, email, &project_path)?;

    println!("{} 创建RMM脚本项目: {}", "🚀".blue().bold(), script_id.yellow().bold());
    println!("  {}: {}", "类型".cyan().bold(), script_type.green().bold());
    println!("  {}: {}", "作者".cyan().bold(), smart_author.green().bold());
    println!("  {}: {}", "邮箱".cyan().bold(), smart_email.green().bold());

    // 创建.rmms标记文件
    create_rmms_file(&project_path, script_type)?;    // 创建脚本文件
    create_script_file(&project_path, script_type)?;

    // 创建脚本元数据和hash文件
    create_script_metadata(&project_path, script_id, script_type, &smart_author, &smart_email)?;

    // 创建README.md
    create_script_readme(&project_path, script_id, script_type, &smart_author)?;

    // 创建脚本元数据和hash文件
    create_script_metadata(&project_path, script_id, script_type, &smart_author, &smart_email)?;

    println!("{}", "\n✅ 脚本项目创建成功！".green().bold());
    println!("下一步:");
    println!("  1. 编辑 {}.{} 实现你的脚本逻辑", script_type, get_script_extension());
    println!("  2. 测试脚本功能");
    println!("  3. 使用 'rmm script publish' 发布脚本");

    Ok(())
}

/// 获取脚本文件扩展名（根据平台）
fn get_script_extension() -> &'static str {
    if cfg!(windows) {
        "ps1"
    } else {
        "sh"
    }
}

/// 创建.rmms标记文件
fn create_rmms_file(project_path: &Path, script_type: &str) -> Result<()> {
    let rmms_path = project_path.join(".rmms");
    
    if rmms_path.exists() {
        println!("  {} 文件 {} 已存在，跳过创建。", "[!]".yellow().bold(), ".rmms".cyan().bold());
        return Ok(());
    }

    let content = format!(r#"# RMM Script 项目标记文件
# 脚本类型配置（3选1）
# 相对路径

# 构建前脚本
{pre_comment}prebuild = prebuild.{ext}

# 构建脚本
{build_comment}build = build.{ext}

# 构建后脚本  
{post_comment}postbuild = postbuild.{ext}
"#,
        ext = get_script_extension(),
        pre_comment = if script_type == "prebuild" { "" } else { "# " },
        build_comment = if script_type == "build" { "" } else { "# " },
        post_comment = if script_type == "postbuild" { "" } else { "# " }
    );

    fs::write(&rmms_path, content)?;
    println!("  ✅ 创建标记文件: {}", rmms_path.display());
    Ok(())
}

/// 创建脚本文件
fn create_script_file(project_path: &Path, script_type: &str) -> Result<()> {
    let script_name = format!("{}.{}", script_type, get_script_extension());
    let script_path = project_path.join(&script_name);
    
    if script_path.exists() {
        println!("  {} 文件 {} 已存在，跳过创建。", "[!]".yellow().bold(), script_name.cyan().bold());
        return Ok(());
    }
    
    let content = if cfg!(windows) {
        // PowerShell脚本模板
        format!(r#"# RMM {} 脚本
# 作者: [作者名]
# 描述: [脚本描述]

param(
    [string]$ProjectPath = ".",
    [string]$Action = "{}"
)

Write-Host "🔧 执行 {} 脚本..." -ForegroundColor Cyan

# 在这里添加你的脚本逻辑
try {{
    # 示例：检查项目路径
    if (!(Test-Path $ProjectPath)) {{
        throw "项目路径不存在: $ProjectPath"
    }}
    
    Write-Host "✅ {} 完成" -ForegroundColor Green
    exit 0
}} catch {{
    Write-Host "❌ {} 失败: $_" -ForegroundColor Red
    exit 1
}}
"#, script_type, script_type, script_type, script_type, script_type)
    } else {
        // Shell脚本模板
        format!(r#"#!/bin/bash
# RMM {} 脚本
# 作者: [作者名]
# 描述: [脚本描述]

set -e  # 遇到错误立即退出

PROJECT_PATH="${{1:-.}}"
ACTION="{}"

echo "🔧 执行 {} 脚本..."

# 在这里添加你的脚本逻辑

# 示例：检查项目路径
if [ ! -d "$PROJECT_PATH" ]; then
    echo "❌ 项目路径不存在: $PROJECT_PATH" >&2
    exit 1
fi

echo "✅ {} 完成"
exit 0
"#, script_type, script_type, script_type, script_type)
    };

    fs::write(&script_path, content)?;
    
    // 在Unix系统上设置执行权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }

    println!("  ✅ 创建脚本文件: {}", script_path.display());
    Ok(())
}

/// 创建脚本项目的README.md
fn create_script_readme(project_path: &Path, script_id: &str, script_type: &str, author: &str) -> Result<()> {
    let readme_path = project_path.join("README.md");
    
    if readme_path.exists() {
        println!("  {} 文件 {} 已存在，跳过创建。", "[!]".yellow().bold(), "README.md".cyan().bold());
        return Ok(());
    }

    let content = format!(r#"# {script_id}

RMM {script_type} 脚本项目

## 描述

[在这里描述你的脚本功能]

## 作者

- **作者**: {author}
- **类型**: {script_type}
- **项目ID**: {script_id}

## 项目结构

```
{script_id}/
├── .rmms                    # RMM脚本项目标记文件
├── {script_type}.{ext}      # 主脚本文件
├── meta.toml               # 脚本元数据文件
├── <hash>.{ext}            # 脚本hash文件
└── README.md               # 项目说明文档
```

## 使用方法

### 本地测试
```bash
# 直接运行脚本
./{script_type}.{ext}

# 或者通过RMM运行 (待实现)
rmm script run {script_type}
```

### 发布到脚本仓库
```bash
# 发布脚本 (待实现)
rmm script publish

# 搜索脚本 (待实现)  
rmm script search <keyword>

# 安装脚本 (待实现)
rmm script install <username/script-id>
```

## 脚本说明

### {script_type} 脚本
- **用途**: [描述脚本的具体用途]
- **触发时机**: [描述何时执行该脚本]
- **输入参数**: [描述脚本的输入参数]
- **输出结果**: [描述脚本的输出结果]

## 元数据信息

脚本的元数据存储在 `meta.toml` 文件中，包含：
- 作者信息和联系方式
- 脚本描述和版本信息
- 脚本类型和hash值
- 创建和更新时间
- username/ID 索引格式

## 开发说明

1. 编辑 `{script_type}.{ext}` 文件实现你的脚本逻辑
2. 测试脚本功能是否正常
3. 更新 `meta.toml` 中的版本和描述信息
4. 使用 `rmm script publish` 发布到脚本仓库

## 许可证

[添加你的许可证信息]
"#, 
        script_id = script_id,
        script_type = script_type,
        author = author,
        ext = get_script_extension()
    );

    fs::write(&readme_path, content)?;
    println!("  ✅ 创建README: {}", readme_path.display());
    Ok(())
}

/// 创建脚本元数据文件和hash文件
fn create_script_metadata(project_path: &Path, script_id: &str, script_type: &str, author: &str, email: &str) -> Result<()> {
    // 读取脚本文件内容来计算hash
    let script_name = format!("{}.{}", script_type, get_script_extension());
    let script_path = project_path.join(&script_name);
    
    if !script_path.exists() {
        anyhow::bail!("脚本文件不存在: {}", script_path.display());
    }
    
    let script_content = fs::read_to_string(&script_path)?;
    
    // 计算hash值
    let mut hasher = DefaultHasher::new();
    script_content.hash(&mut hasher);
    let hash_value = format!("{:x}", hasher.finish());
    
    // 创建hash文件 (hash.扩展名)
    let hash_filename = format!("{}.{}", hash_value, get_script_extension());
    let hash_file_path = project_path.join(&hash_filename);
    
    if hash_file_path.exists() {
        println!("  {} 文件 {} 已存在，跳过创建。", "[!]".yellow().bold(), hash_filename.cyan().bold());
    } else {
        fs::write(&hash_file_path, &script_content)?;
        println!("  ✅ 创建hash文件: {}", hash_file_path.display());
    }
    
    // 获取当前日期
    let current_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    
    // 创建元数据文件 meta.toml
    let meta_path = project_path.join("meta.toml");
    
    if meta_path.exists() {
        println!("  {} 文件 {} 已存在，跳过创建。", "[!]".yellow().bold(), "meta.toml".cyan().bold());
        return Ok(());
    }
    
    let username_id = format!("{}/{}", author.replace(" ", "_").to_lowercase(), script_id);
    
    let meta_content = format!(r#"# RMM Script Meta Configuration
# 脚本项目元数据文件

# 脚本索引格式："username/ID" = "hash"
[scripts]
"{username_id}" = "{hash_value}"

# 脚本元数据
[metadata."{username_id}"]
author = "{author}"
email = "{email}"
description = "RMM {script_type} 脚本"
type = "{script_type}"
version = "1.0.0"
hash = "{hash_value}"
extension = "{extension}"
created = "{current_date}"
updated = "{current_date}"
"#,
        username_id = username_id,
        hash_value = hash_value,
        author = author,
        email = email,
        script_type = script_type,
        extension = get_script_extension(),
        current_date = current_date
    );
    
    fs::write(&meta_path, meta_content)?;
    println!("  ✅ 创建元数据文件: {}", meta_path.display());
    println!("  📄 脚本索引: {} = {}", username_id, hash_value);
    
    Ok(())
}
