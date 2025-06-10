use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use crate::utils::get_rmm_root;

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

impl RmmConfig {    /// 加载配置文件，如果不存在则创建默认配置
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        let mut config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let mut config: RmmConfig = toml::from_str(&content)?;
            
            // 确保版本是最新的
            config.version = get_rmm_version();
            
            // 从环境变量加载GitHub token
            config.github_token = env::var("GITHUB_ACCESS_TOKEN").ok();
            
            // 验证项目路径有效性并同步项目信息
            config.validate_and_sync_projects()?;
            
            // 保存更新后的配置
            config.save()?;
            
            config
        } else {
            let mut config = Self::default();
            config.github_token = env::var("GITHUB_ACCESS_TOKEN").ok();
            config.save()?;
            config
        };
        
        // 从环境变量读取 GitHub 令牌
        config.github_token = env::var("GITHUB_ACCESS_TOKEN").ok();
        
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
    }    /// 验证并同步项目信息
    pub fn validate_and_sync_projects(&mut self) -> Result<()> {
        let mut invalid_projects = Vec::new();
        let mut updated = false;
        
        // 先收集所有需要处理的项目信息
        let projects_to_check: Vec<(String, String)> = self.projects.iter()
            .map(|(name, path)| (name.clone(), path.clone()))
            .collect();
        
        for (name, path) in projects_to_check {
            let project_path = Path::new(&path);
            if !project_path.exists() || !is_rmm_project(project_path) {
                invalid_projects.push(name.clone());
            } else {
                // 同步项目元数据
                if let Err(e) = self.sync_project_metadata(&name, project_path) {
                    eprintln!("警告: 无法同步项目 {} 的元数据: {}", name, e);
                } else {
                    updated = true;
                }
            }
        }
        
        // 移除无效项目
        for name in invalid_projects {
            self.projects.remove(&name);
            updated = true;
        }
        
        // 如果有更新，保存配置
        if updated {
            self.save()?;
        }
        
        Ok(())
    }
    
    /// 同步单个项目的元数据
    fn sync_project_metadata(&self, _project_name: &str, project_path: &Path) -> Result<()> {
        let config_file = project_path.join("rmmproject.toml");
        if !config_file.exists() {
            return Ok(()); // 项目配置文件不存在，跳过同步
        }
        
        // 读取项目配置
        let content = fs::read_to_string(&config_file)?;
        let mut project_config: ProjectConfig = toml::from_str(&content)?;
        
        // 同步RMM版本信息
        project_config.requires_rmm = self.version.clone();
        
        // 保存更新后的项目配置
        let updated_content = toml::to_string_pretty(&project_config)?;
        fs::write(&config_file, updated_content)?;
        
        println!("已同步项目元数据: {}", project_path.display());
        Ok(())
    }
    
    /// 添加项目到配置
    pub fn add_project(&mut self, name: String, path: String) -> Result<()> {
        let project_path = Path::new(&path);
        
        if !project_path.exists() {
            return Err(anyhow!("项目路径不存在: {}", path));
        }
        
        if !is_rmm_project(project_path) {
            return Err(anyhow!("路径 {} 不是一个有效的 RMM 项目", path));
        }
        
        let canonical_path = project_path.canonicalize()?;
        self.projects.insert(name, canonical_path.to_string_lossy().to_string());
        self.save()?;
        
        Ok(())
    }
    
    /// 移除项目
    pub fn remove_project(&mut self, name: &str) -> Result<bool> {
        let removed = self.projects.remove(name).is_some();
        if removed {
            self.save()?;
        }
        Ok(removed)
    }
    
    /// 获取项目路径
    pub fn get_project_path(&self, name: &str) -> Option<&String> {
        self.projects.get(name)
    }
    
    /// 列出所有项目
    pub fn list_projects(&self) -> &HashMap<String, String> {
        &self.projects
    }
      /// 设置用户信息
    pub fn set_user_info(&mut self, username: Option<String>, email: Option<String>) -> Result<()> {
        if let Some(username) = username {
            self.username = username;
        }
        if let Some(email) = email {
            self.email = email;
        }
        self.save()
    }
    
    /// 发现指定目录下的所有 RMM 项目
    pub fn discover_projects(&self, search_path: &Path, max_depth: usize) -> Result<Vec<(String, PathBuf)>> {
        let mut discovered_projects = Vec::new();
        self.discover_projects_recursive(search_path, max_depth, 0, &mut discovered_projects)?;
        Ok(discovered_projects)
    }
    
    /// 递归发现项目
    fn discover_projects_recursive(
        &self,
        current_path: &Path,
        max_depth: usize,
        current_depth: usize,
        projects: &mut Vec<(String, PathBuf)>
    ) -> Result<()> {
        if current_depth > max_depth {
            return Ok(());
        }
        
        // 检查当前目录是否是 RMM 项目
        if is_rmm_project(current_path) {
            let project_name = current_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string();
            
            let canonical_path = current_path.canonicalize()?;
            projects.push((project_name, canonical_path));
        }
        
        // 如果当前目录是项目目录，不再向下搜索
        if is_rmm_project(current_path) {
            return Ok(());
        }
        
        // 递归搜索子目录
        if let Ok(entries) = fs::read_dir(current_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        // 跳过隐藏目录和一些特殊目录
                        if let Some(dir_name) = path.file_name().and_then(|name| name.to_str()) {
                            if dir_name.starts_with('.') || 
                               dir_name == "node_modules" || 
                               dir_name == "target" ||
                               dir_name == "__pycache__" ||
                               dir_name == "build" ||
                               dir_name == "dist" {
                                continue;
                            }
                        }
                        
                        self.discover_projects_recursive(&path, max_depth, current_depth + 1, projects)?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// 同步项目列表（发现新项目并移除无效项目）
    pub fn sync_project_list(&mut self, search_paths: &[PathBuf], max_depth: usize) -> Result<()> {
        println!("🔍 开始同步项目列表...");
        
        // 1. 验证现有项目并移除无效的
        let mut invalid_projects = Vec::new();
        let mut valid_projects = 0;
        
        println!("📋 检查现有项目...");
        for (name, path) in &self.projects {
            let project_path = Path::new(path);
            if !project_path.exists() {
                println!("❌ 项目路径不存在: {} -> {}", name, path);
                invalid_projects.push(name.clone());
            } else if !is_rmm_project(project_path) {
                println!("❌ 无效的 RMM 项目: {} -> {}", name, path);
                invalid_projects.push(name.clone());
            } else {
                println!("✅ 有效项目: {} -> {}", name, path);
                valid_projects += 1;
            }
        }
        
        // 移除无效项目
        for name in &invalid_projects {
            self.projects.remove(name);
        }
        
        if !invalid_projects.is_empty() {
            println!("🧹 已移除 {} 个无效项目", invalid_projects.len());
        }
        
        // 2. 发现新项目
        let mut new_projects = Vec::new();
        let mut discovered_count = 0;
        
        println!("🔍 发现新项目...");
        for search_path in search_paths {
            if !search_path.exists() {
                println!("⚠️  搜索路径不存在: {}", search_path.display());
                continue;
            }
            
            println!("📁 搜索路径: {} (最大深度: {})", search_path.display(), max_depth);
            let discovered = self.discover_projects(search_path, max_depth)?;
            discovered_count += discovered.len();
            
            for (project_name, project_path) in discovered {
                let path_str = project_path.to_string_lossy().to_string();
                
                // 检查是否已经存在
                let is_existing = self.projects.values().any(|existing_path| {
                    Path::new(existing_path).canonicalize().ok() == Some(project_path.clone())
                });
                
                if !is_existing {
                    // 处理名称冲突
                    let mut final_name = project_name.clone();
                    let mut counter = 1;
                    while self.projects.contains_key(&final_name) {
                        final_name = format!("{}_{}", project_name, counter);
                        counter += 1;
                    }
                    
                    new_projects.push((final_name, path_str));
                }
            }
        }
        
        // 添加新项目
        for (name, path) in &new_projects {
            self.projects.insert(name.clone(), path.clone());
            println!("➕ 新增项目: {} -> {}", name, path);
        }
        
        // 3. 保存配置
        if !invalid_projects.is_empty() || !new_projects.is_empty() {
            self.save()?;
        }
        
        // 4. 显示统计信息
        println!("\n📊 同步完成统计:");
        println!("  - 有效项目: {}", valid_projects);
        println!("  - 移除项目: {}", invalid_projects.len());
        println!("  - 发现项目: {}", discovered_count);
        println!("  - 新增项目: {}", new_projects.len());
        println!("  - 总项目数: {}", self.projects.len());
        
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
    pub dependencies: Vec<Dependency>,
    pub authors: Vec<Author>,
    pub scripts: Vec<Script>,
    pub urls: Urls,
    pub build: Option<BuildConfig>,
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
pub struct Script {
    pub name: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Urls {
    pub github: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub prebuild: Option<String>,
    pub build: Option<String>,
    pub postbuild: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    pub git_root: String,
    pub remote_url: String,
    pub username: String,
    pub repo_name: String,
    pub is_in_repo_root: bool,
}

impl ProjectConfig {
    /// 从文件加载配置
    pub fn load_from_file(config_path: &Path) -> Result<Self> {
        if !config_path.exists() {
            return Err(anyhow!("项目配置文件不存在: {}", config_path.display()));
        }
        
        let content = fs::read_to_string(config_path)?;
        let config: ProjectConfig = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// 从项目目录加载配置
    pub fn load_from_dir(project_path: &Path) -> Result<Self> {
        let config_file = project_path.join("rmmproject.toml");
        Self::load_from_file(&config_file)
    }
    
    /// 保存配置到文件
    pub fn save_to_dir(&self, project_path: &Path) -> Result<()> {
        let config_file = project_path.join("rmmproject.toml");
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_file, content)?;
        Ok(())
    }
}

/// 构建配置结构 (.rmmp/Rmake.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RmakeConfig {
    pub build: RmakeBuildConfig,
    pub package: Option<RmakePackageConfig>,
    pub scripts: Option<HashMap<String, String>>,
    pub proxy: Option<RmakeProxyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RmakeBuildConfig {
    pub prebuild: Option<Vec<String>>,
    pub build: Option<Vec<String>>,
    pub postbuild: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub include: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RmakePackageConfig {
    pub zip_name: Option<String>,
    pub tar_name: Option<String>,
    pub compression: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RmakeProxyConfig {
    pub enabled: bool,
    pub auto_select: Option<bool>,
    pub custom_proxy: Option<String>,
}

impl RmakeConfig {
    /// 从项目目录加载构建配置
    pub fn load_from_dir(project_path: &Path) -> Result<Option<Self>> {
        let config_file = project_path.join(".rmmp").join("Rmake.toml");
        if !config_file.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&config_file)?;
        let config: RmakeConfig = toml::from_str(&content)?;
        Ok(Some(config))
    }
    
    /// 保存构建配置
    pub fn save_to_dir(&self, project_path: &Path) -> Result<()> {
        let rmmp_dir = project_path.join(".rmmp");
        fs::create_dir_all(&rmmp_dir)?;
        
        let config_file = rmmp_dir.join("Rmake.toml");
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_file, content)?;
        Ok(())
    }
}

/// 检查路径是否是有效的 RMM 项目
pub fn is_rmm_project(path: &Path) -> bool {
    let project_file = path.join("rmmproject.toml");
    project_file.exists() && project_file.is_file()
}

/// 在当前目录或父目录中查找项目文件
pub fn find_project_file(start_path: &Path) -> Option<PathBuf> {
    let mut current = start_path;
    
    loop {
        let project_file = current.join("rmmproject.toml");
        if project_file.exists() {
            return Some(project_file);
        }
        
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }
      None
}

/// 获取 RMM 版本号（动态从父包获取）
fn get_rmm_version() -> String {
    // 尝试从环境变量获取版本
    if let Ok(version) = env::var("RMM_VERSION") {
        return version;
    }
    
    // 尝试从 Cargo.toml 获取版本
    if let Ok(version) = env::var("CARGO_PKG_VERSION") {
        return version;
    }
    
    // 尝试读取父级 pyproject.toml
    if let Ok(parent_version) = get_parent_package_version() {
        return parent_version;
    }
    
    // 默认版本
    "0.1.0".to_string()
}

/// 从父级包的 pyproject.toml 获取版本
fn get_parent_package_version() -> Result<String> {
    // 查找父级 pyproject.toml
    let current_dir = env::current_dir()?;
    let mut search_path = current_dir.as_path();
    
    loop {
        let pyproject_path = search_path.join("pyproject.toml");
        if pyproject_path.exists() {
            let content = fs::read_to_string(&pyproject_path)?;
            
            // 简单的 TOML 解析来提取版本
            if let Ok(parsed) = toml::from_str::<toml::Value>(&content) {
                if let Some(project) = parsed.get("project") {
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
                                    // 尝试从 __init__.py 读取版本
                                    if let Ok(init_version) = get_version_from_init(search_path) {
                                        return Ok(init_version);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        match search_path.parent() {
            Some(parent) => search_path = parent,
            None => break,
        }
    }
    
    Err(anyhow!("无法找到父级包版本"))
}

/// 从 __init__.py 读取版本
fn get_version_from_init(package_root: &Path) -> Result<String> {
    let init_paths = [
        package_root.join("src").join("pyrmm").join("__init__.py"),
        package_root.join("pyrmm").join("__init__.py"),
        package_root.join("__init__.py"),
    ];
    
    for init_path in &init_paths {
        if init_path.exists() {
            let content = fs::read_to_string(init_path)?;
            
            // 查找版本定义
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("__version__") {
                    // 提取版本字符串
                    if let Some(start) = line.find('"') {
                        if let Some(end) = line[start + 1..].find('"') {
                            return Ok(line[start + 1..start + 1 + end].to_string());
                        }
                    }
                    if let Some(start) = line.find('\'') {
                        if let Some(end) = line[start + 1..].find('\'') {
                            return Ok(line[start + 1..start + 1 + end].to_string());
                        }
                    }
                }
            }
        }
    }
    
    Err(anyhow!("无法从 __init__.py 读取版本"))
}
