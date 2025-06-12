use anyhow::Result;
use std::path::{Path, PathBuf};
use crate::commands::utils::core::project::ProjectConfig;
use crate::commands::utils::core::common::FileSystemManager;
use crate::commands::utils::core::RmakeConfig; // Added RmakeConfig
use crate::commands::utils::init_utils::generate_module_prop; // Added generate_module_prop
use crate::commands::utils::shellcheck; // Added shellcheck
use glob::Pattern; // Added for glob pattern matching
use std::collections::HashSet; // Added for file collection

/// 构建上下文结构
#[derive(Debug, Clone)]
pub struct BuildContext {
    pub project_root: PathBuf,
    pub rmmp_dir: PathBuf,
    pub build_dir: PathBuf,
    pub dist_dir: PathBuf,
}

/// 包信息结构
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub zip_path: PathBuf,
    pub zip_filename: String,
    pub source_path: PathBuf,
    pub source_filename: String,
}

/// 检查路径是否应该被排除
pub fn should_exclude_path(path: &Path, exclude_items: &[&str]) -> bool {
    let path_str = path.to_string_lossy();
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
    
    for exclude in exclude_items {
        if exclude.contains('*') {
            // 简单的通配符匹配
            if exclude.starts_with("*.") && file_name.ends_with(&exclude[1..]) {
                return true;
            }
        } else if path_str.contains(exclude) || file_name == *exclude {
            return true;
        }
    }
    
    false
}

/// 替换模板中的变量
pub fn replace_template_variables(template: &str, config: &ProjectConfig) -> Result<String> {
    let mut result = template.to_string();
    
    // 获取 Git 提交 hash
    let git_hash = "unknown".to_string(); // 简化版本，实际应该获取 git commit hash
    let short_hash = if git_hash.len() >= 8 { &git_hash[..8] } else { &git_hash };
    
    // 获取当前时间
    let now = chrono::Utc::now();
    let date = now.format("%Y%m%d").to_string();
    let datetime = now.format("%Y%m%d_%H%M%S").to_string();
    let timestamp = now.timestamp().to_string();
    
    // 获取作者信息
    let author_name = config.authors.first()
        .map(|a| a.name.as_str())
        .unwrap_or("unknown");
    let author_email = config.authors.first()
        .map(|a| a.email.as_str())
        .unwrap_or("unknown");
    
    // 定义变量映射
    let variables = [        
        ("{id}", config.id.as_str()),
        ("{name}", config.name.as_str()),
        ("{version}", config.version.as_deref().unwrap_or("unknown")),
        ("{version_code}", config.version_code.as_str()),
        ("{author}", author_name),
        ("{email}", author_email),
        ("{hash}", &git_hash),
        ("{short_hash}", short_hash),
        ("{date}", &date),
        ("{datetime}", &datetime),
        ("{timestamp}", &timestamp),
    ];
    
    // 执行替换
    for (var, value) in &variables {
        result = result.replace(var, value);
    }
    
    println!("📝 ZIP 文件名模板: '{}' -> '{}'", template, result);
    
    Ok(result)
}

/// 生成 ZIP 文件名，支持变量替换
pub fn generate_zip_filename(config: &ProjectConfig, rmake_config: Option<&RmakeConfig>) -> Result<String> {
    let template = if let Some(rmake) = rmake_config {
        if let Some(ref package) = rmake.package {
            if let Some(ref zip_name) = package.zip_name {
                if zip_name == "default" {
                    // 使用默认模板
                    "{id}-{version_code}.zip".to_string()
                } else {
                    // 使用自定义模板
                    zip_name.clone()
                }
            } else {
                // 没有指定 zip_name，使用默认模板
                "{id}-{version_code}.zip".to_string()
            }
        } else {
            // 没有 package 配置，使用默认模板
            "{id}-{version_code}.zip".to_string()
        }
    } else {
        // 没有 rmake 配置，使用默认模板
        "{id}-{version_code}.zip".to_string()
    };
    
    // 执行变量替换
    let result = replace_template_variables(&template, config)?;
    
    // 确保文件名以 .zip 结尾
    if result.ends_with(".zip") {
        Ok(result)
    } else {
        Ok(format!("{}.zip", result))
    }
}


