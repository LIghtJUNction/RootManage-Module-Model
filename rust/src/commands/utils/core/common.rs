//! 通用工具模块
//! 
//! 集中管理项目、文件系统、命令执行等通用功能

use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::RmmConfig; // 添加缺失的导入

// ==================== 项目管理器 ====================

/// 项目管理器
pub struct ProjectManager;

impl ProjectManager {
    /// 查找项目配置文件
    /// 从给定目录开始向上搜索 rmmproject.toml 文件
    pub fn find_project_file(start_dir: &Path) -> Result<PathBuf> {
        let mut current_dir = start_dir;
        
        loop {
            let config_file = current_dir.join("rmmproject.toml");
            if config_file.exists() {
                return Ok(config_file);
            }
            
            // 向上一级目录
            match current_dir.parent() {
                Some(parent) => current_dir = parent,
                None => break,
            }
        }
        
        Err(anyhow!("未找到 rmmproject.toml 配置文件"))
    }

    /// 检查目录是否为 RMM 项目
    pub fn is_rmm_project(path: &Path) -> bool {
        path.join("rmmproject.toml").exists()
    }

    /// 获取项目根目录
    pub fn get_project_root(start_dir: &Path) -> Result<PathBuf> {
        let config_file = Self::find_project_file(start_dir)?;
        Ok(config_file.parent().unwrap().to_path_buf())
    }

    /// 验证项目配置
    pub fn validate_project_config(project_path: &Path) -> Result<()> {
        let config_file = project_path.join("rmmproject.toml");
        if !config_file.exists() {
            return Err(anyhow!("项目配置文件不存在: {}", config_file.display()));
        }
        
        // 可以在这里添加更多配置验证逻辑
        Ok(())
    }
}

// ==================== 文件系统管理器 ====================

/// 文件系统管理器
pub struct FileSystemManager;

impl FileSystemManager {
    /// 确保目录存在，如果不存在则创建
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

    /// 复制文件
    pub fn copy_file(src: &Path, dst: &Path) -> Result<()> {
        if let Some(parent) = dst.parent() {
            Self::ensure_dir_exists(parent)?;
        }
        fs::copy(src, dst)?;
        Ok(())
    }

    /// 格式化文件大小为人类可读的格式
    pub fn format_file_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        const THRESHOLD: u64 = 1024;
        
        let mut size = size as f64;
        let mut unit_index = 0;
        
