#![recursion_limit = "256"]

use pyo3::prelude::*;
use std::path::PathBuf;

mod cmds;
mod core;

use cmds::{Commands, RmmBox};
use core::python_bindings::PyRmmCore;
use pyo3::Python;

use clap::{Parser, CommandFactory};
use pyo3::types::PyList;
use colored::*;

/// 🚀 RMM 
#[derive(Parser)]
#[command(color = clap::ColorChoice::Always)]
#[command(styles = get_styles())]
#[command(help_template = "\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
")]
struct Cli {
    #[command(subcommand)]
    /// 命令
    cmd: Option<Commands>,
}
/// CLI 入口函数
#[pyfunction]
fn cli() -> PyResult<()> {
    let args = Cli::parse_from(std::env::args().skip(1));
    match args.cmd {        // 初始化命令
        Some(Commands::Init { project_id }) => {
            // 获取当前目录
            let current_dir = std::env::current_dir().map_err(|e| 
                pyo3::exceptions::PyRuntimeError::new_err(format!("无法获取当前目录: {}", e))
            )?;
            
            // 处理项目ID和路径
            let (actual_project_id, project_path) = if project_id == "." {
                // 如果是 "."，使用当前目录名作为项目ID，在当前目录初始化
                let dir_name = current_dir.file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("无法获取当前目录名"))?;
                (dir_name.to_string(), current_dir)
            } else {
                // 解析路径，可能是相对路径如 ./XXX/YYY
                let target_path = if project_id.starts_with('.') {
                    // 相对路径：./XXX/YYY 或 ../XXX
                    current_dir.join(&project_id).canonicalize()
                        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("无法解析路径 '{}': {}", project_id, e)))?
                } else {
                    // 直接名称：在当前目录下创建
                    current_dir.join(&project_id)
                };
                
                // 从最终路径提取项目ID（目录名）
                let dir_name = target_path.file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("无法获取目标目录名"))?;
                
                // 如果不是相对路径，需要创建目录
                if !project_id.starts_with('.') {
                    if let Err(e) = std::fs::create_dir_all(&target_path) {
                        return Err(pyo3::exceptions::PyRuntimeError::new_err(format!("无法创建项目目录: {}", e)));
                    }
                }
                
                (dir_name.to_string(), target_path)
            };// 从 meta 配置读取作者信息，如果没有则使用默认值
            let core = core::rmm_core::RmmCore::new();
            let (author_name, author_email) = match core.get_meta_config() {
                Ok(meta) => {
                    (meta.username, meta.email)
                }
                Err(_) => {
                    ("unknown".to_string(), "unknown@example.com".to_string())
                }
            };
              match cmds::init::init_project(&project_path, &actual_project_id, &author_name, &author_email) {
                Ok(()) => {
                    // 更新 meta 配置中的 projects (ID = PATH)
                    if let Err(e) = update_meta_projects(&core, &actual_project_id, &project_path) {
                        eprintln!("⚠️ 警告: 无法更新 meta 配置: {}", e);
                    }
                    println!("{} 项目初始化成功！", "✅".green().bold());
                }
                Err(e) => {                    eprintln!("❌ 初始化失败: {}", e);
                    return Err(pyo3::exceptions::PyRuntimeError::new_err(format!("初始化失败: {}", e)));
                }
            }
        },
          // 构建命令
        Some(Commands::Build { project_path, no_auto_fix, script }) => {
            // 确定项目路径
            let target_path = if let Some(path) = project_path {
                PathBuf::from(path)
            } else {
                std::env::current_dir().map_err(|e| 
                    pyo3::exceptions::PyRuntimeError::new_err(format!("无法获取当前目录: {}", e))
                )?
            };
            
            // 规范化路径
            let project_path = target_path.canonicalize().unwrap_or(target_path);
              // 如果指定了脚本，运行脚本；否则运行构建
            if let Some(script_name) = script {
                let core = core::rmm_core::RmmCore::new();
                match core.run_rmake_script(&project_path, &script_name) {
                    Ok(()) => {
                        println!("{} 脚本执行成功！", "✅".green().bold());
                    }
                    Err(e) => {
                        // 如果脚本未找到，列出可用脚本
                        if e.to_string().contains("未找到") {
                            eprintln!("❌ 脚本 '{}' 未找到", script_name);
                            match core.list_rmake_scripts(&project_path) {
                                Ok(scripts) => {
                                    if scripts.is_empty() {
                                        eprintln!("📋 当前项目的Rmake.toml中没有定义任何脚本");
                                    } else {
                                        eprintln!("📋 可用脚本:");
                                        for script in scripts {
                                            eprintln!("   - {}", script);
                                        }
                                    }
                                }
                                Err(_) => {
                                    eprintln!("⚠️  无法读取Rmake.toml配置文件");
                                }
                            }
                        } else {
                            eprintln!("❌ 脚本执行失败: {}", e);
                        }
                        return Err(pyo3::exceptions::PyRuntimeError::new_err(format!("脚本执行失败: {}", e)));
                    }
                }
            } else {
                // 执行构建，传递自动修复参数
                let auto_fix = !no_auto_fix;  // 默认启用自动修复，除非用户明确禁用
                match cmds::build::build_project_with_options(&project_path, auto_fix) {
                    Ok(()) => {
                        println!("{} 构建成功！", "✅".green().bold());
                    }                    Err(e) => {
                        eprintln!("❌ 构建失败: {}", e);
                        return Err(pyo3::exceptions::PyRuntimeError::new_err(format!("构建失败: {}", e)));
                    }
                }
            }        },
        
        // 运行脚本命令
        Some(Commands::Run { project_path, script }) => {
            // 确定项目路径
            let target_path = if let Some(path) = project_path {
                PathBuf::from(path)
            } else {
                std::env::current_dir().map_err(|e| 
                    pyo3::exceptions::PyRuntimeError::new_err(format!("无法获取当前目录: {}", e))
                )?
            };
            
            // 规范化路径
            let project_path = target_path.canonicalize().unwrap_or(target_path);
            
            // 运行脚本
            match cmds::run::run_script(&project_path, script.as_deref()) {
                Ok(()) => {
                    if script.is_some() {
                        println!("{} 脚本执行成功！", "✅".green().bold());
                    }
                }                Err(e) => {
                    eprintln!("❌ 执行失败: {}", e);
                    return Err(pyo3::exceptions::PyRuntimeError::new_err(format!("执行失败: {}", e)));
                }
            }
        },
        
        // 同步项目元数据命令
        Some(Commands::Sync { project_name, projects_only, search_paths, max_depth }) => {
            // 转换 search_paths 为 &str 类型
            let search_paths_refs = search_paths.as_ref().map(|paths| {
                paths.iter().map(|s| s.as_str()).collect::<Vec<&str>>()
            });
            
            // 同步项目
            match cmds::sync::sync_projects(
                project_name.as_deref(),
                projects_only,
                search_paths_refs,
                max_depth,
            ) {
                Ok(()) => {
                    println!("{} 项目同步成功！", "✅".green().bold());
                }
                Err(e) => {
                    eprintln!("❌ 同步失败: {}", e);
                    return Err(pyo3::exceptions::PyRuntimeError::new_err(format!("同步失败: {}", e)));
                }
            }        },
        
        // 显示版本信息
        Some(Commands::Version) => {
            RmmBox::rmm_version();
        },

        // 匹配外部命令
        Some(Commands::External(cmd)) => {
            println!("🤗查询拓展命令: {}", cmd.join(" ").bright_magenta().bold());
            let command_name = cmd.get(0).cloned();
            let module_name = command_name;
              // 尝试导入 Python 模块并执行
            let result = Python::with_gil(|py| {
                if let Some(name) = &module_name {
                    // 限制在 cli 包下查找模块
                    let module_path = format!("pyrmm.cli.{}", name);
                    // 尝试导入模块
                    match PyModule::import(py, &module_path) {
                        Ok(module) => {
                            // 尝试使用与模块名相同的函数作为入口
                            // 如果找不到，则回退到尝试 main 函数
                            let func_result = module.getattr(name).or_else(|_| module.getattr("main"));                            if let Ok(func) = func_result {
                                // 创建参数列表并调用Python函数
                                println!("🐍 找到python命令拓展: {})", name.green());                                // 创建参数列表
                                let list_result = PyList::new(py, &cmd[1..]);
                                if let Ok(args_list) = list_result {
                                    // 将列表包装在一个元组中作为单个参数传递
                                    let result = func.call1((args_list,));
                                    result?;
                                } else {
                                    return Err(pyo3::exceptions::PyValueError::new_err(
                                        "无法创建参数列表".to_string()
                                    ));
                                }
                                Ok(())
                            } else {
                                // 没有找到合适的入口函数，报错
                                Err(pyo3::exceptions::PyAttributeError::new_err(
                                    format!("模块 {} 没有 {} 或 main 函数", name, name)
                                ))
                            }
                        },                        Err(_) => {
                            // 模块导入失败，可能这是个无效命令，显示帮助
                            println!("❌未知命令(Command Not Found): {}", name.red().bold());
                            let mut cmd = Cli::command();
                            cmd.print_help().ok();
                            Ok(())
                        }
                    }
                } else {
                    Err(pyo3::exceptions::PyValueError::new_err("命令参数为空"))
                }
            });
            
            // 处理结果
            result?;
        }         // 没有提供子命令，默认显示带颜色的帮助
        None => {
            let mut cmd = Cli::command();
            cmd.print_help().ok();
        }
    }
    Ok(())
}