/// 构建排除列表
pub fn build_exclude_list(rmake_config: Option<&RmakeConfig>) -> Vec<String> {
    let mut exclude_items = vec![
        ".rmmp".to_string(),           // RMM 元数据目录
        "dist".to_string(), 
        "build".to_string(),
        "target".to_string(),          // Rust 构建产物
        "__pycache__".to_string(),     // Python 缓存
        ".git".to_string(),            // Git 仓库
        "node_modules".to_string(),    // Node.js 依赖
        ".vscode".to_string(),         // VS Code 配置
        ".idea".to_string(),           // IntelliJ IDEA 配置
        "*.zip".to_string(),           // 已构建的模块包
        "*.tar.gz".to_string(),        // 归档文件
        "*.log".to_string(),           // 日志文件
        "Cargo.lock".to_string(),      // Rust 锁定文件
        "Cargo.toml".to_string(),      // Rust 项目文件
        "pyproject.toml".to_string(),  // Python 项目文件
        "uv.lock".to_string(),         // UV 锁定文件
        ".gitignore".to_string(),      // Git 忽略文件
        "rmmproject.toml".to_string()  // RMM 项目配置文件
    ];
    
    // 如果有 Rmake 配置，添加额外的排除项（仅用于模块ZIP打包）
    if let Some(rmake) = rmake_config {
        if let Some(ref excludes) = rmake.build.exclude {
            for exclude in excludes {
                exclude_items.push(exclude.clone());
            }
            println!("📋 使用 Rmake 排除规则（模块打包）: {:?}", excludes);
        }
    }
    
    exclude_items
}
/// 构建项目主流程
pub async fn build_project(config: &ProjectConfig, _output_dir: &Path, user_output_dir: Option<&String>, _debug: bool, skip_shellcheck: bool) -> Result<()> {
    println!("📦 构建模块: {}", config.name);
    
    let build_context = setup_build_environment()?;
    let rmake_config = load_or_create_rmake_config(&build_context.project_root)?;
    
    // 预构建阶段
    if !skip_shellcheck {
        run_shellcheck_validation(&build_context.project_root)?;
    } else {
        println!("⚠️  已跳过 shellcheck 语法检查");
    }
    
    prepare_build_directories(&build_context)?;
    
    // 构建阶段
    execute_build_phase(config, &build_context, rmake_config.as_ref()).await?;
    
    // 打包阶段
    let package_info = create_packages(config, &build_context, rmake_config.as_ref()).await?;
    
    // 后处理阶段
    finalize_build(&package_info, user_output_dir, &build_context.dist_dir)?;
      Ok(())
}

/// 设置构建环境
fn setup_build_environment() -> Result<BuildContext> {
    let project_root = std::env::current_dir()?;
    let rmmp_dir = project_root.join(".rmmp");
    let build_dir = rmmp_dir.join("build");
    let dist_dir = rmmp_dir.join("dist");
    
    // 确保目录存在
    std::fs::create_dir_all(&build_dir)?;
    std::fs::create_dir_all(&dist_dir)?;
    
    Ok(BuildContext {
        project_root,
        rmmp_dir,
        build_dir,
        dist_dir,
    })
}

/// 加载或创建 Rmake 配置
fn load_or_create_rmake_config(project_root: &Path) -> Result<Option<RmakeConfig>> {
    match RmakeConfig::load_from_dir(project_root)? {
        Some(config) => Ok(Some(config)),
        None => {
            println!("📝 未找到 Rmake.toml，创建默认配置...");
            let default_config = RmakeConfig::default();
            default_config.save_to_dir(project_root)?;
            let rmake_path = project_root.join(".rmmp").join("Rmake.toml");
            println!("✅ 已创建默认 Rmake.toml: {}", rmake_path.display());
            Ok(Some(default_config))
        }
    }
}

