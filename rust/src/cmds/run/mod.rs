use anyhow::Result;
use colored::Colorize;
use std::path::Path;

use crate::core::rmm_core::RmmCore;

/// 运行 rmmproject.toml 中定义的脚本
pub fn run_script(project_path: &Path, script_name: Option<&str>) -> Result<()> {
    let core = RmmCore::new();
    
    // 检查项目是否有效
    if !is_valid_project(project_path) {
        anyhow::bail!("当前目录不是有效的 RMM 项目");
    }
    
    if let Some(script) = script_name {
        // 运行指定脚本
        execute_specific_script(&core, project_path, script)
    } else {
        // 列出所有可用脚本
        list_available_scripts(&core, project_path)
    }
}

/// 执行指定的脚本
fn execute_specific_script(core: &RmmCore, project_path: &Path, script_name: &str) -> Result<()> {
    println!("{} 运行脚本: {}", "[🚀]".cyan().bold(), script_name.yellow().bold());
    
    // 读取项目配置
    let project_config = core.get_project_config(project_path)?;
    
    // 检查脚本是否存在
    if let Some(scripts) = &project_config.project.scripts {
        if let Some(script_command) = scripts.get(script_name) {
            println!("{} {}", "[命令]".blue().bold(), script_command.bright_black());
            
            // 执行脚本命令
            execute_command(project_path, script_command)?;
            
            println!("{} 脚本执行完成", "[✅]".green().bold());
            Ok(())
        } else {
            // 脚本未找到，显示可用脚本列表
            eprintln!("{} 脚本 '{}' 未找到", "❌".red().bold(), script_name.yellow());
            list_available_scripts(core, project_path)?;
            anyhow::bail!("脚本 '{}' 未找到", script_name);
        }
    } else {
        anyhow::bail!("项目配置中未定义任何脚本");
    }
}

/// 列出所有可用的脚本
fn list_available_scripts(core: &RmmCore, project_path: &Path) -> Result<()> {
    let project_config = core.get_project_config(project_path)?;
    
    if let Some(scripts) = &project_config.project.scripts {
        if scripts.is_empty() {
            println!("{} 当前项目没有定义任何脚本", "ℹ️".blue().bold());
            println!("{} 你可以在 {} 中添加脚本", 
                "💡".yellow().bold(), 
                "rmmproject.toml".cyan().bold()
            );
            return Ok(());
        }
        
        println!("\n{} 可用脚本:", "📋".blue().bold());
        println!();
        
        // 按字母顺序排序脚本
        let mut script_pairs: Vec<_> = scripts.iter().collect();
        script_pairs.sort_by(|a, b| a.0.cmp(b.0));
        
        for (name, command) in script_pairs {
            println!("  {} {}", 
                name.green().bold(), 
                command.bright_black()
            );
        }
        
        println!();
        println!("{} 使用方法: {} {}", 
            "💡".yellow().bold(),
            "rmm run".cyan().bold(), 
            "<script_name>".yellow()
        );
    } else {
        println!("{} 当前项目没有定义任何脚本", "ℹ️".blue().bold());
        println!("{} 你可以在 {} 中添加脚本配置:", 
            "💡".yellow().bold(), 
            "rmmproject.toml".cyan().bold()
        );
        println!();
        println!("{}[project.scripts]", "  ".dimmed());
        println!("{}hello = \"echo 'hello world!'\"", "  ".dimmed());
    }
    
    Ok(())
}

/// 执行命令
fn execute_command(project_path: &Path, command: &str) -> Result<()> {
    use std::process::Command;
    
    // 执行命令 - 使用系统默认终端
    let mut cmd = if cfg!(target_os = "windows") {
        // Windows: 使用PowerShell避免UNC路径问题
        let mut cmd = Command::new("powershell");
        cmd.arg("-Command")
           .arg(&format!("cd '{}'; {}", project_path.display(), command));
        cmd
    } else {
        // Unix/Linux: 使用sh
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);
        cmd.current_dir(project_path);
        cmd
    };
    
    let output = cmd.output()?;
    
    // 输出命令结果
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    
    if !output.status.success() {
        anyhow::bail!("命令执行失败，退出码: {:?}", output.status.code());
    }
    
    Ok(())
}

/// 检查是否是有效的项目
fn is_valid_project(project_path: &Path) -> bool {
    let rmmp_dir = project_path.join(".rmmp");
    let rmake_file = rmmp_dir.join("Rmake.toml");
    let module_prop = project_path.join("module.prop");
    
    rmmp_dir.exists() && rmake_file.exists() && module_prop.exists()
}

#[cfg(test)]
mod tests {    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_is_valid_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // 创建必要的文件和目录
        let rmmp_dir = project_path.join(".rmmp");
        fs::create_dir_all(&rmmp_dir).unwrap();
        
        let rmake_file = rmmp_dir.join("Rmake.toml");
        fs::write(&rmake_file, "[build]\nscripts = {}").unwrap();
        
        let module_prop = project_path.join("module.prop");
        fs::write(&module_prop, "id=test\nname=Test").unwrap();
        
        assert!(is_valid_project(project_path));
        
        // 删除一个文件，应该返回false
        fs::remove_file(&module_prop).unwrap();
        assert!(!is_valid_project(project_path));
    }

    #[test]
    fn test_run_script_invalid_project() {
        let temp_dir = TempDir::new().unwrap();
        let result = run_script(temp_dir.path(), Some("test"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不是有效的 RMM 项目"));
    }
}
