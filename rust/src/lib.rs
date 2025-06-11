use pyo3::prelude::*;
use clap::{Arg, ArgAction, Command};
use anyhow::Result;
use std::env;

mod config;
mod commands;
mod utils;
mod proxy;
mod adb;
pub mod shellcheck;

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
            // 不要在这里打印错误，因为 clap 已经处理了输出
            // 只有真正的错误才需要打印
            eprintln!("错误: {}", e);
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
        }
    }
}

/// 运行 CLI 的核心逻辑
fn run_cli(args: Vec<String>) -> Result<()> {
    setup_logging()?;
    
    let app = build_cli();
    
    // 使用 get_matches_from 而不是 try_get_matches_from
    // 这样 clap 会自动处理 --help 和 --version 并正常退出
    let matches = match app.try_get_matches_from(args) {
        Ok(matches) => matches,
        Err(err) => {
            // 如果是帮助或版本信息，正常输出并退出
            if err.kind() == clap::error::ErrorKind::DisplayHelp ||
               err.kind() == clap::error::ErrorKind::DisplayVersion {
                print!("{}", err);
                return Ok(());
            }
            // 其他错误则返回错误
            return Err(err.into());
        }
    };
    
    // 初始化配置
    let config = RmmConfig::load()?;    // 路由到不同的命令处理器
    match matches.subcommand() {        Some(("init", sub_matches)) => commands::init::handle_init(&config, sub_matches),
        Some(("build", sub_matches)) => commands::build::handle_build(&config, sub_matches),
        Some(("sync", sub_matches)) => commands::sync::handle_sync(&config, sub_matches),
        Some(("check", sub_matches)) => commands::check::handle_check(&config, sub_matches),
        Some(("publish", sub_matches)) => commands::publish::handle_publish(&config, sub_matches),
        Some(("config", sub_matches)) => commands::config::handle_config(&config, sub_matches),
        Some(("run", sub_matches)) => commands::run::handle_run(&config, sub_matches),
        Some(("device", sub_matches)) => commands::device::handle_device(&config, sub_matches),
        Some(("test", sub_matches)) => commands::test::handle_test(&config, sub_matches),
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
        )        .subcommand(commands::init::build_command())
        .subcommand(commands::build::build_command())
        .subcommand(commands::sync::build_command())
        .subcommand(commands::check::build_command())
        .subcommand(commands::publish::build_command())        .subcommand(commands::config::build_command())
        .subcommand(commands::run::build_command())
        .subcommand(commands::device::build_command())
        .subcommand(commands::test::build_command())
}

/// 获取最快的 GitHub 代理
#[pyfunction]
fn get_fastest_proxy() -> PyResult<Option<String>> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("创建异步运行时失败: {}", e))
    })?;
    
    match rt.block_on(proxy::get_fastest_proxy()) {
        Ok(Some(proxy)) => Ok(Some(proxy.url)),
        Ok(None) => Ok(None),
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("获取代理失败: {}", e)))
    }
}

/// Python 模块定义
#[pymodule]
#[pyo3(name = "rmmcore")]
fn rmmcore(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(cli, m)?)?;
    m.add_function(wrap_pyfunction!(get_fastest_proxy, m)?)?;
    Ok(())
}
