// PyO3 imports
use pyo3::prelude::*;
use pyo3::{pyfunction, pymodule, wrap_pyfunction, PyResult, PyErr, Bound};

// Standard library imports
use std::env;

// Clap imports for CLI
use clap::{Command, Arg, ArgAction};

// Anyhow for error handling
use anyhow::Result;

// Module declarations
mod commands;

// Import configuration and utility types
use commands::utils::core::config::RmmConfig;

const DESCRIPTION: &str = "RMM: 高性能 Magisk/APatch/KernelSU 模块开发工具";

/// 设置日志记录
fn setup_logging() -> Result<()> {
    // 简单的日志设置
    Ok(())
}

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
    let config = RmmConfig::load()?;
    
    // 路由到不同的命令处理器
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
        },
        Some(("config", sub_matches)) => {
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
        },
        Some(("test", sub_matches)) => {
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
            Ok(None)
        }
    }
}

/// 构建 CLI 应用程序
pub fn build_cli() -> Command {
    Command::new("rmm")
        .version("0.2.6")
        .about(DESCRIPTION)
        .long_about("RMM 是一个高性能的 Magisk/APatch/KernelSU 模块开发工具，\
                   提供模块初始化、构建、测试、发布等完整开发流程支持。")
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .help("显示详细输出")
                .action(ArgAction::SetTrue)
                .global(true)
        )
        .arg(
            Arg::new("quiet")
                .long("quiet")
                .short('q')
                .help("静默模式")
                .action(ArgAction::SetTrue)
                .global(true)
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

/// Python 模块定义
#[pymodule]
#[pyo3(name = "rmmcore")]
fn rmmcore(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(cli, m)?)?;
    m.add_function(wrap_pyfunction!(cli_with_output, m)?)?;
    Ok(())
}
