use pyo3::prelude::*;
use clap::{Arg, ArgAction, Command};
use anyhow::Result;
use std::env;

mod config;
mod commands;
mod utils;

use config::RmmConfig;
use utils::setup_logging;

const DESCRIPTION: &str = "RMM: 高性能 Magisk/APatch/KernelSU 模块开发工具";

/// 主 CLI 函数，直接从 Python 调用
#[pyfunction]
#[pyo3(signature = (args=None))]
fn cli(args: Option<Vec<String>>) -> PyResult<()> {
    // 构建参数列表，始终以程序名开头
    let mut final_args = vec!["rmm".to_string()];
    
    // 如果提供了参数，直接使用；否则从环境变量获取
    if let Some(provided_args) = args {
        final_args.extend(provided_args);
    } else {
        // 从命令行获取参数，跳过第一个（程序名）
        final_args.extend(env::args().skip(1));
    }
    
    match run_cli(final_args) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("错误: {}", e);
            std::process::exit(1);
        }
    }
}

/// 运行 CLI 的核心逻辑
fn run_cli(args: Vec<String>) -> Result<()> {
    setup_logging()?;
    
    let app = build_cli();
    let matches = app.try_get_matches_from(args)?;
    
    // 初始化配置
    let config = RmmConfig::load()?;
      // 路由到不同的命令处理器
    match matches.subcommand() {
        Some(("init", sub_matches)) => commands::init::handle_init(&config, sub_matches),
        _ => {
            // 如果没有子命令，显示帮助
            let mut app = build_cli();
            app.print_help()?;
            Ok(())
        }
    }
}

/// 构建 CLI 应用程序
fn build_cli() -> Command {
    Command::new("rmm")
        .version(env!("CARGO_PKG_VERSION"))
        .about(DESCRIPTION)
        .long_about("RMM (Root Module Manager) 是一个高性能的 Magisk/APatch/KernelSU 模块开发工具，使用 Rust 编写以提供卓越的性能。")
        .arg_required_else_help(false)
        .subcommand_required(false)
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .global(true)
                .help("启用详细输出")
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue)                
                .global(true)
                .help("静默模式，只输出错误")
        )
        .subcommand(commands::init::build_command())
}

/// Python 模块定义
#[pymodule]
#[pyo3(name = "rmmcore")]
fn rmmcore(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(cli, m)?)?;
    Ok(())
}
