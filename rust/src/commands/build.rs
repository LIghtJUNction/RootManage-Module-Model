use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::Path;
use crate::config::{RmmConfig, ProjectConfig, RmakeConfig};
use crate::utils::find_or_create_project_config;

/// 构建 build 命令
pub fn build_command() -> Command {
    Command::new("build")
        .about("构建 RMM 项目")
        .long_about("构建当前 RMM 项目，生成可安装的模块包")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("PATH")
                .help("输出目录路径")
        )
        .arg(
            Arg::new("clean")
                .short('c')
                .long("clean")
                .action(ArgAction::SetTrue)
                .help("构建前清理输出目录")
        )
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .action(ArgAction::SetTrue)
                .help("启用调试模式构建")
        )
        .arg(
            Arg::new("skip-shellcheck")
                .long("skip-shellcheck")
                .action(ArgAction::SetTrue)
                .help("跳过 shellcheck 语法检查")
        )
        .arg(
            Arg::new("script")
                .help("要运行的脚本名称（定义在 Rmake.toml 的 [scripts] 中）")
                .value_name("SCRIPT_NAME")
        )
}

/// 处理 build 命令
pub fn handle_build(_config: &RmmConfig, matches: &ArgMatches) -> Result<()> {
    // 查找项目配置文件
    let current_dir = std::env::current_dir()?;
    let project_config_path = find_or_create_project_config(&current_dir)?;
    let project_root = project_config_path.parent().unwrap();
    
    // 检查是否要运行脚本
    if let Some(script_name) = matches.get_one::<String>("script") {
        return run_script(&project_root, script_name);
    }
    
    println!("🔨 开始构建 RMM 项目...");
    println!("📁 项目配置: {}", project_config_path.display());
      // 加载项目配置
    let mut project_config = ProjectConfig::load_from_file(&project_config_path)?;
    
    // 更新版本信息
    crate::utils::update_project_version(&mut project_config)?;
    
    // 保存更新后的配置
    project_config.save_to_dir(&project_config_path.parent().unwrap())?;
      // 获取选项
    let output_dir = matches.get_one::<String>("output");
    let clean = matches.get_flag("clean");
    let debug = matches.get_flag("debug");
    let skip_shellcheck = matches.get_flag("skip-shellcheck");
    
    if debug {
        println!("🐛 调试模式已启用");
    }
    
    if skip_shellcheck {
        println!("⚠️  已跳过 shellcheck 检查");
    }// 确定输出目录 - 默认使用 .rmmp/dist，不复制到用户目录
    let build_output = if let Some(output) = output_dir {
        Path::new(output).to_path_buf()
    } else {
        current_dir.join(".rmmp").join("dist")
    };
    
    if clean && build_output.exists() {
        println!("🧹 清理输出目录: {}", build_output.display());
        std::fs::remove_dir_all(&build_output)?;
    }
    
    // 创建输出目录
    std::fs::create_dir_all(&build_output)?;    // 构建项目
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        build_project(&project_config, &build_output, output_dir, debug, skip_shellcheck).await
    })?;
    
    println!("✅ 构建完成！输出目录: {}", build_output.display());
    
    Ok(())
}

