use clap::{Arg, ArgMatches, Command};
use anyhow::Result;
use std::path::Path;
use crate::config::ProjectConfig;
use crate::utils::find_or_create_project_config;

/// 构建 run 命令
pub fn build_command() -> Command {
    Command::new("run")
        .about("运行项目脚本")
        .long_about("运行在 rmmproject.toml 中定义的脚本，类似于 npm run")
        .arg(
            Arg::new("script")
                .help("要运行的脚本名称")
                .value_name("SCRIPT_NAME")
                .required(false) // 改为可选
        )
        .arg(
            Arg::new("args")
                .help("传递给脚本的额外参数")
                .value_name("ARGS")
                .action(clap::ArgAction::Append)
                .last(true)
        )
}

/// 处理 run 命令
pub fn handle_run(_config: &crate::config::RmmConfig, matches: &ArgMatches) -> Result<String> {
    // 查找项目配置文件
    let current_dir = std::env::current_dir()?;
    let project_config_path = find_or_create_project_config(&current_dir)?;
    let project_root = project_config_path.parent().unwrap();
    
    // 加载项目配置
    let project_config = ProjectConfig::load_from_file(&project_config_path)?;
    
    // 如果没有提供脚本名称，列出所有可用脚本
    if let Some(script_name) = matches.get_one::<String>("script") {
        let extra_args: Vec<&String> = matches.get_many::<String>("args").unwrap_or_default().collect();
        
        println!("🔧 运行脚本: {}", script_name);
        
        // 查找脚本
        let script_command = project_config.scripts.get(script_name)
            .ok_or_else(|| anyhow::anyhow!("❌ 未找到脚本 '{}'", script_name))?;
            
        // 构建完整命令（包含额外参数）
        let mut full_command = script_command.clone();
        if !extra_args.is_empty() {
            full_command.push(' ');
            let args_str: Vec<String> = extra_args.iter().map(|s| s.to_string()).collect();
            full_command.push_str(&args_str.join(" "));
        }
        
        println!("📋 执行命令: {}", full_command);
        
        // 执行脚本命令
        execute_script_command(&full_command, project_root)?;
    } else {
        // 没有提供脚本名称，显示所有可用脚本
        println!("📋 可用脚本:");
        
        if project_config.scripts.is_empty() {
            println!("  (没有定义任何脚本)");
            println!("");
            println!("💡 在 rmmproject.toml 中添加脚本:");
            println!("  [scripts]");
            println!("  build = \"rmm build\"");
            println!("  test = \"echo 'Running tests...'\"");
        } else {
            for (name, command) in &project_config.scripts {
                println!("  {} : {}", name, command);
            }
            println!("");            println!("💡 运行脚本: rmm run <script_name>");
        }
    }
    
    Ok("脚本执行完成".to_string())
}

/// 执行脚本命令
fn execute_script_command(command: &str, working_dir: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("powershell")
            .args(&["-Command", command])
            .current_dir(working_dir)
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("脚本执行失败: {}", stderr);
        }
        
        // 输出命令结果
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            print!("{}", stdout);
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let output = std::process::Command::new("sh")
            .args(&["-c", command])
            .current_dir(working_dir)
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("脚本执行失败: {}", stderr);
        }
        
        // 输出命令结果
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            print!("{}", stdout);
        }
    }
    
    Ok(())
}
