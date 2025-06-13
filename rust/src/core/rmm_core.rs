use anyhow::{Context, Result};
use git2::{Repository, StatusOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use toml;
use walkdir::WalkDir;

/// 缓存项结构
#[derive(Debug, Clone)]
struct CacheItem<T> {
    data: T,
    timestamp: Instant,
    expires_at: Instant,
}

impl<T> CacheItem<T> {
    fn new(data: T, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            data,
            timestamp: now,
            expires_at: now + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// Meta.toml 文件结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaConfig {
    pub email: String,
    pub username: String,
    pub version: String,
    pub projects: HashMap<String, String>,
}

/// RmmProject.toml 文件结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RmmProject {
    pub project: ProjectInfo,
    pub authors: Vec<Author>,
    pub urls: Option<UrlsInfo>,
    #[serde(rename = "build-system")]
    pub build_system: Option<BuildSystem>,
    #[serde(rename = "tool")]
    pub tool: Option<HashMap<String, toml::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectInfo {
    pub id: String,
    pub description: String,
    #[serde(rename = "updateJson")]
    pub update_json: String,
    pub readme: String,
    pub changelog: String,
    pub license: String,
    pub dependencies: Vec<String>,
    pub scripts: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Author {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UrlsInfo {
    pub github: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildSystem {
    pub requires: Vec<String>,
    #[serde(rename = "build-backend")]
    pub build_backend: String,
}

/// Module.prop 文件结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModuleProp {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(rename = "versionCode")]
    pub version_code: String,
    pub author: String,
    pub description: String,
    #[serde(rename = "updateJson")]
    pub update_json: String,
}

/// Rmake.toml 文件结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RmakeConfig {
    pub build: BuildConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildConfig {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub prebuild: Vec<String>,
    pub build: Vec<String>,
    pub postbuild: Vec<String>,
    pub src: Option<SrcConfig>,
    pub scripts: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SrcConfig {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

/// 项目扫描结果
#[derive(Debug, Clone)]
pub struct ProjectScanResult {
    pub name: String,
    pub path: PathBuf,
    pub is_valid: bool,
    pub git_info: Option<GitInfo>,
}

/// Git 仓库信息
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GitInfo {
    pub repo_root: PathBuf,
    pub relative_path: PathBuf,
    pub branch: String,
    pub remote_url: Option<String>,
    pub has_uncommitted_changes: bool,
    pub last_commit_hash: Option<String>,
    pub last_commit_message: Option<String>,
}

/// Git 分析器
pub struct GitAnalyzer;

impl GitAnalyzer {
    /// 分析给定路径的 Git 信息
    pub fn analyze_git_info(path: &Path) -> Result<Option<GitInfo>> {
        let git_root = Self::find_git_root(path)?;
        
        if let Some(repo_root) = git_root {
            let repo = Repository::open(&repo_root)
                .with_context(|| format!("Failed to open Git repository at {}", repo_root.display()))?;
            
            let relative_path = path.strip_prefix(&repo_root)
                .unwrap_or(Path::new(""))
                .to_path_buf();
            
            let branch = Self::get_current_branch(&repo)?;
            let remote_url = Self::get_remote_url(&repo)?;
            let has_uncommitted_changes = Self::has_uncommitted_changes(&repo)?;
            let (last_commit_hash, last_commit_message) = Self::get_last_commit_info(&repo)?;
            
            Ok(Some(GitInfo {
                repo_root,
                relative_path,
                branch,
                remote_url,
                has_uncommitted_changes,
                last_commit_hash,
                last_commit_message,
            }))
        } else {
            Ok(None)
        }
    }
    
    /// 查找 Git 根目录
    pub fn find_git_root(path: &Path) -> Result<Option<PathBuf>> {
        let mut current = path.to_path_buf();
        
        loop {
            let git_dir = current.join(".git");
            if git_dir.exists() {
                return Ok(Some(current));
            }
            
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                break;
            }
        }
        
        Ok(None)
    }
    
    /// 获取当前分支名
    fn get_current_branch(repo: &Repository) -> Result<String> {
        let head = repo.head()
            .with_context(|| "Failed to get HEAD reference")?;
        
        if let Some(branch_name) = head.shorthand() {
            Ok(branch_name.to_string())
        } else {
            Ok("HEAD".to_string())
        }
    }
    
    /// 获取远程仓库 URL
    fn get_remote_url(repo: &Repository) -> Result<Option<String>> {
        let remotes = repo.remotes()
            .with_context(|| "Failed to get remotes")?;
        
        if let Some(remote_name) = remotes.get(0) {
            let remote = repo.find_remote(remote_name)
                .with_context(|| format!("Failed to find remote: {}", remote_name))?;
            
            if let Some(url) = remote.url() {
                return Ok(Some(url.to_string()));
            }
        }
        
        Ok(None)
    }
    
    /// 检查是否有未提交的更改
    fn has_uncommitted_changes(repo: &Repository) -> Result<bool> {
        let mut opts = StatusOptions::new();
        opts.include_ignored(false);
        opts.include_untracked(true);
        
        let statuses = repo.statuses(Some(&mut opts))
            .with_context(|| "Failed to get repository status")?;
        
        Ok(!statuses.is_empty())
    }
    
    /// 获取最后一次提交信息
    fn get_last_commit_info(repo: &Repository) -> Result<(Option<String>, Option<String>)> {
        let head = repo.head()
            .with_context(|| "Failed to get HEAD reference")?;
        
        if let Some(oid) = head.target() {
            let commit = repo.find_commit(oid)
                .with_context(|| "Failed to find commit")?;
            
            let hash = oid.to_string();
            let message = commit.message().unwrap_or("").to_string();
            
            Ok((Some(hash), Some(message)))
        } else {
            Ok((None, None))
        }
    }
}

/// RmmCore 主要结构体
#[derive(Debug)]
pub struct RmmCore {
    rmm_root: PathBuf,
    meta_cache: Arc<Mutex<Option<CacheItem<MetaConfig>>>>,
    project_cache: Arc<Mutex<HashMap<String, CacheItem<RmmProject>>>>,
    cache_ttl: Duration,
    /// Git 信息缓存
    git_cache: Arc<Mutex<HashMap<PathBuf, (GitInfo, Instant)>>>,

}

impl RmmCore {    /// 创建新的 RmmCore 实例
    pub fn new() -> Self {
        Self {
            rmm_root: Self::get_rmm_root_path(),
            meta_cache: Arc::new(Mutex::new(None)),
            project_cache: Arc::new(Mutex::new(HashMap::new())),
            cache_ttl: Duration::from_secs(60), // 60秒缓存
            git_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 功能一：获取 RMM_ROOT 路径
    /// 尝试读取环境变量 RMM_ROOT，如果没有返回默认值：~/data/adb/.rmm/
    pub fn get_rmm_root(&self) -> PathBuf {
        self.rmm_root.clone()
    }    fn get_rmm_root_path() -> PathBuf {
        if let Ok(root) = env::var("RMM_ROOT") {
            PathBuf::from(root)
        } else {
            let home = env::var("HOME")
                .or_else(|_| env::var("USERPROFILE"))
                .unwrap_or_else(|_| String::from("."));
            // 确保路径构建的正确性，强制重新构建路径字符串
            let mut path = PathBuf::from(home);
            path.push("data");
            path.push("adb");
            path.push(".rmm");
            path
        }
    }

    /// 获取 meta.toml 文件路径
    fn get_meta_path(&self) -> PathBuf {
        self.rmm_root.join("meta.toml")
    }    /// 功能二：获取 RMM_ROOT/meta.toml 文件的内容（解析为字典）
    pub fn get_meta_config(&self) -> Result<MetaConfig> {
        // 检查缓存
        {
            let cache = self.meta_cache.lock().unwrap();
            if let Some(cached) = cache.as_ref() {
                if !cached.is_expired() {
                    return Ok(cached.data.clone());
                }
            }
        }

        // 读取并解析文件
        let meta_path = self.get_meta_path();
        let content = fs::read_to_string(&meta_path)
            .with_context(|| format!("Failed to read meta.toml from {}", meta_path.display()))?;
        
        let meta: MetaConfig = toml::from_str(&content)
            .with_context(|| "Failed to parse meta.toml")?;

        // 更新缓存
        {
            let mut cache = self.meta_cache.lock().unwrap();
            *cache = Some(CacheItem::new(meta.clone(), self.cache_ttl));
        }

        Ok(meta)
    }

    /// 功能三：更新 meta.toml 文件的内容
    pub fn update_meta_config(&self, meta: &MetaConfig) -> Result<()> {
        let meta_path = self.get_meta_path();
        
        // 确保目录存在
        if let Some(parent) = meta_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        let content = toml::to_string_pretty(meta)
            .with_context(|| "Failed to serialize meta config")?;
        
        fs::write(&meta_path, content)
            .with_context(|| format!("Failed to write meta.toml to {}", meta_path.display()))?;

        // 更新缓存
        {
            let mut cache = self.meta_cache.lock().unwrap();
            *cache = Some(CacheItem::new(meta.clone(), self.cache_ttl));
        }

        Ok(())
    }

    /// 功能四：返回 meta.toml 文件内容的某个键的值
    pub fn get_meta_value(&self, key: &str) -> Result<Option<toml::Value>> {
        let meta_path = self.get_meta_path();
        let content = fs::read_to_string(&meta_path)
            .with_context(|| format!("Failed to read meta.toml from {}", meta_path.display()))?;
        
        let parsed: toml::Value = toml::from_str(&content)
            .with_context(|| "Failed to parse meta.toml")?;

        Ok(parsed.get(key).cloned())
    }

    /// 功能五：给定项目名，返回路径
    pub fn get_project_path(&self, project_name: &str) -> Result<Option<PathBuf>> {
        let meta = self.get_meta_config()?;
        Ok(meta.projects.get(project_name).map(|p| PathBuf::from(p)))
    }

    /// 功能六：检查各个项目是否有效（判断对应文件夹是否存在且包含 rmmproject.toml 文件）
    pub fn check_projects_validity(&self) -> Result<HashMap<String, bool>> {
        let meta = self.get_meta_config()?;
        let mut results = HashMap::new();

        for (name, path) in &meta.projects {
            let project_path = PathBuf::from(path);
            let is_valid = project_path.exists() && 
                          project_path.is_dir() && 
                          project_path.join("rmmproject.toml").exists();
            results.insert(name.clone(), is_valid);
        }

        Ok(results)
    }

    /// 功能七：给定一个路径和遍历深度，扫描路径下是否含有 rmmp(project)
    pub fn scan_projects(&self, scan_path: &Path, max_depth: Option<usize>) -> Result<Vec<ProjectScanResult>> {
        let mut results = Vec::new();
        
        let walker = if let Some(depth) = max_depth {
            WalkDir::new(scan_path).max_depth(depth)
        } else {
            WalkDir::new(scan_path)
        };

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            
            // 检查是否包含 rmmproject.toml
            let project_file = path.join("rmmproject.toml");
            if project_file.exists() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                // 获取 Git 信息
                let git_info = GitAnalyzer::analyze_git_info(path).ok().flatten();
                
                results.push(ProjectScanResult {
                    name,
                    path: path.to_path_buf(),
                    is_valid: true,
                    git_info,
                });
            }
        }

        Ok(results)
    }

    /// 功能八：双向更新项目列表（将扫描结果同步到 meta.toml）
    pub fn sync_projects(&self, scan_paths: &[&Path], max_depth: Option<usize>) -> Result<()> {
        let mut all_projects = HashMap::new();
        
        // 扫描所有路径
        for &scan_path in scan_paths {
            let scanned = self.scan_projects(scan_path, max_depth)?;
            for project in scanned {
                all_projects.insert(project.name, project.path.to_string_lossy().to_string());
            }
        }

        // 获取当前配置
        let mut meta = self.get_meta_config().unwrap_or_else(|_| MetaConfig {
            email: String::new(),
            username: String::new(),
            version: String::new(),
            projects: HashMap::new(),
        });

        // 更新项目列表
        meta.projects.extend(all_projects);

        // 保存更新
        self.update_meta_config(&meta)?;

        Ok(())
    }

    /// 功能九：读取项目的 rmmproject.toml
    pub fn get_project_config(&self, project_path: &Path) -> Result<RmmProject> {
        let project_key = project_path.to_string_lossy().to_string();
        
        // 检查缓存
        {
            let cache = self.project_cache.lock().unwrap();
            if let Some(cached) = cache.get(&project_key) {
                if !cached.is_expired() {
                    return Ok(cached.data.clone());
                }
            }
        }

        let project_file = project_path.join("rmmproject.toml");
        let content = fs::read_to_string(&project_file)
            .with_context(|| format!("Failed to read rmmproject.toml from {}", project_file.display()))?;
        
        let project: RmmProject = toml::from_str(&content)
            .with_context(|| "Failed to parse rmmproject.toml")?;

        // 更新缓存
        {
            let mut cache = self.project_cache.lock().unwrap();
            cache.insert(project_key, CacheItem::new(project.clone(), self.cache_ttl));
        }

        Ok(project)
    }

    /// 写入项目的 rmmproject.toml
    pub fn update_project_config(&self, project_path: &Path, project: &RmmProject) -> Result<()> {
        let project_file = project_path.join("rmmproject.toml");
        
        let content = toml::to_string_pretty(project)
            .with_context(|| "Failed to serialize project config")?;
        
        fs::write(&project_file, content)
            .with_context(|| format!("Failed to write rmmproject.toml to {}", project_file.display()))?;

        // 更新缓存
        let project_key = project_path.to_string_lossy().to_string();
        {
            let mut cache = self.project_cache.lock().unwrap();
            cache.insert(project_key, CacheItem::new(project.clone(), self.cache_ttl));
        }

        Ok(())
    }

    /// 功能十：读取项目目录下的 module.prop（以 TOML 格式）
    pub fn get_module_prop(&self, project_path: &Path) -> Result<ModuleProp> {
        let prop_file = project_path.join("module.prop");
        let content = fs::read_to_string(&prop_file)
            .with_context(|| format!("Failed to read module.prop from {}", prop_file.display()))?;
        
        let prop: ModuleProp = toml::from_str(&content)
            .with_context(|| "Failed to parse module.prop")?;

        Ok(prop)
    }

    /// 写入项目目录下的 module.prop
    pub fn update_module_prop(&self, project_path: &Path, prop: &ModuleProp) -> Result<()> {
        let prop_file = project_path.join("module.prop");
        
        let content = toml::to_string_pretty(prop)
            .with_context(|| "Failed to serialize module prop")?;
        
        fs::write(&prop_file, content)
            .with_context(|| format!("Failed to write module.prop to {}", prop_file.display()))?;

        Ok(())
    }

    /// 读取项目根目录下的 .rmmp/Rmake.toml 文件
    pub fn get_rmake_config(&self, project_path: &Path) -> Result<RmakeConfig> {
        let rmake_file = project_path.join(".rmmp").join("Rmake.toml");
        let content = fs::read_to_string(&rmake_file)
            .with_context(|| format!("Failed to read Rmake.toml from {}", rmake_file.display()))?;
        
        let rmake: RmakeConfig = toml::from_str(&content)
            .with_context(|| "Failed to parse Rmake.toml")?;

        Ok(rmake)
    }

    /// 写入项目根目录下的 .rmmp/Rmake.toml 文件
    pub fn update_rmake_config(&self, project_path: &Path, rmake: &RmakeConfig) -> Result<()> {
        let rmmp_dir = project_path.join(".rmmp");
        let rmake_file = rmmp_dir.join("Rmake.toml");
        
        // 确保 .rmmp 目录存在
        fs::create_dir_all(&rmmp_dir)
            .with_context(|| format!("Failed to create .rmmp directory at {}", rmmp_dir.display()))?;
        
        let content = toml::to_string_pretty(rmake)
            .with_context(|| "Failed to serialize Rmake config")?;
        
        fs::write(&rmake_file, content)
            .with_context(|| format!("Failed to write Rmake.toml to {}", rmake_file.display()))?;

        Ok(())
    }

    /// 运行Rmake.toml中定义的脚本
    pub fn run_rmake_script(&self, project_path: &Path, script_name: &str) -> Result<()> {
        use std::process::Command;
        
        // 读取Rmake配置
        let rmake = self.get_rmake_config(project_path)?;
        
        // 检查脚本是否存在
        let scripts = rmake.build.scripts.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Rmake.toml中没有定义scripts部分"))?;
        
        let script_command = scripts.get(script_name)
            .ok_or_else(|| anyhow::anyhow!("脚本 '{}' 未找到", script_name))?;
        
        println!("🚀 执行脚本: {}", script_name);        println!("📋 命令: {}", script_command);        
        // 执行命令 - 使用系统默认终端避免UNC路径问题
        let mut cmd = if cfg!(target_os = "windows") {
            // Windows: 使用PowerShell避免UNC路径问题
            let mut cmd = Command::new("powershell");
            cmd.arg("-Command")
               .arg(&format!("cd '{}'; {}", project_path.display(), script_command));
            cmd
        } else {
            // Unix/Linux: 使用sh
            let mut cmd = Command::new("sh");
            cmd.arg("-c").arg(script_command);
            cmd.current_dir(project_path);
            cmd
        };
        
        let output = cmd.output()
            .with_context(|| format!("执行脚本 '{}' 失败", script_name))?;
        
        // 输出结果
        if !output.stdout.is_empty() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
        
        if !output.stderr.is_empty() {
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
        }
        
        // 检查执行结果
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "脚本 '{}' 执行失败，退出代码: {:?}", 
                script_name, 
                output.status.code()
            ));
        }
        
        println!("✅ 脚本 '{}' 执行成功", script_name);
        Ok(())
    }
    
    /// 列出Rmake.toml中所有可用的脚本
    pub fn list_rmake_scripts(&self, project_path: &Path) -> Result<Vec<String>> {
        let rmake = self.get_rmake_config(project_path)?;
        
        let scripts = rmake.build.scripts.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Rmake.toml中没有定义scripts部分"))?;
        
        Ok(scripts.keys().cloned().collect())
    }

    /// 清理过期缓存
    pub fn cleanup_expired_cache(&self) {
        // 清理 meta 缓存
        {
            let mut cache = self.meta_cache.lock().unwrap();
            if let Some(cached) = cache.as_ref() {
                if cached.is_expired() {
                    *cache = None;
                }
            }
        }

        // 清理项目缓存
        {
            let mut cache = self.project_cache.lock().unwrap();
            cache.retain(|_, cached| !cached.is_expired());
        }
    }

    /// 获取缓存统计信息
    pub fn get_cache_stats(&self) -> (bool, usize) {
        let meta_cached = {
            let cache = self.meta_cache.lock().unwrap();
            cache.is_some() && !cache.as_ref().unwrap().is_expired()
        };

        let project_count = {
            let cache = self.project_cache.lock().unwrap();
            cache.len()
        };

        (meta_cached, project_count)
    }
}

impl Default for RmmCore {
    fn default() -> Self {
        Self::new()
    }
}

// 工具函数
impl RmmCore {
    /// 创建默认的 meta.toml 配置
    pub fn create_default_meta(&self, email: &str, username: &str, version: &str) -> MetaConfig {
        MetaConfig {
            email: email.to_string(),
            username: username.to_string(),
            version: version.to_string(),
            projects: HashMap::new(),
        }
    }

    /// 创建默认的项目配置
    pub fn create_default_project(&self, id: &str, username: &str, email: &str) -> RmmProject {
        RmmProject {
            project: ProjectInfo {
                id: id.to_string(),
                description: format!("RMM项目 {}", id),
                update_json: format!("https://raw.githubusercontent.com/YOUR_USERNAME/YOUR_REPOSITORY/main/update.json"),
                readme: "README.MD".to_string(),
                changelog: "CHANGELOG.MD".to_string(),
                license: "LICENSE".to_string(),
                dependencies: vec![],
                scripts: Some({
                    let mut scripts = HashMap::new();
                    scripts.insert("hello".to_string(), "echo 'hello world!'".to_string());
                    scripts
                }),
            },
            authors: vec![Author {
                name: username.to_string(),
                email: email.to_string(),
            }],
            urls: Some(UrlsInfo {
                github: "https://github.com/YOUR_USERNAME/YOUR_REPOSITORY".to_string(),
            }),
            build_system: Some(BuildSystem {
                requires: vec!["rmm>=0.3.0".to_string()],
                build_backend: "rmm".to_string(),
            }),
            tool: None,
        }
    }

    /// 创建默认的 module.prop
    pub fn create_default_module_prop(&self, id: &str, username: &str) -> ModuleProp {
        ModuleProp {
            id: id.to_string(),
            name: id.to_string(),
            version: "v0.1.0".to_string(),
            version_code: "1000000".to_string(),
            author: username.to_string(),
            description: format!("RMM项目 {}", id),
            update_json: "https://raw.githubusercontent.com/YOUR_USERNAME/YOUR_REPOSITORY/main/update.json".to_string(),
        }
    }    /// 创建默认的 Rmake.toml 配置
    pub fn create_default_rmake(&self) -> RmakeConfig {
        let mut default_scripts = HashMap::new();        // 添加跨平台默认脚本
        if cfg!(target_os = "windows") {
            default_scripts.insert("clean".to_string(), "Remove-Item '.rmmp\\build' -Recurse -Force -ErrorAction SilentlyContinue; Remove-Item '.rmmp\\dist' -Recurse -Force -ErrorAction SilentlyContinue; New-Item -Path '.rmmp\\build' -ItemType Directory -Force; New-Item -Path '.rmmp\\dist' -ItemType Directory -Force".to_string());
        } else {
            default_scripts.insert("clean".to_string(), "rm -rf .rmmp/build/* .rmmp/dist/*".to_string());
        }
        
        // 安装模块的手动方式参考：
        // /data/adb/magisk --install-module xxx
        // /data/adb/ksud module install xxx
        // /data/adb/apd module install xxx
        
        RmakeConfig {
            build: BuildConfig {
                include: vec!["../.gitignore".to_string()],
                exclude: vec![
                    ".git".to_string(), 
                    ".rmmp".to_string(), 
                    "*.tmp".to_string(), 
                    "*.log".to_string()
                ],
                prebuild: vec!["echo 'Starting build'".to_string()],
                build: vec!["rmm".to_string()],
                postbuild: vec!["echo 'Build completed'".to_string()],
                src: Some(SrcConfig {
                    include: vec!["# 源代码额外包含的文件，如：\"docs/\"".to_string()],
                    exclude: vec![
                        ".git".to_string(),
                        "*.tmp".to_string(),
                        "*.log".to_string(),
                        "node_modules".to_string(),
                    ],
                }),
                scripts: Some(default_scripts),
            },
        }
    }
}

impl RmmCore {/// 检测给定路径是否在 Git 仓库中，并返回详细信息
    pub fn get_git_info(&self, path: &Path) -> Result<GitInfo> {
        let canonical_path = path.canonicalize()
            .map_err(|e| anyhow::anyhow!("无法获取路径的绝对路径: {}", e))?;
        
        // 检查缓存
        {
            let cache = self.git_cache.lock().unwrap();
            if let Some((git_info, cached_time)) = cache.get(&canonical_path) {
                if cached_time.elapsed() < self.cache_ttl {
                    return Ok(git_info.clone());
                }
            }
        }
        
        let git_info = self.analyze_git_info(&canonical_path)?;
        
        // 更新缓存
        {
            let mut cache = self.git_cache.lock().unwrap();
            cache.insert(canonical_path, (git_info.clone(), Instant::now()));
        }
        
        Ok(git_info)
    }
    
    /// 分析路径的 Git 信息
    fn analyze_git_info(&self, path: &Path) -> Result<GitInfo> {
        let mut current_path = path.to_path_buf();
        let original_path = path.to_path_buf();
        
        // 向上遍历寻找 .git 文件夹
        loop {
            let git_path = current_path.join(".git");
            if git_path.exists() {
                let relative_path = original_path.strip_prefix(&current_path)
                    .unwrap_or(Path::new(""))
                    .to_path_buf();
                
                let mut git_info = GitInfo {
                    repo_root: current_path.clone(),
                    relative_path,
                    branch: String::new(),
                    remote_url: None,
                    has_uncommitted_changes: false,
                    last_commit_hash: None,
                    last_commit_message: None,
                };
                
                // 读取更多 Git 信息
                self.read_git_details(&current_path, &mut git_info)?;
                
                return Ok(git_info);
            }
            
            match current_path.parent() {
                Some(parent) => current_path = parent.to_path_buf(),
                None => break,
            }
        }
        
        // 没有找到 Git 仓库
        Ok(GitInfo::default())
    }
    
    /// 读取 Git 仓库的详细信息
    fn read_git_details(&self, git_root: &Path, git_info: &mut GitInfo) -> Result<()> {
        let git_path = git_root.join(".git");
        
        // 读取当前分支
        if let Ok(head_content) = fs::read_to_string(git_path.join("HEAD")) {
            if let Some(branch) = head_content.strip_prefix("ref: refs/heads/") {
                git_info.branch = branch.trim().to_string();
            }
        }
        
        // 读取远程仓库 URL
        if let Ok(config_content) = fs::read_to_string(git_path.join("config")) {
            git_info.remote_url = self.parse_git_remote_url(&config_content);
        }
        
        // 检查是否有未提交的更改（简单检查）
        git_info.has_uncommitted_changes = self.check_git_status(git_root)?;
        
        // 获取最后一次提交信息
        let (last_commit_hash, last_commit_message) = self.get_last_commit_info(git_root)?;
        git_info.last_commit_hash = last_commit_hash;
        git_info.last_commit_message = last_commit_message;
        
        Ok(())
    }
    
    /// 解析 Git 配置中的远程 URL
    fn parse_git_remote_url(&self, config_content: &str) -> Option<String> {
        for line in config_content.lines() {
            let line = line.trim();
            if line.starts_with("url = ") {
                return Some(line.strip_prefix("url = ")?.to_string());
            }
        }
        None
    }
    
    /// 检查 Git 仓库状态（简化版）
    fn check_git_status(&self, git_root: &Path) -> Result<bool> {
        let git_path = git_root.join(".git");
        
        // 检查 index 文件是否存在且最近被修改
        let index_path = git_path.join("index");
        if let Ok(metadata) = fs::metadata(&index_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    // 如果 index 文件在最近 1 小时内被修改，可能有未提交的更改
                    return Ok(elapsed < Duration::from_secs(3600));
                }
            }
        }
        
        // 检查工作目录中是否有新文件或修改的文件
        // 这里做简化处理，只检查一些常见的指示器
        Ok(false)
    }
    
    /// 获取最后一次提交信息
    fn get_last_commit_info(&self, git_root: &Path) -> Result<(Option<String>, Option<String>)> {
        let repo = Repository::open(git_root)
            .with_context(|| format!("Failed to open Git repository at {}", git_root.display()))?;
        
        let head = repo.head()
            .with_context(|| "Failed to get HEAD reference")?;
        
        if let Some(oid) = head.target() {
            let commit = repo.find_commit(oid)
                .with_context(|| "Failed to find commit")?;
            
            let hash = oid.to_string();
            let message = commit.message().unwrap_or("").to_string();
            
            Ok((Some(hash), Some(message)))
        } else {
            Ok((None, None))
        }
    }
    
    /// 获取项目的 Git 信息
    pub fn get_project_git_info(&self, project_name: &str) -> Result<Option<GitInfo>> {
        if let Some(project_path) = self.get_project_path(project_name)? {
            Ok(Some(self.get_git_info(&project_path)?))
        } else {
            Ok(None)
        }
    }
    
    /// 批量获取所有项目的 Git 信息
    pub fn get_all_projects_git_info(&self) -> Result<HashMap<String, GitInfo>> {
        let meta = self.get_meta_config()?;
        let mut git_info_map = HashMap::new();
        
        for (project_name, _) in &meta.projects {
            if let Ok(Some(git_info)) = self.get_project_git_info(project_name) {
                git_info_map.insert(project_name.clone(), git_info);
            }
        }
        
        Ok(git_info_map)
    }
      /// 检查项目是否在 Git 仓库中
    pub fn is_project_in_git(&self, project_name: &str) -> Result<bool> {
        if let Ok(Some(_git_info)) = self.get_project_git_info(project_name) {
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// 获取项目相对于 Git 根目录的路径
    pub fn get_project_git_relative_path(&self, project_name: &str) -> Result<Option<PathBuf>> {
        if let Ok(Some(git_info)) = self.get_project_git_info(project_name) {
            return Ok(Some(git_info.relative_path));
        }
        Ok(None)
    }
    
    /// 清理 Git 缓存
    pub fn clear_git_cache(&self) {
        let mut cache = self.git_cache.lock().unwrap();
        cache.clear();
    }
    
    /// 清理过期的 Git 缓存项
    pub fn cleanup_expired_git_cache(&self) {
        let mut cache = self.git_cache.lock().unwrap();
        let now = Instant::now();
        cache.retain(|_, (_, cached_time)| now.duration_since(*cached_time) < self.cache_ttl);
    }
}

impl RmmCore {    /// 从meta配置中移除项目
    pub fn remove_project_from_meta(&self, project_name: &str) -> Result<bool> {
        let mut meta = self.get_meta_config()?;
        let removed = meta.projects.remove(project_name).is_some();
        if removed {
            self.update_meta_config(&meta)?;
        }
        Ok(removed)
    }

    /// 从meta配置中移除多个项目
    pub fn remove_projects_from_meta(&self, project_names: &[&str]) -> Result<Vec<String>> {
        let mut meta = self.get_meta_config()?;
        let mut removed = Vec::new();
        
        for &project_name in project_names {
            if meta.projects.remove(project_name).is_some() {
                removed.push(project_name.to_string());
            }
        }
        
        if !removed.is_empty() {
            self.update_meta_config(&meta)?;
        }
        
        Ok(removed)
    }

    /// 移除所有无效的项目
    pub fn remove_invalid_projects(&self) -> Result<Vec<String>> {
        let validity = self.check_projects_validity()?;
        let invalid_projects: Vec<&str> = validity.iter()
            .filter(|(_, is_valid)| !**is_valid)
            .map(|(name, _)| name.as_str())
            .collect();
        
        self.remove_projects_from_meta(&invalid_projects)
    }

    /// 清理所有缓存
    pub fn clear_all_cache(&self) {
        self.clear_git_cache();
        // 注意：meta_cache 和 project_cache 清理在这里可以添加
        // 但目前只有 git_cache 的清理方法可用
    }

    /// 清除所有缓存，强制重新读取
    pub fn clear_cache(&self) {
        {
            let mut cache = self.meta_cache.lock().unwrap();
            *cache = None;
        }
        {
            let mut cache = self.project_cache.lock().unwrap();
            cache.clear();
        }
        {
            let mut cache = self.git_cache.lock().unwrap();
            cache.clear();
        }
    }
}
