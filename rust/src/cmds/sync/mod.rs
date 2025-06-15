use anyhow::Result;
use clap::ArgMatches;
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;

use crate::core::rmm_core::{RmmCore, GitAnalyzer, MetaConfig};

/// 作者信息
#[derive(Debug, Clone, PartialEq)]
struct AuthorInfo {
    name: String,
    email: String,
}

impl AuthorInfo {
    fn is_default(&self) -> bool {
        self.name == "unknown" || self.name == "test" || 
        self.email == "unknown@example.com" || self.email == "test@example.com" ||
        self.name.is_empty() || self.email.is_empty()
    }
    
    fn from_git(path: &Path) -> Option<Self> {
        GitAnalyzer::analyze_git_info(path).ok().flatten().and_then(|git_info| {
            // 从 git config 获取用户信息
            if let Ok(repo) = git2::Repository::open(&git_info.repo_root) {
                let config = repo.config().ok()?;
                let name = config.get_string("user.name").ok()?;
                let email = config.get_string("user.email").ok()?;
                Some(AuthorInfo { name, email })
            } else {
                None
            }
        })
    }
}



/// 检查项目是否有效
fn is_valid_project(project_path: &Path) -> bool {
    project_path.exists() && 
    project_path.is_dir() && 
    project_path.join("rmmproject.toml").exists() &&
    project_path.join(".rmmp").exists() &&
    project_path.join(".rmmp").join("Rmake.toml").exists()
}

/// 版本管理
#[derive(Debug, Clone)]
struct VersionInfo {
    version: String,
    version_code: String,
}

impl VersionInfo {
    fn new(version: &str, version_code: &str) -> Self {
        Self {
            version: version.to_string(),
            version_code: version_code.to_string(),
        }
    }
      /// 智能版本升级 - 支持基于日期和Git的版本管理
    fn smart_bump_version(&mut self, project_path: &Path) {
        // 使用智能版本升级
        self.version = smart_version_bump(&self.version, project_path);
        
        // 生成新的版本代码        self.version_code = generate_version_code(project_path);
    }
    
    /// 从 module.prop 读取版本信息
    fn from_module_prop(project_path: &Path) -> Result<Self> {
        let module_prop_path = project_path.join("module.prop");
        let content = fs::read_to_string(module_prop_path)?;
        
        let mut version = String::new();
        let mut version_code = String::new();
        
        for line in content.lines() {
            if line.starts_with("version=") {
                version = line.trim_start_matches("version=").to_string();
            } else if line.starts_with("versionCode=") {
                version_code = line.trim_start_matches("versionCode=").to_string();
            }
        }
        
        Ok(VersionInfo::new(&version, &version_code))
    }
    
    /// 更新 module.prop 文件
    fn update_module_prop(&self, project_path: &Path) -> Result<()> {
        let module_prop_path = project_path.join("module.prop");
        let content = fs::read_to_string(&module_prop_path)?;
        
        let mut new_content = String::new();
        for line in content.lines() {
            if line.starts_with("version=") {
                new_content.push_str(&format!("version={}\n", self.version));
            } else if line.starts_with("versionCode=") {
                new_content.push_str(&format!("versionCode={}\n", self.version_code));
            } else {
                new_content.push_str(line);
                new_content.push('\n');
            }
        }
        
        fs::write(module_prop_path, new_content)?;
        Ok(())
    }
}

/// 同步项目元数据，清理无效项目并发现新项目
pub fn sync_projects(
    project_name: Option<&str>,
    projects_only: bool,
    fix_version: bool,
    search_paths: Option<Vec<&str>>,
    max_depth: Option<usize>,
) -> Result<()> {
    let core = RmmCore::new();
    
    println!("{} 开始同步项目...", "[🔄]".cyan().bold());
      if let Some(name) = project_name {
        // 同步特定项目
        sync_specific_project(&core, name, fix_version)?;
    } else {
        // 同步所有项目
        sync_all_projects(&core, projects_only, fix_version, search_paths, max_depth)?;
    }
    
    println!("{} 项目同步完成", "[✅]".green().bold());
    Ok(())
}

