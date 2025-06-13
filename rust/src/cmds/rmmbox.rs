use colored::*;

/// RMMBox 基本工具集 - 纯 Rust 结构体设计
#[derive(Debug, Clone)]
pub struct RmmBox;

impl RmmBox {
    /// 显示版本信息
    pub fn rmm_version() {
        println!("{}", env!("CARGO_PKG_DESCRIPTION").green().bold());
        println!("{} Version {}", 
            env!("CARGO_PKG_NAME").cyan().bold(), 
            env!("CARGO_PKG_VERSION").magenta().bold()
        );
        // GITHUB 
        println!("GitHub: {}", env!("CARGO_PKG_HOMEPAGE").blue().bold());
        // 团队
        println!("RMMDEVS: {}", env!("CARGO_PKG_AUTHORS").yellow().bold());
    }    
    
}

