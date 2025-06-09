use std::path::PathBuf;
use std::collections::HashMap;
use anyhow::{Result, Context as AnyhowContext};
use serde::{Deserialize, Serialize};
use colored::*;
use pyo3::prelude::*;

/// 全局上下文，包含命令行全局选项
#[pyclass]
#[derive(Debug, Clone)]
pub struct Context {
    #[pyo3(get, set)]
    pub profile: Option<String>,
    #[pyo3(get, set)]
    pub token: Option<String>,
    #[pyo3(get, set)]
    pub debug: bool,
}

#[pymethods]
impl Context {    #[new]
    pub fn new(profile: Option<String>, token: Option<String>, debug: bool) -> Self {
        Context { profile, token, debug }
    }
    /// 输出调试信息
    pub fn debug(&self, msg: &str) {
        if self.debug {
            eprintln!("{} {}", "[DEBUG]".cyan().bold(), msg);
        }
    }

    /// 输出信息
    pub fn info(&self, msg: &str) {
        println!("{} {}", "[INFO]".green().bold(), msg);
    }

    /// 输出警告
    pub fn warn(&self, msg: &str) {
        eprintln!("{} {}", "[WARN]".yellow().bold(), msg);
    }

    /// 输出错误
    pub fn error(&self, msg: &str) {
        eprintln!("{} {}", "[ERROR]".red().bold(), msg);
    }
}

/// RMM项目配置结构
#[derive(Debug, Serialize, Deserialize)]
pub struct RmmProject {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub id: Option<String>,
    pub versionCode: Option<u32>,
    pub updateJson: Option<String>,
    pub dependencies: Option<HashMap<String, String>>,
    pub build: Option<BuildConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildConfig {
    pub output: Option<String>,
    pub scripts: Option<HashMap<String, String>>,
}

impl RmmProject {
    /// 从文件加载项目配置
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("无法读取项目配置文件: {}", path.as_ref().display()))?;
        
        toml::from_str(&content)
            .with_context(|| "解析项目配置文件失败")
    }

    /// 查找当前目录或父目录中的项目配置文件
    pub fn find_project_file() -> Result<PathBuf> {
        let mut current_dir = std::env::current_dir()?;
        
        loop {
            let rmm_toml = current_dir.join("rmm.toml");
            if rmm_toml.exists() {
                return Ok(rmm_toml);
            }
            
            if !current_dir.pop() {
                break;
            }
        }
        
        anyhow::bail!("在当前目录或任何父目录中都未找到 rmm.toml 文件");
    }

    /// 加载当前项目配置
    pub fn load_current() -> Result<Self> {
        let project_file = Self::find_project_file()?;
        Self::load_from_file(project_file)
    }

    /// 保存项目配置到文件
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .with_context(|| "序列化项目配置失败")?;
        
        std::fs::write(path.as_ref(), content)
            .with_context(|| format!("无法写入项目配置文件: {}", path.as_ref().display()))?;
        
        Ok(())
    }
}

/// 配置管理器
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    /// 获取配置文件路径
    pub fn config_file_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
        
        Ok(home_dir.join(".rmmconfig"))
    }

    /// 加载配置
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        
        if !config_path.exists() {
            return Ok(Self::default());
        }
        
        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("无法读取配置文件: {}", config_path.display()))?;
        
        toml::from_str(&content)
            .with_context(|| "解析配置文件失败")
    }

    /// 保存配置
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;
        
        // 确保父目录存在
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)
            .with_context(|| "序列化配置失败")?;
        
        std::fs::write(&config_path, content)
            .with_context(|| format!("无法写入配置文件: {}", config_path.display()))?;
        
        Ok(())
    }

    /// 获取配置值
    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    /// 设置配置值
    pub fn set(&mut self, key: String, value: String) {
        self.settings.insert(key, value);
    }

    /// 删除配置值
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.settings.remove(key)
    }

    /// 列出所有配置
    pub fn list(&self) -> &HashMap<String, String> {
        &self.settings
    }
}

/// 执行命令行工具
pub fn run_command(cmd: &str, args: &[&str], working_dir: Option<&std::path::Path>) -> Result<String> {
    use std::process::Command;
    
    let mut command = Command::new(cmd);
    command.args(args);
    
    if let Some(dir) = working_dir {
        command.current_dir(dir);
    }
    
    let output = command.output()
        .with_context(|| format!("执行命令失败: {} {}", cmd, args.join(" ")))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("命令执行失败: {}", stderr);
    }
    
    let stdout = String::from_utf8(output.stdout)
        .with_context(|| "命令输出包含无效的UTF-8字符")?;
    
    Ok(stdout)
}

/// 确保目录存在
pub fn ensure_dir_exists<P: AsRef<std::path::Path>>(path: P) -> Result<()> {
    std::fs::create_dir_all(path.as_ref())
        .with_context(|| format!("无法创建目录: {}", path.as_ref().display()))?;
    Ok(())
}

/// 删除目录及其内容
pub fn remove_dir_all<P: AsRef<std::path::Path>>(path: P) -> Result<()> {
    if path.as_ref().exists() {
        std::fs::remove_dir_all(path.as_ref())
            .with_context(|| format!("无法删除目录: {}", path.as_ref().display()))?;
    }
    Ok(())
}

/// 检查文件是否存在
pub fn file_exists<P: AsRef<std::path::Path>>(path: P) -> bool {
    path.as_ref().exists() && path.as_ref().is_file()
}

/// 检查目录是否存在
pub fn dir_exists<P: AsRef<std::path::Path>>(path: P) -> bool {
    path.as_ref().exists() && path.as_ref().is_dir()
}
