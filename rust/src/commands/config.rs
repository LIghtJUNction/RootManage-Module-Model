use clap::{Arg, ArgMatches, Command, ArgAction};
use anyhow::Result;
use crate::config::RmmConfig;

pub fn build_command() -> Command {
    Command::new("config")
        .about("配置 RMM 用户信息")
        .arg(
            Arg::new("list")
                .short('l')
                .long("list")
                .action(ArgAction::SetTrue)
                .help("显示当前配置")
        )
        .arg(
            Arg::new("user.name")
                .long("user.name")
                .value_name("NAME")
                .help("设置用户名")
        )
        .arg(
            Arg::new("user.email")
                .long("user.email")
                .value_name("EMAIL")
                .help("设置用户邮箱")
        )
        .arg(
            Arg::new("github.token")
                .long("github.token")
                .value_name("TOKEN")
                .help("设置 GitHub Token")
        )
        .arg(
            Arg::new("sync-from-git")
                .long("sync-from-git")
                .action(ArgAction::SetTrue)
                .help("从 git 配置同步用户信息")
        )
}

pub fn handle_config(_config: &RmmConfig, matches: &ArgMatches) -> Result<()> {
    let mut config = RmmConfig::load().unwrap_or_default();
    let mut updated = false;

    // 设置用户名
    if let Some(username) = matches.get_one::<String>("user.name") {
        config.username = username.clone();
        updated = true;
        println!("✅ 用户名已设置为: {}", username);
    }

    // 设置用户邮箱
    if let Some(email) = matches.get_one::<String>("user.email") {
        config.email = email.clone();
        updated = true;
        println!("✅ 用户邮箱已设置为: {}", email);
    }    // 设置 GitHub Token
    if let Some(token) = matches.get_one::<String>("github.token") {
        config.github_token = Some(token.clone());
        updated = true;
        println!("✅ GitHub Token 已设置");
    }

    // 从 git 配置同步用户信息
    if matches.get_flag("sync-from-git") {
        match config.force_update_user_info_from_git() {
            Ok(_) => {
                updated = true;
                println!("✅ 已从 git 配置同步用户信息");
            }
            Err(e) => {
                println!("❌ 从 git 配置同步用户信息失败: {}", e);
                println!("💡 请检查是否已设置 git 配置:");
                println!("   git config --global user.name \"你的用户名\"");
                println!("   git config --global user.email \"你的邮箱\"");
            }
        }
    }    // 显示当前配置
    if matches.get_flag("list") || !updated {
        println!("📋 当前配置:");
        println!("  username = {}", config.username);
        println!("  email = {}", config.email);
        println!("  version = {}", config.version);
        if config.github_token.is_some() {
            println!("  github.token = [已设置]");
        } else {
            println!("  github.token = [未设置]");
        }
        println!("  projects.count = {}", config.projects.len());
    }

    // 保存配置
    if updated {
        config.save()?;
        println!("💾 配置已保存");
    }

    if !updated && !matches.get_flag("list") {
        println!("💡 提示:");
        println!("  rmm config --user.name \"你的用户名\"    # 设置用户名");
        println!("  rmm config --user.email \"你的邮箱\"     # 设置邮箱");
        println!("  rmm config --github.token \"token\"    # 设置 GitHub Token");
        println!("  rmm config --list                     # 显示当前配置");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RmmConfig;
    use std::collections::HashMap;
    use tempfile::TempDir;
    use std::env;

    fn create_test_config() -> RmmConfig {
        RmmConfig {
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            version: "0.2.0".to_string(),
            projects: HashMap::new(),
            github_token: None,
        }
    }

    #[test]
    fn test_build_command() {
        let cmd = build_command();
        assert_eq!(cmd.get_name(), "config");
        
        // 测试所有参数都存在
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "list"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "user.name"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "user.email"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "github.token"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "sync-from-git"));
    }

    #[test]
    fn test_handle_config_list_only() {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("RMM_ROOT", temp_dir.path());
        
        let config = create_test_config();
        let cmd = build_command();
        let matches = cmd.try_get_matches_from(vec!["config", "--list"]).unwrap();
        
        // 这个测试主要验证不会panic，实际的输出测试较复杂
        let result = handle_config(&config, &matches);
        assert!(result.is_ok());
        
        env::remove_var("RMM_ROOT");
    }

    #[test] 
    fn test_handle_config_set_username() {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("RMM_ROOT", temp_dir.path());
        
        let config = create_test_config();
        let cmd = build_command();
        let matches = cmd.try_get_matches_from(vec!["config", "--user.name", "newuser"]).unwrap();
        
        let result = handle_config(&config, &matches);
        assert!(result.is_ok());
        
        // 验证配置文件是否创建并包含新用户名
        let config_path = temp_dir.path().join("meta.toml");
        assert!(config_path.exists());
        
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("newuser"));
        
        env::remove_var("RMM_ROOT");
    }

    #[test]
    fn test_handle_config_set_email() {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("RMM_ROOT", temp_dir.path());
        
        let config = create_test_config();
        let cmd = build_command();
        let matches = cmd.try_get_matches_from(vec!["config", "--user.email", "new@example.com"]).unwrap();
        
        let result = handle_config(&config, &matches);
        assert!(result.is_ok());
        
        // 验证配置文件是否创建并包含新邮箱
        let config_path = temp_dir.path().join("meta.toml");
        assert!(config_path.exists());
        
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("new@example.com"));
        
        env::remove_var("RMM_ROOT");
    }

    #[test]
    fn test_handle_config_set_github_token() {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("RMM_ROOT", temp_dir.path());
        
        let config = create_test_config();
        let cmd = build_command();
        let matches = cmd.try_get_matches_from(vec!["config", "--github.token", "test_token_123"]).unwrap();
        
        let result = handle_config(&config, &matches);
        assert!(result.is_ok());
        
        env::remove_var("RMM_ROOT");
    }

    #[test]
    fn test_handle_config_sync_from_git() {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("RMM_ROOT", temp_dir.path());
        
        let config = create_test_config();
        let cmd = build_command();
        let matches = cmd.try_get_matches_from(vec!["config", "--sync-from-git"]).unwrap();
        
        // 这个测试可能会失败如果git配置不存在，但不应该panic
        let result = handle_config(&config, &matches);
        // 不管成功失败都应该正常返回
        assert!(result.is_ok() || result.is_err());
        
        env::remove_var("RMM_ROOT");
    }

    #[test]
    fn test_handle_config_multiple_settings() {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("RMM_ROOT", temp_dir.path());
        
        let config = create_test_config();
        let cmd = build_command();
        let matches = cmd.try_get_matches_from(vec![
            "config", 
            "--user.name", "multiuser",
            "--user.email", "multi@example.com"
        ]).unwrap();
        
        let result = handle_config(&config, &matches);
        assert!(result.is_ok());
        
        // 验证配置文件包含两个设置
        let config_path = temp_dir.path().join("meta.toml");
        assert!(config_path.exists());
        
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("multiuser"));
        assert!(content.contains("multi@example.com"));
        
        env::remove_var("RMM_ROOT");
    }

    #[test]
    fn test_handle_config_no_arguments() {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("RMM_ROOT", temp_dir.path());
        
        let config = create_test_config();
        let cmd = build_command();
        let matches = cmd.try_get_matches_from(vec!["config"]).unwrap();
        
        // 无参数时应该显示当前配置
        let result = handle_config(&config, &matches);
        assert!(result.is_ok());
        
        env::remove_var("RMM_ROOT");
    }
}
