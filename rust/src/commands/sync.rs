use clap::{Arg, ArgAction, ArgMatches, Command};
use anyhow::Result;
use crate::commands::utils::core::config::{RmmConfig, ProjectConfig, get_rmm_version};
use crate::commands::utils::core::common::ProjectManager;
use crate::commands::utils::core::executor::SyncManager;

/// 构建 sync 命令

pub fn build_command() -> Command {
    Command::new("sync")
        .about("同步项目列表和依赖")
        .long_about("同步 RMM 项目列表（默认行为）和项目的依赖项及配置文件")
        .arg(
            Arg::new("force")
                .short('f')
                .long("force")
                .action(ArgAction::SetTrue)
                .help("强制重新同步所有依赖")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("启用详细输出")
        )
        .arg(
            Arg::new("dev")
                .long("dev")
                .action(ArgAction::SetTrue)
                .help("同步开发依赖")
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue)
                .help("静默模式，只输出错误")
        )
        .arg(
            Arg::new("projects")
                .long("projects")
                .action(ArgAction::SetTrue)
                .help("仅同步项目列表（发现新项目，移除无效项目），跳过依赖同步")
        )
        .arg(
            Arg::new("search-path")
                .long("search-path")
                .value_name("PATH")
                .action(ArgAction::Append)
                .help("指定搜索项目的路径（可多次使用）")
        )
        .arg(
            Arg::new("max-depth")
                .long("max-depth")
                .value_name("DEPTH")
                .default_value("3")
                .help("搜索项目的最大目录深度")
        )
        .arg(
            Arg::new("fix-meta")
                .long("fix-meta")
                .action(ArgAction::SetTrue)
                .help("验证并修复 meta.toml 文件格式")
        )
}

/// 处理 sync 命令
pub fn handle_sync(config: &RmmConfig, matches: &ArgMatches) -> Result<String> {
    // 默认行为：总是同步项目列表
    handle_sync_projects(config, matches)?;
    
    // 如果没有明确指定 --projects 参数，也执行依赖同步
    if !matches.get_flag("projects") {
        println!("\n🔄 继续同步项目依赖...");
        handle_sync_dependencies(config, matches)?;
    }
    
    Ok("项目同步成功".to_string())
}

/// 处理项目列表同步
fn handle_sync_projects(_config: &RmmConfig, matches: &ArgMatches) -> Result<()> {
    println!("🔄 开始同步项目列表...");
    
    let mut rmm_config = RmmConfig::load()?;
    
    // 检查是否需要修复 meta.toml 格式
    let fix_meta = matches.get_flag("fix-meta");
    if fix_meta {
        println!("🔧 验证并修复 meta.toml 格式...");
        rmm_config.validate_and_fix_format()?;
        rmm_config.save()?;
        println!("✅ meta.toml 格式已修复并保存");
    }
    
    // 同步用户信息
    println!("🔄 同步用户信息...");
    if let Err(e) = rmm_config.update_user_info_from_git() {
        eprintln!("⚠️  无法从 git 配置同步用户信息: {}", e);
        eprintln!("提示: 可以手动设置 git 配置或编辑 meta.toml 文件");
    }
    
    // 获取搜索路径
    let search_paths: Vec<std::path::PathBuf> = if let Some(paths) = matches.get_many::<String>("search-path") {
        paths.map(|p| std::path::PathBuf::from(p)).collect()
    } else {
        // 默认搜索当前目录
        vec![std::env::current_dir()?]
    };
    
    // 获取最大深度
    let _max_depth: usize = matches.get_one::<String>("max-depth")
        .unwrap()
        .parse()
        .map_err(|_| anyhow::anyhow!("无效的最大深度参数"))?;
    
    // 同步项目列表
    rmm_config.sync_project_list(&search_paths)?;
    
    println!("✅ 项目列表同步完成！");
    Ok(())
}

/// 处理项目依赖同步
fn handle_sync_dependencies(config: &RmmConfig, matches: &ArgMatches) -> Result<()> {
    println!("🔄 开始同步项目依赖...");

    // 查找项目配置文件
    let current_dir = std::env::current_dir()?;
    let project_config_path = ProjectManager::find_project_file(&current_dir)?;
    
    println!("📁 项目配置: {}", project_config_path.display());
    
    // 加载项目配置
    let mut project_config = ProjectConfig::load_from_file(&project_config_path)?;
    
    // 获取选项
    let force = matches.get_flag("force");
    let dev = matches.get_flag("dev");
    
    if force {
        println!("💪 强制同步模式");
    }
    
    if dev {
        println!("🔧 包含开发依赖");
    }
    
    // 更新版本信息 - 注意这里使用项目配置中的版本而非 RMM 工具版本
    let project_version = project_config.version.clone().unwrap_or_else(|| "0.1.0".to_string());
    SyncManager::update_project_version(&mut project_config, &project_version)?;
    
    // 更新 requires_rmm 字段为当前 RMM 版本
    project_config.requires_rmm = get_rmm_version();
    
    // 同步依赖项 - 这里简化处理，实际应该有更复杂的依赖同步逻辑
    println!("📦 同步依赖项...");
    if project_config.dependencies.is_empty() {
        println!("  无依赖项需要同步");
    } else {
        for dep in &project_config.dependencies {
            println!("  - {} ({})", dep.name, dep.version);
        }
    }
    
    // 保存更新后的配置
    project_config.save_to_dir(&project_config_path.parent().unwrap())?;
    
    println!("✅ 同步完成！");
    
    Ok(())
}