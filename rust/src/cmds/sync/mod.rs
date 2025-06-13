use anyhow::Result;
use colored::Colorize;
use std::path::Path;

use crate::core::rmm_core::RmmCore;

/// 检查项目是否有效
fn is_valid_project(project_path: &Path) -> bool {
    project_path.exists() && 
    project_path.is_dir() && 
    project_path.join("rmmproject.toml").exists() &&
    project_path.join(".rmmp").exists() &&
    project_path.join(".rmmp").join("Rmake.toml").exists()
}

/// 同步项目元数据，清理无效项目并发现新项目
pub fn sync_projects(
    project_name: Option<&str>,
    projects_only: bool,
    search_paths: Option<Vec<&str>>,
    max_depth: Option<usize>,
) -> Result<()> {
    let core = RmmCore::new();
    
    println!("{} 开始同步项目...", "[🔄]".cyan().bold());
    
    if let Some(name) = project_name {
        // 同步特定项目
        sync_specific_project(&core, name)?;
    } else {
        // 同步所有项目
        sync_all_projects(&core, projects_only, search_paths, max_depth)?;
    }
    
    println!("{} 项目同步完成", "[✅]".green().bold());
    Ok(())
}

/// 同步特定项目
fn sync_specific_project(core: &RmmCore, project_name: &str) -> Result<()> {
    println!("{} 同步项目: {}", "[📋]".blue().bold(), project_name.yellow().bold());
    
    // 获取当前 meta 配置
    let mut meta = core.get_meta_config()?;
    
    // 检查项目是否存在于 meta 中
    if let Some(project_path) = meta.projects.get(project_name) {
        let project_path = Path::new(project_path);
          // 检查项目是否仍然有效
        if is_valid_project(project_path) {
            println!("  ✅ 项目 {} 有效", project_name.green());
            
            // 更新项目配置
            match core.get_project_config(project_path) {
                Ok(project_config) => {
                    println!("  📄 项目配置已更新");
                    println!("     ID: {}", project_config.project.id.bright_white());                    if !project_config.project.description.is_empty() {
                        println!("     描述: {}", project_config.project.description.bright_black());
                    }
                }
                Err(e) => {
                    println!("  ⚠️  无法读取项目配置: {}", e.to_string().yellow());
                }
            }
        } else {
            println!("  ❌ 项目 {} 无效，从 meta 中移除", project_name.red());
            meta.projects.remove(project_name);
            core.update_meta_config(&meta)?;
        }
    } else {
        println!("  ❓ 项目 {} 不存在于 meta.toml 中", project_name.yellow());
        
        // 尝试在常见位置查找项目
        let rmm_root = core.get_rmm_root();
        let search_paths = vec![
            rmm_root.parent().unwrap_or(&rmm_root),
            Path::new("."),
        ];
        
        for search_path in search_paths {            if let Ok(found_projects) = core.scan_projects(search_path, Some(3)) {
                for project in found_projects {
                    if project.name == project_name {
                        println!("  🔍 找到项目: {}", project.path.display().to_string().green());
                        meta.projects.insert(project.name, project.path.display().to_string());
                        core.update_meta_config(&meta)?;
                        break;
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// 同步所有项目
fn sync_all_projects(
    core: &RmmCore,
    projects_only: bool,
    search_paths: Option<Vec<&str>>,
    max_depth: Option<usize>,
) -> Result<()> {
    // 1. 清理无效项目
    println!("{} 清理无效项目...", "[🗑️]".red().bold());
    let removed_projects = core.remove_invalid_projects()?;
    
    if removed_projects.is_empty() {
        println!("  ✅ 所有项目都有效");
    } else {
        println!("  🗑️  移除了 {} 个无效项目:", removed_projects.len());
        for project in &removed_projects {
            println!("    - {}", project.red());
        }
    }
    
    // 如果只同步项目列表，跳过依赖同步
    if projects_only {
        println!("{} 跳过依赖同步 (projects_only 模式)", "[⏭️]".yellow().bold());
        return Ok(());
    }
    
    // 2. 扫描新项目
    println!("{} 扫描新项目...", "[🔍]".blue().bold());    let search_paths: Vec<std::path::PathBuf> = if let Some(paths) = search_paths {
        paths.into_iter().map(|p| std::path::PathBuf::from(p)).collect()
    } else {
        // 默认搜索路径
        let rmm_root = core.get_rmm_root();
        let parent_path = rmm_root.parent().unwrap_or(&rmm_root).to_path_buf();
        vec![
            parent_path,
            std::path::PathBuf::from("."),
        ]
    };
    
    let max_depth = max_depth.unwrap_or(3);
    let mut new_projects_count = 0;
    let mut total_scanned = 0;
    
    for search_path in &search_paths {
        if !search_path.exists() {
            println!("  ⚠️  路径不存在: {}", search_path.display().to_string().yellow());
            continue;
        }
        
        println!("  📂 扫描路径: {} (深度: {})", 
                 search_path.display().to_string().cyan(), 
                 max_depth.to_string().bright_white());
        
        match core.scan_projects(search_path.as_path(), Some(max_depth)) {
            Ok(found_projects) => {
                total_scanned += found_projects.len();
                
                // 获取当前 meta 配置
                let mut meta = core.get_meta_config()?;
                let mut path_updates = 0;
                  for project in found_projects {
                    let project_name = &project.name;
                    let project_path = &project.path;
                    
                    if let Some(existing_path) = meta.projects.get(project_name) {
                        // 项目已存在，检查路径是否需要更新
                        if existing_path != &project_path.display().to_string() {
                            println!("    🔄 更新项目路径: {}", project_name.yellow());
                            println!("      旧路径: {}", existing_path.bright_black());
                            println!("      新路径: {}", project_path.display().to_string().green());
                            meta.projects.insert(project_name.clone(), project_path.display().to_string());
                            path_updates += 1;
                        }
                    } else {
                        // 新项目
                        println!("    ➕ 发现新项目: {}", project_name.green().bold());
                        println!("      路径: {}", project_path.display().to_string().bright_black());
                        meta.projects.insert(project_name.clone(), project_path.display().to_string());
                        new_projects_count += 1;
                    }
                }
                
                // 更新 meta 配置
                if new_projects_count > 0 || path_updates > 0 {
                    core.update_meta_config(&meta)?;
                }
                
                if path_updates > 0 {
                    println!("    🔄 更新了 {} 个项目路径", path_updates);
                }
            }
            Err(e) => {
                println!("  ❌ 扫描失败: {}", e.to_string().red());
            }
        }
    }
    
    // 3. 显示同步结果
    println!("\n{} 同步结果:", "[📊]".blue().bold());
    println!("  🗑️  移除无效项目: {}", removed_projects.len().to_string().red().bold());
    println!("  ➕ 发现新项目: {}", new_projects_count.to_string().green().bold());
    println!("  📂 总扫描项目: {}", total_scanned.to_string().cyan());
    
    // 4. 显示当前项目列表
    let final_meta = core.get_meta_config()?;
    if !final_meta.projects.is_empty() {
        println!("\n{} 当前项目列表:", "[📋]".blue().bold());
        let mut projects: Vec<_> = final_meta.projects.iter().collect();
        projects.sort_by(|a, b| a.0.cmp(b.0));
        
        for (name, path) in projects {
            let path_obj = Path::new(path);
            let status = if path_obj.exists() && is_valid_project(path_obj) {
                "✅".green()
            } else {
                "❌".red()
            };
            println!("  {} {} {}", status, name.bright_white(), path.bright_black());
        }
    } else {
        println!("\n{} 当前没有项目", "[ℹ️]".blue().bold());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sync_projects_basic() {
        // 测试基本同步功能
        let temp_dir = TempDir::new().unwrap();
        let result = sync_all_projects(
            &RmmCore::new(),
            false,
            Some(vec![temp_dir.path().to_str().unwrap()]),
            Some(2),
        );
        
        // 应该能够成功执行，即使没有找到项目
        assert!(result.is_ok());
    }

    #[test]
    fn test_sync_specific_project() {
        let result = sync_specific_project(&RmmCore::new(), "nonexistent_project");
        // 应该能够处理不存在的项目
        assert!(result.is_ok());
    }
}