///库函数
/// 更新 meta 配置中的项目列表
fn update_meta_projects(core: &core::rmm_core::RmmCore, project_id: &str, project_path: &std::path::Path) -> anyhow::Result<()> {
    let mut meta = core.get_meta_config()?;
    meta.projects.insert(project_id.to_string(), project_path.to_string_lossy().to_string());
    
    // 保存更新后的配置
    let meta_path = core.get_rmm_root().join("meta.toml");
    let meta_content = toml::to_string_pretty(&meta)?;
    std::fs::write(meta_path, meta_content)?;
    
    Ok(())
}

/// 获取 clap 样式配置
fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .header(clap::builder::styling::AnsiColor::Yellow.on_default())
        .usage(clap::builder::styling::AnsiColor::Green.on_default())
        .literal(clap::builder::styling::AnsiColor::Cyan.on_default())
        .placeholder(clap::builder::styling::AnsiColor::Cyan.on_default())
        .error(clap::builder::styling::AnsiColor::Red.on_default())
        .valid(clap::builder::styling::AnsiColor::Green.on_default())
        .invalid(clap::builder::styling::AnsiColor::Red.on_default())
}

/// Python 模块定义
#[pymodule]
fn rmmcore(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // pyrmm.rmmcore.cli
    m.add_function(wrap_pyfunction!(cli, m)?)?;
    
    // 添加 RmmCore 类
    m.add_class::<PyRmmCore>()?;
    
    Ok(())
}