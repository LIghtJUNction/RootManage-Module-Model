use clap::{Arg, ArgMatches, Command};
use anyhow::Result;
use crate::utils::{Context, Config, RmmProject};

pub fn config_command() -> Command {
    Command::new("config")
        .about("Pyrmm Config Command group")
        .subcommand(
            Command::new("ls")
                .about("List all configuration keys")
                .arg(
                    Arg::new("project_name")
                        .help("项目名称 (可选)")
                        .value_name("PROJECT_NAME")
                        .required(false)
                )
        )
        .subcommand(
            Command::new("set")
                .about("Set configuration value")
                .arg(
                    Arg::new("key")
                        .help("配置键")
                        .value_name("KEY")
                        .required(true)
                )
                .arg(
                    Arg::new("value")
                        .help("配置值")
                        .value_name("VALUE")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("delete")
                .about("Delete configuration key")
                .arg(
                    Arg::new("key")
                        .help("要删除的配置键")
                        .value_name("KEY")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("get")
                .about("Get configuration value")
                .arg(
                    Arg::new("key")
                        .help("配置键")
                        .value_name("KEY")
                        .required(true)
                )
        )
}

pub fn handle_config(ctx: &Context, matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("ls", sub_matches)) => handle_config_ls(ctx, sub_matches),
        Some(("set", sub_matches)) => handle_config_set(ctx, sub_matches),
        Some(("delete", sub_matches)) => handle_config_delete(ctx, sub_matches),
        Some(("get", sub_matches)) => handle_config_get(ctx, sub_matches),
        _ => {
            ctx.error("❌ 请指定子命令: ls, set, delete, get");
            Ok(())
        }
    }
}

fn handle_config_ls(ctx: &Context, matches: &ArgMatches) -> Result<()> {
    let project_name = matches.get_one::<String>("project_name");

    if let Some(project_name) = project_name {
        // 显示特定项目的配置
        show_project_config(ctx, project_name)?;
    } else {
        // 显示系统配置
        show_system_config(ctx)?;
    }

    Ok(())
}

fn handle_config_set(ctx: &Context, matches: &ArgMatches) -> Result<()> {
    let key = matches.get_one::<String>("key").unwrap();
    let value = matches.get_one::<String>("value").unwrap();

    let mut config = Config::load()?;
    config.set(key.clone(), value.clone());
    config.save()?;

    ctx.info(&format!("✅ 配置已设置: {} = {}", key, value));

    Ok(())
}

fn handle_config_delete(ctx: &Context, matches: &ArgMatches) -> Result<()> {
    let key = matches.get_one::<String>("key").unwrap();

    let mut config = Config::load()?;
    
    if let Some(old_value) = config.remove(key) {
        config.save()?;
        ctx.info(&format!("✅ 配置已删除: {} (原值: {})", key, old_value));
    } else {
        ctx.warn(&format!("⚠️  配置键不存在: {}", key));
    }

    Ok(())
}

fn handle_config_get(ctx: &Context, matches: &ArgMatches) -> Result<()> {
    let key = matches.get_one::<String>("key").unwrap();

    let config = Config::load()?;
    
    if let Some(value) = config.get(key) {
        println!("{}", value);
    } else {
        ctx.error(&format!("❌ 配置键不存在: {}", key));
    }

    Ok(())
}

fn show_project_config(ctx: &Context, project_name: &str) -> Result<()> {
    // 查找项目路径
    let project_path = find_project_path(project_name)?;
    
    // 加载项目配置
    let rmm_toml = project_path.join("rmm.toml");
    if !rmm_toml.exists() {
        anyhow::bail!("❌ 项目 '{}' 不存在或无法访问", project_name);
    }

    let project = RmmProject::load_from_file(&rmm_toml)?;

    ctx.info(&format!("项目 '{}' 的配置信息:", project_name));
    println!("  name: {}", project.name);
    println!("  version: {}", project.version);
    
    if let Some(author) = &project.author {
        println!("  author: {}", author);
    }
    
    if let Some(description) = &project.description {
        println!("  description: {}", description);
    }
    
    if let Some(id) = &project.id {
        println!("  id: {}", id);
    }
    
    if let Some(version_code) = project.versionCode {
        println!("  versionCode: {}", version_code);
    }
    
    if let Some(update_json) = &project.updateJson {
        println!("  updateJson: {}", update_json);
    }
    
    if let Some(dependencies) = &project.dependencies {
        if !dependencies.is_empty() {
            println!("  dependencies:");
            for (name, version) in dependencies {
                println!("    {}: {}", name, version);
            }
        }
    }
    
    if let Some(build) = &project.build {
        if let Some(output) = &build.output {
            println!("  build.output: {}", output);
        }
        
        if let Some(scripts) = &build.scripts {
            if !scripts.is_empty() {
                println!("  build.scripts:");
                for (name, command) in scripts {
                    println!("    {}: {}", name, command);
                }
            }
        }
    }

    Ok(())
}

fn show_system_config(ctx: &Context) -> Result<()> {
    ctx.info("系统配置:");

    let config = Config::load().unwrap_or_default();
    let settings = config.list();

    if settings.is_empty() {
        println!("  (没有配置项)");
    } else {
        for (key, value) in settings {
            println!("  {}: {}", key, value);
        }
    }

    // 显示一些常见的系统信息
    println!("\n运行时信息:");
    println!("  rmm_version: {}", env!("CARGO_PKG_VERSION"));
    println!("  config_file: {}", Config::config_file_path()?.display());

    if let Ok(current_dir) = std::env::current_dir() {
        println!("  current_dir: {}", current_dir.display());
    }

    Ok(())
}

fn find_project_path(project_name: &str) -> Result<std::path::PathBuf> {
    use std::path::PathBuf;

    // 首先在当前目录查找
    let current_dir = std::env::current_dir()?;
    let project_path = current_dir.join(project_name);
    
    if project_path.exists() && project_path.join("rmm.toml").exists() {
        return Ok(project_path);
    }

    // 在配置的项目路径中查找
    let config = Config::load().unwrap_or_default();
    if let Some(projects_dir) = config.get("projects_dir") {
        let projects_dir = PathBuf::from(projects_dir);
        let project_path = projects_dir.join(project_name);
        
        if project_path.exists() && project_path.join("rmm.toml").exists() {
            return Ok(project_path);
        }
    }

    anyhow::bail!("❌ 项目 '{}' 不存在或无法访问", project_name);
}
