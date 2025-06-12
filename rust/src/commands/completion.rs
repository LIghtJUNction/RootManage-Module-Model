use anyhow::Result;
use clap::{Arg, ArgMatches, Command};
use crate::commands::utils::core::config::RmmConfig;
use crate::commands::utils::core::executor::{CompletionManager, SupportedShell};
use clap_complete::{Shell, generate};
use std::io;

/// 构建 completion 命令
pub fn build_command() -> Command {
    Command::new("completion")
        .about("生成命令补全脚本")
        .long_about("为不同的 shell 生成命令补全脚本，支持 bash、zsh、fish、powershell")
        .arg(
            Arg::new("shell")
                .help("要生成补全脚本的 shell 类型")
                .required(true)
                .value_parser(["bash", "zsh", "fish", "powershell", "cmd"])
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("输出文件路径（默认输出到标准输出）")
                .value_name("FILE")
        )
        .after_help("安装说明:\n  bash: 添加到 ~/.bashrc: eval \"$(rmm completion bash)\"\n  zsh: 添加到 ~/.zshrc: eval \"$(rmm completion zsh)\"\n  fish: 添加到 ~/.config/fish/config.fish: rmm completion fish | source\n  powershell: 添加到 PowerShell 配置: Invoke-Expression (rmm completion powershell)")
}

/// 处理 completion 命令
pub fn handle_completion(_config: &RmmConfig, matches: &ArgMatches) -> Result<String> {
    let shell_str = matches.get_one::<String>("shell").unwrap();
    
    // 转换为 SupportedShell 枚举
    let shell = match shell_str.to_lowercase().as_str() {
        "bash" => SupportedShell::Bash,
        "zsh" => SupportedShell::Zsh,
        "fish" => SupportedShell::Fish,
        "powershell" => SupportedShell::PowerShell,
        "cmd" => SupportedShell::Cmd,
        _ => return Err(anyhow::anyhow!("不支持的 shell: {}", shell_str)),
    };
    
    // 检查是否为不支持的 CMD
    if matches!(shell, SupportedShell::Cmd) {
        println!("❌ Windows CMD 不支持自动补全");
        return Ok("CMD 不支持补全".to_string());
    }
    
    // 转换为 clap 的 Shell 枚举
    let shell_type: Shell = match shell {
        SupportedShell::Bash => Shell::Bash,
        SupportedShell::Zsh => Shell::Zsh,
        SupportedShell::Fish => Shell::Fish,
        SupportedShell::PowerShell => Shell::PowerShell,
        SupportedShell::Cmd => return Err(anyhow::anyhow!("CMD 不支持补全")),
    };
    
    // 获取主命令
    let mut cmd = crate::build_cli();
    let name = cmd.get_name().to_string();
    
    // 生成补全脚本
    if let Some(output_path) = matches.get_one::<String>("output") {
        // 输出到文件
        let mut file = std::fs::File::create(output_path)?;
        generate(shell_type, &mut cmd, name, &mut file);
        println!("✅ 补全脚本已生成到: {}", output_path);
        
        // 显示安装指南
        CompletionManager::print_installation_instructions(shell);
        
        Ok("补全脚本生成成功".to_string())
    } else {
        // 输出到标准输出
        generate(shell_type, &mut cmd, name, &mut io::stdout());
        eprintln!();
        
        // 显示安装指南
        CompletionManager::print_installation_instructions(shell);
        
        Ok("补全脚本输出完成".to_string())
    }
}