/// 准备构建目录
fn prepare_build_directories(context: &BuildContext) -> Result<()> {
    // 清理构建目录
    if context.build_dir.exists() {
        std::fs::remove_dir_all(&context.build_dir)?;
        std::fs::create_dir_all(&context.build_dir)?;
    }
    
    Ok(())
}

/// 执行构建阶段
async fn execute_build_phase(
    config: &ProjectConfig, 
    context: &BuildContext, 
    rmake_config: Option<&RmakeConfig>
) -> Result<()> {    // 执行预构建步骤
    if let Some(rmake) = rmake_config {
        execute_build_steps("prebuild", &rmake.build.pre_build, &context.project_root)?;
    }
    
    // 复制模块文件到构建目录
    copy_module_files_to_build(config, &context.project_root, &context.build_dir, rmake_config)?;
      // 执行构建步骤
    if let Some(rmake) = rmake_config {
        // 将单个目标转换为向量格式
        if let Some(ref target) = rmake.build.target {
            let target_vec = vec![target.clone()];
            execute_build_steps("build", &Some(target_vec), &context.project_root)?;
        }
    }
    
    // 生成 module.prop
    generate_module_prop(config, &context.build_dir)?;
    
    // 执行后构建步骤
    if let Some(rmake) = rmake_config {
        execute_build_steps("postbuild", &rmake.build.post_build, &context.project_root)?;
    }
    
    Ok(())
}

/// 创建包文件
async fn create_packages(
    config: &ProjectConfig, 
    context: &BuildContext, 
    rmake_config: Option<&RmakeConfig>
) -> Result<PackageInfo> {
    // 创建模块 ZIP 包
    let zip_filename = generate_zip_filename(config, rmake_config)?;
    let zip_path = context.dist_dir.join(&zip_filename);
    create_module_zip(&context.build_dir, &zip_path, rmake_config)?;
    
    // 创建源代码 tar.gz 包（使用新的文件名生成逻辑）
    let source_filename = generate_source_filename(config, rmake_config)?;
    let source_path = context.dist_dir.join(&source_filename);
    create_source_archive(&context.project_root, &source_path, rmake_config)?;
    
    println!("📦 模块包: {}", zip_path.display());
    println!("📦 源码包: {}", source_path.display());    
    // 生成 update.json 文件
    println!("📄 生成 update.json...");
    if let Err(e) = crate::commands::utils::utils::generate_update_json(config, &context.project_root, rmake_config).await {
        println!("⚠️  生成 update.json 失败: {}", e);
    }
    
    Ok(PackageInfo {
        zip_path,
        zip_filename,
        source_path,
        source_filename,
    })
}

/// 完成构建
fn finalize_build(
    package_info: &PackageInfo, 
    user_output_dir: Option<&String>, 
    dist_dir: &Path
) -> Result<()> {
    // 只有在用户明确指定输出目录时才复制文件
    if let Some(user_output) = user_output_dir {
        let user_path = Path::new(user_output);
        if user_path != dist_dir {
            std::fs::create_dir_all(user_path)?;
            let output_zip = user_path.join(&package_info.zip_filename);
            let output_source = user_path.join(&package_info.source_filename);
            std::fs::copy(&package_info.zip_path, output_zip)?;
            std::fs::copy(&package_info.source_path, output_source)?;
            println!("📁 已复制到输出目录: {}", user_path.display());
        }
    }
    
    Ok(())
}