        while size >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
            size /= THRESHOLD as f64;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.2} {}", size, UNITS[unit_index])
        }
    }

    /// 检查路径是否应该被排除
    pub fn should_exclude_path(path: &Path, exclude_items: &[&str]) -> bool {
        let path_str = path.to_string_lossy();
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        
        for exclude in exclude_items {
            if exclude.contains('*') {
                // 简单的通配符匹配
                if exclude.starts_with("*.") && file_name.ends_with(&exclude[1..]) {
                    return true;
                }
            } else if path_str.contains(exclude) || file_name == *exclude {
                return true;
            }
        }
        
        false
    }

    /// 使用 glob 模式匹配检查是否匹配
    pub fn matches_pattern(path: &Path, pattern: &str) -> bool {
        use glob::Pattern;
        if let Ok(glob_pattern) = Pattern::new(pattern) {
            glob_pattern.matches_path(path)
        } else {
            false
        }
    }

    /// 根据规则收集文件列表
    pub fn collect_files_with_rules(
        base_dir: &Path,
        include_rules: &[String],
        exclude_rules: &[String],
    ) -> Result<std::collections::HashSet<PathBuf>> {
        use walkdir::WalkDir;
        use std::collections::HashSet;

        let mut collected_files = HashSet::new();

        // 如果没有包含规则，则包含所有文件
        if include_rules.is_empty() {
            for entry in WalkDir::new(base_dir) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    let relative_path = entry.path().strip_prefix(base_dir)?;
                    collected_files.insert(relative_path.to_path_buf());
                }
            }
        } else {
            // 根据包含规则收集文件
            for include_rule in include_rules {
                if include_rule.contains('*') {
                    // 使用 glob 模式匹配
                    for entry in WalkDir::new(base_dir) {
                        let entry = entry?;
                        if entry.file_type().is_file() {
                            let relative_path = entry.path().strip_prefix(base_dir)?;
                            if Self::matches_pattern(relative_path, include_rule) {
                                collected_files.insert(relative_path.to_path_buf());
                            }
                        }
                    }
                } else {
                    // 直接路径匹配
                    let file_path = base_dir.join(include_rule);
                    if file_path.is_file() {
                        collected_files.insert(PathBuf::from(include_rule));
                    }
                }
            }
        }

        // 排除不需要的文件
        collected_files.retain(|path| {
            !exclude_rules.iter().any(|exclude_rule| {
                if exclude_rule.contains('*') {
                    Self::matches_pattern(path, exclude_rule)
                } else {
                    path.to_string_lossy().contains(exclude_rule)
                }
            })
        });

        Ok(collected_files)
    }

    /// 递归复制目录，支持排除规则
    pub fn copy_dir_recursive_with_exclusions(
        src_dir: &Path,
        dst_dir: &Path,
        exclude_items: &[&str],
    ) -> Result<()> {
        use walkdir::WalkDir;

        Self::ensure_dir_exists(dst_dir)?;

        for entry in WalkDir::new(src_dir) {
            let entry = entry?;
            let src_path = entry.path();
            let relative_path = src_path.strip_prefix(src_dir)?;
            let dst_path = dst_dir.join(relative_path);

            // 检查是否应该排除
            if Self::should_exclude_path(relative_path, exclude_items) {
                continue;
            }

            if src_path.is_dir() {
                Self::ensure_dir_exists(&dst_path)?;
            } else {
                Self::copy_file(src_path, &dst_path)?;
            }
        }

        Ok(())
    }

    /// 复制根目录文件到构建目录
    pub fn copy_root_files(
        project_root: &Path,
        build_dir: &Path,
        include_files: &[String],
        exclude_items: &[&str],
    ) -> Result<()> {
        println!("📋 复制根目录文件...");

        for file_pattern in include_files {
            if file_pattern.contains('*') {
                // 处理通配符模式
                use glob::glob;
                let pattern = project_root.join(file_pattern).to_string_lossy().to_string();
                for entry in glob(&pattern)? {
                    let src_path = entry?;
                    if src_path.is_file() {
                        let relative_path = src_path.strip_prefix(project_root)?;
                        if !Self::should_exclude_path(relative_path, exclude_items) {
                            let dst_path = build_dir.join(relative_path);
                            Self::copy_file(&src_path, &dst_path)?;
                            println!("  📄 {}", relative_path.display());
                        }
                    }
                }
            } else {
                // 处理直接文件路径
                let src_path = project_root.join(file_pattern);
                if src_path.exists() && src_path.is_file() {
                    let relative_path = src_path.strip_prefix(project_root)?;
                    if !Self::should_exclude_path(relative_path, exclude_items) {
                        let dst_path = build_dir.join(relative_path);
                        Self::copy_file(&src_path, &dst_path)?;
                        println!("  📄 {}", relative_path.display());
                    }
                }
            }
        }

        Ok(())
    }

    /// 复制 system 目录到构建目录
    pub fn copy_system_directory(
        project_root: &Path,
        build_dir: &Path,
        exclude_items: &[&str],
    ) -> Result<()> {
        let system_dir = project_root.join("system");
        if !system_dir.exists() {
            return Ok(());
        }

        println!("📋 复制 system 目录...");
        let build_system_dir = build_dir.join("system");
        Self::copy_dir_recursive_with_exclusions(&system_dir, &build_system_dir, exclude_items)?;
        
        Ok(())
    }

    /// 复制模块目录到构建目录
    pub fn copy_module_directories(
        project_root: &Path,
        build_dir: &Path,
        exclude_items: &[&str],
    ) -> Result<()> {
        // 复制 system 目录
        Self::copy_system_directory(project_root, build_dir, exclude_items)?;

        // 复制 META-INF 目录
        let meta_inf_dir = project_root.join("META-INF");
        if meta_inf_dir.exists() {
            println!("📋 复制 META-INF 目录...");
            let build_meta_inf_dir = build_dir.join("META-INF");
            Self::copy_dir_recursive_with_exclusions(&meta_inf_dir, &build_meta_inf_dir, exclude_items)?;
        }

        // 复制其他可能的模块目录
        let module_dirs = ["webroot", "zygisk", "riru", "addon.d"];
        for dir_name in &module_dirs {
            let module_dir = project_root.join(dir_name);
            if module_dir.exists() {
                println!("📋 复制 {} 目录...", dir_name);
                let build_module_dir = build_dir.join(dir_name);
                Self::copy_dir_recursive_with_exclusions(&module_dir, &build_module_dir, exclude_items)?;
            }
        }

        Ok(())
    }

    /// 复制模块文件到构建目录
    pub fn copy_module_files_to_build(
        project_root: &Path,
        build_dir: &Path,
        rmake_config: Option<&crate::commands::utils::core::RmakeConfig>,
        exclude_items: &[&str],
    ) -> Result<()> {
        // 获取包含文件列表
        let include_files = if let Some(rmake) = rmake_config {
            if let Some(ref package) = rmake.package {
                package.include.clone().unwrap_or_else(|| vec!["*".to_string()])
            } else {
                vec!["*".to_string()]
            }
        } else {
            vec!["*".to_string()]
        };

        // 如果包含所有文件，复制常见的模块目录和文件
        if include_files.contains(&"*".to_string()) {
            // 复制模块目录
            Self::copy_module_directories(project_root, build_dir, exclude_items)?;
            
            // 复制常见的模块文件
            let common_files = [
                "module.prop", "install.sh", "uninstall.sh", "service.sh",
                "post-fs-data.sh", "customize.sh", "update-binary"
            ];
            
            for file_name in &common_files {
                let src_path = project_root.join(file_name);
                if src_path.exists() {
                    let dst_path = build_dir.join(file_name);
                    Self::copy_file(&src_path, &dst_path)?;
                    println!("  📄 {}", file_name);
                }
            }
        } else {
            // 根据配置的包含文件列表复制
            Self::copy_root_files(project_root, build_dir, &include_files, exclude_items)?;
        }

        Ok(())
    }
}

