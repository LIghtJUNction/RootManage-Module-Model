use clap::{Arg, ArgAction, ArgMatches, Command};
use anyhow::Result;
use crate::commands::utils::core::config::{RmmConfig, ProjectConfig, get_rmm_version};
use crate::commands::utils::core::common::{ProjectManager, CommandExecutor};
use crate::commands::utils::core::executor::{ProjectBuilder, SyncManager};
use std::path::Path;

/// 构建 build 命令
pub fn build_command() -> Command {
    Command::new("build")
        .about("构建 RMM 项目")
        .long_about("构建当前 RMM 项目，生成可安装的模块包")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("PATH")
                .help("输出目录路径")
        )
        .arg(
            Arg::new("clean")
                .short('c')
                .long("clean")
                .action(ArgAction::SetTrue)
                .help("构建前清理输出目录")
        )
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .action(ArgAction::SetTrue)
                .help("启用调试模式构建")
        )
        .arg(
            Arg::new("skip-shellcheck")
                .long("skip-shellcheck")
                .action(ArgAction::SetTrue)
                .help("跳过 shellcheck 语法检查")
        )
        .arg(
            Arg::new("script")
                .help("要运行的脚本名称（定义在 Rmake.toml 的 [scripts] 中）")
                .value_name("SCRIPT_NAME")
        )
}

/// 处理 build 命令
pub fn handle_build(_config: &RmmConfig, matches: &ArgMatches) -> Result<String, anyhow::Error> {
    // 查找项目配置文件
    let current_dir = std::env::current_dir()?;
    let project_config_path = ProjectManager::find_project_file(&current_dir)?;
    let project_root = project_config_path.parent().unwrap();
      // 检查是否要运行脚本
    if let Some(script_name) = matches.get_one::<String>("script") {
        return run_script(&project_root, script_name);
    }
    
    println!("🔨 开始构建 RMM 项目...");    println!("📁 项目配置: {}", project_config_path.display());
    // 加载项目配置
    let mut project_config = ProjectConfig::load_from_file(&project_config_path)?;
    
    // 更新版本信息
    let rmm_version = get_rmm_version(); // Get current RMM version
    SyncManager::update_project_version(&mut project_config, &rmm_version)?; // Pass rmm_version
    
    // 保存更新后的配置
    project_config.save_to_dir(&project_config_path.parent().unwrap())?;
      // 获取选项
    let output_dir = matches.get_one::<String>("output");
    let clean = matches.get_flag("clean");
    let debug = matches.get_flag("debug");
    let skip_shellcheck = matches.get_flag("skip-shellcheck");
    
    if debug {
        println!("🐛 调试模式已启用");
    }
    
    if skip_shellcheck {
        println!("⚠️  已跳过 shellcheck 检查");
    }// 确定输出目录 - 默认使用 .rmmp/dist，不复制到用户目录
    let build_output = if let Some(output) = output_dir {
        Path::new(output).to_path_buf()
    } else {
        current_dir.join(".rmmp").join("dist")
    };
    
    if clean && build_output.exists() {
        println!("🧹 清理输出目录: {}", build_output.display());
        std::fs::remove_dir_all(&build_output)?;
    }
      // 创建输出目录
    std::fs::create_dir_all(&build_output)?;
    
    // 构建项目
    ProjectBuilder::build_project(&project_config, &build_output, output_dir, debug, skip_shellcheck)?;
    
    println!("✅ 构建完成！输出目录: {}", build_output.display());
    
    Ok("项目构建成功".to_string())
}

/// 运行 Rmake 脚本
fn run_script(project_root: &Path, script_name: &str) -> Result<String> {
    println!("🔧 运行脚本: {}", script_name);
      // 这里可以加载 Rmake.toml 并执行指定的脚本
    // 暂时使用简单的实现
    let command = format!("echo 'Running script: {}'", script_name);
    CommandExecutor::execute_script_command(&command, project_root)?;
    
    Ok(format!("脚本 {} 执行完成", script_name))
}

