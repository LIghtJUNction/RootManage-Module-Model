use clap::{Arg, ArgMatches, Command};
use anyhow::Result;
use std::path::PathBuf;
use crate::utils::{Context, RmmProject, Config};

pub fn sync_command() -> Command {
    Command::new("sync")
        .about("同步RMM项目")
        .arg(
            Arg::new("project_name")
                .help("要同步的项目名称 (可选，如果不指定则需要使用 --all 参数)")
                .value_name("PROJECT_NAME")
                .required(false)
        )
        .arg(
            Arg::new("update")
                .short('U')
                .long("update")
                .action(clap::ArgAction::SetTrue)
                .help("如果依赖有升级，将依赖更新到最新版本（包括rmm自己）")
        )
        .arg(
            Arg::new("all")
                .short('a')
                .long("all")
                .action(clap::ArgAction::SetTrue)
                .help("同步所有项目")
        )
        .arg(
            Arg::new("proxy")
                .long("proxy")
                .action(clap::ArgAction::SetTrue)
                .help("获取GitHub代理地址列表并保存到项目元数据")
        )
}

pub fn handle_sync(ctx: &Context, matches: &ArgMatches) -> Result<()> {
    let project_name = matches.get_one::<String>("project_name");
    let update = matches.get_flag("update");
    let sync_all = matches.get_flag("all");
    let proxy = matches.get_flag("proxy");

    if project_name.is_none() && !sync_all {
        anyhow::bail!("❌ 请指定项目名称或使用 --all 参数同步所有项目");
    }

    if sync_all {
        sync_all_projects(ctx, update, proxy)?;
    } else if let Some(name) = project_name {
        sync_single_project(ctx, name, update, proxy)?;
    }

    Ok(())
}

fn sync_all_projects(ctx: &Context, update: bool, proxy: bool) -> Result<()> {
    ctx.info("🔄 同步所有RMM项目...");

    // 查找所有RMM项目
    let projects = find_all_rmm_projects()?;
    
    if projects.is_empty() {
        ctx.warn("❌ 没有找到任何RMM项目");
        return Ok(());
    }

    ctx.info(&format!("📁 找到 {} 个项目:", projects.len()));
    for project_path in &projects {
        ctx.info(&format!("  - {}", project_path.display()));
    }

    for project_path in projects {
        let project_name = project_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        ctx.info(&format!("🔄 同步项目: {}", project_name));
        
        if let Err(e) = sync_project(ctx, &project_path, update, proxy) {
            ctx.error(&format!("❌ 项目 {} 同步失败: {}", project_name, e));
        } else {
            ctx.info(&format!("✅ 项目 {} 同步完成", project_name));
        }
    }

    ctx.info("✅ 所有项目同步完成!");

    Ok(())
}

fn sync_single_project(ctx: &Context, project_name: &str, update: bool, proxy: bool) -> Result<()> {
    ctx.info(&format!("🔄 同步项目: {}", project_name));

    // 查找项目路径
    let project_path = find_project_path(project_name)?;
    
    sync_project(ctx, &project_path, update, proxy)?;
    
    ctx.info(&format!("✅ 项目 {} 同步完成!", project_name));

    Ok(())
}

fn sync_project(ctx: &Context, project_path: &std::path::Path, update: bool, proxy: bool) -> Result<()> {
    // 检查是否是有效的RMM项目
    let rmm_toml = project_path.join("rmm.toml");
    if !rmm_toml.exists() {
        anyhow::bail!("❌ '{}' 不是一个有效的RMM项目", project_path.display());
    }

    // 加载项目配置
    let project = RmmProject::load_from_file(&rmm_toml)?;

    ctx.debug(&format!("📋 项目信息: {} v{}", project.name, project.version));

    // 处理代理更新
    if proxy {
        handle_proxy_update(ctx, project_path)?;
    }

    // 同步依赖
    if let Some(dependencies) = &project.dependencies {
        sync_dependencies(ctx, project_path, dependencies, update)?;
    } else {
        ctx.info("📦 项目没有依赖项");
    }

    // 更新项目元数据
    update_project_metadata(ctx, project_path, &project)?;

    Ok(())
}

fn handle_proxy_update(ctx: &Context, project_path: &std::path::Path) -> Result<()> {
    ctx.info("🌐 正在获取GitHub代理列表...");

    let proxy_list = fetch_github_proxy_list()?;
    
    if proxy_list.is_empty() {
        ctx.warn("⚠️  没有找到可用的代理地址");
        return Ok(());
    }

    ctx.info(&format!("🎯 找到 {} 个代理地址", proxy_list.len()));

    // 保存代理列表到项目元数据
    let metadata_dir = project_path.join(".rmmp");
    std::fs::create_dir_all(&metadata_dir)?;
    
    let proxy_file = metadata_dir.join("github_proxies.txt");
    let proxy_content = proxy_list.join("\n");
    std::fs::write(&proxy_file, proxy_content)?;

    ctx.info(&format!("💾 代理列表已保存到: {}", proxy_file.display()));

    Ok(())
}