/// 构建项目
async fn build_project(config: &ProjectConfig, _output_dir: &Path, user_output_dir: Option<&String>, _debug: bool, skip_shellcheck: bool) -> Result<()> {
    println!("📦 构建模块: {}", config.name);
    
    let project_root = std::env::current_dir()?;
    let rmmp_dir = project_root.join(".rmmp");
    let build_dir = rmmp_dir.join("build");
    let dist_dir = rmmp_dir.join("dist");
    
    // 加载 Rmake 配置
    let rmake_config = crate::config::RmakeConfig::load_from_dir(&project_root)?;    // 确保目录存在
    std::fs::create_dir_all(&build_dir)?;
    std::fs::create_dir_all(&dist_dir)?;
    
    // 运行 shellcheck 检查（在构建前进行，除非被跳过）
    if !skip_shellcheck {
        run_shellcheck_validation(&project_root)?;
    } else {
        println!("⚠️  已跳过 shellcheck 语法检查");
    }
      // 清理构建目录
    if build_dir.exists() {
        std::fs::remove_dir_all(&build_dir)?;
        std::fs::create_dir_all(&build_dir)?;
    }
    
    // 执行预构建步骤
    if let Some(ref rmake) = rmake_config {
        execute_build_steps("prebuild", &rmake.build.prebuild, &project_root)?;
    }
    
    // 复制模块文件到构建目录
    copy_module_files_to_build(config, &project_root, &build_dir, rmake_config.as_ref())?;
    
    // 执行构建步骤
    if let Some(ref rmake) = rmake_config {
        execute_build_steps("build", &rmake.build.build, &project_root)?;
    }
    
    // 生成 module.prop
    generate_module_prop(config, &build_dir)?;
    
    // 执行后构建步骤
    if let Some(ref rmake) = rmake_config {
        execute_build_steps("postbuild", &rmake.build.postbuild, &project_root)?;
    }    // 创建模块 ZIP 包
    let zip_filename = generate_zip_filename(config, rmake_config.as_ref())?;
    
    let zip_path = dist_dir.join(&zip_filename);
    create_module_zip(&build_dir, &zip_path, rmake_config.as_ref())?;
    
    // 创建源代码 tar.gz 包
    let source_filename = format!("{}-{}-source.tar.gz", config.id, config.version_code);
    let source_path = dist_dir.join(&source_filename);
    create_source_archive(&project_root, &source_path)?;println!("📦 模块包: {}", zip_path.display());
    println!("📦 源码包: {}", source_path.display());
    
    // 生成 update.json 文件
    println!("📄 生成 update.json...");
    crate::utils::generate_update_json(config, &project_root, rmake_config.as_ref()).await?;
    
    // 只有在用户明确指定输出目录时才复制文件
    if let Some(user_output) = user_output_dir {
        let user_path = Path::new(user_output);
        if user_path != dist_dir {
            std::fs::create_dir_all(user_path)?;
            let output_zip = user_path.join(&zip_filename);
            let output_source = user_path.join(&source_filename);
            std::fs::copy(&zip_path, output_zip)?;
            std::fs::copy(&source_path, output_source)?;
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
    rmake_config: Option<&crate::config::RmakeConfig>
) -> Result<()> {    // 获取排除列表（合并默认和 Rmake 配置）
    let mut exclude_items = vec![
        ".rmmp",
        "dist", 
        "build",
        "target",
        "__pycache__",
        ".git",
        "node_modules",
        ".vscode",
        ".idea",
        "*.zip",
        "*.tar.gz",
        "*.log",
        "Cargo.lock",
        "Cargo.toml", 
        "pyproject.toml",
        "uv.lock",
        ".gitignore",
        "rmmproject.toml"
    ];
    
    // 如果有 Rmake 配置，添加额外的排除项
    if let Some(rmake) = rmake_config {
        if let Some(ref excludes) = rmake.build.exclude {
            for exclude in excludes {
                exclude_items.push(exclude.as_str());
            }
            println!("📋 使用 Rmake 排除规则: {:?}", excludes);
        }
    }
      // 复制必要的模块文件
    let essential_files = [
        "README.MD", 
        "LICENSE", 
        "CHANGELOG.MD",
        "customize.sh",
        "service.sh",
        "post-fs-data.sh",
        "uninstall.sh"
    ];
    
    for file in &essential_files {
        let src = project_root.join(file);
        if src.exists() {
            let dest = build_dir.join(file);
            std::fs::copy(src, dest)?;
            println!("📄 复制 {}", file);
        }
    }
      // 复制 system 目录（如果存在）
    let system_dir = project_root.join("system");
    if system_dir.exists() {
        copy_dir_recursive_with_exclusions(&system_dir, &build_dir.join("system"), &exclude_items)?;
        println!("📁 复制 system 目录");
    }
    
    // 复制其他模块相关目录
    let module_dirs = ["META-INF", "system_ext", "vendor", "product", "apex", "data"];
    for dir in &module_dirs {
        let src_dir = project_root.join(dir);
        if src_dir.exists() && !should_exclude_path(&src_dir, &exclude_items) {
            copy_dir_recursive_with_exclusions(&src_dir, &build_dir.join(dir), &exclude_items)?;
            println!("📁 复制 {} 目录", dir);
        }
    }
    
    Ok(())
}

/// 检查路径是否应该被排除
fn should_exclude_path(path: &Path, exclude_items: &[&str]) -> bool {
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

/// 生成 module.prop
fn generate_module_prop(config: &ProjectConfig, build_dir: &Path) -> Result<()> {
    let version = config.version.as_deref().unwrap_or("v1.0.0");
    let module_prop_content = format!(
        r#"id={}
name={}
version={}
versionCode={}
author={}
description={}
updateJson={}
"#,        config.id,
        config.name,
        version,
        config.version_code,
        config.authors.first()
            .map(|a| a.name.as_str())
            .unwrap_or("Unknown"),
        config.description.as_deref().unwrap_or(""),
        config.update_json
    );
    
    let module_prop_path = build_dir.join("module.prop");
    std::fs::write(module_prop_path, module_prop_content)?;
    println!("📄 生成 module.prop");
    
    Ok(())
}

/// 创建模块 ZIP 包（使用 Rust 原生库）
fn create_module_zip(build_dir: &Path, zip_path: &Path, rmake_config: Option<&crate::config::RmakeConfig>) -> Result<()> {
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
    
    zip.finish()?;
      // 显示文件大小
    let metadata = std::fs::metadata(zip_path)?;
    let size_str = format_file_size(metadata.len());
    println!("✅ ZIP 包创建完成: {}", size_str);
    
    Ok(())
}

/// 创建源代码归档（使用 Rust 原生库）
fn create_source_archive(project_root: &Path, archive_path: &Path) -> Result<()> {
    use std::fs::File;
    use flate2::{write::GzEncoder, Compression};
    use tar::Builder;
    
    println!("📦 创建源码归档: {}", archive_path.display());
    
    // 创建 gzip 压缩文件
    let tar_gz = File::create(archive_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);
    
    // 默认排除模式
    let exclude_patterns = [
        ".rmmp", "target", "dist", "build", "__pycache__", 
        ".git", "node_modules", ".vscode", ".idea"
    ];
    
    // 遍历项目根目录
    for entry in walkdir::WalkDir::new(project_root)
        .into_iter()
        .filter_entry(|e| {
            // 检查路径是否应该被排除
            let path = e.path();
            let path_str = path.to_string_lossy();
            
            // 排除特定目录和文件
            !exclude_patterns.iter().any(|pattern| {
                if pattern.starts_with("*.") {
                    // 处理文件扩展名模式
                    let ext = &pattern[2..];
                    path_str.ends_with(ext)
                } else {
                    // 处理目录名模式
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| name == *pattern)
                        .unwrap_or(false)
                }
            })
        }) {
        
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            // 计算相对于项目根目录的路径
            let relative_path = path.strip_prefix(project_root)?;
            
            // 添加文件到 tar 归档
            tar.append_path_with_name(path, relative_path)?;
            
            println!("  ✓ {}", relative_path.display());
        }
    }
    
    // 完成归档
    tar.finish()?;
      // 显示文件大小
    let metadata = std::fs::metadata(archive_path)?;
    let size_str = format_file_size(metadata.len());
    println!("✅ 源码归档创建完成: {}", size_str);
    
    Ok(())
}

/// 格式化文件大小
fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        let kb = bytes as f64 / 1024.0;
        format!("{:.2} KB", kb)
    } else if bytes < 1024 * 1024 * 1024 {
        let mb = bytes as f64 / (1024.0 * 1024.0);
        format!("{:.2} MB", mb)
    } else {
        let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        format!("{:.2} GB", gb)
    }
}

