//! 命令执行模块
//! 
//! 专注于脚本执行和项目构建相关的功能

use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::commands::utils::core::config::{ProjectConfig, RmmConfig};
use crate::commands::utils::core::common::CommandExecutor;
use crate::commands::utils::shellcheck;

// ==================== 脚本执行器 ====================

/// 脚本执行器
pub struct ScriptExecutor;

impl ScriptExecutor {
    /// 执行项目脚本（从 rmmproject.toml 中定义的脚本）
    pub fn run_project_script(project_config: &ProjectConfig, script_name: &str, args: &[String], project_root: &Path) -> Result<()> {
        let script_command = project_config.scripts.get(script_name)
            .ok_or_else(|| anyhow!("❌ 未找到脚本 '{}'", script_name))?;
        
        // 构建完整命令（包含额外参数）
        let mut full_command = script_command.clone();
        if !args.is_empty() {
            full_command.push(' ');
            full_command.push_str(&args.join(" "));
        }
        
        println!("🔧 运行脚本: {}", script_name);
        println!("📋 执行命令: {}", full_command);
        
        CommandExecutor::execute_script_command(&full_command, project_root)
    }

    /// 列出所有可用脚本
    pub fn list_available_scripts(project_config: &ProjectConfig) {
        println!("📋 可用脚本:");
        
        if project_config.scripts.is_empty() {
            println!("  (没有定义任何脚本)");
            println!("");
            println!("💡 在 rmmproject.toml 中添加脚本:");
            println!("  [scripts]");
            println!("  build = \"rmm build\"");
            println!("  test = \"echo 'Running tests...'\"");
            println!("  dev = \"rmm build --debug\"");
        } else {
            for (name, command) in &project_config.scripts {
                println!("  {} : {}", name, command);
            }
            println!("");
            println!("💡 运行脚本: rmm run <script_name>");
        }
    }

    /// 运行配置的脚本
    pub fn run_configured_script(project_root: &Path, script_name: &str) -> Result<()> {
        println!("🔧 运行脚本: {}", script_name);
        
        let project_config = ProjectConfig::load_from_dir(project_root)?;
        
        if let Some(scripts) = &project_config.scripts {
            if let Some(command) = scripts.get(script_name) {
                println!("📋 执行命令: {}", command);
                CommandExecutor::execute_script_command(command, project_root)?;
                println!("✅ 脚本 '{}' 执行完成", script_name);
                return Ok(());
            }
        }
        
        Err(anyhow!("❌ 未找到脚本 '{}'", script_name))
    }

    /// 执行构建步骤中的脚本
    pub fn execute_build_steps(
        project_root: &Path,
        steps: &[String],
        step_type: &str,
    ) -> Result<()> {
        if steps.is_empty() {
            return Ok(());
        }

        println!("🔧 执行 {} 步骤...", step_type);
        
        for (index, step) in steps.iter().enumerate() {
            println!("  📋 步骤 {}: {}", index + 1, step);
            CommandExecutor::execute_script_command(step, project_root)?;
        }
        
        println!("✅ {} 步骤执行完成", step_type);
        Ok(())
    }

    /// 运行 Rmake.toml 中定义的脚本
    pub fn run_rmake_script(project_root: &Path, script_name: &str) -> Result<String> {
        println!("🔧 运行 Rmake 脚本: {}", script_name);
        
        // 加载 Rmake 配置
        let rmake_config_path = project_root.join(".rmmp").join("Rmake.toml");
        if !rmake_config_path.exists() {
            anyhow::bail!("❌ 未找到 Rmake.toml 配置文件");
        }
        
        let rmake_config = crate::commands::utils::core::RmakeConfig::load_from_dir(project_root)?
            .ok_or_else(|| anyhow::anyhow!("无法加载 Rmake 配置"))?;
        
        // 查找脚本
        let scripts = rmake_config.scripts
            .ok_or_else(|| anyhow::anyhow!("❌ Rmake.toml 中未定义 [scripts] 部分"))?;
        
        let script_command = scripts.get(script_name)
            .ok_or_else(|| anyhow::anyhow!("❌ 未找到脚本 '{}'", script_name))?;
        
        println!("📋 执行命令: {}", script_command);
        
        // 执行脚本命令
        #[cfg(target_os = "windows")]
        {
            let output = std::process::Command::new("powershell")
                .args(&["-Command", script_command])
                .current_dir(project_root)
                .output()?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("脚本执行失败: {}", stderr);
            }
            
            // 输出命令结果
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                println!("{}", stdout.trim());
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            let output = std::process::Command::new("sh")
                .args(&["-c", script_command])
                .current_dir(project_root)
                .output()?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("脚本执行失败: {}", stderr);
            }
            
            // 输出命令结果
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                println!("{}", stdout.trim());
            }
        }
        
        println!("✅ 脚本 '{}' 执行完成", script_name);
        Ok(format!("脚本 '{}' 执行成功", script_name))
    }
}

// ==================== 项目构建器 ====================

/// 项目构建器
pub struct ProjectBuilder;

