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

/// CLI 函数，返回详细的执行结果，用于 MCP 服务器
#[pyfunction]
#[pyo3(signature = (args=None))]
fn cli_with_output(args: Option<Vec<String>>) -> PyResult<Option<String>> {
    // 构建参数列表，始终以程序名开头
    let mut final_args = vec!["rmm".to_string()];
    
    // 如果提供了参数，直接使用；否则从环境变量获取
    if let Some(provided_args) = args {
        final_args.extend(provided_args);
    } else {
        // 从命令行获取参数，跳过第一个（程序名）
        final_args.extend(env::args().skip(1));
    }
      match run_cli_with_output(final_args) {
        Ok(output) => Ok(output),
        Err(e) => {
            eprintln!("错误: {}", e);
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
        }
    }
}

/// 调用 Python 发布函数
#[pyfunction]
fn publish_to_github(config_json: String) -> PyResult<bool> {
    use pyo3::types::PyModule;
    
    pyo3::Python::with_gil(|py| {        // 导入 publisher 模块
        let publisher_module = PyModule::import(py, "pyrmm.publisher")?;
        
        // 导入 json 模块
        let json = PyModule::import(py, "json")?;
        
        // 调用 json.loads 函数
        let config_dict = json.getattr("loads")?.call1((config_json,))?;
        
        // 调用 publish_to_github 函数
        let result = publisher_module.getattr("publish_to_github")?.call1((config_dict,))?;
        
        Ok(result.extract::<bool>()?)
    })
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
    match matches.subcommand() {        
        Some(("init", sub_matches)) => {
            match commands::init::handle_init(&config, sub_matches) {
                Ok(_result) => Ok(()),
                Err(e) => Err(e)
            }
        },
        Some(("build", sub_matches)) => {
            match commands::build::handle_build(&config, sub_matches) {
                Ok(_result) => Ok(()),
                Err(e) => Err(e)
            }
        },
        Some(("sync", sub_matches)) => {
            match commands::sync::handle_sync(&config, sub_matches) {
                Ok(_result) => Ok(()),
                Err(e) => Err(e)
            }
        },
        Some(("check", sub_matches)) => {
            // check 命令返回 String，我们需要特殊处理
            match commands::check::handle_check(&config, sub_matches) {
                Ok(_result) => {
                    // 在这里 _result 是 String，但我们不需要打印它，因为命令内部已经处理了输出
                    Ok(())
                },
                Err(e) => Err(e)
            }
        },
        Some(("publish", sub_matches)) => {
            match commands::publish::handle_publish(&config, sub_matches) {
                Ok(_result) => Ok(()),
                Err(e) => Err(e)
            }
        },        Some(("config", sub_matches)) => {
            match commands::config::handle_config(&config, sub_matches) {
                Ok(_result) => Ok(()),
                Err(e) => Err(e)
            }
        },
        Some(("run", sub_matches)) => {
            match commands::run::handle_run(&config, sub_matches) {
                Ok(_result) => Ok(()),
                Err(e) => Err(e)
            }
        },
        Some(("device", sub_matches)) => {
            match commands::device::handle_device(&config, sub_matches) {
                Ok(_result) => Ok(()),
                Err(e) => Err(e)
            }
        },
        Some(("clean", sub_matches)) => {
            match commands::clean::handle_clean(&config, sub_matches) {
                Ok(_result) => Ok(()),
                Err(e) => Err(e)
            }
        },        Some(("test", sub_matches)) => {
            match commands::test::handle_test(&config, sub_matches) {
                Ok(_result) => Ok(()),
                Err(e) => Err(e)
            }
        },
        Some(("completion", sub_matches)) => {
            match commands::completion::handle_completion(&config, sub_matches) {
                Ok(_result) => Ok(()),
                Err(e) => Err(e)
            }
        },
        Some(("mcp", sub_matches)) => {
            match commands::mcp::handle_mcp(&config, sub_matches) {
                Ok(_result) => Ok(()),
                Err(e) => Err(e)
            }
        },
        _ => {
            // 如果没有子命令，显示帮助
            let mut app = build_cli();
            app.print_help()?;
            Ok(())
        }
    }
}

/// 运行 CLI 的核心逻辑，返回详细的输出结果
fn run_cli_with_output(args: Vec<String>) -> Result<Option<String>> {
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
                return Ok(None);
            }
            // 其他错误则返回错误
            return Err(err.into());
        }
    };
    
    // 初始化配置
    let config = RmmConfig::load()?;

    // 路由到不同的命令处理器，并收集输出结果
    match matches.subcommand() {        
        Some(("init", sub_matches)) => {
            match commands::init::handle_init(&config, sub_matches) {
                Ok(result) => Ok(Some(result)),
                Err(e) => Err(e)
            }
        },
        Some(("build", sub_matches)) => {
            match commands::build::handle_build(&config, sub_matches) {
                Ok(result) => Ok(Some(result)),
                Err(e) => Err(e)
            }
        },
        Some(("sync", sub_matches)) => {
            match commands::sync::handle_sync(&config, sub_matches) {
                Ok(result) => Ok(Some(result)),
                Err(e) => Err(e)
            }
        },
        Some(("check", sub_matches)) => {
            match commands::check::handle_check(&config, sub_matches) {
                Ok(result) => Ok(Some(result)),
                Err(e) => Err(e)
            }
        },
        Some(("publish", sub_matches)) => {
            match commands::publish::handle_publish(&config, sub_matches) {
                Ok(result) => Ok(Some(result)),
                Err(e) => Err(e)
            }
        },
        Some(("config", sub_matches)) => {
            match commands::config::handle_config(&config, sub_matches) {
                Ok(result) => Ok(Some(result)),
                Err(e) => Err(e)
            }
        },
        Some(("run", sub_matches)) => {
            match commands::run::handle_run(&config, sub_matches) {
                Ok(result) => Ok(Some(result)),
                Err(e) => Err(e)
            }
        },
        Some(("device", sub_matches)) => {
            match commands::device::handle_device(&config, sub_matches) {
                Ok(result) => Ok(Some(result)),
                Err(e) => Err(e)
            }
        },
        Some(("clean", sub_matches)) => {
            match commands::clean::handle_clean(&config, sub_matches) {
                Ok(result) => Ok(Some(result)),
                Err(e) => Err(e)
            }
        },
        Some(("test", sub_matches)) => {
            match commands::test::handle_test(&config, sub_matches) {
                Ok(result) => Ok(Some(result)),
                Err(e) => Err(e)
            }
        },
        Some(("completion", sub_matches)) => {
            match commands::completion::handle_completion(&config, sub_matches) {
                Ok(result) => Ok(Some(result)),
                Err(e) => Err(e)
            }
        },
        Some(("mcp", sub_matches)) => {
            match commands::mcp::handle_mcp(&config, sub_matches) {
                Ok(result) => Ok(Some(result)),
                Err(e) => Err(e)
            }
        },
        _ => {
            // 如果没有子命令，显示帮助
            let mut app = build_cli();
            app.print_help()?;
            Ok(Some("帮助信息已显示".to_string()))
        }
    }
}

/// 构建 CLI 应用程序
pub fn build_cli() -> Command {
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
        .subcommand(commands::clean::build_command())        .subcommand(commands::test::build_command())
        .subcommand(commands::completion::build_command())
        .subcommand(commands::mcp::build_command())
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
    m.add_function(wrap_pyfunction!(cli_with_output, m)?)?;
    m.add_function(wrap_pyfunction!(get_fastest_proxy, m)?)?;
    m.add_function(wrap_pyfunction!(publish_to_github, m)?)?;
    Ok(())
}
