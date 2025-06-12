use anyhow::Result;
use clap::{Arg, ArgAction, Command, ArgMatches};
use crate::commands::utils::core::config::RmmConfig;
use crate::commands::utils::core::executor::CheckManager;

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
pub fn handle_test(_config: &RmmConfig, matches: &ArgMatches) -> Result<String> {
    let current_dir = std::env::current_dir()?;
    let verbose = matches.get_flag("verbose");
    let shellcheck_only = matches.get_flag("shellcheck");
    
    println!("🧪 开始测试 RMM 项目...");
    println!("📁 项目目录: {}", current_dir.display());
    
    let mut all_tests_passed = true;
    
    // 运行 shellcheck 检查
    if shellcheck_only || !shellcheck_only {  // 总是运行 shellcheck
        match CheckManager::check_shell_syntax(&current_dir) {
            Ok(result) => {
                if verbose {
                    println!("{}", result);
                }
                // 检查结果是否表示成功
                if result.contains("✅") {
                    println!("✅ Shell 脚本语法检查通过");
                } else {
                    println!("❌ Shell 脚本语法检查失败");
                    all_tests_passed = false;
                }
            }
            Err(e) => {
                println!("❌ Shell 脚本检查错误: {}", e);
                all_tests_passed = false;
            }
        }
    }
    
    // 可以在这里添加其他测试类型
    if !shellcheck_only {
        // 预留其他测试类型的空间
        println!("📋 其他测试类型将在未来版本中添加");
    }    if all_tests_passed {
        println!("✅ 所有测试通过！");
        Ok("项目测试通过".to_string())
    } else {
        println!("❌ 部分测试失败！");
        Ok("项目测试失败".to_string())
    }
}

