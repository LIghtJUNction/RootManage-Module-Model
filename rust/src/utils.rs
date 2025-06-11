use anyhow::{Result, anyhow};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// 获取 RMM 根目录
pub fn get_rmm_root() -> Result<PathBuf> {
    // 优先使用环境变量
    if let Ok(rmm_root) = env::var("RMM_ROOT") {
        let path = PathBuf::from(rmm_root);
        return Ok(path);
    }
    
    // 检查是否在Android环境（/data/adb 存在）
    let android_path = PathBuf::from("/data/adb/.rmm");
    if android_path.parent().map(|p| p.exists()).unwrap_or(false) {
        return Ok(android_path);
    }
    
    // Windows 用户目录 - 使用 ~/data/adb/.rmm
    if let Ok(userprofile) = env::var("USERPROFILE") {
        let win_path = PathBuf::from(userprofile).join("data").join("adb").join(".rmm");
        return Ok(win_path);
    }
    
    // Unix/Linux 用户主目录 - 使用 ~/data/adb/.rmm
    if let Ok(home) = env::var("HOME") {
        let home_path = PathBuf::from(home).join("data").join("adb").join(".rmm");
        return Ok(home_path);
    }
    
    // 最后的备选方案：当前目录
    let current_dir = env::current_dir()?;
    Ok(current_dir.join("data").join("adb").join(".rmm"))
}

/// 设置日志记录
pub fn setup_logging() -> Result<()> {
    // 简单的日志设置，可以根据需要扩展
    env::set_var("RUST_LOG", "info");
    Ok(())
}

/// 获取动态版本号
pub fn get_dynamic_version() -> String {
    // 1. 尝试从环境变量获取
    if let Ok(version) = env::var("RMM_VERSION") {
        return version;
    }
    
    // 2. 尝试从 Cargo.toml 获取（编译时）
    if let Ok(version) = env::var("CARGO_PKG_VERSION") {
        return version;
    }
    
    // 3. 尝试从父级 Python 包获取
    if let Ok(version) = get_parent_python_version() {
        return version;
    }
    
    // 4. 默认版本
    "0.1.0".to_string()
}

/// 从父级 Python 包获取版本
fn get_parent_python_version() -> Result<String> {
    let current_dir = env::current_dir()?;
    let mut search_path = current_dir.as_path();
    
    // 向上查找 pyproject.toml
    loop {
        let pyproject_path = search_path.join("pyproject.toml");
        if pyproject_path.exists() {
            // 尝试解析 pyproject.toml
            if let Ok(version) = parse_pyproject_version(&pyproject_path) {
                return Ok(version);
            }
        }
        
        match search_path.parent() {
            Some(parent) => search_path = parent,
            None => break,
        }
    }
    
    Err(anyhow!("无法找到父级 Python 包版本"))
}

/// 解析 pyproject.toml 中的版本信息
fn parse_pyproject_version(pyproject_path: &Path) -> Result<String> {
    let content = fs::read_to_string(pyproject_path)?;
    
    // 尝试解析 TOML
    if let Ok(parsed) = toml::from_str::<toml::Value>(&content) {
        if let Some(project) = parsed.get("project") {
            // 检查静态版本
            if let Some(version) = project.get("version") {
                if let Some(version_str) = version.as_str() {
                    return Ok(version_str.to_string());
                }
            }
            
            // 检查动态版本
            if let Some(dynamic) = project.get("dynamic") {
                if let Some(dynamic_arr) = dynamic.as_array() {
                    for item in dynamic_arr {
                        if item.as_str() == Some("version") {
                            // 尝试从 hatch 配置获取版本
                            if let Ok(hatch_version) = get_hatch_version(&parsed, pyproject_path) {
                                return Ok(hatch_version);
                            }
                        }
                    }
                }
            }
        }
    }
    
    Err(anyhow!("无法解析 pyproject.toml 中的版本"))
}

/// 从 hatch 配置获取版本
fn get_hatch_version(parsed: &toml::Value, pyproject_path: &Path) -> Result<String> {    if let Some(tool) = parsed.get("tool") {
        if let Some(hatch) = tool.get("hatch") {
            if let Some(version) = hatch.get("version") {
                if let Some(path) = version.get("path") {
                    if let Some(path_str) = path.as_str() {
                        // 构建版本文件的完整路径
                        let version_file = pyproject_path.parent()
                            .ok_or_else(|| anyhow!("无法获取 pyproject.toml 的父目录"))?
                            .join(path_str);
                        
                        return extract_version_from_file(&version_file);
                    }
                }
            }
        }
    }
    
    Err(anyhow!("无法从 hatch 配置获取版本"))
}

