use clap::{ValueEnum};
use clap_complete::{Shell};

/// 支持的 Shell 类型
#[derive(Debug, Clone, ValueEnum)]
pub enum SupportedShell {
    /// Bash shell
    Bash,
    /// Zsh shell  
    Zsh,
    /// Fish shell
    Fish,
    /// PowerShell
    Powershell,
    /// Elvish shell
    Elvish,
}

impl From<SupportedShell> for Shell {
    fn from(shell: SupportedShell) -> Self {
        match shell {
            SupportedShell::Bash => Shell::Bash,
            SupportedShell::Zsh => Shell::Zsh,
            SupportedShell::Fish => Shell::Fish,
            SupportedShell::Powershell => Shell::PowerShell,
            SupportedShell::Elvish => Shell::Elvish,
        }
    }
}
/// 打印安装说明
pub fn print_installation_instructions(shell: &SupportedShell, output_path: Option<&str>) {
    eprintln!("\n📋 安装说明:");
    
    match shell {
        SupportedShell::Bash => {
            eprintln!("将补全脚本添加到您的 .bashrc 或 .bash_profile:");
            if let Some(path) = output_path {
                eprintln!("  source {}", path);
            } else {
                eprintln!("  rmm completion bash > ~/.rmm_completion.bash");
                eprintln!("  echo 'source ~/.rmm_completion.bash' >> ~/.bashrc");
            }
            eprintln!("\n或者直接加载到当前会话:");
            eprintln!("  eval \"$(rmm completion bash)\"");
        }
        SupportedShell::Zsh => {
            eprintln!("对于 zsh，有几种方式安装补全:");
            eprintln!("1. 添加到您的 .zshrc:");
            if let Some(path) = output_path {
                eprintln!("   echo 'source {}' >> ~/.zshrc", path);
            } else {
                eprintln!("   echo 'eval \"$(rmm completion zsh)\"' >> ~/.zshrc");
            }
            eprintln!("2. 或者放置到 zsh 补全目录 (推荐):");
            eprintln!("   rmm completion zsh > ~/.zsh/completions/_rmm");
            eprintln!("   确保 ~/.zsh/completions 在您的 fpath 中");
        }
        SupportedShell::Fish => {
            eprintln!("对于 fish shell:");
            if let Some(path) = output_path {
                eprintln!("  cp {} ~/.config/fish/completions/rmm.fish", path);
            } else {
                eprintln!("  rmm completion fish > ~/.config/fish/completions/rmm.fish");
            }
        }
        SupportedShell::Powershell => {
            eprintln!("对于 PowerShell:");
            eprintln!("1. 找到您的 PowerShell 配置文件位置:");
            eprintln!("   $PROFILE");
            eprintln!("2. 将补全脚本添加到配置文件:");
            if let Some(path) = output_path {
                eprintln!("   . {}", path);
            } else {
                eprintln!("   rmm completion powershell | Out-String | Invoke-Expression");
            }
        }
        SupportedShell::Elvish => {
            eprintln!("对于 Elvish shell:");
            if let Some(path) = output_path {
                eprintln!("将以下内容添加到您的 ~/.elvish/rc.elv:");
                eprintln!("  use {}", path);
            } else {
                eprintln!("  rmm completion elvish > ~/.elvish/completions/rmm.elv");
                eprintln!("然后在 ~/.elvish/rc.elv 中添加:");
                eprintln!("  use ~/.elvish/completions/rmm");
            }
        }
    }
    
    eprintln!("\n💡 提示: 安装后需要重新启动 shell 或执行 'source' 命令以激活补全");
}


/// 获取 shell 安装帮助信息
pub fn get_shell_installation_help() -> &'static str {
    r#"
EXAMPLES:
    # 生成 bash 补全脚本并输出到标准输出
    rmm completion bash

    # 生成 zsh 补全脚本并保存到文件
    rmm completion zsh -o ~/.zsh/completions/_rmm

    # 生成 PowerShell 补全脚本
    rmm completion powershell > $PROFILE.CurrentUserAllHosts

    # 临时启用补全（bash）
    eval "$(rmm completion bash)"

SUPPORTED SHELLS:
    bash        Bourne Again Shell (最常用的 Linux shell)
    zsh         Z Shell (macOS 默认 shell)
    fish        Friendly Interactive Shell
    powershell  PowerShell (Windows 默认)
    elvish      Elvish Shell

INSTALLATION:
    生成的补全脚本需要被您的 shell 加载才能生效。
    每种 shell 的安装方法略有不同，请参考命令输出中的安装说明。
"#
}
