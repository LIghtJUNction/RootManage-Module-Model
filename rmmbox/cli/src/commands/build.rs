use clap::{Arg, ArgMatches, Command};
use anyhow::Result;
use std::path::PathBuf;
use std::fs::File;
use crate::utils::{Context, RmmProject, ensure_dir_exists, remove_dir_all, run_command};

pub fn build_command() -> Command {
    Command::new("build")
        .about("构建RMM项目")
        .arg(
            Arg::new("project_name")
                .help("要构建的项目名称 (可选，如果不指定则构建当前目录的项目)")
                .value_name("PROJECT_NAME")
                .required(false)
        )
        .arg(
            Arg::new("path")
                .short('p')
                .long("path")
                .value_name("PATH")
                .help("指定项目路径")
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("OUTPUT")
                .help("指定输出目录")
        )
        .arg(
            Arg::new("clean")
                .short('c')
                .long("clean")
                .action(clap::ArgAction::SetTrue)
                .help("构建前清理输出目录")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("显示详细构建信息")
        )
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .action(clap::ArgAction::SetTrue)
                .help("启用调试模式")
        )
}

pub fn handle_build(ctx: &Context, matches: &ArgMatches) -> Result<()> {
    let project_name = matches.get_one::<String>("project_name");
    let path = matches.get_one::<String>("path").map(PathBuf::from);
    let output = matches.get_one::<String>("output").map(PathBuf::from);
    let clean = matches.get_flag("clean");
    let verbose = matches.get_flag("verbose") || ctx.debug;
    let debug = matches.get_flag("debug") || ctx.debug;

    // 确定项目路径
    let project_path = if let Some(path) = path {
        path
    } else if let Some(name) = project_name {
        // 这里需要实现通过项目名称查找路径的逻辑
        // 暂时使用当前目录下的子目录
        std::env::current_dir()?.join(name)
    } else {
        std::env::current_dir()?
    };

    let project_name = project_name
        .map(|s| s.clone())
        .unwrap_or_else(|| {
            project_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

    ctx.info(&format!("🔨 正在构建项目: {}", project_name));
    ctx.info(&format!("📁 项目路径: {}", project_path.display()));

    // 检查是否是有效的RMM项目
    let rmm_toml = project_path.join("rmm.toml");
    if !rmm_toml.exists() {
        anyhow::bail!(
            "❌ 错误: '{}' 不是一个有效的RMM项目。\n请确保项目目录包含 rmm.toml 文件。",
            project_path.display()
        );
    }

    // 加载项目配置
    let project = RmmProject::load_from_file(&rmm_toml)?;

    if verbose {
        ctx.info("🔍 详细模式已启用");
    }
    if debug {
        ctx.info("🐛 调试模式已启用");
    }
    if clean {
        ctx.info("🧹 清理模式已启用");
    }

    // 设置输出目录
    let output_dir = output.unwrap_or_else(|| project_path.join(".rmmp").join("dist"));
    ctx.info(&format!("📦 输出目录: {}", output_dir.display()));

    // 生成新版本
    ctx.info(&format!("📝 正在为项目 {} 生成新版本...", project_name));
    let old_version = &project.version;
    ctx.info(&format!("🔄 当前版本: {}", old_version));

    // 版本生成逻辑 (简化版本)
    let new_version = generate_new_version(old_version)?;
    let version_code = generate_version_code(&new_version)?;
    ctx.info(&format!("📋 新版本信息: {} (版本代码: {})", new_version, version_code));

    // 清理输出目录
    if clean && output_dir.exists() {
        if verbose {
            ctx.info(&format!("🧹 清理输出目录: {}", output_dir.display()));
        }
        remove_dir_all(&output_dir)?;
    }

    // 确保输出目录存在
    ensure_dir_exists(&output_dir)?;

    // 执行构建
    let start_time = std::time::Instant::now();
    
    let result = build_project(&project, &project_path, &output_dir, verbose, debug)?;
    
    let build_time = start_time.elapsed();

    if result.success {
        ctx.info(&format!("✅ 项目 '{}' 构建成功！", project_name));
        
        if !result.output_files.is_empty() {
            ctx.info("📦 生成的文件:");
            for output_file in &result.output_files {
                let file_path = PathBuf::from(output_file);
                if file_path.extension().map(|ext| ext == "zip").unwrap_or(false) {
                    ctx.info(&format!("  🗜️  模块包: {}", output_file));
                } else if output_file.ends_with(".tar.gz") {
                    ctx.info(&format!("  📄 源代码包: {}", output_file));
                } else {
                    ctx.info(&format!("  📦 文件: {}", output_file));
                }
            }
        }
        
        ctx.info(&format!("⏱️  构建时间: {:.2}秒", build_time.as_secs_f64()));
    } else {
        anyhow::bail!("❌ 项目 '{}' 构建失败: {}", project_name, result.error.unwrap_or_else(|| "未知错误".to_string()));
    }

    Ok(())
}

#[derive(Debug)]
struct BuildResult {
    success: bool,
    output_files: Vec<String>,
    error: Option<String>,
}

fn build_project(
    project: &RmmProject,
    project_path: &std::path::Path,
    output_dir: &std::path::Path,
    verbose: bool,
    debug: bool,
) -> Result<BuildResult> {
    let mut output_files = Vec::new();

    // 创建模块包zip文件
    let zip_file = output_dir.join(format!("{}-{}.zip", project.name, project.version));
    create_module_zip(project, project_path, &zip_file, verbose)?;
    output_files.push(zip_file.to_string_lossy().to_string());

    // 创建源代码包 (如果需要)
    if should_create_source_package(project) {
        let source_file = output_dir.join(format!("{}-{}.tar.gz", project.name, project.version));
        create_source_package(project_path, &source_file, verbose)?;
        output_files.push(source_file.to_string_lossy().to_string());
    }

    Ok(BuildResult {
        success: true,
        output_files,
        error: None,
    })
}

fn create_module_zip(
    project: &RmmProject,
    project_path: &std::path::Path,
    output_file: &std::path::Path,
    verbose: bool,
) -> Result<()> {
    use std::fs::File;
    use zip::write::{FileOptions, ZipWriter};
    use zip::CompressionMethod;

    let file = File::create(output_file)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default().compression_method(CompressionMethod::Deflated);

    // 添加module.prop文件
    let module_prop = create_module_prop(project)?;
    zip.start_file("module.prop", options)?;
    std::io::Write::write_all(&mut zip, module_prop.as_bytes())?;

    // 添加其他必要文件
    add_project_files_to_zip(&mut zip, project_path, verbose)?;

    zip.finish()?;
    Ok(())
}

fn create_module_prop(project: &RmmProject) -> Result<String> {
    let mut prop = String::new();
    prop.push_str(&format!("id={}\n", project.id.as_ref().unwrap_or(&project.name)));
    prop.push_str(&format!("name={}\n", project.name));
    prop.push_str(&format!("version={}\n", project.version));
    prop.push_str(&format!("versionCode={}\n", project.versionCode.unwrap_or(1)));
    
    if let Some(author) = &project.author {
        prop.push_str(&format!("author={}\n", author));
    }
    
    if let Some(description) = &project.description {
        prop.push_str(&format!("description={}\n", description));
    }
    
    if let Some(update_json) = &project.updateJson {
        prop.push_str(&format!("updateJson={}\n", update_json));
    }

    Ok(prop)
}

fn add_project_files_to_zip(
    zip: &mut zip::ZipWriter<File>,
    project_path: &std::path::Path,
    verbose: bool,
) -> Result<()> {
    use std::fs;
    use std::io::Read;
    use zip::write::FileOptions;

    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);

    // 添加常见的模块文件
    let common_files = ["service.sh", "post-fs-data.sh", "uninstall.sh", "customize.sh"];
    
    for file_name in &common_files {
        let file_path = project_path.join(file_name);
        if file_path.exists() {
            if verbose {
                println!("添加文件: {}", file_name);
            }
            
            let mut file = fs::File::open(&file_path)?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;
            
            zip.start_file(*file_name, options)?;
            std::io::Write::write_all(zip, &contents)?;
        }
    }

    // 递归添加system目录和META-INF目录
    if project_path.join("system").exists() {
        add_directory_to_zip(zip, &project_path.join("system"), "system", verbose)?;
    }
    
    if project_path.join("META-INF").exists() {
        add_directory_to_zip(zip, &project_path.join("META-INF"), "META-INF", verbose)?;
    }

    Ok(())
}

fn add_directory_to_zip(
    zip: &mut zip::ZipWriter<File>,
    dir_path: &std::path::Path,
    zip_path: &str,
    verbose: bool,
) -> Result<()> {
    use std::fs;
    use std::io::Read;
    use zip::write::FileOptions;

    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let zip_file_path = format!("{}/{}", zip_path, name.to_string_lossy());

        if path.is_file() {
            if verbose {
                println!("添加文件: {}", zip_file_path);
            }
            
            let mut file = fs::File::open(&path)?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;
            
            zip.start_file(&zip_file_path, options)?;
            std::io::Write::write_all(zip, &contents)?;
        } else if path.is_dir() {
            add_directory_to_zip(zip, &path, &zip_file_path, verbose)?;
        }
    }

    Ok(())
}

