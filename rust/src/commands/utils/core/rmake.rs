//! RMake 配置模块
//! 
//! 包含构建和打包相关的配置结构

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use std::path::Path;
use std::fs;

/// RMake 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RmakeConfig {
    pub build: BuildConfig,
    pub package: Option<PackageConfig>,
    pub scripts: Option<HashMap<String, String>>,
    pub proxy: Option<ProxyConfig>,
    pub source_package: Option<SourcePackageConfig>, // Added source_package
}

/// 构建配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub target: Option<String>,
    pub output_dir: Option<String>,
    pub exclude: Option<Vec<String>>,
    pub include: Option<Vec<String>>,
    pub pre_build: Option<Vec<String>>,
    pub post_build: Option<Vec<String>>,
}

/// 打包配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    pub compression: Option<String>,
    pub zip_name: Option<String>,
}

/// 代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub auto_select: Option<bool>,
    pub custom_proxy: Option<String>,
}

/// Source Package Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcePackageConfig {
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub name_template: Option<String>, // e.g., "{id}-{version}-source.tar.gz"
}

impl Default for RmakeConfig {
    fn default() -> Self {
        Self {
            build: BuildConfig::default(),
            package: Some(PackageConfig::default()),
            scripts: Some(HashMap::new()), // Changed from None
            proxy: Some(ProxyConfig::default()),
            source_package: Some(SourcePackageConfig::default()), // Added default for source_package
        }
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            target: None, // Changed from "magisk" to None - target should not be a command
            output_dir: Some("dist".to_string()),
            exclude: Some(vec![
                ".git".to_string(),
                "target".to_string(),
                "*.log".to_string(),
                ".vscode".to_string(),
                ".idea".to_string(),
                "node_modules".to_string(),
                "__pycache__".to_string(),
            ]),
            include: Some(Vec::new()), // Changed from None
            pre_build: Some(vec!["echo 'Pre-build step'".to_string()]), // Added default pre_build
            post_build: Some(vec!["echo 'Post-build step'".to_string()]), // Added default post_build
        }
    }
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            compression: Some("deflate".to_string()),
            zip_name: Some("default".to_string()),
        }
    }
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: true, // User preference from prompt
            auto_select: Some(true),
            custom_proxy: None,
        }
    }
}

impl Default for SourcePackageConfig {
    fn default() -> Self {
        Self {
            include: Some(Vec::new()), // By default, include nothing specific, meaning exclude logic will be primary
            exclude: Some(vec![         // Common exclusions for source archives
                ".git/".to_string(),
                ".rmmp/".to_string(),
                "target/".to_string(),
                "dist/".to_string(),
                "build/".to_string(),
                "__pycache__/".to_string(),
                "*.pyc".to_string(),
                "*.pyd".to_string(),
                "*.zip".to_string(),
                "*.tar.gz".to_string(), // Exclude other archives
                "*.log".to_string(),
                ".DS_Store".to_string(),
            ]),
            name_template: Some("{id}-{version_code}-source.tar.gz".to_string()),
        }
    }
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