/// 同步特定项目
fn sync_specific_project(core: &RmmCore, project_name: &str, fix_version: bool) -> Result<()> {
    println!("{} 同步项目: {}", "[📋]".blue().bold(), project_name.yellow().bold());
    
    // 获取当前 meta 配置
    let mut meta = core.get_meta_config()?;
    
    // 检查项目是否存在于 meta 中
    if let Some(project_path_str) = meta.projects.get(project_name).cloned() {
        let project_path = Path::new(&project_path_str);
        
        // 检查项目是否仍然有效
        if is_valid_project(project_path) {
            println!("  ✅ 项目 {} 有效", project_name.green());
              // 执行完整的项目同步
            sync_project_metadata(core, project_path, fix_version, &mut meta)?;
            
        } else {
            println!("  ❌ 项目 {} 无效，从 meta 中移除", project_name.red());
            meta.projects.remove(project_name);
            core.update_meta_config(&meta)?;
        }
    } else {
        println!("  ❓ 项目 {} 不存在于 meta.toml 中", project_name.yellow());
        
        // 尝试在常见位置查找项目
        search_and_add_project(core, project_name, &mut meta)?;
    }
    
    Ok(())
}

/// 搜索并添加项目
fn search_and_add_project(core: &RmmCore, project_name: &str, meta: &mut crate::core::rmm_core::MetaConfig) -> Result<()> {
    let rmm_root = core.get_rmm_root();
    let search_paths = vec![
        rmm_root.parent().unwrap_or(&rmm_root),
        Path::new("."),
    ];
    
    for search_path in search_paths {
        if let Ok(found_projects) = core.scan_projects(search_path, Some(3)) {
            for project in found_projects {
                if project.name == project_name {
                    println!("  🔍 找到项目: {}", project.path.display().to_string().green());
                    meta.projects.insert(project.name, project.path.display().to_string());
                    core.update_meta_config(meta)?;
                    return Ok(());
                }
            }
        }
    }
    Ok(())
}

/// 同步项目元数据（版本、作者信息等）
fn sync_project_metadata(core: &RmmCore, project_path: &Path, fix_version: bool, meta: &mut crate::core::rmm_core::MetaConfig) -> Result<()> {
    println!("  🔄 同步项目元数据...");
    
    // 1. 版本管理
    println!("    📦 检查版本信息...");
    if let Err(e) = sync_version_info(core, project_path, fix_version) {
        println!("    ⚠️  版本同步失败: {}", e.to_string().yellow());
    }
    
    // 2. 作者信息同步
    println!("    👤 检查作者信息...");
    if let Err(e) = sync_author_info(core, project_path, meta) {
        println!("    ⚠️  作者信息同步失败: {}", e.to_string().yellow());
    }
    
    // 3. 更新项目配置显示
    match core.get_project_config(project_path) {
        Ok(project_config) => {
            println!("  📄 项目配置已更新");
            println!("     ID: {}", project_config.project.id.bright_white());
            if !project_config.project.description.is_empty() {
                println!("     描述: {}", project_config.project.description.bright_black());
            }
            
            // 显示作者信息
            if !project_config.authors.is_empty() {
                let author = &project_config.authors[0];
                println!("     作者: {} <{}>", author.name.bright_cyan(), author.email.bright_black());
            }
        }
        Err(e) => {
            println!("  ⚠️  无法读取项目配置: {}", e.to_string().yellow());
        }
    }
    
    Ok(())
}

