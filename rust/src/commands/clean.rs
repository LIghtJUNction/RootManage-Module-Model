use clap::{Arg, ArgAction, ArgMatches, Command};
use crate::commands::utils::core::config::RmmConfig;
use anyhow::Result;
use crate::commands::utils::core::executor::CleanManager;
use std::path::Path;

/// 构建 clean 命令
pub fn build_command() -> Command {
    Command::new("clean")
        .about("清理临时文件和日志")
        .long_about("清理 RMM 项目产生的临时文件、构建缓存、日志文件等")
        .arg(
            Arg::new("all")
                .short('a')
                .long("all")
                .action(ArgAction::SetTrue)
                .help("清理所有文件（包括构建产物）")
        )
        .arg(
            Arg::new("logs")
                .short('l')
                .long("logs")
                .action(ArgAction::SetTrue)
                .help("仅清理日志文件")
        )
        .arg(
            Arg::new("cache")
                .short('c')
                .long("cache")
                .action(ArgAction::SetTrue)
                .help("仅清理缓存文件")
        )
        .arg(
            Arg::new("build")
                .short('b')
                .long("build")
                .action(ArgAction::SetTrue)
                .help("仅清理构建产物")
        )
        .arg(
            Arg::new("dry_run")
                .short('n')
                .long("dry-run")
                .action(ArgAction::SetTrue)
                .help("预览将要删除的文件，但不实际删除")
        )
        .arg(
            Arg::new("force")
                .short('f')
                .long("force")
                .action(ArgAction::SetTrue)
                .help("强制删除，不提示确认")
        )
}

/// 处理 clean 命令
pub fn handle_clean(_config: &RmmConfig, matches: &ArgMatches) -> Result<String> {
    let all = matches.get_flag("all");
    let logs_only = matches.get_flag("logs");
    let cache_only = matches.get_flag("cache");
    let build_only = matches.get_flag("build");
    let dry_run = matches.get_flag("dry_run");
    let force = matches.get_flag("force");

    // 确定清理范围
    let clean_logs = all || logs_only || (!cache_only && !build_only);
    let clean_cache = all || cache_only || (!logs_only && !build_only);
    let clean_build = all || build_only || (!logs_only && !cache_only);

    if dry_run {
        println!("🔍 预览模式 - 以下文件/目录将被删除:");
    } else {
        println!("🧹 开始清理 RMM 项目文件...");
    }

    let mut operations_count = 0usize;

    // 清理日志文件
    if clean_logs {
        println!("\n📋 清理日志文件:");
        if !dry_run {
            CleanManager::clean_directory(Path::new("logs"))?;
            operations_count += 1;
        } else {
            if Path::new("logs").exists() {
                println!("  - logs/ 目录");
                operations_count += 1;
            }
        }
    }

    // 清理缓存文件
    if clean_cache {
        println!("\n🗂️  清理缓存文件:");
        let cache_dirs = [
            ".rmmp/cache",
            "target/debug/incremental", 
            "__pycache__",
            "src/pyrmm/__pycache__",
            "src/pyrmm/cli/__pycache__",
            "src/pyrmm/ai/__pycache__"
        ];
        
        for cache_dir in &cache_dirs {
            let path = Path::new(cache_dir);
            if !dry_run {
                CleanManager::clean_directory(path)?;
                operations_count += 1;
            } else {
                if path.exists() {
                    println!("  - {} 目录", cache_dir);
                    operations_count += 1;
                }
            }
        }
    }

    // 清理构建产物
    if clean_build {
        println!("\n📦 清理构建产物:");
        let build_dirs = [
            ".rmmp/dist",
            ".rmmp/temp"
        ];
        
        for build_dir in &build_dirs {
            let path = Path::new(build_dir);
            if !dry_run {
                CleanManager::clean_directory(path)?;
                operations_count += 1;
            } else {
                if path.exists() {
                    println!("  - {} 目录", build_dir);
                    operations_count += 1;
                }
            }
        }
        
        // 清理特定文件
        let build_files = ["update.json"];
        for build_file in &build_files {
            let path = Path::new(build_file);
            if !dry_run {
                CleanManager::clean_file(path)?;
                operations_count += 1;
            } else {
                if path.exists() {
                    println!("  - {} 文件", build_file);
                    operations_count += 1;
                }
            }
        }
        
        // Rust 构建产物
        if all {
            let rust_dirs = ["target/debug", "target/release", "target/wheels"];
            for rust_dir in &rust_dirs {
                let path = Path::new(rust_dir);
                if !dry_run {
                    CleanManager::clean_directory(path)?;
                    operations_count += 1;
                } else {
                    if path.exists() {
                        println!("  - {} 目录", rust_dir);
                        operations_count += 1;
                    }
                }
            }
        }
    }

    // 显示统计信息
    if dry_run {
        println!("\n📊 预览统计:");
        println!("  操作数量: {} 个", operations_count);
        println!("\n💡 使用 'rmm clean' 实际执行清理");
    } else {
        if operations_count > 0 {
            if !force && operations_count > 5 {
                use std::io::{self, Write};
                print!("⚠️  即将执行 {} 个清理操作，确认继续? (y/N): ", operations_count);
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                
                if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes" | "是") {
                    println!("❌ 清理已取消");
                    return Ok("清理已取消".to_string());
                }
            }
            
            println!("\n✅ 清理完成!");
            println!("  执行操作: {} 个", operations_count);
        } else {
            println!("\n✨ 没有找到需要清理的文件");
        }
    }
    
    Ok("项目清理完成".to_string())
}
