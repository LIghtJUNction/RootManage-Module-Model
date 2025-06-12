//! 核心数据结构和配置模块
//! 
//! 包含 RMM 项目的核心配置结构、项目配置、构建配置等

pub mod config;
pub mod project;
pub mod rmake;
pub mod adb;
pub mod common;   // 新增：通用工具函数
pub mod executor; // 新增：命令执行模块
pub mod package;  // 新增：包管理模块

// Re-export key configurations and utilities
pub use rmake::RmakeConfig;
pub use common::{CommandExecutor, ConfigManager, FileSystemManager, GitManager, ProjectManager, VersionManager};
pub use executor::{CheckManager, CleanManager, CompletionManager, DeviceManager, ProjectBuilder, PublishManager, ScriptExecutor, SyncManager};
pub use project::{ProjectConfig};
pub use config::{RmmConfig};
pub use package::{PackageManager, PackageInfo};