fn should_create_source_package(_project: &RmmProject) -> bool {
    // 简化版本：总是创建源代码包
    true
}

fn create_source_package(
    project_path: &std::path::Path,
    output_file: &std::path::Path,
    verbose: bool,
) -> Result<()> {
    // 使用tar命令创建源代码包
    let args = vec![
        "czf",
        output_file.to_str().unwrap(),
        "-C",
        project_path.parent().unwrap().to_str().unwrap(),
        project_path.file_name().unwrap().to_str().unwrap(),
    ];
    
    if verbose {
        println!("创建源代码包: tar {}", args.join(" "));
    }
    
    run_command("tar", &args, None)?;
    Ok(())
}

fn generate_new_version(current_version: &str) -> Result<String> {
    // 简化的版本生成逻辑：增加最后一个数字
    let parts: Vec<&str> = current_version.split('.').collect();
    if parts.len() >= 3 {
        let major: u32 = parts[0].parse().unwrap_or(1);
        let minor: u32 = parts[1].parse().unwrap_or(0);
        let patch: u32 = parts[2].parse().unwrap_or(0) + 1;
        Ok(format!("{}.{}.{}", major, minor, patch))
    } else {
        Ok(format!("{}.1", current_version))
    }
}

fn generate_version_code(version: &str) -> Result<u32> {
    // 简化的版本代码生成：将版本号转换为整数
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 3 {
        let major: u32 = parts[0].parse().unwrap_or(1);
        let minor: u32 = parts[1].parse().unwrap_or(0);
        let patch: u32 = parts[2].parse().unwrap_or(0);
        Ok(major * 10000 + minor * 100 + patch)
    } else {
        Ok(10001)
    }
}
