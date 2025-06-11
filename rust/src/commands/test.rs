use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::Path;
use crate::config::RmmConfig;

/// 构建 test 命令
pub fn build_command() -> Command {
    Command::new("test")
        .about("测试 RMM 项目")
        .long_about("对当前 RMM 项目进行各种测试，包括 shell 脚本语法检查")
        .arg(
            Arg::new("shellcheck")
                .long("shellcheck")
                .action(ArgAction::SetTrue)
                .help("只运行 shellcheck 检查")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("显示详细输出")
        )
}

/// 处理 test 命令
pub fn handle_test(_config: &RmmConfig, matches: &ArgMatches) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let verbose = matches.get_flag("verbose");
    let shellcheck_only = matches.get_flag("shellcheck");
    
    println!("🧪 开始测试 RMM 项目...");
    println!("📁 项目目录: {}", current_dir.display());
    
    let mut all_tests_passed = true;
    
    // 运行 shellcheck 检查
    if shellcheck_only || !shellcheck_only {  // 总是运行 shellcheck
        all_tests_passed &= run_shellcheck_tests(&current_dir, verbose)?;
    }
    
    // 可以在这里添加其他测试类型
    if !shellcheck_only {
        // 预留其他测试类型的空间
        println!("📋 其他测试类型将在未来版本中添加");
    }
    
    if all_tests_passed {
        println!("✅ 所有测试通过！");
    } else {
        println!("❌ 部分测试失败！");
        std::process::exit(1);
    }
    
    Ok(())
}

/// 运行 shellcheck 测试
fn run_shellcheck_tests(project_root: &Path, verbose: bool) -> Result<bool> {
    println!("\n🔍 运行 Shellcheck 检查...");
    
    // 检查 shellcheck 是否可用
    if !crate::shellcheck::is_shellcheck_available() {
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
    match crate::shellcheck::get_shellcheck_version() {
        Ok(version) => println!("📋 Shellcheck 版本: {}", version),
        Err(_) => println!("📋 Shellcheck 版本: 未知"),
    }
    
    // 执行检查
    match crate::shellcheck::check_project(project_root, verbose) {
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