/// 复制模块文件到构建目录
fn copy_module_files_to_build(
    _config: &ProjectConfig, 
    project_root: &Path, 
    build_dir: &Path, 
    rmake_config: Option<&RmakeConfig>
) -> Result<()> {
    // 使用新的 include/exclude 逻辑
    if let Some(rmake) = rmake_config {
        let include_patterns = &rmake.build.include;
        let exclude_patterns = &rmake.build.exclude;
        
        // 如果有明确的 include 模式且不为空，使用新的收集逻辑
        if let Some(includes) = include_patterns {
            if !includes.is_empty() {
                println!("📋 使用自定义 include/exclude 规则复制模块文件");
                let collected_files = collect_files_with_rules(project_root, include_patterns, exclude_patterns)?;
                
                for file_path in collected_files {
                    let relative_path = file_path.strip_prefix(project_root)?;
                    let dest_path = build_dir.join(relative_path);
                    
                    // 确保目标目录存在
                    if let Some(dest_parent) = dest_path.parent() {
                        std::fs::create_dir_all(dest_parent)?;
                    }
                    
                    std::fs::copy(&file_path, &dest_path)?;
                    println!("📄 复制 {}", relative_path.display());
                }
                
                return Ok(());
            }
        }
    }
    
    // 回退到原有逻辑（基于 exclude 的方式）
    println!("📋 使用传统 exclude 规则复制模块文件");
    let exclude_items = build_exclude_list(rmake_config);
    
    copy_root_files(project_root, build_dir, &exclude_items)?;
    copy_system_directory(project_root, build_dir, &exclude_items)?;
    copy_module_directories(project_root, build_dir, &exclude_items)?;
    
    Ok(())
}

/// 复制项目根目录中的文件（应用排除规则）
fn copy_root_files(project_root: &Path, build_dir: &Path, exclude_items: &[String]) -> Result<()> {
    let exclude_refs: Vec<&str> = exclude_items.iter().map(|s| s.as_str()).collect();
    
    // 遍历项目根目录中的所有条目
    for entry in std::fs::read_dir(project_root)? {
        let entry = entry?;
        let path = entry.path();
        
        // 只处理文件，不处理目录
        if path.is_file() {
            // 检查是否应该排除此文件
            if !should_exclude_path(&path, &exclude_refs) {
                let file_name = path.file_name().unwrap();
                let dest = build_dir.join(file_name);
                std::fs::copy(&path, &dest)?;
                println!("📄 复制 {}", file_name.to_string_lossy());
            } else {
                println!("🚫 跳过 {} (被排除)", path.file_name().unwrap().to_string_lossy());
            }
        }
    }
    
    Ok(())
}

/// 复制 system 目录
fn copy_system_directory(project_root: &Path, build_dir: &Path, exclude_items: &[String]) -> Result<()> {
    let system_dir = project_root.join("system");
    if system_dir.exists() {
        let build_system_dir = build_dir.join("system");
        std::fs::create_dir_all(&build_system_dir)?;
        
        if system_dir.read_dir()?.next().is_some() {
            // 目录不为空，复制内容
            let exclude_refs: Vec<&str> = exclude_items.iter().map(|s| s.as_str()).collect();
            copy_dir_recursive_with_exclusions(&system_dir, &build_system_dir, &exclude_refs)?;
            println!("📁 复制 system 目录（含文件）");
        } else {
            // 目录为空，只创建目录结构
            println!("📁 创建空 system 目录");
        }
    }
    
    Ok(())
}

/// 复制其他模块相关目录
fn copy_module_directories(project_root: &Path, build_dir: &Path, exclude_items: &[String]) -> Result<()> {
    let module_dirs = ["META-INF", "system_ext", "vendor", "product", "apex", "data"];
    let exclude_refs: Vec<&str> = exclude_items.iter().map(|s| s.as_str()).collect();
    
    for dir in &module_dirs {
    let src_dir = project_root.join(dir);
        if src_dir.exists() && !should_exclude_path(&src_dir, &exclude_refs) {
            copy_dir_recursive_with_exclusions(&src_dir, &build_dir.join(dir), &exclude_refs)?;
            println!("📁 复制 {} 目录", dir);
        }
    }
    
    Ok(())
}