// ==================== 命令执行器 ====================

/// 命令执行器
pub struct CommandExecutor;

impl CommandExecutor {
    /// 执行脚本命令
    pub fn execute_script_command(command: &str, working_dir: &Path) -> Result<()> {
        println!("🔧 在目录 {} 中执行: {}", working_dir.display(), command);
        
        #[cfg(target_os = "windows")]
        {
            let output = Command::new("powershell")
                .args(&["-Command", command])
                .current_dir(working_dir)
                .output()?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow!("脚本执行失败: {}", stderr));
            }
            
            // 输出命令结果
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                println!("{}", stdout);
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            let output = Command::new("sh")
                .args(&["-c", command])
                .current_dir(working_dir)
                .output()?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow!("脚本执行失败: {}", stderr));
            }
            
            // 输出命令结果
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                println!("{}", stdout);
            }
        }
        
        Ok(())
    }

    /// 运行外部命令
    pub fn run_command(command: &str, args: &[&str], cwd: Option<&Path>) -> Result<std::process::Output> {
        let mut cmd = Command::new(command);
        cmd.args(args);
        
        if let Some(working_dir) = cwd {
            cmd.current_dir(working_dir);
        }
        
        let output = cmd.output()?;
        Ok(output)
    }

    /// 检查命令是否可用
    pub fn is_command_available(command: &str) -> bool {
        Command::new(command)
            .arg("--version")
            .output()
            .is_ok()
    }

    /// 检查必需的工具是否可用
    pub fn check_required_tools() -> Result<Vec<String>> {
        let mut missing_tools = Vec::new();
        
        let tools = ["git"];
        for tool in &tools {
            if !Self::is_command_available(tool) {
                missing_tools.push(tool.to_string());
            }
        }
        
        Ok(missing_tools)
    }
}

// ==================== Git 管理器 ====================

/// Git 信息结构
#[derive(Debug, Clone)]
pub struct GitInfo {
    pub username: String,
    pub repo_name: String,
    pub remote_url: String,
    pub branch: String,
}

/// Git 管理器
pub struct GitManager;

impl GitManager {
    /// 获取 Git 信息
    pub fn get_git_info(project_path: &Path) -> Option<GitInfo> {
        // 检查是否为 Git 仓库
        if !project_path.join(".git").exists() {
            return None;
        }
        
        // 获取远程 URL
        let remote_url = Command::new("git")
            .args(&["remote", "get-url", "origin"])
            .current_dir(project_path)
            .output()
            .ok()?
            .stdout;
        
        let remote_url = String::from_utf8_lossy(&remote_url).trim().to_string();
        
        // 解析用户名和仓库名
        let (username, repo_name) = Self::parse_git_url(&remote_url)?;
        
        // 获取当前分支
        let branch = Command::new("git")
            .args(&["branch", "--show-current"])
            .current_dir(project_path)
            .output()
            .ok()?
            .stdout;
        
        let branch = String::from_utf8_lossy(&branch).trim().to_string();
        
        Some(GitInfo {
            username,
            repo_name,
            remote_url,
            branch,
        })
    }

