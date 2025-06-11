use clap::{Arg, ArgAction, Command};
use anyhow::Result;
use std::env;

mod config;
mod commands;
mod utils;
mod proxy;
mod adb;
mod shellcheck;

use config::RmmConfig;
use utils::setup_logging;

const DESCRIPTION: &str = "RMM: 高性能 Magisk/APatch/KernelSU 模块开发工具";

fn main() -> Result<()> {    // 设置日志
    let _ = setup_logging();
    
    let args: Vec<String> = env::args().collect();
    run_cli(args)
}

/// 主 CLI 函数
fn run_cli(args: Vec<String>) -> Result<()> {
    let app = build_cli();
    let matches = app.try_get_matches_from(args)?;
    
    // 加载配置
    let config = RmmConfig::load().unwrap_or_else(|_| {
        println!("⚠️  配置文件不存在，将创建默认配置");
        RmmConfig::default()
    });    // 处理子命令
    match matches.subcommand() {
        Some(("init", sub_matches)) => {
            commands::init::handle_init(&config, sub_matches)?;
        }
        Some(("build", sub_matches)) => {
            commands::build::handle_build(&config, sub_matches)?;
        }
        Some(("sync", sub_matches)) => {
            commands::sync::handle_sync(&config, sub_matches)?;
        }
        Some(("check", sub_matches)) => {
            commands::check::handle_check(&config, sub_matches)?;
        }
        Some(("publish", sub_matches)) => {
            commands::publish::handle_publish(&config, sub_matches)?;
        }        Some(("config", sub_matches)) => {
            commands::config::handle_config(&config, sub_matches)?;
        }        Some(("run", sub_matches)) => {
            commands::run::handle_run(&config, sub_matches)?;
        }        Some(("device", sub_matches)) | Some(("devices", sub_matches)) => {
            commands::device::handle_device(&config, sub_matches)?;
        }        Some(("clean", sub_matches)) => {
            commands::clean::handle_clean(&config, sub_matches)?;
        }
        Some(("test", sub_matches)) => {
            commands::test::handle_test(&config, sub_matches)?;
        }
        Some(("completion", sub_matches)) => {
            commands::completion::handle_completion(&config, sub_matches)?;
        }
        Some(("mcp", sub_matches)) => {
            commands::mcp::handle_mcp(&config, sub_matches)?;
        }
        _ => {
            println!("{}", DESCRIPTION);
            println!("使用 'rmm <command> --help' 查看具体命令的帮助信息");
        }
    }
    
    Ok(())
}

/// 构建 CLI 应用
fn build_cli() -> Command {
    Command::new("rmm")
        .version(env!("CARGO_PKG_VERSION"))
        .about(DESCRIPTION)
        .subcommand_required(false)
        .arg_required_else_help(false)        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("启用详细输出"),
        )
        .arg(
            Arg::new("token")
                .long("token")
                .value_name("TOKEN")
                .help("GitHub Personal Access Token")
                .global(true),
        )        
        .subcommand(commands::init::build_command())
        .subcommand(commands::build::build_command())
        .subcommand(commands::sync::build_command())
        .subcommand(commands::check::build_command())
        .subcommand(commands::publish::build_command())
        .subcommand(commands::config::build_command())
        .subcommand(commands::run::build_command())
        .subcommand(commands::device::build_command())
        .subcommand(commands::clean::build_command())
        .subcommand(commands::test::build_command())
        .subcommand(commands::completion::build_command())
        .subcommand(commands::mcp::build_command())
}