/// 创建模块 ZIP 包（使用 Rust 原生库）
fn create_module_zip(build_dir: &Path, zip_path: &Path, rmake_config: Option<&RmakeConfig>) -> Result<()> {
    use std::fs::File;
    use zip::{ZipWriter, write::FileOptions, CompressionMethod};
    
    // 获取压缩级别
    let (compression_method, compression_level) = if let Some(rmake) = rmake_config {
        if let Some(ref package_config) = rmake.package {
            match package_config.compression.as_deref().unwrap_or("default") {
                "none" => (CompressionMethod::Stored, None),
                "fast" => (CompressionMethod::Deflated, Some(1)),
                "default" => (CompressionMethod::Deflated, Some(6)),
                "best" => (CompressionMethod::Deflated, Some(9)),
                _ => (CompressionMethod::Deflated, Some(6)),
            }
        } else {
            (CompressionMethod::Deflated, Some(6))
        }
    } else {
        (CompressionMethod::Deflated, Some(6))
    };
    
    println!("📦 创建 ZIP 包: {}", zip_path.display());
    println!("🗜️  压缩方法: {:?}, 级别: {:?}", compression_method, compression_level);
    
    let file = File::create(zip_path)?;
    let mut zip = ZipWriter::new(file);
    
    // 遍历构建目录中的所有文件
    for entry in walkdir::WalkDir::new(build_dir) {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            // 计算相对路径
            let relative_path = path.strip_prefix(build_dir)?;
            let relative_path_str = relative_path.to_string_lossy().replace('\\', "/");
              // 设置文件选项
            let mut options = FileOptions::<()>::default()
                .compression_method(compression_method);
            
            if let Some(level) = compression_level {
                options = options.compression_level(Some(level));
            }
              // 添加文件到 ZIP
            zip.start_file(&relative_path_str, options)?;
            let file_content = std::fs::read(path)?;
            std::io::Write::write_all(&mut zip, &file_content)?;
            
            println!("  ✓ {}", relative_path_str);
        }
    }
    
    zip.finish()?;    // 显示文件大小
    let metadata = std::fs::metadata(zip_path)?;
    let size_str = FileSystemManager::format_file_size(metadata.len());
    println!("✅ ZIP 包创建完成: {}", size_str);
    
    Ok(())
}