/// 从文件中提取版本信息
fn extract_version_from_file(file_path: &Path) -> Result<String> {
    if !file_path.exists() {
        return Err(anyhow!("版本文件不存在: {}", file_path.display()));
    }
    
    let content = fs::read_to_string(file_path)?;
    
    // 查找版本定义
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("__version__") {
            return extract_version_from_line(line);
        }
    }
    
    Err(anyhow!("在文件中未找到 __version__ 定义"))
}

/// 从代码行中提取版本字符串
fn extract_version_from_line(line: &str) -> Result<String> {
    // 查找双引号
    if let Some(start) = line.find('"') {
        if let Some(end) = line[start + 1..].find('"') {
            return Ok(line[start + 1..start + 1 + end].to_string());
        }
    }
    
    // 查找单引号
    if let Some(start) = line.find('\'') {
        if let Some(end) = line[start + 1..].find('\'') {
            return Ok(line[start + 1..start + 1 + end].to_string());
        }
    }
    
    Err(anyhow!("无法从行中提取版本: {}", line))
}

/// 确保目录存在
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// 递归删除目录
pub fn remove_dir_all(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

/// 运行外部命令
pub fn run_command(command: &str, args: &[&str], cwd: Option<&Path>) -> Result<std::process::Output> {
    let mut cmd = std::process::Command::new(command);
    cmd.args(args);
    
    if let Some(working_dir) = cwd {
        cmd.current_dir(working_dir);
    }
    
    let output = cmd.output()?;
    Ok(output)
}

/// 检查命令是否可用
pub fn is_command_available(command: &str) -> bool {
    std::process::Command::new(command)
        .arg("--version")
        .output()
        .is_ok()
}

/// 获取 Git 信息
pub fn get_git_info(project_path: &Path) -> Option<GitInfo> {
    // 查找 Git 仓库根目录
    let mut current = project_path;
    let mut git_root = None;
    
    loop {
        if current.join(".git").exists() {
            git_root = Some(current);
            break;
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }
    
    let git_root = git_root?;
    let is_in_repo_root = git_root == project_path;
    
    // 获取远程信息
    let remote_info = get_git_remote_info(git_root)?;
    
    Some(GitInfo {
        git_root: git_root.to_string_lossy().to_string(),
        remote_url: remote_info.url,
        username: remote_info.username,
        repo_name: remote_info.repo_name,
        is_in_repo_root,
    })
}

/// Git 信息结构
#[derive(Debug, Clone)]
pub struct GitInfo {
    pub git_root: String,
    pub remote_url: String,
    pub username: String,
    pub repo_name: String,
    pub is_in_repo_root: bool,
}

/// Git 远程信息
#[derive(Debug, Clone)]
struct RemoteInfo {
    pub url: String,
    pub username: String,
    pub repo_name: String,
}

/// 获取 Git 远程信息
fn get_git_remote_info(git_root: &Path) -> Option<RemoteInfo> {
    let config_path = git_root.join(".git").join("config");
    if !config_path.exists() {
        return None;
    }
    
    let content = fs::read_to_string(&config_path).ok()?;
    parse_git_remote(&content, "origin")
}

/// 解析 Git 配置中的远程信息
fn parse_git_remote(config_content: &str, remote_name: &str) -> Option<RemoteInfo> {
    let section_header = format!("[remote \"{}\"]", remote_name);
    let mut in_remote_section = false;
    let mut url = None;
    
    for line in config_content.lines() {
        let line = line.trim();
        
        if line == section_header {
            in_remote_section = true;
        } else if line.starts_with('[') && line.ends_with(']') {
            in_remote_section = false;
        } else if in_remote_section && line.starts_with("url = ") {
            url = Some(line.strip_prefix("url = ")?.to_string());
            break;
        }
    }
    
    let url = url?;
    let (username, repo_name) = parse_github_url(&url)?;
    
    Some(RemoteInfo {
        url,
        username,
        repo_name,
    })
}

/// 解析 GitHub URL
fn parse_github_url(url: &str) -> Option<(String, String)> {
    use regex::Regex;
    
    let patterns = [
        r"https://github\.com/([^/]+)/([^/]+?)(?:\.git)?/?$",
        r"git@github\.com:([^/]+)/([^/]+?)(?:\.git)?/?$",
        r"ssh://git@github\.com/([^/]+)/([^/]+?)(?:\.git)?/?$",
    ];
    
    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(captures) = re.captures(url) {
                let username = captures.get(1)?.as_str().to_string();
                let repo_name = captures.get(2)?.as_str().to_string();
                return Some((username, repo_name));
            }
        }
    }
    
    None
}

