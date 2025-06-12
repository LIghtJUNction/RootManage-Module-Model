use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::Path;
use crate::commands::utils::core::config::RmmConfig;
use crate::commands::utils::core::common::{FileSystemManager, GitManager};
use crate::commands::utils::init_utils::*;

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

pub fn handle_init(config: &RmmConfig, matches: &ArgMatches) -> Result<String> {
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
    FileSystemManager::ensure_dir_exists(path)?;    // 检测 Git 信息
    let git_info_tuple = GitManager::get_git_info(path)
        .map(|git_info| (git_info.username, git_info.repo_name));
    
    // 使用RMM配置中的用户信息作为默认值
    let author_name = &config.username;
    let author_email = &config.email;
    
    // 创建项目配置
    let project_config = create_project_config(&project_name, author_name, author_email, &config.version, git_info_tuple)?;
    
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
        println!("📦 已创建基础项目结构");    }    // 创建基础文件
    create_basic_files(path, &project_name, author_name)?;
    
    // 创建 Rmake.toml
    create_rmake_toml(path, &project_name)?;
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
    
    Ok(format!("项目 {} 初始化成功", project_name))
}