/// 创建源代码归档（使用 Rust 原生库）
fn create_source_archive(project_root: &Path, archive_path: &Path, rmake_config: Option<&RmakeConfig>) -> Result<()> {
    use std::fs::File;
    use flate2::{write::GzEncoder, Compression};
    use tar::Builder;
    
    println!("📦 创建源码归档: {}", archive_path.display());
    
    // 创建 gzip 压缩文件
    let tar_gz = File::create(archive_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);
    
    // 使用新的 include/exclude 逻辑
    if let Some(rmake) = rmake_config {
        if let Some(ref source_package) = rmake.source_package {
            let include_patterns = &source_package.include;
            let exclude_patterns = &source_package.exclude;
            
            // 如果有明确的 include 模式且不为空，使用新的收集逻辑
            if let Some(includes) = include_patterns {
                if !includes.is_empty() {
                    println!("📋 使用自定义 include/exclude 规则创建源码归档");
                    let collected_files = collect_files_with_rules(project_root, include_patterns, exclude_patterns)?;                    for file_path in collected_files {
                        let relative_path = file_path.strip_prefix(project_root)?;
                        tar.append_path_with_name(&file_path, relative_path)?;
                        println!("  ✓ {}", relative_path.display());
                    }
                      // 完成归档并确保数据被写入
                    let inner = tar.into_inner()?;
                    inner.finish()?;
                    
                    // 显示文件大小
                    let metadata = std::fs::metadata(archive_path)?;
                    let size_str = FileSystemManager::format_file_size(metadata.len());
                    println!("✅ 源码归档创建完成: {}", size_str);
                    
                    return Ok(());
                }
            }
            
            // 使用 exclude 逻辑
            if let Some(excludes) = exclude_patterns {
                if !excludes.is_empty() {
                    println!("📋 使用自定义 exclude 规则创建源码归档");
                    
                    for entry in walkdir::WalkDir::new(project_root) {
                        let entry = entry?;
                        let path = entry.path();
                        
                        if path.is_file() {
                            let relative_path = path.strip_prefix(project_root)?;
                            let relative_path_str = relative_path.to_string_lossy().replace('\\', "/");
                            
                            // 检查是否应该排除
                            let should_exclude = excludes.iter().any(|pattern| {
                                matches_pattern(&relative_path_str, pattern)
                            });                            if !should_exclude {
                                tar.append_path_with_name(path, relative_path)?;
                                println!("  ✓ {}", relative_path.display());
                            }
                        }
                    }
                      // 完成归档并确保数据被写入
                    let inner = tar.into_inner()?;
                    inner.finish()?;
                    
                    // 显示文件大小
                    let metadata = std::fs::metadata(archive_path)?;
                    let size_str = FileSystemManager::format_file_size(metadata.len());
                    println!("✅ 源码归档创建完成: {}", size_str);
                    
                    return Ok(());
                }
            }
        }
    }
      // 回退到原有的简化排除逻辑
    println!("📋 使用默认排除规则创建源码归档");
    let should_exclude = |relative_path: &str| -> bool {
        // 排除 .rmmp/dist 和 .rmmp/build 目录，但保留 Rmake.toml
        if relative_path.starts_with(".rmmp/dist") || relative_path.starts_with(".rmmp\\dist") ||
           relative_path.starts_with(".rmmp/build") || relative_path.starts_with(".rmmp\\build") {
            return true;
        }
        
        // 保留 .rmmp/Rmake.toml 文件
        if relative_path == ".rmmp/Rmake.toml" || relative_path == ".rmmp\\Rmake.toml" {
            return false;
        }
        
        // 排除其他 .rmmp 目录内容（除了 Rmake.toml）
        if relative_path.starts_with(".rmmp/") || relative_path.starts_with(".rmmp\\") {
            return true;
        }
        
        // 排除特定目录
        let path_components: Vec<&str> = relative_path.split(['/', '\\']).collect();
        path_components.iter().any(|&component| {
            matches!(component, 
                "target" | "__pycache__" | ".git" | "node_modules" | 
                ".vscode" | ".idea" | ".github"
            )
        })
    };
    
    // 遍历项目根目录
    for entry in walkdir::WalkDir::new(project_root) {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            // 计算相对于项目根目录的路径
            let relative_path = match path.strip_prefix(project_root) {
                Ok(rel) => rel,
                Err(_) => continue,
            };
            let relative_path_str = relative_path.to_string_lossy();
            
            // 检查是否应该排除
            if should_exclude(&relative_path_str) {
                continue;
            }
            
            // 添加文件到 tar 归档
            tar.append_path_with_name(path, relative_path)?;
            
            println!("  ✓ {}", relative_path.display());
        }
    }    // 完成归档并刷新编码器
    tar.finish()?;
    
    // 显示文件大小
    let metadata = std::fs::metadata(archive_path)?;
    let size_str = FileSystemManager::format_file_size(metadata.len());
    println!("✅ 源码归档创建完成: {}", size_str);
    
    Ok(())
}

/// 检查路径是否匹配 glob 模式
fn matches_pattern(path: &str, pattern: &str) -> bool {
    if let Ok(glob_pattern) = Pattern::new(pattern) {
        glob_pattern.matches(path)
    } else {
        // 如果不是有效的 glob 模式，则执行简单的字符串匹配
        path.contains(pattern) || path.ends_with(pattern)
    }
}

