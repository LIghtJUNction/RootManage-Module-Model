use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::Path;
use crate::config::{RmmConfig, ProjectConfig};

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
pub fn handle_sync(config: &RmmConfig, matches: &ArgMatches) -> Result<()> {
    // 默认行为：总是同步项目列表
    handle_sync_projects(config, matches)?;
    
    // 如果没有明确指定 --projects 参数，也执行依赖同步
    if !matches.get_flag("projects") {
        println!("\n🔄 继续同步项目依赖...");
        handle_sync_dependencies(config, matches)?;
    }
    
    Ok(())
}

/// 处理项目列表同步
fn handle_sync_projects(_config: &RmmConfig, matches: &ArgMatches) -> Result<()> {
    println!("🔄 开始同步项目列表...");
    
    let mut rmm_config = RmmConfig::load()?;
    
    // 检查是否需要修复 meta.toml 格式
    let fix_meta = matches.get_flag("fix-meta");
    if fix_meta {
        println!("🔧 验证并修复 meta.toml 格式...");
        let fixed = rmm_config.validate_and_fix_format()?;
        if fixed {
            rmm_config.save()?;
            println!("✅ meta.toml 格式已修复并保存");
        } else {
            println!("✅ meta.toml 格式正常，无需修复");
        }
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
    let max_depth: usize = matches.get_one::<String>("max-depth")
        .unwrap()
        .parse()
        .map_err(|_| anyhow::anyhow!("无效的最大深度参数"))?;
      // 同步项目列表
    rmm_config.sync_project_list(&search_paths, max_depth)?;
    
    println!("✅ 项目列表同步完成！");
    Ok(())
}

/// 处理项目依赖同步
fn handle_sync_dependencies(config: &RmmConfig, matches: &ArgMatches) -> Result<()> {
    println!("🔄 开始同步项目依赖...");

    // 查找项目配置文件
    let current_dir = std::env::current_dir()?;
    let project_config_path = find_project_config(&current_dir)?;
    
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
    
    // 更新版本信息
    crate::utils::update_project_version(&mut project_config)?;
    
    // 同步RMM版本信息
    sync_rmm_metadata(config, &mut project_config)?;
    
    // 同步依赖
    sync_dependencies(&project_config, force, dev)?;
    
    // 保存更新后的配置
    project_config.save_to_dir(&project_config_path.parent().unwrap())?;
    
    println!("✅ 同步完成！");
    
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
    
    anyhow::bail!("未找到 rmmproject.toml 配置文件。请确保在 RMM 项目根目录中运行此命令。");
}

/// 同步RMM元数据
fn sync_rmm_metadata(config: &RmmConfig, project_config: &mut ProjectConfig) -> Result<()> {
    println!("📋 同步RMM元数据...");
    
    // 更新requires_rmm版本
    let old_version = project_config.requires_rmm.clone();
    project_config.requires_rmm = config.version.clone();
    
    if old_version != project_config.requires_rmm {
        println!("🔄 更新RMM版本要求: {} -> {}", old_version, project_config.requires_rmm);
    } else {
        println!("✅ RMM版本要求已是最新: {}", project_config.requires_rmm);
    }
    
    // 将当前项目添加到全局 meta.toml 的项目列表中
    let mut rmm_config = RmmConfig::load()?;
    let current_dir = std::env::current_dir()?;
    
    // 使用新的方法添加当前项目
    rmm_config.add_current_project(&project_config.id, &current_dir)?;
    
    Ok(())
}

/// 同步依赖
fn sync_dependencies(config: &ProjectConfig, _force: bool, _include_dev: bool) -> Result<()> {
    println!("📦 同步依赖项...");
    
    // 显示当前依赖
    if !config.dependencies.is_empty() {
        println!("依赖项:");
        for dep in &config.dependencies {
            println!("  - {} ({})", dep.name, dep.version);
        }
    } else {
        println!("  无依赖项");
    }
    
    Ok(())
}
