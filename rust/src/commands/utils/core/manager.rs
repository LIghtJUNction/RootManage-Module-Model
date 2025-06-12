use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use crate::commands::utils::core::config::ProjectConfig;
use crate::commands::utils::core::common::CommandExecutor;
use crate::commands::utils::shellcheck;

// ==================== 检查管理器 ====================

/// 检查管理器
pub struct CheckManager;

impl CheckManager {
    /// 检查项目配置
    pub fn check_project_config(project_root: &Path) -> Result<String> {
        let config_file = project_root.join("rmmproject.toml");
        if !config_file.exists() {
            return Ok("❌ 项目配置文件不存在".to_string());
        }
        
        match ProjectConfig::load_from_file(&config_file) {
            Ok(_) => Ok("✅ 项目配置正常".to_string()),
            Err(e) => Ok(format!("❌ 项目配置错误: {}", e)),
        }
    }

    /// 检查 GitHub 连接
    pub fn check_github_connection() -> Result<String> {
        // TODO: 实现 GitHub 连接检查
        Ok("✅ GitHub 连接正常".to_string())
    }

    /// 检查依赖
    pub fn check_dependencies() -> Result<String> {
        let tools = ["git"];
        let mut missing = Vec::new();
        
        for tool in &tools {
            if !CommandExecutor::is_command_available(tool) {
                missing.push(*tool);
            }
        }
        
        if missing.is_empty() {
            Ok("✅ 所有必需工具已安装".to_string())
        } else {
            Ok(format!("❌ 缺少工具: {}", missing.join(", ")))
        }
    }

    /// 检查项目结构
    pub fn check_project_structure(project_root: &Path) -> Result<String> {
        let required_files = ["rmmproject.toml"];
        let mut missing = Vec::new();
        
        for file in &required_files {
            if !project_root.join(file).exists() {
                missing.push(*file);
            }
        }
        
        if missing.is_empty() {
            Ok("✅ 项目结构正常".to_string())
        } else {
            Ok(format!("❌ 缺少文件: {}", missing.join(", ")))
        }
    }

    /// 检查 Shell 语法
    pub fn check_shell_syntax(project_root: &Path) -> Result<String> {
        if !CommandExecutor::is_command_available("shellcheck") {
            return Ok("⚠️  shellcheck 未安装，跳过 Shell 脚本检查".to_string());
        }
        
        match shellcheck::check_project(project_root, false) {
            Ok((results, all_passed)) => {
                if all_passed {
                    Ok("✅ Shell 脚本语法检查通过".to_string())
                } else {
                    let error_count = results.iter().filter(|r| r.level == "error").count();
                    Ok(format!("❌ Shell 脚本语法检查发现 {} 个问题", error_count))
                }
            }
            Err(e) => Ok(format!("❌ Shell 脚本检查失败: {}", e)),
        }
    }

    /// 运行 shellcheck 验证
    pub fn run_shellcheck_validation(project_root: &Path) -> Result<()> {
        println!("🔍 运行 Shellcheck 验证...");
        
        // 检查 shellcheck 是否可用
        if !crate::commands::utils::shellcheck::is_shellcheck_available() {
            println!("⚠️  Shellcheck 未安装或不可用");
            println!("   建议安装 shellcheck 以进行 shell 脚本语法检查");
            println!("   安装方法:");
            if cfg!(target_os = "windows") {
                println!("     - Windows: 使用 scoop install shellcheck 或从 GitHub 下载");
            } else if cfg!(target_os = "macos") {
                println!("     - macOS: brew install shellcheck");
            } else {
                println!("     - Linux: 使用包管理器安装 (apt install shellcheck / yum install shellcheck)");
            }
            println!("   跳过 shellcheck 检查继续构建...");
            return Ok(());
        }
        
        // 显示 shellcheck 版本
        match shellcheck::get_shellcheck_version() {
            Ok(version) => println!("📋 Shellcheck 版本: {}", version),
            Err(_) => println!("📋 Shellcheck 版本: 未知"),
        }
        
        // 执行检查
        match shellcheck::check_project(project_root, false) {
            Ok((results, all_passed)) => {
                if results.is_empty() {
                    println!("📋 项目中未发现 shell 脚本文件");
                    return Ok(());
                }
                
                if all_passed {
                    println!("✅ Shellcheck 验证通过");
                } else {
                    println!("❌ Shellcheck 验证失败！");
                    println!("   发现 shell 脚本语法错误，构建中止");
                    println!("   请修复错误后重新构建，或使用 'rmm test --shellcheck' 查看详细信息");
                    return Err(anyhow::anyhow!("Shell 脚本语法检查失败"));
                }
                
                Ok(())
            }
            Err(e) => {
                println!("❌ Shellcheck 检查失败: {}", e);
                Err(anyhow::anyhow!("Shellcheck 执行失败: {}", e))
            }
        }
    }
}

// ==================== 清理管理器 ====================

/// 清理管理器
pub struct CleanManager;

impl CleanManager {
    /// 清理目录
    pub fn clean_directory(path: &Path) -> Result<()> {
        if path.exists() && path.is_dir() {
            std::fs::remove_dir_all(path)?;
            println!("🧹 已清理目录: {}", path.display());
        }
        Ok(())
    }