/// 根据 include/exclude 规则收集文件列表
fn collect_files_with_rules(
    project_root: &Path,
    include_patterns: &Option<Vec<String>>,
    exclude_patterns: &Option<Vec<String>>,
) -> Result<HashSet<PathBuf>> {
    let mut collected_files = HashSet::new();
    
    // 如果有 include 模式，则只包含匹配的文件
    if let Some(includes) = include_patterns {
        if !includes.is_empty() {
            println!("📋 使用 include 模式: {:?}", includes);
            
            for entry in walkdir::WalkDir::new(project_root) {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_file() {
                    let relative_path = path.strip_prefix(project_root)?;
                    let relative_path_str = relative_path.to_string_lossy().replace('\\', "/");
                    
                    // 检查是否匹配任何 include 模式
                    for pattern in includes {
                        if matches_pattern(&relative_path_str, pattern) {
                            collected_files.insert(path.to_path_buf());
                            break;
                        }
                    }
                }
            }
        } else {
            // include 列表为空，包含所有文件
            for entry in walkdir::WalkDir::new(project_root) {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    collected_files.insert(path.to_path_buf());
                }
            }
        }
    } else {
        // 没有 include 规则，包含所有文件
        for entry in walkdir::WalkDir::new(project_root) {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                collected_files.insert(path.to_path_buf());
            }
        }
    }
    
    // 应用 exclude 规则
    if let Some(excludes) = exclude_patterns {
        if !excludes.is_empty() {
            println!("📋 应用 exclude 模式: {:?}", excludes);
            
            collected_files.retain(|path| {
                let relative_path = path.strip_prefix(project_root).unwrap_or(path);
                let relative_path_str = relative_path.to_string_lossy().replace('\\', "/");
                
                // 如果匹配任何 exclude 模式，则移除此文件
                !excludes.iter().any(|pattern| matches_pattern(&relative_path_str, pattern))
            });
        }
    }
    
    Ok(collected_files)
}

/// 生成源代码包文件名，支持变量替换
fn generate_source_filename(config: &ProjectConfig, rmake_config: Option<&RmakeConfig>) -> Result<String> {
    let template = if let Some(rmake) = rmake_config {
        if let Some(ref source_package) = rmake.source_package {
            if let Some(ref name_template) = source_package.name_template {
                name_template.clone()
            } else {
                "{id}-{version_code}-source.tar.gz".to_string()
            }
        } else {
            "{id}-{version_code}-source.tar.gz".to_string()
        }
    } else {
        "{id}-{version_code}-source.tar.gz".to_string()
    };
    
    // 打印源码归档文件名模板信息
    let result = replace_template_variables(&template, config)?;
    println!("📝 源码归档文件名模板: '{}' -> '{}'", template, result);
      // 确保文件名以 .tar.gz 结尾
    if result.ends_with(".tar.gz") {
        Ok(result)
    } else {
        Ok(format!("{}.tar.gz", result))
    }
}

/// 递归复制目录（带排除规则）
fn copy_dir_recursive_with_exclusions(src: &Path, dest: &Path, exclude_items: &[&str]) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        
        // 检查是否应该排除此文件/目录
        if should_exclude_path(&src_path, exclude_items) {
            continue;
        }
        
        let dest_path = dest.join(entry.file_name());
        
        if src_path.is_dir() {
            copy_dir_recursive_with_exclusions(&src_path, &dest_path, exclude_items)?;
        } else {
            std::fs::copy(&src_path, &dest_path)?;
        }
    }
    
    Ok(())
}

/// 执行构建步骤
fn execute_build_steps(
    step_name: &str, 
    commands: &Option<Vec<String>>, 
    working_dir: &Path
) -> Result<()> {
    if let Some(cmds) = commands {
        if !cmds.is_empty() {
            println!("🔧 执行 {} 步骤...", step_name);
            for cmd in cmds {
                println!("  > {}", cmd);
                
                // 在 Windows 上使用 PowerShell 执行命令
                let output = std::process::Command::new("powershell")
                    .args(&["-Command", cmd])
                    .current_dir(working_dir)
                    .output()?;
                
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    anyhow::bail!("{} 步骤失败: {}", step_name, stderr);
                }
                
                // 输出命令结果
                let stdout = String::from_utf8_lossy(&output.stdout);
                if !stdout.trim().is_empty() {
                    println!("    {}", stdout.trim());
                }
            }
            println!("✅ {} 步骤完成", step_name);
        }
    }
    Ok(())
}

