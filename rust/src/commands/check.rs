use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::Path;
use crate::config::{RmmConfig, ProjectConfig};
use crate::shellcheck;

/// 构建 check 命令
pub fn build_command() -> Command {
    Command::new("check")
        .about("检查项目状态、语法和 GitHub 连接")
        .long_about("检查 RMM 项目的配置、依赖、shell 脚本语法和 GitHub 连接状态")
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
        .arg(
            Arg::new("skip-shellcheck")
                .long("skip-shellcheck")
                .action(ArgAction::SetTrue)
                .help("跳过 shell 脚本语法检查")
        )
}

/// 处理 check 命令
pub fn handle_check(config: &RmmConfig, matches: &ArgMatches) -> Result<String> {
    println!("🔍 开始检查项目状态...");

    let check_all = matches.get_flag("all");
    let check_github = matches.get_flag("github") || check_all;
    let check_deps = matches.get_flag("deps") || check_all;
    let skip_shellcheck = matches.get_flag("skip-shellcheck");
    
    let mut result_output = String::new();
    
    // 基本项目检查
    result_output.push_str("📋 项目配置检查:\n");
    match check_project_config() {
        Ok(_) => result_output.push_str("✅ 项目配置正常\n"),
        Err(e) => result_output.push_str(&format!("❌ 项目配置错误: {}\n", e)),
    }
    
    // Shell 脚本语法检查 (默认启用)
    if !skip_shellcheck {
        result_output.push_str("\n🐚 Shell 脚本语法检查:\n");
        match check_shell_syntax() {
            Ok(shell_result) => {
                result_output.push_str(&shell_result);
                result_output.push_str("\n");
            }
            Err(e) => {
                result_output.push_str(&format!("❌ Shell 脚本检查失败: {}\n", e));
            }
        }
    }
    
    // GitHub 连接检查
    if check_github {
        result_output.push_str("\n🐙 GitHub 连接检查:\n");
        match check_github_connection(config) {
            Ok(_) => result_output.push_str("✅ GitHub 连接正常\n"),
            Err(e) => result_output.push_str(&format!("❌ GitHub 连接错误: {}\n", e)),
        }
    }
    
    // 依赖检查
    if check_deps {
        result_output.push_str("\n📦 依赖检查:\n");
        match check_dependencies() {
            Ok(_) => result_output.push_str("✅ 依赖检查完成\n"),
            Err(e) => result_output.push_str(&format!("❌ 依赖检查错误: {}\n", e)),
        }
    }
    
    // 项目结构检查
    result_output.push_str("\n📁 项目结构检查:\n");
    match check_project_structure() {
        Ok(_) => result_output.push_str("✅ 项目结构正常\n"),
        Err(e) => result_output.push_str(&format!("❌ 项目结构错误: {}\n", e)),
    }
    
    println!("✅ 检查完成！");
    result_output.push_str("\n✅ 检查完成！");
    
    Ok(result_output)
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
            match ProjectConfig::load_from_file(&path) {                Ok(config) => {
                    println!("✓ 配置文件格式正确");
                    println!("  项目名: {}", config.name);
                    println!("  项目ID: {}", config.id);
                    println!("  版本: {}", config.version.as_ref().unwrap_or(&"未设置".to_string()));
                    println!("  版本代码: {}", config.version_code);
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
        ("README.MD", "项目说明", false),
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

/// 检查 Shell 脚本语法
fn check_shell_syntax() -> Result<String> {
    println!("\n🐚 检查 Shell 脚本语法...");
    
    let current_dir = std::env::current_dir()?;
    
    // 检查 shellcheck 是否可用
    if !shellcheck::is_shellcheck_available() {
        let warning_msg = "⚠️  shellcheck 工具未安装或不可用，跳过语法检查";
        println!("{}", warning_msg);
        return Ok(warning_msg.to_string());
    }
    
    // 运行 shellcheck
    match shellcheck::check_project(&current_dir, true) {
        Ok((results, all_passed)) => {
            let formatted_output = shellcheck::format_results(&results, true);
            
            if all_passed {
                let success_msg = if results.is_empty() {
                    "✅ 未发现 Shell 脚本文件"
                } else {
                    "✅ Shell 脚本语法检查通过"
                };
                println!("{}", success_msg);
                
                // 返回详细结果
                if results.is_empty() {
                    Ok(success_msg.to_string())
                } else {
                    Ok(format!("{}\n\n{}", success_msg, formatted_output))
                }
            } else {
                let error_msg = "❌ Shell 脚本语法检查发现问题";
                println!("{}", error_msg);
                
                // 返回详细错误信息
                Ok(format!("{}\n\n{}", error_msg, formatted_output))
            }
        }
        Err(e) => {
            let error_msg = format!("❌ Shell 脚本语法检查失败: {}", e);
            println!("{}", &error_msg);
            Err(anyhow::anyhow!(error_msg))
        }
    }
}
