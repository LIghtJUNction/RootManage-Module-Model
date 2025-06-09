use clap::{Arg, ArgMatches, Command};
use anyhow::Result;
use std::path::Path;
use std::process::Command as StdCommand;
use crate::utils::{Context, RmmProject};

pub fn run_command() -> Command {
    Command::new("run")
        .about("运行项目脚本 (灵感来自npm)")
        .arg(
            Arg::new("script_name")
                .help("要运行的脚本名称")
                .value_name("SCRIPT_NAME")
                .required(false)
        )
        .arg(
            Arg::new("args")
                .help("传递给脚本的参数")
                .value_name("ARGS")
                .action(clap::ArgAction::Append)
                .last(true) // 允许在脚本名称后接受任意参数
        )
}

pub fn handle_run(ctx: &Context, matches: &ArgMatches) -> Result<()> {
    let script_name = matches.get_one::<String>("script_name");
    let script_args: Vec<&String> = matches.get_many::<String>("args").unwrap_or_default().collect();

    // 获取当前项目信息
    let current_dir = std::env::current_dir()?;
    let project = match RmmProject::load_current() {
        Ok(project) => project,
        Err(_) => {
            anyhow::bail!("❌ 当前目录不是一个有效的RMM项目");
        }
    };

    if script_name.is_none() {
        // 显示可用的脚本列表
        show_available_scripts(ctx, &project)?;
        return Ok(());
    }

    let script_name = script_name.unwrap();

    // 查找脚本配置
    let script_command = find_script_command(&project, script_name)?;

    ctx.info(&format!("🚀 运行脚本: {}", script_name));
    ctx.debug(&format!("📜 脚本命令: {}", script_command));

    // 执行脚本
    execute_script(ctx, &script_command, &script_args, &current_dir)?;

    ctx.info(&format!("✅ 脚本 '{}' 执行完成", script_name));

    Ok(())
}

fn show_available_scripts(ctx: &Context, project: &RmmProject) -> Result<()> {
    ctx.info("📜 可用的脚本:");

    if let Some(build_config) = &project.build {
        if let Some(scripts) = &build_config.scripts {
            if scripts.is_empty() {
                ctx.info("  (没有定义脚本)");
            } else {
                for (name, command) in scripts {
                    ctx.info(&format!("  {} - {}", name, command));
                }
            }
        } else {
            ctx.info("  (没有定义脚本)");
        }
    } else {
        ctx.info("  (没有定义脚本)");
    }

    ctx.info("\n使用方法: rmm run <script_name> [args...]");

    Ok(())
}

fn find_script_command(project: &RmmProject, script_name: &str) -> Result<String> {
    if let Some(build_config) = &project.build {
        if let Some(scripts) = &build_config.scripts {
            if let Some(command) = scripts.get(script_name) {
                return Ok(command.clone());
            }
        }
    }

    // 检查是否是预定义的脚本
    match script_name {
        "build" => Ok("rmm build".to_string()),
        "clean" => Ok("rmm clean".to_string()),
        "test" => Ok("rmm test".to_string()),
        _ => anyhow::bail!("❌ 脚本 '{}' 未找到。使用 'rmm run' 查看可用脚本", script_name),
    }
}

fn execute_script(
    ctx: &Context,
    script_command: &str,
    script_args: &[&String],
    working_dir: &Path,
) -> Result<()> {
    // 解析脚本命令
    let parts: Vec<&str> = script_command.split_whitespace().collect();
    if parts.is_empty() {
        anyhow::bail!("❌ 脚本命令为空");
    }

    let mut command = StdCommand::new(parts[0]);
    
    // 添加脚本命令的参数
    if parts.len() > 1 {
        command.args(&parts[1..]);
    }
    
    // 添加用户传递的参数
    if !script_args.is_empty() {
        command.args(script_args);
    }

    // 设置工作目录
    command.current_dir(working_dir);

    // 继承标准输入输出，让用户可以与脚本交互
    command.stdin(std::process::Stdio::inherit());
    command.stdout(std::process::Stdio::inherit());
    command.stderr(std::process::Stdio::inherit());

    ctx.debug(&format!("🔧 执行命令: {} {:?}", parts[0], command));

    // 执行命令
    let status = command.status()?;

    if !status.success() {
        let exit_code = status.code().unwrap_or(-1);
        anyhow::bail!("❌ 脚本执行失败，退出代码: {}", exit_code);
    }

    Ok(())
}

// 扩展功能：支持脚本钩子
pub fn run_script_hook(ctx: &Context, hook_name: &str) -> Result<()> {
    if let Ok(project) = RmmProject::load_current() {
        let script_name = format!("pre{}", hook_name);
        if script_exists(&project, &script_name) {
            ctx.debug(&format!("🪝 运行前置钩子: {}", script_name));
            run_script_by_name(ctx, &project, &script_name)?;
        }

        let script_name = format!("post{}", hook_name);
        if script_exists(&project, &script_name) {
            ctx.debug(&format!("🪝 运行后置钩子: {}", script_name));
            run_script_by_name(ctx, &project, &script_name)?;
        }
    }

    Ok(())
}

fn script_exists(project: &RmmProject, script_name: &str) -> bool {
    if let Some(build_config) = &project.build {
        if let Some(scripts) = &build_config.scripts {
            return scripts.contains_key(script_name);
        }
    }
    false
}

fn run_script_by_name(ctx: &Context, project: &RmmProject, script_name: &str) -> Result<()> {
    let script_command = find_script_command(project, script_name)?;
    let current_dir = std::env::current_dir()?;
    execute_script(ctx, &script_command, &[], &current_dir)
}