fn fetch_github_proxy_list() -> Result<Vec<String>> {
    // 这里应该从实际的代理服务获取列表
    // 暂时返回一些常见的GitHub代理地址作为示例
    Ok(vec![
        "https://ghproxy.com/".to_string(),
        "https://github.com.cnpmjs.org/".to_string(),
        "https://hub.fastgit.xyz/".to_string(),
        "https://github.bajins.com/".to_string(),
    ])
}

fn sync_dependencies(
    ctx: &Context,
    project_path: &std::path::Path,
    dependencies: &std::collections::HashMap<String, String>,
    update: bool,
) -> Result<()> {
    if dependencies.is_empty() {
        return Ok(());
    }

    ctx.info(&format!("📦 同步 {} 个依赖项...", dependencies.len()));

    let deps_dir = project_path.join(".rmmp").join("deps");
    std::fs::create_dir_all(&deps_dir)?;

    for (dep_name, dep_version) in dependencies {
        ctx.info(&format!("  📥 同步依赖: {} v{}", dep_name, dep_version));

        let dep_path = deps_dir.join(dep_name);
        
        if dep_path.exists() && !update {
            ctx.debug(&format!("    ⏭️  依赖 {} 已存在，跳过", dep_name));
            continue;
        }

        // 下载或更新依赖
        download_dependency(ctx, dep_name, dep_version, &dep_path)?;
        
        ctx.info(&format!("    ✅ 依赖 {} 同步完成", dep_name));
    }

    Ok(())
}

fn download_dependency(
    ctx: &Context,
    dep_name: &str,
    dep_version: &str,
    dep_path: &std::path::Path,
) -> Result<()> {
    // 这里应该实现实际的依赖下载逻辑
    // 可以从GitHub、本地仓库或其他源下载
    
    ctx.debug(&format!("正在下载依赖: {} v{}", dep_name, dep_version));
    
    // 创建依赖目录
    std::fs::create_dir_all(dep_path)?;
    
    // 创建一个简单的标记文件表示依赖已下载
    let version_file = dep_path.join(".version");
    std::fs::write(version_file, dep_version)?;
    
    // 这里可以添加实际的下载逻辑，比如：
    // - 从GitHub下载release
    // - 从本地仓库复制
    // - 从包管理器安装
    
    Ok(())
}

fn update_project_metadata(ctx: &Context, project_path: &std::path::Path, project: &RmmProject) -> Result<()> {
    let metadata_dir = project_path.join(".rmmp");
    std::fs::create_dir_all(&metadata_dir)?;

    // 保存项目信息快照
    let metadata = serde_json::json!({
        "name": project.name,
        "version": project.version,
        "sync_time": chrono::Utc::now().to_rfc3339(),
        "rmm_version": env!("CARGO_PKG_VERSION")
    });

    let metadata_file = metadata_dir.join("sync_info.json");
    std::fs::write(&metadata_file, serde_json::to_string_pretty(&metadata)?)?;

    ctx.debug(&format!("📋 项目元数据已更新: {}", metadata_file.display()));

    Ok(())
}

fn find_all_rmm_projects() -> Result<Vec<PathBuf>> {
    let mut projects = Vec::new();
    let current_dir = std::env::current_dir()?;

    // 递归搜索当前目录及子目录中的rmm.toml文件
    find_rmm_projects_recursive(&current_dir, &mut projects, 0, 3)?; // 限制搜索深度为3

    Ok(projects)
}

fn find_rmm_projects_recursive(
    dir: &std::path::Path,
    projects: &mut Vec<PathBuf>,
    current_depth: usize,
    max_depth: usize,
) -> Result<()> {
    if current_depth > max_depth {
        return Ok(());
    }

    // 检查当前目录是否包含rmm.toml
    let rmm_toml = dir.join("rmm.toml");
    if rmm_toml.exists() {
        projects.push(dir.to_path_buf());
        return Ok(()); // 找到项目后不再搜索子目录
    }

    // 搜索子目录
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() && !path.file_name().unwrap().to_str().unwrap().starts_with('.') {
                    find_rmm_projects_recursive(&path, projects, current_depth + 1, max_depth)?;
                }
            }
        }
    }

    Ok(())
}

fn find_project_path(project_name: &str) -> Result<PathBuf> {
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

    // 在所有已知项目中查找
    let all_projects = find_all_rmm_projects()?;
    for project_path in all_projects {
        if project_path.file_name().and_then(|name| name.to_str()) == Some(project_name) {
            return Ok(project_path);
        }
    }

    anyhow::bail!("❌ 找不到项目: {}", project_name);
}