/// 同步版本信息
fn sync_version_info(core: &RmmCore, project_path: &Path, fix_version: bool) -> Result<()> {
    if let Ok(mut version_info) = VersionInfo::from_module_prop(project_path) {
        println!("    📦 当前版本: {} ({})", version_info.version.bright_green(), version_info.version_code.bright_black());
        
        // 🔥 重要修复：首先检查 update.json 是否与 module.prop 一致
        let update_json_path = project_path.join(".rmmp/dist/update.json");
        let needs_sync = if update_json_path.exists() {
            match fs::read_to_string(&update_json_path) {
                Ok(content) => {
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                        let update_version = json_value.get("version").and_then(|v| v.as_str()).unwrap_or("");
                        let update_version_code = json_value.get("versionCode")
                            .and_then(|v| v.as_i64())
                            .map(|v| v.to_string())
                            .unwrap_or_default();
                        
                        // 检查版本是否不一致
                        if update_version != version_info.version || update_version_code != version_info.version_code {
                            println!("    ⚠️  检测到版本不一致:");
                            println!("       module.prop: {} ({})", version_info.version.bright_cyan(), version_info.version_code.bright_cyan());
                            println!("       update.json: {} ({})", update_version.bright_yellow(), update_version_code.bright_yellow());
                            true
                        } else {
                            false
                        }
                    } else {
                        println!("    ⚠️  update.json 格式错误，需要重新同步");
                        true
                    }
                }
                Err(_) => {
                    println!("    ⚠️  无法读取 update.json，需要重新同步");
                    true
                }
            }
        } else {
            println!("    ⚠️  update.json 不存在，需要创建");
            true
        };        
        if needs_sync {
            println!("    🔄 同步版本信息到 update.json...");
            sync_update_json(project_path, &version_info)?;
            println!("    ✅ 版本信息已同步");
        }
        
        // 🔥 重要修复：默认情况下只同步版本信息，--fix-version 参数控制是否跳过版本升级
        if !fix_version {
            // 执行智能版本升级
            let old_version = version_info.version.clone();
            let old_code = version_info.version_code.clone();
            
            version_info.smart_bump_version(project_path);
            
            // 检查是否有变化
            if version_info.version != old_version || version_info.version_code != old_code {
                version_info.update_module_prop(project_path)?;
                sync_update_json(project_path, &version_info)?;
                println!("    🆙 版本已升级: {} ({}) -> {} ({})", 
                    old_version.bright_black(), old_code.bright_black(),
                    version_info.version.bright_green(), version_info.version_code.bright_green());
                
                // 检查全局版本（但不修改）
                if let Err(e) = check_global_version(core, &version_info.version) {
                    println!("    ⚠️  检查全局版本失败: {}", e.to_string().yellow());
                }
            } else {
                println!("    ℹ️  版本无需升级");
                
                // 即使版本不升级，也显示项目版本信息
                if let Err(e) = check_global_version(core, &version_info.version) {
                    println!("    ⚠️  检查版本信息失败: {}", e.to_string().yellow());
                }
            }
        } else {
            println!("    🔧 --fix-version 模式：仅修复版本不一致，跳过版本升级");
            
            // 显示项目版本信息
            if let Err(e) = check_global_version(core, &version_info.version) {
                println!("    ⚠️  检查版本信息失败: {}", e.to_string().yellow());
            }
        }
    }
    Ok(())
}

/// 同步作者信息
fn sync_author_info(core: &RmmCore, project_path: &Path, meta: &mut crate::core::rmm_core::MetaConfig) -> Result<()> {
    // 获取各来源的作者信息
    let meta_author = AuthorInfo {
        name: meta.username.clone(),
        email: meta.email.clone(),
    };
    
    let project_config = core.get_project_config(project_path)?;
    let project_author = if !project_config.authors.is_empty() {
        let author = &project_config.authors[0];
        AuthorInfo {
            name: author.name.clone(),
            email: author.email.clone(),
        }
    } else {
        AuthorInfo {
            name: "unknown".to_string(),
            email: "unknown@example.com".to_string(),
        }
    };
    
    let git_author = AuthorInfo::from_git(project_path);
    
    // 应用同步逻辑
    apply_author_sync_logic(&meta_author, &project_author, &git_author, core, project_path, meta)?;
    
    Ok(())
}

