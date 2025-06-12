
/// 执行脚本命令
use anyhow::Result;
use std::path::Path;
use crate::commands::utils::shellcheck;

pub fn execute_script_command(command: &str, working_dir: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("powershell")
            .args(&["-Command", command])
            .current_dir(working_dir)
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("脚本执行失败: {}", stderr);
        }
        
        // 输出命令结果
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            print!("{}", stdout);
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(working_dir)
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("脚本执行失败: {}", stderr);
        }
        
        // 输出命令结果
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            print!("{}", stdout);
        }
    }
    
    Ok(())
}

/// 运行 shellcheck 测试
pub fn run_shellcheck_tests(project_root: &Path, verbose: bool) -> Result<bool> {    println!("\n🔍 运行 Shellcheck 检查...");
    
    // 检查 shellcheck 是否可用
    if !shellcheck::is_shellcheck_available() {
        println!("⚠️  Shellcheck 未安装或不可用");
        println!("   请安装 shellcheck 以进行 shell 脚本语法检查");
        println!("   安装方法:");
        if cfg!(target_os = "windows") {
            println!("     - Windows: 使用 scoop install shellcheck 或从 GitHub 下载");
        } else if cfg!(target_os = "macos") {
            println!("     - macOS: brew install shellcheck");
        } else {
            println!("     - Linux: 使用包管理器安装 (apt install shellcheck / yum install shellcheck)");
        }
        println!("   跳过 shellcheck 检查...");
        return Ok(true);  // 不作为错误，只是警告
    }
    
    // 显示 shellcheck 版本
    match shellcheck::get_shellcheck_version() {
        Ok(version) => println!("📋 Shellcheck 版本: {}", version),
        Err(_) => println!("📋 Shellcheck 版本: 未知"),
    }
    
    // 执行检查
    match shellcheck::check_project(project_root, verbose) {
        Ok((results, all_passed)) => {
            if results.is_empty() {
                println!("📋 项目中未发现 shell 脚本文件");
                return Ok(true);
            }
            
            if all_passed {
                println!("✅ Shellcheck 检查通过");
            } else {
                println!("⚠️  Shellcheck 检查发现问题（作为警告，不影响测试结果）");
            }
            
            Ok(true)  // 在测试模式下，shellcheck 问题只作为警告
        }
        Err(e) => {
            println!("❌ Shellcheck 检查失败: {}", e);
            Ok(false)
        }
    }
}