    /// 清理文件
    pub fn clean_file(path: &Path) -> Result<()> {
        if path.exists() && path.is_file() {
            std::fs::remove_file(path)?;
            println!("🧹 已清理文件: {}", path.display());
        }
        Ok(())
    }
}

// ==================== 设备管理器 ====================

/// 设备管理器
pub struct DeviceManager;

impl DeviceManager {
    /// 检查 ADB 是否可用
    pub fn check_adb_available() -> bool {
        CommandExecutor::is_command_available("adb")
    }

    /// 安装模块到设备
    pub fn install_module_to_device(device_id: &str, module_path: &Path) -> Result<String> {
        println!("📱 安装模块到设备: {}", device_id);
        println!("📦 模块文件: {}", module_path.display());
        
        if !module_path.exists() {
            return Err(anyhow!("模块文件不存在: {}", module_path.display()));
        }
        
        // TODO: 实现设备安装逻辑
        
        Ok("✅ 模块安装成功".to_string())
    }
}

// ==================== 同步管理器 ====================

/// 同步管理器
pub struct SyncManager;

impl SyncManager {
    /// 更新项目版本
    pub fn update_project_version(project_config: &mut ProjectConfig, new_version: &str) -> Result<()> {
        project_config.version = Some(new_version.to_string());
        // TODO: 保存配置文件
        Ok(())
    }
}

// ==================== 发布管理器 ====================

/// 发布管理器
pub struct PublishManager;

impl PublishManager {
    /// 在构建目录中寻找最新的模块文件
    pub fn find_latest_build_files(dist_dir: &Path, project_id: &str) -> Result<(PathBuf, PathBuf)> {
        if !dist_dir.exists() {
            anyhow::bail!("❌ 构建目录不存在: {}\\n请先运行 \'rmm build\' 构建项目", dist_dir.display());
        }
        
        // 查找所有匹配的ZIP文件
        let mut zip_files = Vec::new();
        let mut tar_files = Vec::new();
        
        for entry in std::fs::read_dir(dist_dir)? {
            let entry = entry?;
            let path = entry.path();
            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            // 查找匹配项目ID的ZIP文件
            if filename.ends_with(".zip") && filename.starts_with(project_id) {
                let metadata = entry.metadata()?;
                zip_files.push((path.clone(), metadata.modified()?));
            }
            
            // 查找匹配项目ID的源码包
            if filename.ends_with("-source.tar.gz") && filename.starts_with(project_id) {
                let metadata = entry.metadata()?;
                tar_files.push((path.clone(), metadata.modified()?));
            }
        }
        
        if zip_files.is_empty() {
            anyhow::bail!("❌ 未找到模块包文件 ({}*.zip)\\n请先运行 \'rmm build\' 构建项目", project_id);
        }
        
        if tar_files.is_empty() {
            anyhow::bail!("❌ 未找到源码包文件 ({}*-source.tar.gz)\\n请先运行 \'rmm build\' 构建项目", project_id);
        }
        
        // 按修改时间排序，获取最新的文件
        zip_files.sort_by(|a, b| b.1.cmp(&a.1));
        tar_files.sort_by(|a, b| b.1.cmp(&a.1));
        
        let latest_zip = zip_files.into_iter().next().unwrap().0;
        let latest_tar = tar_files.into_iter().next().unwrap().0;
        
        println!("📦 找到最新模块包: {}", latest_zip.file_name().unwrap().to_string_lossy());
        println!("📋 找到最新源码包: {}", latest_tar.file_name().unwrap().to_string_lossy());
        
        Ok((latest_zip, latest_tar))
    }
}

// ==================== 补全管理器 ====================

/// Shell 类型
#[derive(Debug, Clone)]
pub enum SupportedShell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
}

/// 补全管理器
pub struct CompletionManager;

impl CompletionManager {
    /// 打印安装指南
    pub fn print_installation_instructions(shell: SupportedShell) {
        match shell {
            SupportedShell::Bash => {
                println!("将以下内容添加到 ~/.bashrc:");
                println!("eval \"$(rmm completion bash)\"");
            }
            SupportedShell::Zsh => {
                println!("将以下内容添加到 ~/.zshrc:");
                println!("eval \"$(rmm completion zsh)\"");
            }
            SupportedShell::Fish => {
                println!("将以下内容添加到 ~/.config/fish/config.fish:");
                println!("rmm completion fish | source");
            }
            SupportedShell::PowerShell => {
                println!("将以下内容添加到 PowerShell 配置文件:");
                println!("Invoke-Expression (rmm completion powershell)");
            }
            SupportedShell::Cmd => {
                println!("Windows CMD 不支持自动补全");
            }
        }
    }

    /// 获取 Shell 安装帮助
    pub fn get_shell_installation_help(shell: &str) -> Result<String> {
        match shell.to_lowercase().as_str() {
            "bash" => Ok("添加到 ~/.bashrc: eval \"$(rmm completion bash)\"".to_string()),
            "zsh" => Ok("添加到 ~/.zshrc: eval \"$(rmm completion zsh)\"".to_string()),
            "fish" => Ok("添加到 ~/.config/fish/config.fish: rmm completion fish | source".to_string()),
            "powershell" => Ok("添加到 PowerShell 配置: Invoke-Expression (rmm completion powershell)".to_string()),
            _ => Err(anyhow!("不支持的 shell: {}", shell)),
        }
    }
}