/// 生成版本号和版本代码
pub fn generate_version_info() -> Result<(String, String)> {
    use chrono::{Utc, Datelike};
    
    let now = Utc::now();
    let year = now.year();
    let month = now.month();
    let day = now.day();
    
    // 生成版本代码：年份+月份+日期+两位序列(从00开始)
    let version_code = format!("{:04}{:02}{:02}00", year, month, day);
    
    // 获取 Git commit hash
    let commit_hash = get_git_commit_hash().unwrap_or_else(|_| "unknown".to_string());
    
    // 生成版本号：v0.1.0-{commit_hash前8位}
    let short_hash = if commit_hash.len() >= 8 {
        &commit_hash[..8]
    } else {
        &commit_hash
    };
    let version = format!("v0.1.0-{}", short_hash);
    
    Ok((version, version_code))
}

/// 获取当前 Git commit hash
pub fn get_git_commit_hash() -> Result<String> {
    use git2::Repository;
    
    let current_dir = std::env::current_dir()?;
    let mut search_path = current_dir.as_path();
    
    // 向上搜索 Git 仓库
    loop {
        let git_dir = search_path.join(".git");
        if git_dir.exists() {
            // 找到 Git 仓库，尝试打开
            if let Ok(repo) = Repository::open(search_path) {
                // 获取 HEAD 引用
                if let Ok(head) = repo.head() {
                    if let Some(oid) = head.target() {
                        return Ok(oid.to_string());
                    }
                }
            }
            break;
        }
        
        match search_path.parent() {
            Some(parent) => search_path = parent,
            None => break,
        }
    }
    
    anyhow::bail!("无法获取 Git commit hash")
}

/// 更新项目的版本信息
pub fn update_project_version(config: &mut crate::config::ProjectConfig) -> Result<()> {
    let (version, version_code) = generate_version_info()?;
    
    // 更新版本号
    config.version = Some(version.clone());
    
    // 检查是否需要更新版本代码
    let today_prefix = &version_code[..8]; // YYYYMMDD
    let current_prefix = if config.version_code.len() >= 8 {
        &config.version_code[..8]
    } else {
        ""
    };
    
    if today_prefix != current_prefix {
        // 新的一天，重置为00
        config.version_code = version_code;
    } else {
        // 同一天，递增序列号
        let current_seq: u32 = if config.version_code.len() >= 10 {
            config.version_code[8..].parse().unwrap_or(0)
        } else {
            0
        };
        
        let new_seq = (current_seq + 1).min(99); // 最大99
        config.version_code = format!("{}{:02}", today_prefix, new_seq);
    }
    
    println!("🔄 更新版本信息: 版本号={}, 版本代码={}", version, config.version_code);
    
    Ok(())
}

/// 检测当前项目是否在 Git 仓库根目录
pub fn detect_git_repo_info() -> Result<Option<GitRepoInfo>> {
    let current_dir = std::env::current_dir()?;
    let mut search_path = current_dir.as_path();
    
    // 向上搜索 .git 目录
    loop {
        let git_dir = search_path.join(".git");
        if git_dir.exists() {
            // 找到 Git 仓库根目录
            let is_in_repo_root = search_path == current_dir;
              // 读取 Git 配置获取远程仓库信息
            if let Some(git_info) = get_git_info(search_path) {
                return Ok(Some(GitRepoInfo {
                    repo_root: search_path.to_path_buf(),
                    is_in_repo_root,
                    remote_url: git_info.remote_url,
                    username: git_info.username,
                    repo_name: git_info.repo_name,
                }));
            }
        }
        
        match search_path.parent() {
            Some(parent) => search_path = parent,
            None => break,
        }
    }
    
    Ok(None)
}

