use anyhow::{Result, anyhow};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// 获取 RMM 根目录
pub fn get_rmm_root() -> Result<PathBuf> {
    // 优先使用环境变量
    if let Ok(rmm_root) = env::var("RMM_ROOT") {
        let path = PathBuf::from(rmm_root);
        if path.exists() {
            return Ok(path);
        }
    }
    
    // 默认路径：Android 环境
    let android_path = PathBuf::from("/data/adb/.rmm");
    if android_path.exists() {
        return Ok(android_path);
    }
    
    // 默认路径：用户主目录
    if let Ok(home) = env::var("HOME") {
        let home_path = PathBuf::from(home).join("data").join("adb").join(".rmm");
        return Ok(home_path);
    }
    
    // Windows 用户目录
    if let Ok(userprofile) = env::var("USERPROFILE") {
        let win_path = PathBuf::from(userprofile).join("data").join("adb").join(".rmm");
        return Ok(win_path);
    }
    
    // 最后的备选方案：当前目录
    let current_dir = env::current_dir()?;
    Ok(current_dir.join(".rmm"))
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
fn get_hatch_version(parsed: &toml::Value, pyproject_path: &Path) -> Result<String> {
    if let Some(tool) = parsed.get("tool") {
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
