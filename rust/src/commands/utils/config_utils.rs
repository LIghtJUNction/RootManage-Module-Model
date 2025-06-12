

use anyhow::Result;
use crate::commands::utils::core::config::RmmConfig;

pub fn validate_config(config: &RmmConfig) -> Result<()> {
    if config.username.trim().is_empty() {
        return Err(anyhow::anyhow!("用户名不能为空"));
    }
    
    if config.email.trim().is_empty() {
        return Err(anyhow::anyhow!("邮箱不能为空"));
    }
    
    Ok(())
}

pub fn display_config(config: &RmmConfig) -> Result<String> {
    let output = format!(
        "RMM 配置:\n用户名: {}\n邮箱: {}\n版本: {}",
        config.username,
        config.email,
        config.version
    );
    Ok(output)
}
