use anyhow::Result;
use std::path::PathBuf;
use crate::core::RmmCore;

/// 示例：RmmCore 使用演示
pub fn main() -> Result<()> {
    println!("🚀 RmmCore 功能演示开始");

    // 创建 RmmCore 实例
    let core = RmmCore::new();
    
    println!("📁 RMM_ROOT 路径: {}", core.get_rmm_root().display());

    // 演示创建默认配置
    println!("\n📝 创建默认 Meta 配置...");
    let meta = core.create_default_meta(
        "example@gmail.com", 
        "example_user", 
        "0.1.0"
    );
    
    // 保存配置
    match core.update_meta_config(&meta) {
        Ok(_) => println!("✅ Meta 配置保存成功"),
        Err(e) => println!("❌ Meta 配置保存失败: {}", e),
    }

    // 读取配置
    match core.get_meta_config() {
        Ok(loaded_meta) => {
            println!("📖 读取 Meta 配置成功:");
            println!("   📧 Email: {}", loaded_meta.email);
            println!("   👤 Username: {}", loaded_meta.username);
            println!("   🔢 Version: {}", loaded_meta.version);
            println!("   📦 Projects: {} 个", loaded_meta.projects.len());
        }
        Err(e) => println!("❌ 读取 Meta 配置失败: {}", e),
    }

    // 演示 Git 信息检测
    println!("\n🔍 检测当前目录的 Git 信息...");
    let current_dir = std::env::current_dir()?;
    match core.get_git_info(&current_dir) {
        Ok(git_info) => {
            println!("📊 Git 信息:");
            if !git_info.repo_root.as_os_str().is_empty() {
                println!("   🏠 仓库根目录: {}", git_info.repo_root.display());
                println!("   📍 相对路径: {}", git_info.relative_path.display());
                if !git_info.branch.is_empty() {
                    println!("   🌿 当前分支: {}", git_info.branch);
                }
                if let Some(remote_url) = &git_info.remote_url {
                    println!("   🌐 远程URL: {}", remote_url);
                }
                println!("   📝 有未提交更改: {}", git_info.has_uncommitted_changes);
            } else {
                println!("   ⚠️  当前目录不在 Git 仓库中");
            }
        }
        Err(e) => println!("❌ Git 信息检测失败: {}", e),
    }

    // 演示项目扫描
    println!("\n🔍 扫描当前目录的项目...");
    match core.scan_projects(&current_dir, Some(3)) {
        Ok(projects) => {
            println!("📊 找到 {} 个项目:", projects.len());
            for project in &projects {
                println!("   📁 {}: {} (有效: {})", 
                    project.name, 
                    project.path.display(), 
                    project.is_valid
                );
                
                // 显示项目的 Git 信息
                if let Some(git_info) = &project.git_info {
                    if !git_info.repo_root.as_os_str().is_empty() {
                        println!("      🔗 Git: {} (分支: {})", 
                            git_info.repo_root.display(),
                            git_info.branch
                        );
                    }
                }
            }
        }
        Err(e) => println!("❌ 项目扫描失败: {}", e),
    }

    // 演示缓存统计
    let (meta_cached, project_count) = core.get_cache_stats();
    println!("\n📈 缓存统计:");
    println!("   🗂️  Meta 缓存: {}", if meta_cached { "已缓存" } else { "未缓存" });
    println!("   📁 项目缓存: {} 个", project_count);

    // 演示创建示例项目配置
    println!("\n📄 创建示例项目配置...");
    let project = core.create_default_project("example_project", "example_user", "example@gmail.com");
    println!("✅ 项目配置创建成功:");
    println!("   🆔 ID: {}", project.project.id);
    println!("   📝 描述: {}", project.project.description);
    println!("   👥 作者: {}", project.authors[0].name);

    // 演示创建 Module.prop
    println!("\n📋 创建示例 Module.prop...");
    let module_prop = core.create_default_module_prop("example_module", "example_user");
    println!("✅ Module.prop 创建成功:");
    println!("   🆔 ID: {}", module_prop.id);
    println!("   📛 名称: {}", module_prop.name);
    println!("   🔢 版本: {}", module_prop.version);

    // 演示创建 Rmake 配置
    println!("\n⚙️  创建示例 Rmake 配置...");
    let rmake = core.create_default_rmake();
    println!("✅ Rmake 配置创建成功:");
    println!("   📦 包含文件: {:?}", rmake.build.include);
    println!("   🚫 排除文件: {:?}", rmake.build.exclude);
    println!("   🔨 构建命令: {:?}", rmake.build.build);

    println!("\n🎉 RmmCore 功能演示完成！");
    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_full_workflow() -> Result<()> {
        // 设置临时目录
        let temp_dir = tempdir()?;
        unsafe {
            std::env::set_var("RMM_ROOT", temp_dir.path());
        }
        
        let core = RmmCore::new();
        
        // 1. 创建和保存 meta 配置
        let meta = core.create_default_meta("test@example.com", "testuser", "1.0.0");
        core.update_meta_config(&meta)?;
        
        // 2. 验证可以读取配置
        let loaded_meta = core.get_meta_config()?;
        assert_eq!(loaded_meta.email, "test@example.com");
        
        // 3. 创建测试项目
        let project_dir = temp_dir.path().join("test_project");
        fs::create_dir_all(&project_dir)?;
        
        let project = core.create_default_project("test_project", "testuser", "test@example.com");
        core.update_project_config(&project_dir, &project)?;
        
        // 4. 验证项目配置
        let loaded_project = core.get_project_config(&project_dir)?;
        assert_eq!(loaded_project.project.id, "test_project");
        
        // 5. 创建 module.prop
        let module_prop = core.create_default_module_prop("test_module", "testuser");
        core.update_module_prop(&project_dir, &module_prop)?;
        
        let loaded_prop = core.get_module_prop(&project_dir)?;
        assert_eq!(loaded_prop.id, "test_module");
        
        // 6. 创建 Rmake 配置
        let rmake = core.create_default_rmake();
        core.update_rmake_config(&project_dir, &rmake)?;
        
        let loaded_rmake = core.get_rmake_config(&project_dir)?;
        assert!(loaded_rmake.build.include.contains(&"rmm".to_string()));
        
        // 7. 测试 Git 信息
        let git_info = core.get_git_info(&project_dir)?;
        println!("项目 Git 信息: {:?}", git_info);
        
        println!("✅ 完整工作流测试通过");
        Ok(())
    }
    
    #[test]
    fn test_project_scanning_and_sync_with_git() -> Result<()> {
        let temp_dir = tempdir()?;
        unsafe {
            std::env::set_var("RMM_ROOT", temp_dir.path());
        }
        
        let core = RmmCore::new();
        
        // 创建多个测试项目
        let projects = ["project1", "project2", "project3"];
        for project_name in &projects {
            let project_dir = temp_dir.path().join(project_name);
            fs::create_dir_all(&project_dir)?;
            fs::write(project_dir.join("rmmproject.toml"), "")?;
        }
        
        // 扫描项目（包含 Git 信息）
        let scanned = core.scan_projects(temp_dir.path(), Some(2))?;
        assert_eq!(scanned.len(), 3);
        
        // 检查每个项目的 Git 信息
        for project in &scanned {
            println!("项目 {}: Git 信息 = {:?}", project.name, project.git_info);
            assert!(project.git_info.is_some());
        }
        
        // 同步项目到 meta 配置
        let scan_paths = vec![temp_dir.path()];
        core.sync_projects(&scan_paths, Some(2))?;
        
        // 验证同步结果
        let meta = core.get_meta_config()?;
        for project_name in &projects {
            assert!(meta.projects.contains_key(*project_name));
        }
        
        // 检查项目有效性
        let validity = core.check_projects_validity()?;
        for project_name in &projects {
            assert_eq!(validity.get(*project_name), Some(&true));
        }
        
        println!("✅ 项目扫描和同步（含Git）测试通过");
        Ok(())
    }
    
    #[test]
    fn test_cache_performance() -> Result<()> {
        let temp_dir = tempdir()?;
        unsafe {
            std::env::set_var("RMM_ROOT", temp_dir.path());
        }
        
        let core = RmmCore::new();
        
        // 创建配置
        let meta = core.create_default_meta("cache@test.com", "cacheuser", "1.0.0");
        core.update_meta_config(&meta)?;
        
        // 第一次读取（从文件）
        let start = std::time::Instant::now();
        let _meta1 = core.get_meta_config()?;
        let first_read_time = start.elapsed();
        
        // 第二次读取（从缓存）
        let start = std::time::Instant::now();
        let _meta2 = core.get_meta_config()?;
        let cached_read_time = start.elapsed();
        
        // 缓存读取应该更快
        println!("首次读取时间: {:?}", first_read_time);
        println!("缓存读取时间: {:?}", cached_read_time);
        
        // 验证缓存状态
        let (meta_cached, _) = core.get_cache_stats();
        assert!(meta_cached);
        
        println!("✅ 缓存性能测试通过");
        Ok(())
    }

    #[test]
    fn test_git_functionality() -> Result<()> {
        let core = RmmCore::new();
        let current_dir = std::env::current_dir()?;
        
        // 测试 Git 信息获取
        let git_info = core.get_git_info(&current_dir)?;
        println!("当前目录 Git 信息: {:?}", git_info);
        
        // 测试 Git 缓存
        let start = std::time::Instant::now();
        let _git_info1 = core.get_git_info(&current_dir)?;
        let first_time = start.elapsed();
        
        let start = std::time::Instant::now();
        let _git_info2 = core.get_git_info(&current_dir)?;
        let cached_time = start.elapsed();
        
        println!("首次 Git 信息获取: {:?}", first_time);
        println!("缓存 Git 信息获取: {:?}", cached_time);
        
        println!("✅ Git 功能测试通过");
        Ok(())
    }
}
