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
    }

    // 设置 GitHub Token
    if let Some(token) = matches.get_one::<String>("github.token") {
        config.github_token = Some(token.clone());
        updated = true;
        println!("✅ GitHub Token 已设置");
    }

    // 显示当前配置
    if matches.get_flag("list") || !updated {
        println!("📋 当前配置:");
        println!("  user.name = {}", config.username);
        println!("  user.email = {}", config.email);
        println!("  rmm.version = {}", config.version);
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