/// 应用作者信息同步逻辑
fn apply_author_sync_logic(    meta_author: &AuthorInfo,
    project_author: &AuthorInfo, 
    git_author: &Option<AuthorInfo>,
    core: &RmmCore,
    _project_path: &Path,  // 标记为未使用但保留接口兼容性
    meta: &mut crate::core::rmm_core::MetaConfig
) -> Result<()> {
    
    let meta_is_default = meta_author.is_default();
    let project_is_default = project_author.is_default();
    
    match (meta_is_default, project_is_default) {
        (true, true) => {
            // 两者都是默认值
            if let Some(git_info) = git_author {
                println!("    🔄 从 Git 仓库同步作者信息: {} <{}>", 
                    git_info.name.bright_cyan(), git_info.email.bright_black());
                
                // 更新 meta 配置
                meta.username = git_info.name.clone();
                meta.email = git_info.email.clone();
                core.update_meta_config(meta)?;
                
                // 更新项目配置（这里需要实现更新项目配置的逻辑）
                println!("    💡 建议手动更新项目配置以同步作者信息");
            } else {
                println!("    ⚠️  作者信息均为默认值，且未检测到 Git 仓库");
                println!("    💡 建议执行以下操作之一:");
                println!("       • 使用 'git config user.name \"Your Name\"' 和 'git config user.email \"your@email.com\"' 设置 Git 用户信息");
                println!("       • 手动编辑 meta.toml 设置全局作者信息");
                println!("       • 手动编辑 rmmproject.toml 设置项目作者信息");
            }
        },
        (true, false) => {
            // meta 是默认值，项目不是 - 将项目信息同步到 meta
            println!("    � 将项目作者信息同步到全局配置: {} <{}>", 
                project_author.name.bright_cyan(), project_author.email.bright_black());
            
            meta.username = project_author.name.clone();
            meta.email = project_author.email.clone();
            core.update_meta_config(meta)?;
        },
        (false, true) => {
            // meta 不是默认值，项目是 - 将 meta 信息同步到项目
            println!("    📥 将全局配置同步到项目作者信息: {} <{}>", 
                meta_author.name.bright_cyan(), meta_author.email.bright_black());
            
            // 这里需要实现更新项目配置的逻辑
            println!("    💡 建议手动更新项目配置以同步作者信息");
        },
        (false, false) => {
            // 两者都不是默认值
            if *meta_author == *project_author {
                println!("    ✅ 作者信息已同步: {} <{}>", 
                    meta_author.name.bright_cyan(), meta_author.email.bright_black());
            } else {
                println!("    ℹ️  检测到不同的作者信息，可能是他人项目，保持现有配置");
                println!("       全局: {} <{}>", meta_author.name.bright_black(), meta_author.email.bright_black());
                println!("       项目: {} <{}>", project_author.name.bright_black(), project_author.email.bright_black());
            }
        }
    }
    
    Ok(())
}

