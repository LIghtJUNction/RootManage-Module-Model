use clap::{Arg, ArgMatches, Command};
use anyhow::Result;
use crate::config::{RmmConfig, ProjectConfig, RmakeConfig};
use std::path::Path;
use serde_json::json;
use pyo3::prelude::*;
use pyo3::types::PyModule;

pub fn build_command() -> Command {
    Command::new("publish")
        .about("发布模块到 GitHub Release")
        .arg(
            Arg::new("token")
                .long("token")
                .help("GitHub Personal Access Token")
                .value_name("TOKEN")
        )
        .arg(
            Arg::new("draft")
                .long("draft")
                .help("创建草稿发布")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("prerelease")
                .long("prerelease")
                .help("标记为预发布版本")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("message")
                .short('m')
                .long("message")
                .help("自定义发布说明")
                .value_name("MESSAGE")
        )
}

pub fn handle_publish(_config: &RmmConfig, matches: &ArgMatches) -> Result<()> {
    println!("🚀 准备发布模块到 GitHub...");
    
    // 检查 GitHub token (优先级: --token > GITHUB_ACCESS_TOKEN > GITHUB_TOKEN)
    let github_token = matches.get_one::<String>("token")
        .map(|s| s.clone())
        .or_else(|| std::env::var("GITHUB_ACCESS_TOKEN").ok())
        .or_else(|| std::env::var("GITHUB_TOKEN").ok());
    
    if github_token.is_none() {
        anyhow::bail!(
            "❌ 未找到 GitHub Token\n请通过以下方式之一设置：\n  1. 使用 --token 参数: rmm publish --token your_token_here\n  2. 设置环境变量: export GITHUB_ACCESS_TOKEN=your_token_here\n  3. 设置环境变量: export GITHUB_TOKEN=your_token_here"
        );
    }
    
    // 设置环境变量供 Python 脚本使用
    if let Some(token) = github_token {
        std::env::set_var("GITHUB_TOKEN", &token);
    }
      // 查找项目配置文件
    let current_dir = std::env::current_dir()?;
    let project_config_path = crate::config::find_project_file(&current_dir)?;
      // 加载项目配置
    let project_config = ProjectConfig::load_from_file(&project_config_path)?;
      // 加载 Rmake 配置（如果存在）
    let project_root = project_config_path.parent().unwrap();
    let rmake_config = RmakeConfig::load_from_dir(&project_root)?;
      // 获取版本信息（从项目配置中读取，而不是重新生成）
    let version = project_config.version.clone()
        .unwrap_or_else(|| "v0.1.0".to_string());
    let _version_code = project_config.version_code.clone();
      // 获取 Git 仓库信息
    let git_info = crate::utils::get_git_info(&project_root)
        .ok_or_else(|| anyhow::anyhow!("无法获取 Git 仓库信息"))?;
    let repo_name = format!("{}/{}", git_info.username, git_info.repo_name);
    
    // 构建输出路径
    let dist_dir = project_root.join(".rmmp").join("dist");    // 查找生成的文件 - 自动寻找最新的文件
    let (module_zip_path, source_tar_path) = find_latest_build_files(&dist_dir, &project_config.id)?;
    
    // 检查文件是否存在
    if !module_zip_path.exists() {
        anyhow::bail!("❌ 模块包不存在: {}\n请先运行 'rmm build' 构建项目", module_zip_path.display());
    }
    
    if !source_tar_path.exists() {
        anyhow::bail!("❌ 源码包不存在: {}\n请先运行 'rmm build' 构建项目", source_tar_path.display());
    }
    
    // 读取 CHANGELOG 作为 release body
    let changelog_path = project_root.join("CHANGELOG.md");
    let release_body = if changelog_path.exists() {
        std::fs::read_to_string(&changelog_path).unwrap_or_else(|_| {
            format!("## {} 发布说明\n\n此版本包含最新的功能更新和修复。", version)
        })
    } else {
        format!("## {} 发布说明\n\n此版本包含最新的功能更新和修复。", version)
    };
    
    // 自定义发布说明
    let final_release_body = if let Some(custom_message) = matches.get_one::<String>("message") {
        format!("{}\n\n---\n\n{}", custom_message, release_body)
    } else {
        release_body
    };
    
    // 检查是否启用代理功能
    let enable_proxy = rmake_config.as_ref()
        .and_then(|r| r.proxy.as_ref())
        .map(|p| p.enabled)
        .unwrap_or(false);
    
    // 准备发布配置
    let config_data = json!({
        "repo_name": repo_name,
        "version": version,
        "release_name": format!("{} - {}", project_config.name, version),
        "release_body": final_release_body,
        "module_zip_path": module_zip_path.to_string_lossy(),
        "source_tar_path": source_tar_path.to_string_lossy(),
        "enable_proxy": enable_proxy,
        "draft": matches.get_flag("draft"),
        "prerelease": matches.get_flag("prerelease")
    });
      // 获取文件名用于显示
    let zip_filename = module_zip_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("未知");
    let source_filename = source_tar_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("未知");
    
    println!("📦 版本: {}", version);
    println!("📁 仓库: {}", repo_name);
    println!("📄 模块包: {}", zip_filename);
    println!("📋 源码包: {}", source_filename);
    
    if enable_proxy {
        println!("🚀 已启用代理加速链接");
    }
    
    if matches.get_flag("draft") {
        println!("📝 模式: 草稿发布");
    }
    
    if matches.get_flag("prerelease") {
        println!("🧪 模式: 预发布版本");
    }
      // 查找 Python 发布脚本
    // let publisher_script = find_publisher_script(&project_root)?;
      // 调用 Python 发布函数 (通过 Rust 扩展模块)
    println!("🔄 正在发布...");
    let result = pyo3::Python::with_gil(|py| -> Result<bool> {
        // 导入 publisher 模块
        let publisher_module = PyModule::import(py, "pyrmm.publisher")
            .map_err(|e| anyhow::anyhow!("导入发布模块失败: {}", e))?;        // 将 JSON 配置转换为 Python 字典
        let json_str = config_data.to_string();
        
        // 导入 json 模块
        let json = PyModule::import(py, "json")
            .map_err(|e| anyhow::anyhow!("导入 json 模块失败: {}", e))?;
        
        // 调用 json.loads 函数
        let config_dict = json.getattr("loads")
            .map_err(|e| anyhow::anyhow!("获取 json.loads 函数失败: {}", e))?
            .call1((json_str,))
            .map_err(|e| anyhow::anyhow!("JSON 解析失败: {}", e))?;
        
        // 调用 publish_to_github 函数
        let result = publisher_module
            .getattr("publish_to_github")
            .map_err(|e| anyhow::anyhow!("找不到 publish_to_github 函数: {}", e))?
            .call1((config_dict,))
            .map_err(|e| anyhow::anyhow!("调用发布函数失败: {}", e))?;
          // 提取返回值
        result.extract::<bool>()
            .map_err(|e| anyhow::anyhow!("提取返回值失败: {}", e))
    })?;
    
    if result {
        println!("✅ 发布完成！");
        Ok(())
    } else {
        anyhow::bail!("❌ 发布失败");
    }
}

