use clap::{ArgMatches, Command};
use anyhow::Result;
use crate::utils::{Context, remove_dir_all, dir_exists};

pub fn clean_command() -> Command {
    Command::new("clean")
        .about("清理项目文件")
        .subcommand(
            Command::new("dist")
                .about("清理构建输出目录")
        )
        .subcommand(
            Command::new("tags")
                .about("清理项目标签")
        )
        .subcommand(
            Command::new("all")
                .about("清理所有生成的文件")
        )
}

pub fn handle_clean(ctx: &Context, matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("dist", _)) => clean_dist(ctx),
        Some(("tags", _)) => clean_tags(ctx),
        Some(("all", _)) => clean_all(ctx),
        _ => {
            ctx.error("❌ 请指定子命令: dist, tags, all");
            Ok(())
        }
    }
}

fn clean_dist(ctx: &Context) -> Result<()> {
    ctx.info("🧹 清理构建输出目录...");

    let current_dir = std::env::current_dir()?;
    let dist_dir = current_dir.join(".rmmp").join("dist");

    if dir_exists(&dist_dir) {
        remove_dir_all(&dist_dir)?;
        ctx.info(&format!("✅ 已清理: {}", dist_dir.display()));
    } else {
        ctx.info("ℹ️  构建输出目录不存在，无需清理");
    }

    Ok(())
}

fn clean_tags(ctx: &Context) -> Result<()> {
    ctx.info("🧹 清理项目标签...");

    let current_dir = std::env::current_dir()?;
    let tags_dir = current_dir.join(".rmmp").join("tags");

    if dir_exists(&tags_dir) {
        remove_dir_all(&tags_dir)?;
        ctx.info(&format!("✅ 已清理: {}", tags_dir.display()));
    } else {
        ctx.info("ℹ️  标签目录不存在，无需清理");
    }

    Ok(())
}

fn clean_all(ctx: &Context) -> Result<()> {
    ctx.info("🧹 清理所有生成的文件...");

    let current_dir = std::env::current_dir()?;
    let rmmp_dir = current_dir.join(".rmmp");

    if dir_exists(&rmmp_dir) {
        remove_dir_all(&rmmp_dir)?;
        ctx.info(&format!("✅ 已清理: {}", rmmp_dir.display()));
    } else {
        ctx.info("ℹ️  没有找到需要清理的文件");
    }

    // 清理其他临时文件
    let temp_files = ["*.tmp", "*.log", "*.cache"];
    for pattern in &temp_files {
        // 这里可以添加清理临时文件的逻辑
        ctx.debug(&format!("检查临时文件: {}", pattern));
    }

    ctx.info("✅ 清理完成");

    Ok(())
}
