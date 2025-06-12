
use anyhow::Result;
use clap::{Arg, ArgMatches, Command};
use crate::commands::utils::core::config::{RmmConfig, ProjectConfig};
use crate::commands::utils::core::common::ProjectManager;
use crate::commands::utils::core::executor::ScriptExecutor;

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
pub fn handle_run(_config: &RmmConfig, matches: &ArgMatches) -> Result<String> {    // 查找项目配置文件
    let current_dir = std::env::current_dir()?;
    let project_config_path = ProjectManager::find_project_file(&current_dir)?;
    let project_root = project_config_path.parent().unwrap();
    
    // 加载项目配置
    let project_config = ProjectConfig::load_from_file(&project_config_path)?;
      // 如果提供了脚本名称，运行脚本
    if let Some(script_name) = matches.get_one::<String>("script") {
        let extra_args: Vec<String> = matches.get_many::<String>("args")
            .unwrap_or_default()
            .map(|s| s.clone())
            .collect();
        
        ScriptExecutor::run_project_script(&project_config, script_name, &extra_args, project_root)?;
    } else {
        // 没有提供脚本名称，显示所有可用脚本
        ScriptExecutor::list_available_scripts(&project_config);
    }
    
    Ok("脚本执行完成".to_string())
}