/// 同步所有项目
fn sync_all_projects(
    core: &RmmCore,
    projects_only: bool,
    fix_version: bool,
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
    
    // 2. 清理重复项目（指向相同路径的不同项目名）
    println!("{} 清理重复项目...", "[🔄]".yellow().bold());
    let duplicate_removed = remove_duplicate_projects(core)?;
    
    if duplicate_removed.is_empty() {
        println!("  ✅ 没有重复项目");
    } else {
        println!("  🗑️  移除了 {} 个重复项目:", duplicate_removed.len());
        for project in &duplicate_removed {
            println!("    - {}", project.yellow());
        }
    }
    
    // 如果只同步项目列表，跳过依赖同步
    if projects_only {
        println!("{} 跳过依赖同步 (projects_only 模式)", "[⏭️]".yellow().bold());
        return Ok(());
    }
    
    // 3. 扫描新项目
    println!("{} 扫描新项目...", "[🔍]".blue().bold());
    let search_paths: Vec<std::path::PathBuf> = if let Some(paths) = search_paths {
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
                      if let Some(existing_path) = meta.projects.get(project_name) {                        // 项目已存在，检查路径是否需要更新
                        let normalized_path = normalize_path(project_path);
                        
                        // 防止空路径
                        let safe_path = if normalized_path.is_empty() {
                            ".".to_string()
                        } else {
                            normalized_path
                        };
                        
                        if existing_path != &safe_path {
                            println!("    🔄 更新项目路径: {}", project_name.yellow());
                            println!("      旧路径: {}", existing_path.bright_black());
                            println!("      新路径: {}", safe_path.green());
                            meta.projects.insert(project_name.clone(), safe_path);
                            path_updates += 1;
                        }
                          // 为现有项目执行元数据同步
                        if !projects_only {
                            println!("    🔄 同步项目 {} 的元数据", project_name.cyan());
                            if let Err(e) = sync_project_metadata(core, project_path, fix_version, &mut meta) {
                                println!("    ⚠️  同步失败: {}", e.to_string().yellow());
                            }
                        }} else {                        // 新项目 - 检查是否与现有项目路径重复
                        let normalized_path = normalize_path(project_path);
                        
                        // 防止空路径
                        let safe_path = if normalized_path.is_empty() {
                            ".".to_string()
                        } else {
                            normalized_path
                        };
                        
                        let is_duplicate_path = meta.projects.values().any(|existing_path| {
                            // 标准化路径比较
                            let existing_canonical = std::path::Path::new(existing_path)
                                .canonicalize()
                                .unwrap_or_else(|_| std::path::PathBuf::from(existing_path));
                            let new_canonical = project_path.canonicalize()
                                .unwrap_or_else(|_| project_path.clone());
                            existing_canonical == new_canonical
                        });
                        
                        if is_duplicate_path {
                            println!("    ⚠️  跳过重复路径的项目: {} -> {}", project_name.yellow(), safe_path.bright_black());
                            continue;
                        }
                        
                        // 真正的新项目
                        println!("    ➕ 发现新项目: {}", project_name.green().bold());
                        println!("      路径: {}", safe_path.bright_black());
                        meta.projects.insert(project_name.clone(), safe_path);
                        new_projects_count += 1;
                          // 为新项目也执行元数据同步
                        if !projects_only {
                            println!("    🔄 同步新项目 {} 的元数据", project_name.cyan());
                            if let Err(e) = sync_project_metadata(core, project_path, fix_version, &mut meta) {
                                println!("    ⚠️  同步失败: {}", e.to_string().yellow());
                            }
                        }
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
    
    // 4. 显示同步结果
    println!("\n{} 同步结果:", "[📊]".blue().bold());
    println!("  🗑️  移除无效项目: {}", removed_projects.len().to_string().red().bold());
    println!("  🔄 移除重复项目: {}", duplicate_removed.len().to_string().yellow().bold());
    println!("  ➕ 发现新项目: {}", new_projects_count.to_string().green().bold());
    println!("  📂 总扫描项目: {}", total_scanned.to_string().cyan());
    
    // 5. 显示当前项目列表
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

/// 移除重复项目（指向相同路径的不同名称）
fn remove_duplicate_projects(core: &RmmCore) -> Result<Vec<String>> {
    let mut meta = core.get_meta_config()?;
    let mut path_to_names: std::collections::HashMap<std::path::PathBuf, Vec<String>> = std::collections::HashMap::new();
    let mut removed_names = Vec::new();
    
    // 收集所有路径和对应的项目名
    for (name, path_str) in &meta.projects {
        let path = std::path::Path::new(path_str);
        let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        
        path_to_names.entry(canonical_path).or_insert_with(Vec::new).push(name.clone());
    }
    
    // 找出重复的路径
    for (path, names) in path_to_names {
        if names.len() > 1 {
            // 对于重复的路径，保留第一个有效的项目名，移除其他的
            println!("  🔍 发现重复路径: {}", path.display().to_string().yellow());
            
            // 按名称排序，优先保留较短的或更规范的名称
            let mut sorted_names = names.clone();
            sorted_names.sort_by(|a, b| {
                // 优先级：非默认名称 > 较短名称 > 字母序
                let a_is_default = a == "unknown" || a == "test";
                let b_is_default = b == "unknown" || b == "test";
                
                match (a_is_default, b_is_default) {
                    (true, false) => std::cmp::Ordering::Greater,   // b 优先
                    (false, true) => std::cmp::Ordering::Less,      // a 优先
                    _ => a.len().cmp(&b.len()).then(a.cmp(b))       // 长度然后字母序
                }
            });
            
            let keep_name = &sorted_names[0];
            println!("    ✅ 保留项目: {}", keep_name.green());
            
            for name in &sorted_names[1..] {
                println!("    🗑️  移除重复项目: {}", name.red());
                meta.projects.remove(name);
                removed_names.push(name.clone());
            }
        }
    }
    
    // 更新配置
    if !removed_names.is_empty() {
        core.update_meta_config(&meta)?;
    }
    
    Ok(removed_names)
}

/// 标准化路径格式，返回绝对路径
fn normalize_path(path: &Path) -> String {
    // 首先尝试 canonicalize 获取绝对路径
    if let Ok(canonical) = path.canonicalize() {
        let path_str = canonical.display().to_string();
        
        // 移除Windows长路径前缀 \\?\
        if path_str.starts_with(r"\\?\") {
            path_str[4..].to_string()
        } else {
            path_str
        }
    } else {
        // 如果 canonicalize 失败，手动构建绝对路径
        let path_str = path.display().to_string();
        
        // 移除Windows长路径前缀 \\?\
        let clean_path = if path_str.starts_with(r"\\?\") {
            &path_str[4..]
        } else {
            &path_str
        };
        
        let clean_path_buf = std::path::PathBuf::from(clean_path);
        
        if clean_path_buf.is_absolute() {
            clean_path.to_string()
        } else {
            // 相对路径转绝对路径
            if let Ok(current_dir) = std::env::current_dir() {
                current_dir.join(&clean_path_buf).display().to_string()
            } else {
                clean_path.to_string()
            }
        }
    }
}

/// 生成基于日期和递增的版本代码
fn generate_version_code(project_path: &Path) -> String {
    // 获取当前日期 YYYYMMDD 格式
    let now = chrono::Local::now();
    let date_str = now.format("%Y%m%d").to_string();
    
    // 尝试从现有版本代码中提取递增数字
    if let Ok(current_version) = VersionInfo::from_module_prop(project_path) {
        let current_code = &current_version.version_code;
        
        // 如果当前版本代码是今天的日期开头，提取并递增后缀
        if current_code.starts_with(&date_str) {
            let suffix = &current_code[date_str.len()..];
            if let Ok(num) = suffix.parse::<u32>() {
                return format!("{}{:02}", date_str, num + 1);
            }
        }
    }
    
    // 默认从01开始
    format!("{}01", date_str)
}

/// 智能版本升级 - 修正版本格式，patch使用Git提交hash
fn smart_version_bump(current_version: &str, project_path: &Path) -> String {
    // 移除可能的 'v' 前缀进行处理
    let version_without_v = current_version.trim_start_matches('v');
    
    // 获取Git提交hash作为patch
    let patch_hash = if let Ok(Some(git_info)) = GitAnalyzer::analyze_git_info(project_path) {
        if let Ok(repo) = git2::Repository::open(&git_info.repo_root) {
            if let Ok(head) = repo.head() {
                if let Some(commit) = head.target() {
                    let commit_str = commit.to_string();
                    if commit_str.len() >= 8 {
                        commit_str[..8].to_string() // 使用8位commit hash
                    } else {
                        "unknown".to_string()
                    }
                } else {
                    "unknown".to_string()
                }
            } else {
                "unknown".to_string()
            }
        } else {
            "unknown".to_string()
        }
    } else {
        "unknown".to_string()
    };
    
    // 检查当前版本是否已经包含patch部分
    if let Some(dash_pos) = version_without_v.find('-') {
        // 已经有patch部分，只升级基础版本号（不重复添加patch）
        let base_version = &version_without_v[..dash_pos];
        let parts: Vec<&str> = base_version.split('.').collect();
        
        if parts.len() >= 3 {
            if let (Ok(major), Ok(minor), Ok(patch)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>()
            ) {
                return format!("v{}.{}.{}-{}", major, minor, patch + 1, patch_hash);
            }
        }
        
        // 如果解析失败，直接替换hash部分
        return format!("v{}-{}", base_version, patch_hash);
    }
    
    // 解析版本号 (major.minor.patch)
    let parts: Vec<&str> = version_without_v.split('.').collect();
    
    if parts.len() >= 3 {
        // 标准的三段版本号
        if let (Ok(major), Ok(minor), Ok(patch)) = (
            parts[0].parse::<u32>(),
            parts[1].parse::<u32>(),
            parts[2].parse::<u32>()
        ) {
            // 升级patch版本，添加commit hash
            return format!("v{}.{}.{}-{}", major, minor, patch + 1, patch_hash);
        }
    } else if parts.len() == 2 {
        // 两段版本号，添加patch
        if let (Ok(major), Ok(minor)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            return format!("v{}.{}.1-{}", major, minor, patch_hash);
        }
    }
      // 如果解析失败，使用默认逻辑
    format!("v{}-{}", version_without_v, patch_hash)
}

/// 同步版本信息到update.json
fn sync_update_json(project_path: &Path, version_info: &VersionInfo) -> Result<()> {
    // 🔥 修复：需要同步所有 update.json 文件
    let update_json_paths = vec![
        project_path.join(".rmmp/dist/update.json"),
        project_path.join(".rmmp/build/update.json"),
        project_path.join(".rmmp/source-build/update.json"),
    ];
    
    let mut updated_count = 0;
    
    for update_json_path in update_json_paths {
        if update_json_path.exists() {
            let content = fs::read_to_string(&update_json_path)?;
            
            // 解析JSON
            if let Ok(mut json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                // 更新版本信息
                if let Some(obj) = json_value.as_object_mut() {
                    obj.insert("version".to_string(), serde_json::Value::String(version_info.version.clone()));
                    
                    // 将版本代码转换为数字
                    if let Ok(version_code_num) = version_info.version_code.parse::<i64>() {
                        obj.insert("versionCode".to_string(), serde_json::Value::Number(serde_json::Number::from(version_code_num)));
                    }
                    
                    // 🔥 新增：同步更新 zipUrl 中的版本信息
                    if let Some(zip_url) = obj.get("zipUrl").and_then(|v| v.as_str()) {
                        // 更新 zipUrl 中的版本标签和版本代码
                        let updated_zip_url = update_zip_url_version(zip_url, &version_info.version, &version_info.version_code);
                        obj.insert("zipUrl".to_string(), serde_json::Value::String(updated_zip_url));
                    }
                    
                    // 写回文件，保持格式美观
                    let formatted_json = serde_json::to_string_pretty(&json_value)?;
                    fs::write(&update_json_path, formatted_json)?;
                    
                    // 获取相对路径用于显示
                    let relative_path = update_json_path.strip_prefix(project_path)
                        .unwrap_or(&update_json_path)
                        .display();
                    println!("    📄 已同步版本信息到 {}", relative_path);
                    updated_count += 1;
                }
            }
        }
    }
      if updated_count == 0 {
        println!("    ⚠️  未找到任何 update.json 文件");
    } else {
        println!("    ✅ 共同步了 {} 个 update.json 文件", updated_count);
    }
    
    Ok(())
}

/// 更新 zipUrl 中的版本信息
fn update_zip_url_version(zip_url: &str, new_version: &str, new_version_code: &str) -> String {
    use regex::Regex;
    
    // 1. 更新版本标签 (如 v0.1.8-357fe85b -> v0.1.10-357fe85b)
    let version_regex = Regex::new(r"/releases/download/v[^/]+/").unwrap();
    let version_tag = if new_version.starts_with('v') {
        new_version.to_string()
    } else {
        format!("v{}", new_version)
    };
    
    let mut updated_url = version_regex.replace(zip_url, &format!("/releases/download/{}/", version_tag)).to_string();
    
    // 2. 更新文件名中的版本代码 (如 rmmp-2025061507-arm64.zip -> rmmp-2025061510-arm64.zip)
    let version_code_regex = Regex::new(r"([a-zA-Z\-]+)-(\d{10})([-a-zA-Z0-9]*\.zip)").unwrap();
    if let Some(caps) = version_code_regex.captures(&updated_url) {
        let prefix = caps.get(1).map_or("", |m| m.as_str());
        let suffix = caps.get(3).map_or("", |m| m.as_str());
        let new_filename = format!("{}-{}{}", prefix, new_version_code, suffix);
        
        // 替换文件名部分
        let filename_regex = Regex::new(r"/([^/]+\.zip)$").unwrap();
        updated_url = filename_regex.replace(&updated_url, &format!("/{}", new_filename)).to_string();
    }
    
    updated_url
}

/// 检查全局版本但不自动更新 - 全局版本应该由CLI写死，不应该被sync命令修改
fn check_global_version(_core: &RmmCore, project_version: &str) -> Result<()> {
    // 🔥 重要修复：sync命令不应该修改全局版本！
    // 全局版本是CLI工具本身的版本，应该写死在代码中，不应该被项目版本影响
    
    // 仅显示信息，不做任何修改
    println!("    ℹ️  项目版本: {} (全局版本由CLI工具管理，不自动同步)", 
             project_version.bright_green());
    
    Ok(())
}