    /// 解析 Git URL 获取用户名和仓库名
    fn parse_git_url(url: &str) -> Option<(String, String)> {
        // 简单的 GitHub URL 解析
        if url.contains("github.com") {
            // 处理 HTTPS URL: https://github.com/user/repo.git
            if let Some(start) = url.find("github.com/") {
                let path = &url[start + 11..];
                let parts: Vec<&str> = path.trim_end_matches(".git").split('/').collect();
                if parts.len() >= 2 {
                    return Some((parts[0].to_string(), parts[1].to_string()));
                }
            }
            // 处理 SSH URL: git@github.com:user/repo.git
            else if let Some(start) = url.find("github.com:") {
                let path = &url[start + 11..];
                let parts: Vec<&str> = path.trim_end_matches(".git").split('/').collect();
                if parts.len() >= 2 {
                    return Some((parts[0].to_string(), parts[1].to_string()));
                }
            }
        }
        
        None
    }

    /// 获取 Git 用户信息
    pub fn get_git_user_info() -> (String, String) {
        let name = Command::new("git")
            .args(&["config", "user.name"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "Unknown User".to_string());

        let email = Command::new("git")
            .args(&["config", "user.email"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "user@example.com".to_string());

        (name, email)
    }
}

// ==================== 版本管理器 ====================

/// 版本管理器
pub struct VersionManager;

impl VersionManager {
    /// 生成版本信息
    pub fn generate_version_info() -> Result<(String, String)> {
        // 使用系统时间生成版本信息
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        
        // 简单的版本生成：使用天数作为版本
        let days = now.as_secs() / (24 * 3600);
        let version = format!("v0.1.{}", days % 1000);
        let version_code = format!("{:010}", days);
        
        Ok((version, version_code))
    }
}

// ==================== 配置管理器 ====================

/// 配置管理器
pub struct ConfigManager;

impl ConfigManager {
    /// 验证配置
    pub fn validate_config(config: &RmmConfig) -> Result<()> {
        if config.username.trim().is_empty() {
            return Err(anyhow!("用户名不能为空"));
        }
        
        if config.email.trim().is_empty() {
            return Err(anyhow!("邮箱不能为空"));
        }
        
        Ok(())
    }

    /// 显示配置
    pub fn display_config(config: &RmmConfig) -> Result<String> {
        let output = format!(
            "RMM 配置:\n用户名: {}\n邮箱: {}\n版本: {}",
            config.username,
            config.email,
            config.version
        );
        Ok(output)
    }

    /// 加载或创建 Rmake 配置
    pub fn load_or_create_rmake_config(project_root: &Path) -> Result<Option<crate::commands::utils::core::RmakeConfig>> {
        let rmake_path = project_root.join(".rmmp").join("Rmake.toml");
        
        if rmake_path.exists() {
            // 加载现有配置
            println!("📋 加载 Rmake 配置: {}", rmake_path.display());
            match crate::commands::utils::core::RmakeConfig::load_from_dir(project_root) {
                Ok(Some(config)) => {
                    println!("✅ Rmake 配置加载成功");
                    Ok(Some(config))
                }
                Ok(None) => {
                    println!("⚠️  Rmake 配置文件为空或无效");
                    Ok(None)
                }
                Err(e) => {
                    println!("❌ 加载 Rmake 配置失败: {}", e);
                    Err(e)
                }
            }
        } else {
            println!("📋 未找到 Rmake.toml，将使用默认配置");
            Ok(None)
        }
    }

    /// 构建排除列表
    pub fn build_exclude_list(rmake_config: Option<&crate::commands::utils::core::RmakeConfig>) -> Vec<String> {
        let mut exclude_items = vec![
            ".git".to_string(),
            ".gitignore".to_string(),
            ".rmmp".to_string(),
            "dist".to_string(),
            "build".to_string(),
            "*.log".to_string(),
            "*.tmp".to_string(),
            ".DS_Store".to_string(),
            "Thumbs.db".to_string(),
        ];

        // 添加 Rmake 配置中的排除项
        if let Some(rmake) = rmake_config {
            if let Some(ref package) = rmake.package {
                if let Some(ref exclude) = package.exclude {
                    exclude_items.extend(exclude.clone());
                }
            }
        }

        println!("📋 排除文件列表: {:?}", exclude_items);
        exclude_items
    }
}
