use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use crate::commands::utils::core::config::RmmConfig;
use crate::commands::utils::core::common::ProjectManager;
use crate::commands::utils::core::executor::CheckManager;

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
      // 获取项目根目录
    let current_dir = std::env::current_dir()?;
    let project_file_path = ProjectManager::find_project_file(&current_dir).ok();
    let project_root = project_file_path
        .as_ref()
        .and_then(|p| p.parent())
        .unwrap_or(&current_dir);
    
    // 基本项目检查
    result_output.push_str("📋 项目配置检查:\n");
    match CheckManager::check_project_config(project_root) {
        Ok(result) => result_output.push_str(&format!("{}\n", result)),
        Err(e) => result_output.push_str(&format!("❌ 项目配置错误: {}\n", e)),
    }
    
    // Shell 脚本语法检查 (默认启用)
    if !skip_shellcheck {
        result_output.push_str("\n🐚 Shell 脚本语法检查:\n");
        match CheckManager::check_shell_syntax(project_root) {
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
        match CheckManager::check_github_connection() {
            Ok(result) => result_output.push_str(&format!("{}\n", result)),
            Err(e) => result_output.push_str(&format!("❌ GitHub 连接错误: {}\n", e)),
        }
    }
    
    // 依赖检查
    if check_deps {
        result_output.push_str("\n📦 依赖检查:\n");
        match CheckManager::check_dependencies() {
            Ok(result) => result_output.push_str(&format!("{}\n", result)),
            Err(e) => result_output.push_str(&format!("❌ 依赖检查错误: {}\n", e)),
        }
    }
    
    // 项目结构检查
    result_output.push_str("\n📁 项目结构检查:\n");
    match CheckManager::check_project_structure(project_root) {
        Ok(result) => result_output.push_str(&format!("{}\n", result)),
        Err(e) => result_output.push_str(&format!("❌ 项目结构错误: {}\n", e)),
    }
    
    println!("✅ 检查完成！");
    result_output.push_str("\n✅ 检查完成！");
    
    Ok(result_output)
}
