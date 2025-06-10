use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::Path;
use crate::config::{RmmConfig, ProjectConfig};

/// 构建 check 命令
pub fn build_command() -> Command {
    Command::new("check")
        .about("检查项目状态和 GitHub 连接")
        .long_about("检查 RMM 项目的配置、依赖和 GitHub 连接状态")
        .arg(
            Arg::new("all")
                .short('a')
                .long("all")
                .action(ArgAction::SetTrue)
                .help("执行所有检查")
        )
        .arg(
            Arg::new("github")
                .short('g')
                .long("github")
                .action(ArgAction::SetTrue)
                .help("检查 GitHub 连接")
        )
        .arg(
            Arg::new("deps")
                .short('d')
                .long("deps")
                .action(ArgAction::SetTrue)
                .help("检查依赖项")
        )
}

/// 处理 check 命令
pub fn handle_check(config: &RmmConfig, matches: &ArgMatches) -> Result<()> {
    println!("🔍 开始检查项目状态...");

    let check_all = matches.get_flag("all");
    let check_github = matches.get_flag("github") || check_all;
    let check_deps = matches.get_flag("deps") || check_all;
    
    // 基本项目检查
    check_project_config()?;
    
    // GitHub 连接检查
    if check_github {
        check_github_connection(config)?;
    }
    
    // 依赖检查
    if check_deps {
        check_dependencies()?;
    }
    
    // 项目结构检查
    check_project_structure()?;
    
    println!("✅ 检查完成！");
    
    Ok(())
}

/// 检查项目配置
fn check_project_config() -> Result<()> {
    println!("\n📋 检查项目配置...");
    
    let current_dir = std::env::current_dir()?;
    let config_path = find_project_config(&current_dir);
    
    match config_path {
        Ok(path) => {
            println!("✓ 找到项目配置: {}", path.display());
            
            // 尝试加载配置
            match ProjectConfig::load_from_file(&path) {
                Ok(config) => {
                    println!("✓ 配置文件格式正确");
                    println!("  项目名: {}", config.name);
                    println!("  项目ID: {}", config.id);
                    println!("  版本: {}", config.version_code);
                    println!("  作者: {}", config.authors.first().map(|a| a.name.as_str()).unwrap_or("未设置"));
                }
                Err(e) => {
                    println!("✗ 配置文件格式错误: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ {}", e);
        }
    }
    
    Ok(())
}

/// 检查 GitHub 连接
fn check_github_connection(_config: &RmmConfig) -> Result<()> {
    println!("\n🐙 检查 GitHub 连接...");
    
    // 检查 GitHub token
    if let Ok(token) = std::env::var("GITHUB_ACCESS_TOKEN") {
        if !token.is_empty() {
            println!("✓ 找到 GitHub Access Token");
            
            // 这里可以添加实际的 GitHub API 连接测试
            println!("  (GitHub API 连接测试需要实现)");
        } else {
            println!("⚠ GITHUB_ACCESS_TOKEN 环境变量为空");
        }
    } else {
        println!("⚠ 未设置 GITHUB_ACCESS_TOKEN 环境变量");
        println!("  提示: 设置此变量以启用 GitHub 功能");
    }
    
    // 检查 Git 仓库
    if Path::new(".git").exists() {
        println!("✓ 当前目录是 Git 仓库");
        
        // 检查远程仓库
        if let Ok(output) = std::process::Command::new("git")
            .args(&["remote", "get-url", "origin"])
            .output()
        {            if output.status.success() {
                let remote_url_bytes = String::from_utf8_lossy(&output.stdout);
                let remote_url = remote_url_bytes.trim();
                println!("✓ 远程仓库: {}", remote_url);
                
                if remote_url.contains("github.com") {
                    println!("✓ 这是一个 GitHub 仓库");
                } else {
                    println!("⚠ 这不是 GitHub 仓库");
                }
            }
        }
    } else {
        println!("⚠ 当前目录不是 Git 仓库");
    }
    
    Ok(())
}

/// 检查依赖项
fn check_dependencies() -> Result<()> {
    println!("\n📦 检查依赖项...");
    
    let current_dir = std::env::current_dir()?;
    if let Ok(config_path) = find_project_config(&current_dir) {
        if let Ok(config) = ProjectConfig::load_from_file(&config_path) {
            if config.dependencies.is_empty() {
                println!("ℹ 项目无依赖项");
            } else {
                println!("依赖项列表:");
                for dep in &config.dependencies {
                    println!("  - {} ({})", dep.name, dep.version);
                    
                    // 检查依赖是否存在
                    let dep_path = Path::new("deps").join(&dep.name);
                    if dep_path.exists() {
                        println!("    ✓ 已安装");
                    } else {
                        println!("    ✗ 未安装");
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// 检查项目结构
fn check_project_structure() -> Result<()> {
    println!("\n📁 检查项目结构...");
    
    let required_files = [
        ("module.prop", "模块属性文件", true),
        ("customize.sh", "安装脚本", false),
        ("system/", "系统文件目录", false),
        ("README.md", "项目说明", false),
        ("LICENSE", "许可证文件", false),
    ];
    
    for (file, description, required) in &required_files {
        let path = Path::new(file);
        if path.exists() {
            println!("✓ {}: {}", description, file);
        } else if *required {
            println!("✗ 缺少必需文件 {}: {}", description, file);
        } else {
            println!("⚠ 缺少可选文件 {}: {}", description, file);
        }
    }
    
    Ok(())
}

/// 查找项目配置文件
fn find_project_config(start_dir: &Path) -> Result<std::path::PathBuf> {
    let mut current = start_dir;
    
    loop {
        let config_path = current.join("rmmproject.toml");
        if config_path.exists() {
            return Ok(config_path);
        }
        
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }
    
    anyhow::bail!("未找到 rmmproject.toml 配置文件");
}