/// Git 仓库信息
#[derive(Debug, Clone)]
pub struct GitRepoInfo {
    pub repo_root: PathBuf,
    pub is_in_repo_root: bool,
    #[allow(dead_code)]
    pub remote_url: String,
    pub username: String,
    pub repo_name: String,
}

/// 生成 update.json 文件
pub async fn generate_update_json(
    config: &crate::config::ProjectConfig,
    project_root: &Path,
    rmake_config: Option<&crate::config::RmakeConfig>,
) -> Result<()> {
    use serde_json::json;
    
    // 检测 Git 仓库信息
    let git_info = detect_git_repo_info()?;
    
    if git_info.is_none() {
        println!("⚠️  未检测到 Git 仓库，跳过 update.json 生成");
        return Ok(());
    }
    
    let git_info = git_info.unwrap();
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
    };
    
    let zip_filename = format!("{}-{}.zip", config.id, config.version_code);
    let changelog_filename = "CHANGELOG.MD";
    
    // 构建原始 URL
    let zip_url = format!(
        "https://raw.githubusercontent.com/{}/{}/main{}/{}",
        git_info.username, git_info.repo_name, base_path, zip_filename
    );
    
    let changelog_url = format!(
        "https://raw.githubusercontent.com/{}/{}/main{}/{}",
        git_info.username, git_info.repo_name, base_path, changelog_filename
    );
    
    // 检查是否需要应用代理
    let (final_zip_url, final_changelog_url) = if let Some(rmake) = rmake_config {
        if let Some(proxy_config) = &rmake.proxy {
            if proxy_config.enabled {
                let proxy = if let Some(custom_proxy) = &proxy_config.custom_proxy {                // 使用自定义代理
                Some(crate::proxy::GithubProxy {
                    url: custom_proxy.clone(),
                    server: "custom".to_string(),
                    ip: "".to_string(),
                    location: "".to_string(),
                    latency: 0,
                    speed: 0.0,
                })
                } else if proxy_config.auto_select.unwrap_or(true) {
                    // 自动选择最快代理
                    println!("🔍 正在获取最快的 GitHub 代理...");
                    match crate::proxy::get_fastest_proxy().await {
                        Ok(proxy_opt) => {
                            if let Some(proxy) = &proxy_opt {
                                println!("✅ 选择代理: {} (速度: {:.2})", proxy.url, proxy.speed);
                            }
                            proxy_opt
                        }
                        Err(e) => {
                            println!("⚠️  获取代理失败: {}, 将使用原始链接", e);
                            None
                        }
                    }
                } else {
                    None
                };
                
                (
                    crate::proxy::apply_proxy_to_url(&zip_url, proxy.as_ref()),
                    crate::proxy::apply_proxy_to_url(&changelog_url, proxy.as_ref()),
                )
            } else {
                (zip_url, changelog_url)
            }
        } else {
            (zip_url, changelog_url)
        }
    } else {
        (zip_url, changelog_url)
    };
    
    // 创建 update.json 内容
    let update_json = json!({
        "versionCode": config.version_code.parse::<u32>().unwrap_or(1),
        "version": config.version.clone(),
        "zipUrl": final_zip_url,
        "changelog": final_changelog_url
    });
    
    // 写入 update.json 文件
    let update_json_path = project_root.join("update.json");
    let content = serde_json::to_string_pretty(&update_json)?;
    std::fs::write(&update_json_path, content)?;
    
    println!("📄 生成 update.json: {}", update_json_path.display());
    println!("🔗 模块下载链接: {}", final_zip_url);
    
    Ok(())
}

/// Git 用户信息结构
#[derive(Debug, Clone)]
pub struct GitUserInfo {
    pub name: String,
    pub email: String,
}

/// 从 git 配置中获取用户信息
pub fn get_git_user_info() -> Result<GitUserInfo> {
    // 首先尝试从环境变量获取
    if let (Ok(name), Ok(email)) = (std::env::var("GIT_AUTHOR_NAME"), std::env::var("GIT_AUTHOR_EMAIL")) {
        return Ok(GitUserInfo { name, email });
    }
    
    if let (Ok(name), Ok(email)) = (std::env::var("GIT_COMMITTER_NAME"), std::env::var("GIT_COMMITTER_EMAIL")) {
        return Ok(GitUserInfo { name, email });
    }

    // 尝试使用 git2 库从配置中获取
    match get_git_user_from_config() {
        Ok(user_info) => Ok(user_info),
        Err(_) => {
            // 如果无法从 git 配置获取，尝试从全局 git 配置获取
            match get_git_user_from_command() {
                Ok(user_info) => Ok(user_info),
                Err(e) => Err(anyhow!(
                    "无法获取 git 用户信息: {}。请设置 git 配置：\n\
                     git config --global user.name \"Your Name\"\n\
                     git config --global user.email \"your.email@example.com\"", e
                ))
            }
        }
    }
}

