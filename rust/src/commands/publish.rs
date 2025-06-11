use clap::{Arg, ArgMatches, Command};
use anyhow::Result;
use crate::config::{RmmConfig, ProjectConfig, RmakeConfig};
use std::path::Path;
use std::process::Command as StdCommand;
use serde_json::json;

pub fn build_command() -> Command {
    Command::new("publish")
        .about("发布模块到 GitHub Release")
        .arg(
            Arg::new("token")
                .long("token")
                .help("GitHub Personal Access Token")
                .value_name("TOKEN")
        )
        .arg(
            Arg::new("draft")
                .long("draft")
                .help("创建草稿发布")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("prerelease")
                .long("prerelease")
                .help("标记为预发布版本")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("message")
                .short('m')
                .long("message")
                .help("自定义发布说明")
                .value_name("MESSAGE")
        )
}

pub fn handle_publish(_config: &RmmConfig, matches: &ArgMatches) -> Result<()> {
    println!("🚀 准备发布模块到 GitHub...");
    
    // 检查 GitHub token (优先级: --token > GITHUB_ACCESS_TOKEN > GITHUB_TOKEN)
    let github_token = matches.get_one::<String>("token")
        .map(|s| s.clone())
        .or_else(|| std::env::var("GITHUB_ACCESS_TOKEN").ok())
        .or_else(|| std::env::var("GITHUB_TOKEN").ok());
    
    if github_token.is_none() {
        anyhow::bail!(
            "❌ 未找到 GitHub Token\n请通过以下方式之一设置：\n  1. 使用 --token 参数: rmm publish --token your_token_here\n  2. 设置环境变量: export GITHUB_ACCESS_TOKEN=your_token_here\n  3. 设置环境变量: export GITHUB_TOKEN=your_token_here"
        );
    }
    
    // 设置环境变量供 Python 脚本使用
    if let Some(token) = github_token {
        std::env::set_var("GITHUB_TOKEN", &token);
    }
      // 查找项目配置文件
    let current_dir = std::env::current_dir()?;
    let project_config_path = crate::config::find_project_file(&current_dir)?;
      // 加载项目配置
    let project_config = ProjectConfig::load_from_file(&project_config_path)?;
      // 加载 Rmake 配置（如果存在）
    let project_root = project_config_path.parent().unwrap();
    let rmake_config = RmakeConfig::load_from_dir(&project_root)?;
      // 获取版本信息（从项目配置中读取，而不是重新生成）
    let version = project_config.version.clone()
        .unwrap_or_else(|| "v0.1.0".to_string());
    let version_code = project_config.version_code.clone();
      // 获取 Git 仓库信息
    let git_info = crate::utils::get_git_info(&project_root)
        .ok_or_else(|| anyhow::anyhow!("无法获取 Git 仓库信息"))?;
    let repo_name = format!("{}/{}", git_info.username, git_info.repo_name);
    
    // 构建输出路径
    let dist_dir = project_root.join(".rmmp").join("dist");
      // 查找生成的文件
    let zip_filename = if let Some(ref rmake) = rmake_config {
        rmake.package.as_ref()
            .and_then(|p| p.zip_name.as_ref())
            .map(|name| format!("{}.zip", name))
            .unwrap_or_else(|| format!("{}.zip", project_config.id))
    } else {
        format!("{}.zip", project_config.id)
    };
    
    let source_filename = format!("{}-{}-source.tar.gz", project_config.id, version_code);
    
    let module_zip_path = dist_dir.join(&zip_filename);
    let source_tar_path = dist_dir.join(&source_filename);
    
    // 检查文件是否存在
    if !module_zip_path.exists() {
        anyhow::bail!("❌ 模块包不存在: {}\n请先运行 'rmm build' 构建项目", module_zip_path.display());
    }
    
    if !source_tar_path.exists() {
        anyhow::bail!("❌ 源码包不存在: {}\n请先运行 'rmm build' 构建项目", source_tar_path.display());
    }
    
    // 读取 CHANGELOG 作为 release body
    let changelog_path = project_root.join("CHANGELOG.md");
    let release_body = if changelog_path.exists() {
        std::fs::read_to_string(&changelog_path).unwrap_or_else(|_| {
            format!("## {} 发布说明\n\n此版本包含最新的功能更新和修复。", version)
        })
    } else {
        format!("## {} 发布说明\n\n此版本包含最新的功能更新和修复。", version)
    };
    
    // 自定义发布说明
    let final_release_body = if let Some(custom_message) = matches.get_one::<String>("message") {
        format!("{}\n\n---\n\n{}", custom_message, release_body)
    } else {
        release_body
    };
    
    // 检查是否启用代理功能
    let enable_proxy = rmake_config.as_ref()
        .and_then(|r| r.proxy.as_ref())
        .map(|p| p.enabled)
        .unwrap_or(false);
    
    // 准备发布配置
    let config_data = json!({
        "repo_name": repo_name,
        "version": version,
        "release_name": format!("{} - {}", project_config.name, version),
        "release_body": final_release_body,
        "module_zip_path": module_zip_path.to_string_lossy(),
        "source_tar_path": source_tar_path.to_string_lossy(),
        "enable_proxy": enable_proxy,
        "draft": matches.get_flag("draft"),
        "prerelease": matches.get_flag("prerelease")
    });
    
    println!("📦 版本: {}", version);
    println!("📁 仓库: {}", repo_name);
    println!("📄 模块包: {}", zip_filename);
    println!("📋 源码包: {}", source_filename);
    
    if enable_proxy {
        println!("🚀 已启用代理加速链接");
    }
    
    if matches.get_flag("draft") {
        println!("📝 模式: 草稿发布");
    }
    
    if matches.get_flag("prerelease") {
        println!("🧪 模式: 预发布版本");
    }
    
    // 查找 Python 发布脚本
    let publisher_script = find_publisher_script(&project_root)?;
    
    // 调用 Python 发布脚本
    println!("🔄 正在发布...");
    let output = StdCommand::new("python")
        .arg(&publisher_script)
        .arg(&config_data.to_string())
        .output()?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            print!("{}", stdout);
        }
        println!("✅ 发布完成！");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        if !stdout.trim().is_empty() {
            print!("{}", stdout);
        }
        
        if !stderr.trim().is_empty() {
            anyhow::bail!("发布失败: {}", stderr);
        } else {
            anyhow::bail!("发布失败，原因未知");
        }
    }
}

/// 查找 Python 发布脚本路径
fn find_publisher_script(project_root: &Path) -> Result<std::path::PathBuf> {
    // 搜索路径列表
    let search_paths = [
        project_root.join("src").join("pyrmm").join("publisher.py"),
        project_root.parent().unwrap().join("src").join("pyrmm").join("publisher.py"),
        std::env::current_dir()?.join("src").join("pyrmm").join("publisher.py"),
        std::env::current_dir()?.parent().unwrap().join("src").join("pyrmm").join("publisher.py"),
    ];
    
    for path in &search_paths {
        if path.exists() {
            return Ok(path.clone());
        }
    }
    
    anyhow::bail!("未找到 Python 发布脚本 publisher.py");
}
