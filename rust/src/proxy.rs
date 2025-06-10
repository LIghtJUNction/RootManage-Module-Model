use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// GitHub 代理信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubProxy {
    pub url: String,
    pub server: String,
    pub ip: String,
    pub location: String,
    pub latency: u32,
    pub speed: f64,
}

/// 代理 API 响应
#[derive(Debug, Deserialize)]
struct ProxyApiResponse {
    code: u32,
    msg: String,
    data: Vec<GithubProxy>,
    total: u32,
    update_time: String,
}

/// 获取可用的 GitHub 代理列表
pub async fn get_github_proxies() -> Result<Vec<GithubProxy>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    
    let response = client
        .get("https://api.akams.cn/github")
        .send()
        .await?;
    
    if !response.status().is_success() {
        anyhow::bail!("获取代理列表失败: HTTP {}", response.status());
    }
    
    let api_response: ProxyApiResponse = response.json().await?;
    
    if api_response.code != 200 {
        anyhow::bail!("API 返回错误: {}", api_response.msg);
    }
    
    Ok(api_response.data)
}

/// 获取最快的 GitHub 代理
pub async fn get_fastest_proxy() -> Result<Option<GithubProxy>> {
    let proxies = get_github_proxies().await?;
    
    if proxies.is_empty() {
        return Ok(None);
    }
    
    // 按速度排序，选择最快的
    let mut sorted_proxies = proxies;
    sorted_proxies.sort_by(|a, b| b.speed.partial_cmp(&a.speed).unwrap_or(std::cmp::Ordering::Equal));
    
    Ok(sorted_proxies.into_iter().next())
}

/// 应用代理到 URL
pub fn apply_proxy_to_url(url: &str, proxy: Option<&GithubProxy>) -> String {
    match proxy {
        Some(proxy_info) => {
            if url.starts_with("https://raw.githubusercontent.com/") || 
               url.starts_with("https://github.com/") {
                format!("{}/{}", proxy_info.url, url)
            } else {
                url.to_string()
            }
        }
        None => url.to_string(),
    }
}

/// 测试代理连接性
pub async fn test_proxy_connectivity(proxy: &GithubProxy) -> Result<bool> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    
    let test_url = format!("{}/https://api.github.com", proxy.url);
    
    match client.head(&test_url).send().await {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_get_proxies() {
        match get_github_proxies().await {
            Ok(proxies) => {
                println!("获取到 {} 个代理", proxies.len());
                for proxy in proxies.iter().take(3) {
                    println!("代理: {} (速度: {:.2})", proxy.url, proxy.speed);
                }
            }
            Err(e) => {
                println!("获取代理失败: {}", e);
            }
        }
    }
    
    #[tokio::test]
    async fn test_fastest_proxy() {
        match get_fastest_proxy().await {
            Ok(Some(proxy)) => {
                println!("最快代理: {} (速度: {:.2})", proxy.url, proxy.speed);
            }
            Ok(None) => {
                println!("没有可用代理");
            }
            Err(e) => {
                println!("获取最快代理失败: {}", e);
            }
        }
    }
}
