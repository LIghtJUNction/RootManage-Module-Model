//! RMM 核心配置数据结构
//! 
//! 定义 RMM 的核心配置结构

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// RMM 主配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RmmConfig {
    pub email: String,
    pub username: String,
    pub version: String,
    pub projects: HashMap<String, String>,
    /// GitHub 访问令牌（运行时从环境变量读取，不存储在配置文件中）
    #[serde(skip)]
    pub github_token: Option<String>,
}

impl Default for RmmConfig {
    fn default() -> Self {
        Self {
            email: "email".to_string(),
            username: "username".to_string(),
            version: get_rmm_version(),
            projects: HashMap::new(),
            github_token: None,
        }
    }
}

impl RmmConfig {
    /// 加载配置文件，如果不存在则创建默认配置
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        let mut config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let mut config: RmmConfig = toml::from_str(&content)?;
            
            // 确保版本是最新的
            config.version = get_rmm_version();
            
            // 从环境变量加载GitHub token
            config.github_token = env::var("GITHUB_ACCESS_TOKEN").ok()
                .or_else(|| env::var("GITHUB_TOKEN").ok());
            
            config
        } else {
            let default_config = Self::default();
            default_config.save()?;
            default_config
        };
        
        // 从环境变量读取 GitHub 令牌
        config.github_token = env::var("GITHUB_ACCESS_TOKEN").ok()
            .or_else(|| env::var("GITHUB_TOKEN").ok());
        
        Ok(config)
    }
      
    /// 保存配置到文件
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        
        Ok(())
    }
      
    /// 获取配置文件路径
    pub fn config_path() -> Result<PathBuf> {
        let rmm_root = get_rmm_root()?;
        Ok(rmm_root.join("meta.toml"))
    }

    /// 添加项目到配置
    pub fn add_project(&mut self, name: String, path: String) -> Result<()> {
        let project_path = Path::new(&path);
        
        if !project_path.exists() {
            return Err(anyhow!("项目路径不存在: {}", path));
        }
        
        let canonical_path = project_path.canonicalize()?;
        self.projects.insert(name, canonical_path.to_string_lossy().to_string());
        self.save()?;
        
        Ok(())
    }
    
    /// 将当前项目添加到全局配置中
    pub fn add_current_project(&mut self, project_id: &str, project_path: &Path) -> Result<()> {
        let canonical_path = project_path.canonicalize()?;
        let path_str = canonical_path.to_string_lossy().to_string();
        
        // 检查项目是否已存在（按路径）
        let project_exists = self.projects.values().any(|path| {
            Path::new(path).canonicalize().map(|p| p == canonical_path).unwrap_or(false)
        });
        
        if !project_exists {
            // 添加项目到列表
            self.projects.insert(project_id.to_string(), path_str.clone());
            self.save()?;
            println!("➕ 已将项目添加到全局配置: {} -> {}", project_id, path_str);
        } else {
            // 检查是否需要更新项目ID映射
            let current_id_path = self.projects.get(project_id);
            if current_id_path.is_none() || current_id_path != Some(&path_str) {
                // 添加或更新当前项目ID和路径的映射
                self.projects.insert(project_id.to_string(), path_str.clone());
                self.save()?;
                println!("🔄 已更新项目映射: {} -> {}", project_id, path_str);
            } else {
                println!("✅ 项目已在全局配置中: {} -> {}", project_id, path_str);
            }
        }
        
        Ok(())
    }
    
    /// 从 git 配置强制更新用户信息
    pub fn force_update_user_info_from_git(&mut self) -> Result<()> {
        use std::process::Command;
        
        // 获取 git 用户名
        let output = Command::new("git")
            .args(&["config", "--global", "user.name"])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("无法获取 git 用户名"));
        }
        
        let git_username = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if git_username.is_empty() {
            return Err(anyhow!("git 用户名为空"));
        }
        
        // 获取 git 邮箱
        let output = Command::new("git")
            .args(&["config", "--global", "user.email"])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow!("无法获取 git 邮箱"));
        }
        
        let git_email = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if git_email.is_empty() {
            return Err(anyhow!("git 邮箱为空"));
        }
        
        // 更新配置
        self.username = git_username;
        self.email = git_email;
        
        Ok(())
    }

    /// 更新用户信息从 git 配置（非强制）
    pub fn update_user_info_from_git(&mut self) -> Result<()> {
        self.force_update_user_info_from_git()
    }

    /// 验证并修复配置格式
    pub fn validate_and_fix_format(&mut self) -> Result<()> {
        // 确保版本是最新的
        self.version = get_rmm_version();
        
        // 验证邮箱格式
        if !self.email.contains('@') {
            self.email = "email@example.com".to_string();
        }
        
        // 验证用户名不为空
        if self.username.trim().is_empty() {
            self.username = "username".to_string();
        }
        
        Ok(())
    }

    /// 同步项目列表
    pub fn sync_project_list(&mut self, search_paths: &[PathBuf]) -> Result<()> {
        use crate::commands::utils::core::common::ProjectManager;
        
        for search_path in search_paths {            if !search_path.exists() {
                continue;
            }
            
            // 递归搜索项目
            if let Ok(entries) = std::fs::read_dir(search_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() && ProjectManager::is_rmm_project(&path) {
                        if let Some(project_name) = path.file_name().and_then(|n| n.to_str()) {
                            let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
                            self.projects.insert(
                                project_name.to_string(),
                                canonical_path.to_string_lossy().to_string()
                            );
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// 项目配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub requires_rmm: String,
    pub version: Option<String>,
    #[serde(rename = "versionCode")]
    pub version_code: String,
    #[serde(rename = "updateJson")]
    pub update_json: String,
    pub readme: String,
    pub changelog: String,
    pub license: String,
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
    pub authors: Vec<Author>,
    #[serde(default)]
    pub scripts: HashMap<String, String>,
    pub urls: Urls,
    pub build: Option<BuildConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<GitInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Urls {
    pub github: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub target: Option<String>,
    pub prebuild: Option<Vec<String>>,
    pub build: Option<Vec<String>>,
    pub postbuild: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    pub url: String,
    pub branch: String,
    pub commit: String,
    pub git_root: String,
    pub remote_url: String,
    pub username: String,
    pub repo_name: String,
    pub is_in_repo_root: bool,
}

impl ProjectConfig {
    /// 从项目目录加载配置
    pub fn load_from_dir(project_path: &Path) -> Result<Self> {
        let config_file = project_path.join("rmmproject.toml");
        Self::load_from_file(&config_file)
    }

    /// 从文件加载配置
    pub fn load_from_file(config_path: &Path) -> Result<Self> {
        if !config_path.exists() {
            return Err(anyhow!("项目配置文件不存在: {}", config_path.display()));
        }
        
        let content = fs::read_to_string(config_path)?;
        let config: ProjectConfig = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// 保存配置到文件
    pub fn save_to_dir(&self, project_path: &Path) -> Result<()> {
        let config_file = project_path.join("rmmproject.toml");
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_file, content)?;
        Ok(())
    }
}

/// 获取 RMM 版本
pub fn get_rmm_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 获取 RMM 根目录
pub fn get_rmm_root() -> Result<PathBuf> {
    // 1. 首先检查环境变量 RMM_ROOT
    if let Ok(rmm_root) = env::var("RMM_ROOT") {
        let path = PathBuf::from(rmm_root);
        if path.exists() {
            return Ok(path);
        }
    }
    
    // 2. 使用默认路径
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    
    let rmm_root = PathBuf::from(home).join("data").join("adb").join(".rmm");
    
    // 确保目录存在
    if !rmm_root.exists() {
        fs::create_dir_all(&rmm_root)?;
    }
    
    Ok(rmm_root)
}
