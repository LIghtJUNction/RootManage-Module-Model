use clap::{Arg, ArgAction, ArgMatches, Command};
use anyhow::Result;
use crate::config::RmmConfig;
use std::path::Path;
use std::fs;

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
pub fn handle_clean(_config: &RmmConfig, matches: &ArgMatches) -> Result<()> {
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
        println!("🔍 预览模式 - 以下文件将被删除:");
    } else {
        println!("🧹 开始清理 RMM 项目文件...");
    }

    let mut total_size = 0u64;
    let mut file_count = 0usize;

    // 清理日志文件
    if clean_logs {
        println!("\n📋 清理日志文件:");
        total_size += clean_directory("logs", &["*.log", "*.txt"], dry_run, &mut file_count)?;
        total_size += clean_directory(".", &["*.log"], dry_run, &mut file_count)?;
    }

    // 清理缓存文件
    if clean_cache {
        println!("\n🗂️  清理缓存文件:");
        total_size += clean_directory(".rmmp/cache", &["*"], dry_run, &mut file_count)?;
        total_size += clean_directory("target/debug/incremental", &["*"], dry_run, &mut file_count)?;
        total_size += clean_directory("__pycache__", &["*"], dry_run, &mut file_count)?;
        total_size += clean_directory("src/pyrmm/__pycache__", &["*"], dry_run, &mut file_count)?;
        total_size += clean_directory("src/pyrmm/cli/__pycache__", &["*"], dry_run, &mut file_count)?;
    }

    // 清理构建产物
    if clean_build {
        println!("\n📦 清理构建产物:");
        total_size += clean_directory(".rmmp/dist", &["*"], dry_run, &mut file_count)?;
        total_size += clean_directory(".rmmp/temp", &["*"], dry_run, &mut file_count)?;
        total_size += clean_file("update.json", dry_run, &mut file_count)?;
        
        // Rust 构建产物
        if all {
            total_size += clean_directory("target/debug", &["*"], dry_run, &mut file_count)?;
            total_size += clean_directory("target/release", &["*"], dry_run, &mut file_count)?;
            total_size += clean_directory("target/wheels", &["*"], dry_run, &mut file_count)?;
        }
    }

    // 显示统计信息
    let size_mb = total_size as f64 / 1024.0 / 1024.0;
    
    if dry_run {
        println!("\n📊 预览统计:");
        println!("  文件数量: {} 个", file_count);
        println!("  总大小: {:.2} MB", size_mb);
        println!("\n💡 使用 'rmm clean' 实际执行清理");
    } else {
        if file_count > 0 {
            if !force && file_count > 10 {
                use std::io::{self, Write};
                print!("⚠️  即将删除 {} 个文件 ({:.2} MB)，确认继续? (y/N): ", file_count, size_mb);
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                
                if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes" | "是") {
                    println!("❌ 清理已取消");
                    return Ok(());
                }
            }
            
            println!("\n✅ 清理完成!");
            println!("  已删除文件: {} 个", file_count);
            println!("  释放空间: {:.2} MB", size_mb);
        } else {
            println!("\n✨ 没有找到需要清理的文件");
        }
    }

    Ok(())
}

/// 清理目录下的文件
fn clean_directory(dir_path: &str, patterns: &[&str], dry_run: bool, file_count: &mut usize) -> Result<u64> {
    let path = Path::new(dir_path);
    
    if !path.exists() {
        return Ok(0);
    }

    let mut total_size = 0u64;

    if path.is_dir() {
        let entries = fs::read_dir(path)?;
        
        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();
            
            if should_clean_file(&entry_path, patterns) {
                let metadata = entry.metadata()?;
                total_size += metadata.len();
                *file_count += 1;
                
                if dry_run {
                    println!("  🗑️  {}", entry_path.display());
                } else {
                    if entry_path.is_dir() {
                        fs::remove_dir_all(&entry_path)?;
                        println!("  🗂️  已删除目录: {}", entry_path.display());
                    } else {
                        fs::remove_file(&entry_path)?;
                        println!("  📄 已删除文件: {}", entry_path.display());
                    }
                }
            }
        }
        
        // 如果目录为空且不是根目录，则删除目录本身
        if !dry_run && dir_path != "." && dir_path != ".rmmp" {
            if let Ok(entries) = fs::read_dir(path) {
                if entries.count() == 0 {
                    fs::remove_dir(path)?;
                    println!("  🗂️  已删除空目录: {}", path.display());
                }
            }
        }
    }

    Ok(total_size)
}

/// 清理单个文件
fn clean_file(file_path: &str, dry_run: bool, file_count: &mut usize) -> Result<u64> {
    let path = Path::new(file_path);
    
    if !path.exists() {
        return Ok(0);
    }

    let metadata = path.metadata()?;
    let size = metadata.len();
    *file_count += 1;

    if dry_run {
        println!("  🗑️  {}", path.display());
    } else {
        fs::remove_file(path)?;
        println!("  📄 已删除文件: {}", path.display());
    }

    Ok(size)
}

/// 检查文件是否应该被清理
fn should_clean_file(path: &Path, patterns: &[&str]) -> bool {
    if patterns.contains(&"*") {
        return true;
    }

    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    for pattern in patterns {
        if pattern.ends_with("*") {
            let prefix = &pattern[..pattern.len()-1];
            if file_name.starts_with(prefix) {
                return true;
            }
        } else if pattern.starts_with("*") {
            let suffix = &pattern[1..];
            if file_name.ends_with(suffix) {
                return true;
            }
        } else if file_name == *pattern {
            return true;
        }
    }

    false
}