/// 使用 git2 库从配置中获取用户信息
fn get_git_user_from_config() -> Result<GitUserInfo> {
    // 尝试打开当前目录的 git 仓库
    let repo = match git2::Repository::open(".") {
        Ok(repo) => repo,
        Err(_) => {
            // 如果当前目录不是 git 仓库，尝试打开全局配置
            return get_git_user_from_global_config();
        }
    };

    // 获取仓库配置
    let config = repo.config()?;
    
    let name = config.get_string("user.name")
        .map_err(|_| anyhow!("未找到 user.name 配置"))?;
    let email = config.get_string("user.email")
        .map_err(|_| anyhow!("未找到 user.email 配置"))?;

    Ok(GitUserInfo { name, email })
}

/// 从全局 git 配置获取用户信息
fn get_git_user_from_global_config() -> Result<GitUserInfo> {
    let config = git2::Config::open_default()?;
    
    let name = config.get_string("user.name")
        .map_err(|_| anyhow!("未找到全局 user.name 配置"))?;
    let email = config.get_string("user.email")
        .map_err(|_| anyhow!("未找到全局 user.email 配置"))?;

    Ok(GitUserInfo { name, email })
}

/// 通过命令行 git 获取用户信息（备用方案）
fn get_git_user_from_command() -> Result<GitUserInfo> {
    use std::process::Command;

    let name_output = Command::new("git")
        .args(&["config", "--global", "user.name"])
        .output()
        .map_err(|e| anyhow!("执行 git config 命令失败: {}", e))?;

    let email_output = Command::new("git")
        .args(&["config", "--global", "user.email"])
        .output()
        .map_err(|e| anyhow!("执行 git config 命令失败: {}", e))?;

    if !name_output.status.success() {
        return Err(anyhow!("git config user.name 命令失败"));
    }

    if !email_output.status.success() {
        return Err(anyhow!("git config user.email 命令失败"));
    }

    let name = String::from_utf8(name_output.stdout)
        .map_err(|e| anyhow!("解析 user.name 输出失败: {}", e))?
        .trim()
        .to_string();

    let email = String::from_utf8(email_output.stdout)
        .map_err(|e| anyhow!("解析 user.email 输出失败: {}", e))?
        .trim()
        .to_string();

    if name.is_empty() || email.is_empty() {
        return Err(anyhow!("git 用户名或邮箱为空"));
    }

    Ok(GitUserInfo { name, email })
}

/// 查找项目配置文件，如果不存在则创建默认的
pub fn find_or_create_project_config(start_dir: &Path) -> Result<PathBuf> {
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
    
    // 如果找不到配置文件，在当前目录创建默认的 rmmproject.toml
    let config_path = start_dir.join("rmmproject.toml");
    create_default_project_config(&config_path)?;
    
    println!("✨ 已创建默认的 rmmproject.toml 配置文件");
    println!("💡 您可以编辑此文件来自定义项目设置");
    
    Ok(config_path)
}

/// 创建默认的项目配置文件
pub fn create_default_project_config(config_path: &Path) -> Result<()> {
    let dir_name = config_path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("my_project");
    
    let default_config = format!(r#"# RMM 项目配置文件
id = "{}"
name = "{}"
description = "一个 RMM 项目"
version = "v0.1.0"
versionCode = "1000000"
requires_rmm = ">=0.2.0"
readme = "README.MD"
changelog = "CHANGELOG.MD"
license = "LICENSE"
updateJson = "https://raw.githubusercontent.com/YOUR_USERNAME/YOUR_REPOSITORY/main/update.json"
dependencies = []

[[authors]]
name = "Your Name"
email = "your.email@example.com"

[scripts]
build = "rmm build"

[urls]
github = "https://github.com/YOUR_USERNAME/YOUR_REPOSITORY"
"#, dir_name, dir_name);

    std::fs::write(config_path, default_config)?;
    Ok(())
}
