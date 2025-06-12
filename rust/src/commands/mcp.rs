use clap::{Arg, ArgAction, Command};
use anyhow::Result;
use crate::commands::utils::core::config::RmmConfig;
use pyo3::types::{PyAnyMethods, PyDictMethods};

pub fn build_command() -> Command {
    Command::new("mcp")
        .about("启动 RMM MCP 服务器")
        .long_about("启动 Model Context Protocol (MCP) 服务器，提供 AI 助手集成功能")
        .arg(
            Arg::new("transport")
                .short('t')
                .long("transport")
                .value_name("TYPE")
                .help("传输方式: stdio (标准输入输出) 或 sse (Server-Sent Events)")
                .value_parser(["stdio", "sse"])
                .default_value("stdio")
        )
        .arg(
            Arg::new("host")
                .long("host")
                .value_name("HOST")
                .help("服务器监听地址 (仅在 sse 模式下使用)")
                .default_value("localhost")
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("服务器监听端口 (仅在 sse 模式下使用)")
                .value_parser(clap::value_parser!(u16))
                .default_value("8000")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("启用详细日志输出")
        )
}

pub fn handle_mcp(_config: &RmmConfig, matches: &clap::ArgMatches) -> Result<String> {
    let transport = matches.get_one::<String>("transport").unwrap();
    let host = matches.get_one::<String>("host").unwrap();
    let port = matches.get_one::<u16>("port").unwrap();
    let verbose = matches.get_flag("verbose");

    // 调用 Python 的 MCP 服务器
    pyo3::Python::with_gil(|py| {
        // 导入 rmmcp 模块
        let rmmcp_module = match py.import("pyrmm.ai.rmmcp") {
            Ok(module) => module,
            Err(e) => {
                eprintln!("错误: 无法导入 MCP 模块: {}", e);
                eprintln!("请确保已安装 mcp 依赖: pip install 'mcp[cli]>=1.9.3'");
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyImportError, _>(
                    "MCP 模块导入失败"
                ));
            }
        };

        // 获取启动函数
        let start_mcp_server = rmmcp_module.getattr("start_mcp_server")?;

        // 准备参数
        let kwargs = pyo3::types::PyDict::new(py);
        kwargs.set_item("transport", transport)?;
        kwargs.set_item("host", host)?;
        kwargs.set_item("port", *port)?;
        kwargs.set_item("verbose", verbose)?;

        if verbose {
            println!("🚀 启动 RMM MCP 服务器 ({} 模式)...", transport);
            if transport == "sse" {
                println!("📍 地址: {}:{}", host, port);
            }
        }        // 调用启动函数
        match start_mcp_server.call((), Some(&kwargs)) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("❌ MCP 服务器启动失败: {}", e);
                Err(e)
            }
        }
    })
    .map_err(|e| anyhow::anyhow!("MCP 服务器错误: {}", e))?;

    Ok("MCP 服务器启动成功".to_string())
}