/// 递归复制目录（带排除规则）
fn copy_dir_recursive_with_exclusions(src: &Path, dest: &Path, exclude_items: &[&str]) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        
        // 检查是否应该排除这个文件/目录
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
fn run_script(project_root: &Path, script_name: &str) -> Result<()> {
    println!("🔧 运行脚本: {}", script_name);
    
    // 加载 Rmake 配置
    let rmake_config_path = project_root.join(".rmmp").join("Rmake.toml");
    if !rmake_config_path.exists() {
        anyhow::bail!("❌ 未找到 Rmake.toml 配置文件");
    }
    
    let rmake_config = crate::config::RmakeConfig::load_from_dir(project_root)?
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
    
    println!("✅ 脚本 '{}' 执行完成", script_name);
    
    Ok(())
}

/// 生成 ZIP 文件名，支持变量替换
fn generate_zip_filename(config: &ProjectConfig, rmake_config: Option<&RmakeConfig>) -> Result<String> {    let template = if let Some(rmake) = rmake_config {
        if let Some(ref package) = rmake.package {
            if let Some(ref zip_name) = package.zip_name {
                if zip_name == "default" {
                    // 使用默认规则：包含版本代码
                    format!("{}-{}.zip", config.id, config.version_code)
                } else {
                    // 使用自定义模板
                    zip_name.clone()
                }
            } else {
                // 没有指定 zip_name，使用默认规则：包含版本代码
                format!("{}-{}.zip", config.id, config.version_code)
            }
        } else {
            // 没有 package 配置，使用默认规则：包含版本代码
            format!("{}-{}.zip", config.id, config.version_code)
        }
    } else {
        // 没有 rmake 配置，使用默认规则：包含版本代码
        format!("{}-{}.zip", config.id, config.version_code)
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

/// 替换模板中的变量
fn replace_template_variables(template: &str, config: &ProjectConfig) -> Result<String> {
    let mut result = template.to_string();
    
    // 获取 Git 提交 hash
    let git_hash = crate::utils::get_git_commit_hash().unwrap_or_else(|_| "unknown".to_string());
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
    let variables = [        ("{id}", config.id.as_str()),
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

/// 运行 shellcheck 验证
fn run_shellcheck_validation(project_root: &Path) -> Result<()> {
    println!("🔍 运行 Shellcheck 验证...");
    
    // 检查 shellcheck 是否可用
    if !crate::shellcheck::is_shellcheck_available() {
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
    match crate::shellcheck::get_shellcheck_version() {
        Ok(version) => println!("📋 Shellcheck 版本: {}", version),
        Err(_) => println!("📋 Shellcheck 版本: 未知"),
    }
      // 执行检查
    match crate::shellcheck::check_project(project_root, false) {
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
