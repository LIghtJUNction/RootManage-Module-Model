use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;
use serde_json::json;
use crate::commands::utils::core::config::ProjectConfig;
use crate::commands::utils::core::rmake::RmakeConfig;

/// 确保目录存在，如果不存在则创建
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// 格式化文件大小为人类可读的格式
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: u64 = 1024;
    
    if size == 0 {
        return "0 B".to_string();
    }
    
    let mut size = size as f64;
    let mut unit_index = 0;
    
    while size >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD as f64;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// 获取 Git 信息（简化版）
pub fn get_git_info(_path: &Path) -> Option<(String, String)> {
    // 简化版本，返回默认值
    // 实际实现中可以调用 git 命令获取用户名和邮箱
    Some((
        "Unknown User".to_string(),
        "user@example.com".to_string(),
    ))
}

/// 检查 ADB 是否可用
pub fn check_adb_available() -> bool {
    // 简化版本，假设 ADB 可用
    // 实际实现中可以尝试运行 adb version 命令
    true
}

/// Git 仓库信息结构
#[derive(Debug, Clone)]
pub struct GitRepoInfo {
    pub username: String,
    pub repo_name: String,
    pub repo_root: std::path::PathBuf,
    pub is_in_repo_root: bool,
}

/// 检测Git仓库信息
pub fn detect_git_repo_info() -> Result<Option<GitRepoInfo>> {
    // 获取远程origin的URL
    let output = Command::new("git")
        .args(&["remote", "get-url", "origin"])
        .output()
        .map_err(|_| anyhow::anyhow!("无法执行git命令"))?;

    if !output.status.success() {
        return Ok(None);
    }

    let remote_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    // 解析GitHub URL格式: https://github.com/username/repo.git 或 git@github.com:username/repo.git
    let (username, repo_name) = if remote_url.starts_with("https://github.com/") {
        let path = remote_url.strip_prefix("https://github.com/").unwrap();
        let path = path.strip_suffix(".git").unwrap_or(path);
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            return Ok(None);
        }
    } else if remote_url.starts_with("git@github.com:") {
        let path = remote_url.strip_prefix("git@github.com:").unwrap();
        let path = path.strip_suffix(".git").unwrap_or(path);
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            return Ok(None);
        }
    } else {
        return Ok(None);
    };

    // 获取仓库根目录
    let repo_root_output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|_| anyhow::anyhow!("无法获取git仓库根目录"))?;

    if !repo_root_output.status.success() {
        return Ok(None);
    }

    let repo_root = std::path::PathBuf::from(String::from_utf8_lossy(&repo_root_output.stdout).trim());
    let current_dir = std::env::current_dir()?;
    let is_in_repo_root = current_dir == repo_root;

    Ok(Some(GitRepoInfo {
        username,
        repo_name,
        repo_root,
        is_in_repo_root,
    }))
}

/// 生成 update.json 文件
pub async fn generate_update_json(
    config: &ProjectConfig,
    project_root: &Path,
    _rmake_config: Option<&RmakeConfig>,
) -> Result<()> {
    // 检测 Git 仓库信息
    let git_info = match detect_git_repo_info()? {
        Some(info) => info,
        None => {
            println!("⚠️  未检测到 Git 仓库，跳过 update.json 生成");
            return Ok(());
        }
    };

    println!("📁 检测到 Git 仓库: {}/{}", git_info.username, git_info.repo_name);

    // 构建基础 URL
    let base_path = if git_info.is_in_repo_root {
        String::new()
    } else {
        // 计算相对路径
        let current_dir = std::env::current_dir()?;
        let relative_path = current_dir
            .strip_prefix(&git_info.repo_root)
            .map_err(|_| anyhow::anyhow!("无法计算相对路径"))?;
        format!("/{}", relative_path.to_string_lossy().replace('\\', "/"))
    };    let zip_filename = format!("{}-{}.zip", config.id, config.version_code);
    let changelog_filename = "CHANGELOG.MD";

    // 构建 URL - ZIP文件在 .rmmp/dist/ 目录中
    let zip_url = format!(
        "https://raw.githubusercontent.com/{}/{}/main{}/.rmmp/dist/{}",
        git_info.username, git_info.repo_name, base_path, zip_filename
    );

    // CHANGELOG.MD 在项目根目录
    let changelog_url = format!(
        "https://raw.githubusercontent.com/{}/{}/main{}/{}",
        git_info.username, git_info.repo_name, base_path, changelog_filename
    );

    // 创建 update.json 内容
    let update_json = json!({
        "versionCode": config.version_code.parse::<u32>().unwrap_or(1),
        "version": config.version.clone(),
        "zipUrl": zip_url,
        "changelog": changelog_url
    });

    // 写入 update.json 文件
    let update_json_path = project_root.join("update.json");
    let content = serde_json::to_string_pretty(&update_json)?;
    std::fs::write(&update_json_path, content)?;

    println!("📄 生成 update.json: {}", update_json_path.display());
    println!("🔗 模块下载链接: {}", zip_url);

    Ok(())
}

/// 查找或创建项目配置
pub fn find_or_create_project_config(start_dir: &Path) -> Result<ProjectConfig> {
    // 先尝试查找现有配置
    if let Ok(config_path) = find_project_file(start_dir) {
        return ProjectConfig::load_from_file(&config_path);
    }
    
    // 如果找不到，创建默认配置
    let _project_name = start_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unnamed_project");
    
    anyhow::bail!("未找到 rmmproject.toml 配置文件，请先运行 'rmm init' 初始化项目")
}

/// 查找项目配置文件
pub fn find_project_file(start_dir: &Path) -> Result<std::path::PathBuf> {
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
    
    anyhow::bail!("未找到 rmmproject.toml 配置文件")
}

/// 检查是否是 RMM 项目
pub fn is_rmm_project(path: &Path) -> bool {
    path.join("rmmproject.toml").exists()
}

/// 获取 RMM 根目录
pub fn get_rmm_root() -> Result<std::path::PathBuf> {
    use std::env;
    
    // 1. 首先检查环境变量 RMM_ROOT
    if let Ok(rmm_root) = env::var("RMM_ROOT") {
        let path = std::path::PathBuf::from(rmm_root);
        if path.exists() {
            return Ok(path);
        }
    }
    
    // 2. 使用默认路径
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    
    let rmm_root = std::path::PathBuf::from(home).join("data").join("adb").join(".rmm");
    
    // 确保目录存在
    if !rmm_root.exists() {
        fs::create_dir_all(&rmm_root)?;
    }
    
    Ok(rmm_root)
}

/// 获取 Git 用户信息
pub fn get_git_user_info() -> (String, String) {
    let name = std::process::Command::new("git")
        .args(&["config", "user.name"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "username".to_string());
    
    let email = std::process::Command::new("git")
        .args(&["config", "user.email"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "email@example.com".to_string());
    
    (name, email)
}