/// 在构建目录中寻找最新的模块文件
fn find_latest_build_files(dist_dir: &Path, project_id: &str) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
    if !dist_dir.exists() {
        anyhow::bail!("❌ 构建目录不存在: {}\n请先运行 'rmm build' 构建项目", dist_dir.display());
    }
    
    // 查找所有匹配的ZIP文件
    let mut zip_files = Vec::new();
    let mut tar_files = Vec::new();
    
    for entry in std::fs::read_dir(dist_dir)? {
        let entry = entry?;
        let path = entry.path();
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
          // 查找匹配项目ID的ZIP文件
        if filename.ends_with(".zip") && filename.starts_with(project_id) {
            let metadata = entry.metadata()?;
            zip_files.push((path.clone(), metadata.modified()?));
        }
        
        // 查找匹配项目ID的源码包
        if filename.ends_with("-source.tar.gz") && filename.starts_with(project_id) {
            let metadata = entry.metadata()?;
            tar_files.push((path.clone(), metadata.modified()?));
        }
    }
    
    if zip_files.is_empty() {
        anyhow::bail!("❌ 未找到模块包文件 ({}*.zip)\n请先运行 'rmm build' 构建项目", project_id);
    }
    
    if tar_files.is_empty() {
        anyhow::bail!("❌ 未找到源码包文件 ({}*-source.tar.gz)\n请先运行 'rmm build' 构建项目", project_id);
    }
    
    // 按修改时间排序，获取最新的文件
    zip_files.sort_by(|a, b| b.1.cmp(&a.1));
    tar_files.sort_by(|a, b| b.1.cmp(&a.1));
    
    let latest_zip = zip_files.into_iter().next().unwrap().0;
    let latest_tar = tar_files.into_iter().next().unwrap().0;
    
    println!("📦 找到最新模块包: {}", latest_zip.file_name().unwrap().to_string_lossy());
    println!("📋 找到最新源码包: {}", latest_tar.file_name().unwrap().to_string_lossy());
    
    Ok((latest_zip, latest_tar))
}