/// 运行 Rmake.toml 中定义的脚本
pub fn run_script(project_root: &Path, script_name: &str) -> Result<String> {
    println!("🔧 运行脚本: {}", script_name);
    
    // 加载 Rmake 配置
    let rmake_config_path = project_root.join(".rmmp").join("Rmake.toml");    if !rmake_config_path.exists() {
        anyhow::bail!("❌ 未找到 Rmake.toml 配置文件");
    }
    
    let rmake_config = RmakeConfig::load_from_dir(project_root)?
        .ok_or_else(|| anyhow::anyhow!("无法加载 Rmake 配置"))?;
    
    // 查找脚本
    let scripts = rmake_config.scripts
        .ok_or_else(|| anyhow::anyhow!("❌ Rmake.toml 中未定义 [scripts] 部分"))?;
    
    let script_command = scripts.get(script_name)
        .ok_or_else(|| anyhow::anyhow!("❌ 未找到脚本 '{}'", script_name))?;
    
    println!("📋 执行命令: {}", script_command);
    
    // 执行脚本命令
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("powershell")
            .args(&["-Command", script_command])
            .current_dir(project_root)
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("脚本执行失败: {}", stderr);
        }
        
        // 输出命令结果
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            println!("{}", stdout.trim());
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let output = std::process::Command::new("sh")
            .args(&["-c", script_command])
            .current_dir(project_root)
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("脚本执行失败: {}", stderr);
        }
        
        // 输出命令结果
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            println!("{}", stdout.trim());
        }
    }
    println!("✅ 脚本 '{}' 执行完成", script_name);    Ok(format!("脚本 '{}' 执行成功", script_name))
}

/// 运行 shellcheck 验证
fn run_shellcheck_validation(project_root: &Path) -> Result<()> {
    println!("🔍 运行 Shellcheck 验证...");
      // 检查 shellcheck 是否可用
    if !crate::commands::utils::shellcheck::is_shellcheck_available() {
        println!("⚠️  Shellcheck 未安装或不可用");
        println!("   建议安装 shellcheck 以进行 shell 脚本语法检查");
        println!("   安装方法:");
        if cfg!(target_os = "windows") {
            println!("     - Windows: 使用 scoop install shellcheck 或从 GitHub 下载");
        } else if cfg!(target_os = "macos") {
            println!("     - macOS: brew install shellcheck");
        } else {
            println!("     - Linux: 使用包管理器安装 (apt install shellcheck / yum install shellcheck)");
        }
        println!("   跳过 shellcheck 检查继续构建...");
        return Ok(());
    }
      // 显示 shellcheck 版本
    match shellcheck::get_shellcheck_version() {
        Ok(version) => println!("📋 Shellcheck 版本: {}", version),
        Err(_) => println!("📋 Shellcheck 版本: 未知"),
    }
      // 执行检查
    match shellcheck::check_project(project_root, false) {
        Ok((results, all_passed)) => {
            if results.is_empty() {
                println!("📋 项目中未发现 shell 脚本文件");
                return Ok(());
            }
            
            if all_passed {
                println!("✅ Shellcheck 验证通过");
            } else {
                println!("❌ Shellcheck 验证失败！");
                println!("   发现 shell 脚本语法错误，构建中止");
                println!("   请修复错误后重新构建，或使用 'rmm test --shellcheck' 查看详细信息");
                return Err(anyhow::anyhow!("Shell 脚本语法检查失败"));
            }
            
            Ok(())
        }
        Err(e) => {
            println!("❌ Shellcheck 检查失败: {}", e);
            Err(anyhow::anyhow!("Shellcheck 执行失败: {}", e))
        }
    }
}
