use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use crate::utils::{get_rmm_root, get_git_user_info};

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
            
            // 不再自动从 git 配置更新全局用户信息，避免安全风险
            
            // 只在明确要求同步时才验证项目路径，避免每次加载都清理项目
            // config.validate_and_sync_projects()?;
            
            config} else {
            let mut config = Self::default();
            config.github_token = env::var("GITHUB_ACCESS_TOKEN").ok();
            
            // 注意：不自动从 git 配置更新全局用户信息
            // git 信息只应该用于项目级别的作者信息，避免安全风险
            println!("⚠️  使用默认配置，请使用 'rmm config --user.name \"你的名字\"' 设置用户信息");
            
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
            
            // 创建RMM必要的目录结构
            Self::ensure_rmm_directories(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        
        Ok(())
    }
    
    /// 确保RMM目录结构完整
    fn ensure_rmm_directories(rmm_root: &Path) -> Result<()> {
        let directories = [
            "bin",      // 二进制文件
            "cache",    // 缓存文件
            "tmp",      // 临时文件
            "data",     // 数据文件
            "backup",   // 备份文件
            "logs",     // 日志文件
        ];
        
        for dir in &directories {
            let dir_path = rmm_root.join(dir);
            if !dir_path.exists() {
                fs::create_dir_all(&dir_path)?;
                println!("📁 创建目录: {}", dir_path.display());
            }
        }
        
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn remove_project(&mut self, name: &str) -> Result<bool> {
        let removed = self.projects.remove(name).is_some();
        if removed {
            self.save()?;
        }
        Ok(removed)
    }
      /// 获取项目路径
    #[allow(dead_code)]
    pub fn get_project_path(&self, name: &str) -> Option<&String> {
        self.projects.get(name)
    }
    
    /// 列出所有项目
    #[allow(dead_code)]
    pub fn list_projects(&self) -> &HashMap<String, String> {
        &self.projects
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
    
    /// 从 git 配置更新用户信息
    pub fn update_user_info_from_git(&mut self) -> Result<()> {
        // 只有当前是占位符值时才更新
        let should_update = self.username == "username" || self.email == "email" || 
                           self.username.is_empty() || self.email.is_empty();
        
        if should_update {
            let git_user = get_git_user_info()?;
            self.username = git_user.name;
            self.email = git_user.email;
            println!("✅ 已从 git 配置更新用户信息: {} <{}>", self.username, self.email);
        }
        
        Ok(())
    }
    
    /// 强制从 git 配置更新用户信息（即使不是占位符）
    pub fn force_update_user_info_from_git(&mut self) -> Result<()> {
        let git_user = get_git_user_info()?;
        self.username = git_user.name;
        self.email = git_user.email;
        println!("✅ 已强制从 git 配置更新用户信息: {} <{}>", self.username, self.email);
        Ok(())
    }
      /// 刷新配置（重新从文件加载）
    #[allow(dead_code)]
    pub fn refresh(&mut self) -> Result<()> {
        let refreshed_config = Self::load()?;
        *self = refreshed_config;
        Ok(())
    }

    /// 检查配置文件是否有更新（基于修改时间）
    #[allow(dead_code)]
    pub fn has_config_changed(&self) -> Result<bool> {
        let config_path = Self::config_path()?;
        if !config_path.exists() {
            return Ok(false);
        }
          let metadata = std::fs::metadata(&config_path)?;
        if let Ok(_modified) = metadata.modified() {
            // 这里可以存储上次加载的时间并比较
            // 简化版本：总是返回true，表示需要检查
            return Ok(true);
        }
        
        Ok(false)
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
                // 移除旧的路径映射（如果存在不同ID指向同一路径）
                let keys_to_remove: Vec<String> = self.projects.iter()
                    .filter(|(_, path)| {
                        Path::new(path).canonicalize().map(|p| p == canonical_path).unwrap_or(false)
                    })
                    .map(|(key, _)| key.clone())
                    .collect();
                
                for key in keys_to_remove {
                    if key != project_id {  // 不要移除当前项目ID
                        self.projects.remove(&key);
                    }
                }
                
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

    /// 验证并修复 meta.toml 文件格式
    pub fn validate_and_fix_format(&mut self) -> Result<bool> {
        let mut fixed = false;
        
        // 检查项目部分的格式
        let mut valid_projects = HashMap::new();
        let mut invalid_entries = Vec::new();
        
        for (id, path) in &self.projects {
            let path_obj = Path::new(path);
            
            // 检查路径是否存在
            if !path_obj.exists() {
                println!("⚠️  项目路径不存在: {} -> {}", id, path);
                invalid_entries.push(id.clone());
                continue;
            }
            
            // 检查是否是有效的 RMM 项目
            if !is_rmm_project(path_obj) {
                println!("⚠️  无效的 RMM 项目: {} -> {}", id, path);
                invalid_entries.push(id.clone());
                continue;
            }
            
            // 规范化路径
            match path_obj.canonicalize() {
                Ok(canonical_path) => {
                    let canonical_str = canonical_path.to_string_lossy().to_string();
                    if canonical_str != *path {
                        println!("🔧 规范化路径: {} -> {}", path, canonical_str);
                        fixed = true;
                    }
                    valid_projects.insert(id.clone(), canonical_str);
                }
                Err(_) => {
                    println!("⚠️  无法规范化路径: {} -> {}", id, path);
                    invalid_entries.push(id.clone());
                }
            }
        }
        
        // 移除无效条目
        for id in &invalid_entries {
            self.projects.remove(id);
            fixed = true;
            println!("🗑️  移除无效项目: {}", id);
        }
        
        // 更新有效项目
        if fixed {
            self.projects = valid_projects;
        }
        
        Ok(fixed)
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
    pub prebuild: Option<Vec<String>>,
    pub build: Option<Vec<String>>,
    pub postbuild: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
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
    #[allow(dead_code)]
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

    /// 从当前 Git 仓库信息添加作者到项目配置（不覆盖现有作者）
    pub fn add_git_author_if_not_exists(&mut self, project_path: &Path) -> Result<()> {
        // 只有在项目目录内的 Git 仓库才处理
        if let Some(_git_info) = crate::utils::get_git_info(project_path) {
            // 尝试从当前仓库的 Git 配置获取用户信息
            if let Ok(current_git_user) = crate::utils::get_git_user_info() {
                let new_author = Author {
                    name: current_git_user.name.clone(),
                    email: current_git_user.email.clone(),
                };
                
                // 检查是否已经存在相同的作者
                let author_exists = self.authors.iter().any(|author| {
                    author.email == new_author.email || 
                    (author.name == new_author.name && author.email == new_author.email)
                });
                
                if !author_exists {
                    self.authors.push(new_author);
                    println!("✅ 已添加 Git 用户作为项目作者: {} <{}>", current_git_user.name, current_git_user.email);
                    println!("💡 这只影响当前项目，不会修改全局配置");
                } else {
                    println!("ℹ️  Git 用户已是项目作者: {} <{}>", current_git_user.name, current_git_user.email);
                }
            }
        }
        
        Ok(())
    }
}

/// Rmake 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RmakeConfig {
    pub build: BuildConfig,
    pub package: Option<PackageConfig>,
    pub scripts: Option<HashMap<String, String>>,
    pub proxy: Option<ProxyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    pub compression: Option<String>,
    pub zip_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub auto_select: Option<bool>,
    pub custom_proxy: Option<String>,
}

impl RmakeConfig {
    /// 从项目目录加载 Rmake 配置
    pub fn load_from_dir(project_path: &Path) -> Result<Option<Self>> {
        let rmake_path = project_path.join(".rmmp").join("Rmake.toml");
        if !rmake_path.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&rmake_path)?;
        let config: RmakeConfig = toml::from_str(&content)?;
        Ok(Some(config))
    }
    
    /// 保存配置到文件
    pub fn save_to_dir(&self, project_path: &Path) -> Result<()> {
        let rmmp_dir = project_path.join(".rmmp");
        fs::create_dir_all(&rmmp_dir)?;
        
        let rmake_path = rmmp_dir.join("Rmake.toml");
        let content = toml::to_string_pretty(self)?;
        fs::write(&rmake_path, content)?;
        Ok(())
    }
}

/// 获取 RMM 版本
pub fn get_rmm_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 检查是否是 RMM 项目
pub fn is_rmm_project(path: &Path) -> bool {
    path.join("rmmproject.toml").exists()
}

/// 查找项目配置文件
pub fn find_project_file(start_dir: &Path) -> Result<PathBuf> {
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

/// 返回默认的 Rmake 配置
pub fn create_default_rmake_config() -> RmakeConfig {
    use std::collections::HashMap;

    let mut scripts = HashMap::new();
    scripts.insert("test".to_string(), "echo 'Running tests...'".to_string());
    scripts.insert("lint".to_string(), "echo 'Linting code...'".to_string());

    RmakeConfig {
        build: BuildConfig {
            prebuild: Some(vec!["echo 'Pre-build step'".to_string()]),
            build: Some(vec!["echo 'Main build step'".to_string()]),
            postbuild: Some(vec!["echo 'Post-build step'".to_string()]),
            exclude: Some(vec![
                ".git".to_string(),
                "target".to_string(),
                "*.log".to_string(),
                ".vscode".to_string(),
                ".idea".to_string(),
                "node_modules".to_string(),
                "__pycache__".to_string(),
                ".github".to_string()
            ]),
        },
        package: Some(PackageConfig {
            compression: Some("default".to_string()),
            zip_name: Some("default".to_string()),
        }),
        scripts: Some(scripts),
        proxy: Some(ProxyConfig {
            enabled: true,
            auto_select: Some(true),
            custom_proxy: None,
        }),
    }
}
