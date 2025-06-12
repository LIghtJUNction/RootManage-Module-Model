

use anyhow::Result;
use crate::commands::utils::core::config::RmmConfig;
use crate::commands::utils::core::project::{ProjectConfig, find_project_config};
use crate::commands::utils::shellcheck;
use std::path::Path;

/// 检查项目配置
pub fn check_project_config() -> Result<()> {
    println!("\n📋 检查项目配置...");
    
    let current_dir = std::env::current_dir()?;
    let config_path = find_project_config(&current_dir);
    
    match config_path {
        Ok(path) => {
            println!("✓ 找到项目配置: {}", path.display());
            
            // 尝试加载配置
            match ProjectConfig::load_from_file(&path) {                Ok(config) => {
                    println!("✓ 配置文件格式正确");
                    println!("  项目名: {}", config.name);
                    println!("  项目ID: {}", config.id);
                    println!("  版本: {}", config.version.as_ref().unwrap_or(&"未设置".to_string()));
                    println!("  版本代码: {}", config.version_code);
                    println!("  作者: {}", config.authors.first().map(|a| a.name.as_str()).unwrap_or("未设置"));
                }
                Err(e) => {
                    println!("✗ 配置文件格式错误: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ {}", e);
        }
    }
    
    Ok(())
}

/// 检查 GitHub 连接
pub fn check_github_connection(_config: &RmmConfig) -> Result<()> {
    println!("\n🐙 检查 GitHub 连接...");
    
    // 检查 GitHub token
    if let Ok(token) = std::env::var("GITHUB_ACCESS_TOKEN") {
        if !token.is_empty() {
            println!("✓ 找到 GitHub Access Token");
            
            // 这里可以添加实际的 GitHub API 连接测试
            println!("  (GitHub API 连接测试需要实现)");
        } else {
            println!("⚠ GITHUB_ACCESS_TOKEN 环境变量为空");
        }
    } else {
        println!("⚠ 未设置 GITHUB_ACCESS_TOKEN 环境变量");
        println!("  提示: 设置此变量以启用 GitHub 功能");
    }
    
    // 检查 Git 仓库
    if Path::new(".git").exists() {
        println!("✓ 当前目录是 Git 仓库");
        
        // 检查远程仓库
        if let Ok(output) = std::process::Command::new("git")
            .args(&["remote", "get-url", "origin"])
            .output()
        {            if output.status.success() {
                let remote_url_bytes = String::from_utf8_lossy(&output.stdout);
                let remote_url = remote_url_bytes.trim();
                println!("✓ 远程仓库: {}", remote_url);
                
                if remote_url.contains("github.com") {
                    println!("✓ 这是一个 GitHub 仓库");
                } else {
                    println!("⚠ 这不是 GitHub 仓库");
                }
            }
        }
    } else {
        println!("⚠ 当前目录不是 Git 仓库");
    }
    
    Ok(())
}

/// 检查依赖项
pub fn check_dependencies() -> Result<()> {
    println!("\n📦 检查依赖项...");
    
    let current_dir = std::env::current_dir()?;
    if let Ok(config_path) = find_project_config(&current_dir) {
        if let Ok(config) = ProjectConfig::load_from_file(&config_path) {
            if config.dependencies.is_empty() {
                println!("ℹ 项目无依赖项");
            } else {
                println!("依赖项列表:");
                for dep in &config.dependencies {
                    println!("  - {} ({})", dep.name, dep.version);
                    
                    // 检查依赖是否存在
                    let dep_path = Path::new("deps").join(&dep.name);
                    if dep_path.exists() {
                        println!("    ✓ 已安装");
                    } else {
                        println!("    ✗ 未安装");
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// 检查项目结构
pub fn check_project_structure() -> Result<()> {
    println!("\n📁 检查项目结构...");
    
    let required_files = [
        ("module.prop", "模块属性文件", true),
        ("customize.sh", "安装脚本", false),
        ("system/", "系统文件目录", false),
        ("README.MD", "项目说明", false),
        ("LICENSE", "许可证文件", false),
    ];
    
    for (file, description, required) in &required_files {
        let path = Path::new(file);
        if path.exists() {
            println!("✓ {}: {}", description, file);
        } else if *required {
            println!("✗ 缺少必需文件 {}: {}", description, file);
        } else {
            println!("⚠ 缺少可选文件 {}: {}", description, file);
        }
    }
    
    Ok(())
}

/// 检查 Shell 脚本语法
pub fn check_shell_syntax() -> Result<String> {
    println!("\n🐚 检查 Shell 脚本语法...");
    
    let current_dir = std::env::current_dir()?;
    
    // 检查 shellcheck 是否可用
    if !shellcheck::is_shellcheck_available() {
        let warning_msg = "⚠️  shellcheck 工具未安装或不可用，跳过语法检查";
        println!("{}", warning_msg);
        return Ok(warning_msg.to_string());
    }
      // 运行 shellcheck
    match shellcheck::check_project(&current_dir, true) {
        Ok((results, all_passed)) => {
            let formatted_output = shellcheck::format_results(&results);
            
            if all_passed {
                let success_msg = if results.is_empty() {
                    "✅ 未发现 Shell 脚本文件"
                } else {
                    "✅ Shell 脚本语法检查通过"
                };
                println!("{}", success_msg);
                
                // 返回详细结果
                if results.is_empty() {
                    Ok(success_msg.to_string())
                } else {
                    Ok(format!("{}\n\n{}", success_msg, formatted_output))
                }
            } else {
                let error_msg = "❌ Shell 脚本语法检查发现问题";
                println!("{}", error_msg);
                
                // 返回详细错误信息
                Ok(format!("{}\n\n{}", error_msg, formatted_output))
            }
        }
        Err(e) => {
            let error_msg = format!("❌ Shell 脚本语法检查失败: {}", e);
            println!("{}", &error_msg);
            Err(anyhow::anyhow!(error_msg))
        }
    }
}
