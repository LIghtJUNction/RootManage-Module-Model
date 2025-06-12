//! 项目配置数据结构
//! 
//! 重新导出核心配置模块中的项目相关结构

// 从核心配置模块导入项目相关的配置结构
pub use super::config::{
    ProjectConfig, 
    Author, 
    Urls, 
    BuildConfig, 
    GitInfo,
};

use anyhow::Result;
use std::path::{Path, PathBuf};

/// 查找项目配置文件
pub fn find_project_config(start_dir: &Path) -> Result<PathBuf> {
    crate::commands::utils::utils::find_project_file(start_dir)
}