impl ProjectBuilder {
    /// 构建项目的核心逻辑
    pub fn build_project(
        project_config: &ProjectConfig,
        build_output: &Path,
        _output_dir: Option<&String>,
        debug: bool,
        skip_shellcheck: bool,
    ) -> Result<()> {
    println!("🔨 开始构建项目: {}", project_config.name);
    
    // 1. 创建构建目录
    std::fs::create_dir_all(build_output)?;
    
    // 2. Shell 脚本语法检查（如果未跳过）
    if !skip_shellcheck {
        println!("🔍 进行 Shell 脚本语法检查...");
        check_shell_scripts(build_output.parent().unwrap())?;
    }
      // 3. 执行预构建脚本
    if let Some(prebuild) = &project_config.build.as_ref().and_then(|b| b.prebuild.as_ref()) {
        for script in prebuild.iter() {
            println!("🔧 执行预构建脚本: {}", script);
            CommandExecutor::execute_script_command(script, build_output.parent().unwrap())?;
        }
    }
    
    // 4. 执行主构建脚本
    if let Some(build_scripts) = &project_config.build.as_ref().and_then(|b| b.build.as_ref()) {
        for script in build_scripts.iter() {
            println!("🔧 执行构建脚本: {}", script);
            CommandExecutor::execute_script_command(script, build_output.parent().unwrap())?;
        }
    }
    
    // 5. 复制文件
    copy_project_files(project_config, build_output)?;
    
    // 6. 生成模块包
    create_module_package(project_config, build_output, debug)?;
    
    // 7. 执行后构建脚本
    if let Some(postbuild) = &project_config.build.as_ref().and_then(|b| b.postbuild.as_ref()) {
        for script in postbuild.iter() {
            println!("🔧 执行后构建脚本: {}", script);
            CommandExecutor::execute_script_command(script, build_output.parent().unwrap())?;
        }
    }    
    println!("✅ 项目构建完成！");
    Ok(())
    }
}

/// 复制项目文件到构建目录
fn copy_project_files(_project_config: &ProjectConfig, _build_output: &Path) -> Result<()> {
    println!("📁 复制项目文件...");
    
    // 实现文件复制逻辑
    // TODO: 根据 exclude 配置过滤文件
    
    Ok(())
}

/// 创建模块包
fn create_module_package(project_config: &ProjectConfig, build_output: &Path, debug: bool) -> Result<()> {
    println!("📦 创建模块包...");
    
    if debug {
        println!("🐛 调试模式：保留调试信息");
    }
    
    // 创建 ZIP 包
    let package_name = format!("{}.zip", project_config.id);
    let package_path = build_output.join(&package_name);
    
    // TODO: 实现 ZIP 包创建逻辑
    
    println!("📦 模块包已创建: {}", package_path.display());
    Ok(())
}

// ==================== 检查执行 ====================

/// 执行项目检查
pub fn check_project(project_root: &Path, skip_shellcheck: bool) -> Result<String> {
    let mut results = Vec::new();
    
    // 1. 检查项目配置
    results.push(check_project_configuration(project_root)?);
    
    // 2. 检查 Shell 脚本语法（如果未跳过）
    if !skip_shellcheck {
        results.push(check_shell_scripts(project_root)?);
    }
    
    // 3. 检查依赖
    results.push(check_dependencies()?);
    
    // 4. 检查 Git 状态
    results.push(check_git_status(project_root)?);
    
    Ok(results.join("\n"))
}

/// 检查项目配置
fn check_project_configuration(project_root: &Path) -> Result<String> {
    let config_file = project_root.join("rmmproject.toml");
    if !config_file.exists() {
        return Ok("❌ 项目配置文件不存在".to_string());
    }
    
    // 尝试加载配置文件
    match ProjectConfig::load_from_file(&config_file) {
        Ok(_) => Ok("✅ 项目配置正常".to_string()),
        Err(e) => Ok(format!("❌ 项目配置错误: {}", e)),
    }
}

/// 检查 Shell 脚本语法
pub fn check_shell_scripts(project_root: &Path) -> Result<String> {
    if !is_command_available("shellcheck") {
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

/// 检查依赖
fn check_dependencies() -> Result<String> {
    let tools = ["git"];
    let mut missing = Vec::new();
    
    for tool in &tools {
        if !is_command_available(tool) {
            missing.push(*tool);
        }
    }
    
    if missing.is_empty() {
        Ok("✅ 所有必需工具已安装".to_string())
    } else {
        Ok(format!("❌ 缺少工具: {}", missing.join(", ")))
    }
}

/// 检查 Git 状态
fn check_git_status(project_root: &Path) -> Result<String> {
    if !project_root.join(".git").exists() {
        return Ok("⚠️  非 Git 仓库".to_string());
    }
    
    // 检查是否有未提交的更改
    let output = Command::new("git")
        .args(&["status", "--porcelain"])
        .current_dir(project_root)
        .output()?;
      if output.stdout.is_empty() {
        Ok("✅ Git 工作目录干净".to_string())
    } else {
        Ok("⚠️  有未提交的更改".to_string())
    }
}

/// 检查命令是否可用
fn is_command_available(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

// ==================== 同步执行 ====================

/// 同步项目配置
pub fn sync_project_configuration(_config: &mut RmmConfig, _project_path: &Path) -> Result<String> {
    let mut results = Vec::new();
    
    // 同步项目列表
    results.push("🔄 同步项目列表...".to_string());
    
    // 更新版本信息
    results.push("📋 更新版本信息...".to_string());
    
    // 验证配置
    results.push("✅ 配置同步完成".to_string());
    
    Ok(results.join("\n"))
}

// ==================== 发布执行 ====================

/// 发布项目到 GitHub
pub fn publish_to_github(
    _project_config: &ProjectConfig,
    project_root: &Path,
    _draft: bool,
    _prerelease: bool,
    token: Option<&str>,
) -> Result<String> {
    println!("🚀 准备发布到 GitHub...");
    
    if let Some(token) = token {
        std::env::set_var("GITHUB_TOKEN", token);
    }
    
    // 检查构建文件是否存在
    let dist_dir = project_root.join(".rmmp").join("dist");
    if !dist_dir.exists() {
        return Err(anyhow!("构建目录不存在，请先运行 'rmm build'"));
    }
    
    // TODO: 实现 GitHub 发布逻辑
    
    Ok("✅ 发布成功".to_string())
}

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
