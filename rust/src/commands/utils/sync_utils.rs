/// 同步RMM元数据
use anyhow::Result;
use crate::commands::utils::core::config::RmmConfig;
use crate::commands::utils::core::project::ProjectConfig;

/// 更新项目版本
pub fn update_project_version(project_config: &mut ProjectConfig) -> Result<()> {
    println!("🔄 更新项目版本...");
    
    // 自动递增版本代码
    let old_version_code = project_config.version_code.clone();
    
    // 解析当前版本代码为数字，递增，然后转回字符串
    let current_code: u32 = project_config.version_code.parse().unwrap_or(1000000);
    let new_code = current_code + 1;
    project_config.version_code = new_code.to_string();
    
    println!("✅ 版本代码更新: {} -> {}", old_version_code, project_config.version_code);
    println!("✅ 当前版本: {:?}", project_config.version);
    
    Ok(())
}

pub fn sync_rmm_metadata(config: &RmmConfig, project_config: &mut ProjectConfig) -> Result<()> {
    println!("📋 同步RMM元数据...");
    
    // 更新requires_rmm版本
    let old_version = project_config.requires_rmm.clone();
    project_config.requires_rmm = config.version.clone();
    
    if old_version != project_config.requires_rmm {
        println!("🔄 更新RMM版本要求: {} -> {}", old_version, project_config.requires_rmm);
    } else {
        println!("✅ RMM版本要求已是最新: {}", project_config.requires_rmm);
    }
    
    // 将当前项目添加到全局 meta.toml 的项目列表中
    let mut rmm_config = RmmConfig::load()?;
    let current_dir = std::env::current_dir()?;
    
    // 使用新的方法添加当前项目
    rmm_config.add_current_project(&project_config.id, &current_dir)?;
    
    Ok(())
}

/// 同步依赖
pub fn sync_dependencies(config: &ProjectConfig, _force: bool, _include_dev: bool) -> Result<()> {
    println!("📦 同步依赖项...");
    
    // 显示当前依赖
    if !config.dependencies.is_empty() {
        println!("依赖项:");
        for dep in &config.dependencies {
            println!("  - {} ({})", dep.name, dep.version);
        }
    } else {
        println!("  无依赖项");
    }
    
    Ok(())
}
