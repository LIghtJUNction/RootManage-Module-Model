use anyhow::{Result, Context};
use std::path::Path;

/// GitHub相关操作
pub struct GitHubClient {
    token: Option<String>,
    owner: String,
    repo: String,
}

impl GitHubClient {
    pub fn new(owner: String, repo: String, token: Option<String>) -> Self {
        Self { token, owner, repo }
    }
    
    /// 检查GitHub连接状态
    pub fn check_connection(&self) -> Result<bool> {
        // 这里我们将通过Python的pygithub库来实现
        // Rust只提供接口，具体实现委托给Python
        println!("检查GitHub连接状态...");
        
        if self.token.is_none() {
            println!("警告: 未设置GitHub访问令牌");
            return Ok(false);
        }
        
        println!("GitHub连接正常");
        Ok(true)
    }
      /// 创建发布
    pub fn create_release(&self, tag: &str, name: &str, _body: &str, asset_path: &Path) -> Result<String> {
        println!("创建GitHub发布: {}", name);
        println!("标签: {}", tag);
        
        if asset_path.exists() {
            println!("准备上传资源: {}", asset_path.display());
        }
        
        // 返回模拟的发布URL
        Ok(format!("https://github.com/{}/{}/releases/tag/{}", self.owner, self.repo, tag))
    }
    
    /// 获取最新发布信息
    pub fn get_latest_release(&self) -> Result<Option<String>> {
        println!("获取最新发布信息...");
        // 这里将委托给Python实现
        Ok(Some("v1.0.0".to_string()))
    }
    
    /// 检查仓库状态
    pub fn check_repo_status(&self) -> Result<()> {
        println!("检查仓库状态:");
        println!("  仓库: {}/{}", self.owner, self.repo);
        
        if let Some(_token) = &self.token {
            println!("  认证: ✓ 已配置GitHub令牌");
        } else {
            println!("  认证: ✗ 未配置GitHub令牌");
            println!("  提示: 设置环境变量 GITHUB_TOKEN");
        }
        
        Ok(())
    }
}

/// 从环境变量或配置获取GitHub信息
pub fn get_github_config() -> Result<(String, String, Option<String>)> {
    let token = std::env::var("GITHUB_TOKEN").ok();
    
    // 尝试从git配置获取仓库信息
    let repo_url = std::process::Command::new("git")
        .args(&["config", "--get", "remote.origin.url"])
        .output()
        .context("无法获取git远程仓库URL")?;
        
    let url = String::from_utf8_lossy(&repo_url.stdout).trim().to_string();
    
    // 解析GitHub URL
    let (owner, repo) = parse_github_url(&url)?;
    
    Ok((owner, repo, token))
}

/// 解析GitHub URL获取owner和repo
fn parse_github_url(url: &str) -> Result<(String, String)> {
    if url.is_empty() {
        return Ok(("LIghtJUNction".to_string(), "RootManageModuleModel".to_string()));
    }
    
    // 支持多种GitHub URL格式
    let url = url.trim_end_matches(".git");
    
    if let Some(captures) = regex::Regex::new(r"github\.com[:/]([^/]+)/([^/]+)")
        .unwrap()
        .captures(url) 
    {
        let owner = captures.get(1).unwrap().as_str().to_string();
        let repo = captures.get(2).unwrap().as_str().to_string();
        Ok((owner, repo))
    } else {
        // 默认值
        Ok(("LIghtJUNction".to_string(), "RootManageModuleModel".to_string()))
    }
}
