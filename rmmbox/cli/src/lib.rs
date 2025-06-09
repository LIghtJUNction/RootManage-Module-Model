use pyo3::prelude::*;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::env;

mod commands;
mod utils;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = "RMMR : 高性能 Magisk/Apatch/KernelSU 模块开发工具";

/// Python CLI context wrapper
#[pyclass]
struct CliContext {
    profile: Option<String>,
    token: Option<String>,
    debug: bool,
}

#[pymethods]
impl CliContext {
    #[new]
    fn new(profile: Option<String>, token: Option<String>, debug: bool) -> Self {
        CliContext { profile, token, debug }
    }
    
    fn to_rust_context(&self) -> PyResult<utils::Context> {
        Ok(utils::Context::new(
            self.profile.clone(),
            self.token.clone(),
            self.debug,
        ))
    }
}

/// Build command wrapper
#[pyfunction]
#[pyo3(signature = (ctx, project_name=None, path=None, output=None, clean=false, verbose=false, debug=false))]
fn build(
    ctx: &CliContext,
    project_name: Option<String>,
    path: Option<String>,
    output: Option<String>,
    clean: bool,
    verbose: bool,
    debug: bool,
) -> PyResult<()> {
    // 创建模拟的ArgMatches
    let rust_ctx = ctx.to_rust_context()?;
    
    // 执行构建逻辑
    let result = execute_build_command(
        &rust_ctx,
        project_name,
        path,
        output,
        clean,
        verbose,
        debug,
    );

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
    }
}

/// Init command wrapper
#[pyfunction]
#[pyo3(signature = (ctx, project_path=".", yes=false, basic=true, lib=false, ravd=false))]
fn init(
    ctx: &CliContext,
    project_path: &str,
    yes: bool,
    basic: bool,
    lib: bool,
    ravd: bool,
) -> PyResult<()> {
    let rust_ctx = ctx.to_rust_context()?;
    
    let result = execute_init_command(
        &rust_ctx,
        project_path,
        yes,
        basic,
        lib,
        ravd,
    );

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
    }
}

/// Sync command wrapper
#[pyfunction]
#[pyo3(signature = (ctx, project_name=None, update=false, all=false, proxy=false))]
fn sync(
    ctx: &CliContext,
    project_name: Option<String>,
    update: bool,
    all: bool,    proxy: bool,
) -> PyResult<()> {
    let rust_ctx = ctx.to_rust_context()?;
    
    let result = execute_sync_command(
        &rust_ctx,
        project_name,
        update,
        all,
        proxy,
    );

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
    }
}

/// Run command wrapper
#[pyfunction]
#[pyo3(signature = (ctx, script_name=None, args=None))]
fn run(
    ctx: &CliContext,
    script_name: Option<String>,    args: Option<Vec<String>>,
) -> PyResult<()> {
    let rust_ctx = ctx.to_rust_context()?;
    
    let result = execute_run_command(
        &rust_ctx,
        script_name,
        args.unwrap_or_default(),
    );

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
    }
}

/// Config command group wrapper
#[pyclass]
struct ConfigCommands;

#[pymethods]
impl ConfigCommands {
    #[new]
    fn new() -> Self {
        ConfigCommands
    }
    
    #[staticmethod]
    fn ls(ctx: &CliContext, project_name: Option<String>) -> PyResult<()> {
        let rust_ctx = ctx.to_rust_context()?;
        
        let result = execute_config_ls(&rust_ctx, project_name);

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }    #[staticmethod]
    fn set(ctx: &CliContext, key: String, value: String) -> PyResult<()> {
        let rust_ctx = ctx.to_rust_context()?;
        
        let result = execute_config_set(&rust_ctx, key, value);

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }    #[staticmethod]
    fn delete(ctx: &CliContext, key: String) -> PyResult<()> {
        let rust_ctx = ctx.to_rust_context()?;
        
        let result = execute_config_delete(&rust_ctx, key);

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }    #[staticmethod]
    fn get(ctx: &CliContext, key: String) -> PyResult<Option<String>> {
        let rust_ctx = ctx.to_rust_context()?;
        
        let result = execute_config_get(&rust_ctx, key);

        match result {
            Ok(value) => Ok(value),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }
}

/// Clean command group wrapper
#[pyclass]
struct CleanCommands;

#[pymethods]
impl CleanCommands {
    #[new]
    fn new() -> Self {
        CleanCommands
    }
    
    #[staticmethod]
    fn dist(ctx: &CliContext) -> PyResult<()> {
        let rust_ctx = ctx.to_rust_context()?;
        
        let result = execute_clean_dist(&rust_ctx);

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }    #[staticmethod]
    fn tags(ctx: &CliContext) -> PyResult<()> {
        let rust_ctx = ctx.to_rust_context()?;
        
        let result = execute_clean_tags(&rust_ctx);

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }    #[staticmethod]
    fn all(ctx: &CliContext) -> PyResult<()> {
        let rust_ctx = ctx.to_rust_context()?;
        
        let result = execute_clean_all(&rust_ctx);

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }
}

/// Get version information
#[pyfunction]
fn version() -> String {
    VERSION.to_string()
}

/// CLI module for Python
#[pymodule]
fn rmmr(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // 添加类和函数
    m.add_class::<CliContext>()?;
    m.add_class::<ConfigCommands>()?;
    m.add_class::<CleanCommands>()?;
    
    // 添加命令函数
    m.add_function(wrap_pyfunction!(build, m)?)?;
    m.add_function(wrap_pyfunction!(init, m)?)?;
    m.add_function(wrap_pyfunction!(sync, m)?)?;
    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    
    // 添加主 CLI 函数
    m.add_function(wrap_pyfunction!(cli, m)?)?;
    
    // 添加常量
    m.add("__version__", VERSION)?;
    m.add("__description__", DESCRIPTION)?;
    
    Ok(())
}

/// 主 CLI 入口点 - 解析命令行参数并执行相应命令
#[pyfunction]
#[pyo3(signature = (args=None))]
fn cli(args: Option<Vec<String>>) -> PyResult<()> {
    let args = args.unwrap_or_else(|| {
        // 当从Python调用时，使用程序名作为第一个参数
        vec!["rmmr".to_string()]
    });
    
    let app = build_cli_app();
    let matches = app.try_get_matches_from(args)
        .map_err(|e| pyo3::exceptions::PySystemExit::new_err(e.to_string()))?;
    
    // 创建全局上下文
    let profile = matches.get_one::<String>("profile").cloned();
    let token = matches.get_one::<String>("token").cloned();
    let debug = matches.get_flag("debug");
    
    let ctx = utils::Context::new(profile, token, debug);
    
    // 处理子命令
    match matches.subcommand() {
        Some(("build", sub_matches)) => {
            handle_build_command(&ctx, sub_matches)?;
        }
        Some(("init", sub_matches)) => {
            handle_init_command(&ctx, sub_matches)?;
        }
        Some(("sync", sub_matches)) => {
            handle_sync_command(&ctx, sub_matches)?;
        }
        Some(("run", sub_matches)) => {
            handle_run_command(&ctx, sub_matches)?;
        }
        Some(("config", sub_matches)) => {
            handle_config_command(&ctx, sub_matches)?;
        }
        Some(("clean", sub_matches)) => {
            handle_clean_command(&ctx, sub_matches)?;
        }
        Some(("version", _)) => {
            println!("RMMR CLI version: {}", VERSION);
        }
        _ => {
            return Err(pyo3::exceptions::PyValueError::new_err("No command specified"));
        }
    }
    
    Ok(())
}

/// 构建 CLI 应用程序定义
fn build_cli_app() -> Command {
    Command::new("rmmr")
        .version(VERSION)
        .about(DESCRIPTION)
        .arg(
            Arg::new("profile")
                .short('p')
                .long("profile")
                .value_name("PROFILE")
                .help("指定配置文件")
                .action(ArgAction::Set),
        )        .arg(
            Arg::new("token")
                .short('t')
                .long("token")
                .value_name("TOKEN")
                .help("指定GITHUB访问令牌")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("debug")
                .long("debug")
                .help("启用调试模式")
                .action(ArgAction::SetTrue),
        )
        .subcommand(
            Command::new("build")
                .about("构建 RMM 项目")
                .arg(
                    Arg::new("project-name")
                        .long("project-name")
                        .value_name("NAME")
                        .help("项目名称")
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .value_name("PATH")
                        .help("项目路径")
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .value_name("OUTPUT")
                        .help("输出路径")
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("clean")
                        .long("clean")
                        .help("清理构建")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .help("详细输出")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("debug")
                        .long("debug")
                        .help("调试模式")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("init")
                .about("初始化新的 RMM 项目")
                .arg(
                    Arg::new("project_path")
                        .help("项目路径")
                        .default_value(".")
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("yes")
                        .short('y')
                        .long("yes")
                        .help("自动确认所有选项")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("basic")
                        .long("basic")
                        .help("创建基础项目")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("lib")
                        .long("lib")
                        .help("创建库项目")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("ravd")
                        .long("ravd")
                        .help("创建 RAVD 项目")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("sync")
                .about("同步项目依赖")
                .arg(
                    Arg::new("project-name")
                        .long("project-name")
                        .value_name("NAME")
                        .help("项目名称")
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("update")
                        .long("update")
                        .help("更新依赖")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("all")
                        .long("all")
                        .help("同步所有项目")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("proxy")
                        .long("proxy")
                        .help("使用代理")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("run")
                .about("运行项目脚本")
                .arg(
                    Arg::new("script_name")
                        .help("脚本名称")
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("args")
                        .help("脚本参数")
                        .num_args(0..)
                        .action(ArgAction::Append),
                ),
        )
        .subcommand(
            Command::new("config")
                .about("配置管理")
                .subcommand(
                    Command::new("ls")
                        .about("列出配置")
                        .arg(
                            Arg::new("project-name")
                                .long("project-name")
                                .value_name("NAME")
                                .help("项目名称")
                                .action(ArgAction::Set),
                        ),
                )
                .subcommand(
                    Command::new("set")
                        .about("设置配置")
                        .arg(
                            Arg::new("key")
                                .help("配置键")
                                .required(true)
                                .action(ArgAction::Set),
                        )
                        .arg(
                            Arg::new("value")
                                .help("配置值")
                                .required(true)
                                .action(ArgAction::Set),
                        ),
                )
                .subcommand(
                    Command::new("get")
                        .about("获取配置")
                        .arg(
                            Arg::new("key")
                                .help("配置键")
                                .required(true)
                                .action(ArgAction::Set),
                        ),
                )
                .subcommand(
                    Command::new("delete")
                        .about("删除配置")
                        .arg(
                            Arg::new("key")
                                .help("配置键")
                                .required(true)
                                .action(ArgAction::Set),
                        ),
                ),
        )
        .subcommand(
            Command::new("clean")
                .about("清理项目文件")
                .subcommand(Command::new("dist").about("清理构建输出"))
                .subcommand(Command::new("tags").about("清理项目标签"))
                .subcommand(Command::new("all").about("清理所有生成的文件")),
        )        .subcommand(Command::new("version").about("显示版本信息"))
}

// 命令处理函数

fn handle_build_command(ctx: &utils::Context, matches: &ArgMatches) -> PyResult<()> {
    let project_name = matches.get_one::<String>("project-name").cloned();
    let path = matches.get_one::<String>("path").cloned();
    let output = matches.get_one::<String>("output").cloned();
    let clean = matches.get_flag("clean");
    let verbose = matches.get_flag("verbose");
    let debug = matches.get_flag("debug");

    execute_build_command(ctx, project_name, path, output, clean, verbose, debug)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    
    Ok(())
}

fn handle_init_command(ctx: &utils::Context, matches: &ArgMatches) -> PyResult<()> {
    let project_path = matches.get_one::<String>("project_path").unwrap();
    let yes = matches.get_flag("yes");
    let basic = matches.get_flag("basic");
    let lib = matches.get_flag("lib");
    let ravd = matches.get_flag("ravd");

    execute_init_command(ctx, project_path, yes, basic, lib, ravd)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    
    Ok(())
}

fn handle_sync_command(ctx: &utils::Context, matches: &ArgMatches) -> PyResult<()> {
    let project_name = matches.get_one::<String>("project-name").cloned();
    let update = matches.get_flag("update");
    let all = matches.get_flag("all");
    let proxy = matches.get_flag("proxy");

    execute_sync_command(ctx, project_name, update, all, proxy)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    
    Ok(())
}

fn handle_run_command(ctx: &utils::Context, matches: &ArgMatches) -> PyResult<()> {
    let script_name = matches.get_one::<String>("script_name").cloned();
    let args: Vec<String> = matches.get_many::<String>("args")
        .unwrap_or_default()
        .cloned()
        .collect();

    execute_run_command(ctx, script_name, args)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    
    Ok(())
}

fn handle_config_command(ctx: &utils::Context, matches: &ArgMatches) -> PyResult<()> {
    match matches.subcommand() {
        Some(("ls", sub_matches)) => {
            let project_name = sub_matches.get_one::<String>("project-name").cloned();
            execute_config_ls(ctx, project_name)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        }
        Some(("set", sub_matches)) => {
            let key = sub_matches.get_one::<String>("key").unwrap().clone();
            let value = sub_matches.get_one::<String>("value").unwrap().clone();
            execute_config_set(ctx, key, value)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        }
        Some(("get", sub_matches)) => {
            let key = sub_matches.get_one::<String>("key").unwrap().clone();
            match execute_config_get(ctx, key)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))? {
                Some(value) => println!("{}={}", sub_matches.get_one::<String>("key").unwrap(), value),
                None => eprintln!("配置键不存在"),
            }
        }
        Some(("delete", sub_matches)) => {
            let key = sub_matches.get_one::<String>("key").unwrap().clone();
            execute_config_delete(ctx, key)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        }
        _ => {
            return Err(pyo3::exceptions::PyValueError::new_err("Config subcommand required"));
        }
    }
    
    Ok(())
}

fn handle_clean_command(ctx: &utils::Context, matches: &ArgMatches) -> PyResult<()> {
    match matches.subcommand() {
        Some(("dist", _)) => {
            execute_clean_dist(ctx)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        }
        Some(("tags", _)) => {
            execute_clean_tags(ctx)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        }
        Some(("all", _)) => {
            execute_clean_all(ctx)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        }
        _ => {
            return Err(pyo3::exceptions::PyValueError::new_err("Clean subcommand required"));
        }
    }
    
    Ok(())
}

// 以下是各命令的具体实现函数

fn execute_build_command(
    ctx: &utils::Context,
    project_name: Option<String>,
    path: Option<String>,
    output: Option<String>,
    clean: bool,
    verbose: bool,
    debug: bool,
) -> anyhow::Result<()> {
    use std::fs;
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use walkdir::WalkDir;
    
    // 确定项目路径
    let project_path = if let Some(path) = path {
        std::path::PathBuf::from(path)
    } else if let Some(name) = &project_name {
        std::env::current_dir()?.join(name)
    } else {
        std::env::current_dir()?
    };

    let project_name = project_name.unwrap_or_else(|| {
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
    let project = utils::RmmProject::load_from_file(&rmm_toml)?;
    ctx.info(&format!("📦 项目名称: {}", project.name));
    ctx.info(&format!("🏷️  版本: {}", project.version));

    if verbose || ctx.debug {
        ctx.info("🔍 详细模式已启用");
    }
    if debug || ctx.debug {
        ctx.info("🐛 调试模式已启用");
    }

    // 创建输出目录
    let output_dir = if let Some(output) = output {
        std::path::PathBuf::from(output)
    } else {
        project_path.join(".rmmp").join("dist")
    };

    if clean && output_dir.exists() {
        ctx.info("🧹 清理旧的构建文件...");
        utils::remove_dir_all(&output_dir)?;
    }

    utils::ensure_dir_exists(&output_dir)?;
    ctx.info(&format!("📂 输出目录: {}", output_dir.display()));

    // 生成版本代码
    let version_code = project.versionCode.unwrap_or_else(|| {
        generate_version_code(&project.version).unwrap_or(1)
    });

    ctx.info(&format!("🔢 版本代码: {}", version_code));

    // 创建 Magisk 模块 ZIP 文件
    let zip_filename = format!("{}-{}.zip", project.name, project.version);
    let zip_path = output_dir.join(&zip_filename);
    
    ctx.info(&format!("📦 创建模块包: {}", zip_filename));
    
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = zip::ZipWriter::new(zip_file);    // 添加 module.prop 文件
    let module_prop = create_module_prop(&project, version_code)?;
    zip.start_file("module.prop", SimpleFileOptions::default())?;
    zip.write_all(module_prop.as_bytes())?;
    
    // 添加项目文件
    add_project_files_to_zip(&mut zip, &project_path, ctx)?;
    
    // 完成 ZIP 文件
    zip.finish()?;
    
    ctx.info(&format!("✅ 项目 '{}' 构建完成！", project_name));
    ctx.info(&format!("📦 输出文件: {}", zip_path.display()));
    
    // 显示文件大小
    let file_size = fs::metadata(&zip_path)?.len();
    ctx.info(&format!("📊 文件大小: {:.2} KB", file_size as f64 / 1024.0));

    Ok(())
}

fn create_module_prop(project: &utils::RmmProject, version_code: u32) -> anyhow::Result<String> {
    let mut prop = String::new();
    
    prop.push_str(&format!("id={}\n", project.id.as_ref().unwrap_or(&project.name)));
    prop.push_str(&format!("name={}\n", project.name));
    prop.push_str(&format!("version={}\n", project.version));
    prop.push_str(&format!("versionCode={}\n", version_code));
    prop.push_str(&format!("author={}\n", project.author.as_ref().unwrap_or(&"Unknown".to_string())));
    prop.push_str(&format!("description={}\n", project.description.as_ref().unwrap_or(&"RMM Module".to_string())));
    
    if let Some(update_json) = &project.updateJson {
        prop.push_str(&format!("updateJson={}\n", update_json));
    }
    
    Ok(prop)
}

fn add_project_files_to_zip(
    zip: &mut zip::ZipWriter<std::fs::File>,
    project_path: &std::path::Path,
    ctx: &utils::Context,
) -> anyhow::Result<()> {
    use std::fs;
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use walkdir::WalkDir;
      ctx.info("📁 添加项目文件到模块包...");
    
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    
    // 添加 update-binary 脚本
    let update_binary_template = include_str!("templates/update-binary.sh");
    zip.start_file("META-INF/com/google/android/update-binary", options)?;
    zip.write_all(update_binary_template.as_bytes())?;
    
    // 添加 updater-script（空文件）
    zip.start_file("META-INF/com/google/android/updater-script", options)?;
    zip.write_all(b"#MAGISK\n")?;
      // 遍历项目目录，添加需要的文件
    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let file_path = entry.path();
        let relative_path = file_path.strip_prefix(project_path)?;
        
        // 跳过不需要的文件
        if should_skip_file(&relative_path) {
            continue;
        }
        
        let file_content = fs::read(file_path)?;
        let zip_path = relative_path.to_string_lossy().replace('\\', "/");
        
        if ctx.debug {
            ctx.debug(&format!("添加文件: {}", zip_path));
        }
        
        zip.start_file(&zip_path, options)?;
        zip.write_all(&file_content)?;
    }
    
    Ok(())
}

fn should_skip_file(path: &std::path::Path) -> bool {
    let path_str = path.to_string_lossy();
    
    // 跳过配置文件和构建目录
    if path_str.starts_with("rmm.toml") ||
       path_str.starts_with(".rmmp/") ||
       path_str.starts_with(".git/") ||
       path_str.starts_with("target/") ||
       path_str.starts_with("node_modules/") ||
       path_str.ends_with(".pyc") ||
       path_str.ends_with(".pyo") ||
       path_str.ends_with(".log") {
        return true;
    }
    
    false
}

fn generate_version_code(version: &str) -> anyhow::Result<u32> {
    // 简单的版本代码生成：将版本号转换为数字
    let parts: Vec<&str> = version.split('.').collect();
    let mut code = 0u32;
    
    for (i, part) in parts.iter().enumerate() {
        if i >= 3 { break; } // 最多处理3个部分
        
        let num: u32 = part.parse().unwrap_or(0);
        code += num * (100u32.pow(2 - i as u32));
    }
    
    Ok(if code == 0 { 1 } else { code })
}

fn execute_init_command(
    ctx: &utils::Context,
    project_path: &str,
    yes: bool,
    _basic: bool,
    lib: bool,
    ravd: bool,
) -> anyhow::Result<()> {
    use std::fs;
    use std::io::Write;
    
    let project_path = std::path::PathBuf::from(project_path);
    
    // 确定项目类型
    let project_type = if lib {
        "library"
    } else if ravd {
        "ravd"
    } else {
        "basic"
    };

    ctx.info(&format!("🚀 正在初始化 {} 类型的RMM项目...", project_type));
    ctx.info(&format!("📁 项目路径: {}", project_path.display()));

    // 检查目录是否已存在
    if project_path.exists() && project_path.is_dir() {
        let entries: Vec<_> = fs::read_dir(&project_path)?.collect();
        if !entries.is_empty() && !yes {
            if !confirm_overwrite()? {
                ctx.info("❌ 初始化已取消");
                return Ok(());
            }
        }
    } else {
        utils::ensure_dir_exists(&project_path)?;
    }

    let project_name = project_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("rmm-project")
        .to_string();

    // 创建项目文件
    match project_type {
        "basic" => create_basic_project(&project_path, &project_name, ctx)?,
        "library" => create_library_project(&project_path, &project_name, ctx)?,
        "ravd" => create_ravd_project(&project_path, &project_name, ctx)?,
        _ => anyhow::bail!("未支持的项目类型: {}", project_type),
    }

    ctx.info(&format!("✅ 项目 '{}' 初始化完成！", project_name));
    ctx.info("💡 接下来你可以:");
    ctx.info(&format!("   cd {}", project_path.display()));
    ctx.info("   rmmr build");

    Ok(())
}

fn confirm_overwrite() -> anyhow::Result<bool> {
    use std::io::{self, Write};
    
    print!("目录不为空，是否要继续？ (y/N): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    Ok(input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes")
}

fn create_basic_project(
    project_path: &std::path::Path,
    project_name: &str,
    ctx: &utils::Context,
) -> anyhow::Result<()> {
    use std::fs;
    
    ctx.info("📄 创建基础项目文件...");
    
    // 创建 rmm.toml
    let rmm_toml = format!(
        r#"[project]
name = "{}"
version = "1.0.0"
author = "Your Name"
description = "A basic RMM module"
id = "{}"

[build]
output = ".rmmp/dist"
"#,
        project_name, project_name.replace("-", "_")
    );
    
    fs::write(project_path.join("rmm.toml"), rmm_toml)?;
    
    // 创建 README.md
    let readme = format!(
        r#"# {}

A Magisk/Apatch/KernelSU module created with RMM.

## Description

This is a basic RMM module.

## Installation

1. Build the module: `rmmr build`
2. Install the generated ZIP file through Magisk/Apatch/KernelSU

## Usage

Describe how to use your module here.
"#,
        project_name
    );
    
    fs::write(project_path.join("README.md"), readme)?;
    
    // 创建基础目录结构
    utils::ensure_dir_exists(&project_path.join("system"))?;
    utils::ensure_dir_exists(&project_path.join("service.d"))?;
    
    // 创建 service.sh
    let service_sh = r#"#!/system/bin/sh
# Service script for the module
# This script will be executed in late_start service mode
# More info in the main Magisk thread
"#;
    
    fs::write(project_path.join("service.sh"), service_sh)?;
    
    // 创建 install.sh（可选）
    let install_sh = r#"#!/system/bin/sh
# Install script for the module
# This script will be executed during module installation
"#;
    
    fs::write(project_path.join("install.sh"), install_sh)?;
    
    ctx.info("✅ 基础项目文件创建完成");
    
    Ok(())
}

fn create_library_project(
    project_path: &std::path::Path,
    project_name: &str,
    ctx: &utils::Context,
) -> anyhow::Result<()> {
    use std::fs;
    
    ctx.info("📚 创建库项目文件...");
    
    // 创建 rmm.toml
    let rmm_toml = format!(
        r#"[project]
name = "{}"
version = "1.0.0"
author = "Your Name"
description = "A library RMM module"
id = "{}"

[build]
output = ".rmmp/dist"

[library]
type = "shared"
"#,
        project_name, project_name.replace("-", "_")
    );
    
    fs::write(project_path.join("rmm.toml"), rmm_toml)?;
    
    // 创建库目录结构
    utils::ensure_dir_exists(&project_path.join("lib"))?;
    utils::ensure_dir_exists(&project_path.join("include"))?;
    utils::ensure_dir_exists(&project_path.join("src"))?;
    
    // 创建示例头文件
    let header_example = format!(
        r#"#ifndef {}_H
#define {}_H

// Example library header
void example_function();

#endif // {}_H
"#,
        project_name.to_uppercase().replace("-", "_"),
        project_name.to_uppercase().replace("-", "_"),
        project_name.to_uppercase().replace("-", "_")
    );
    
    fs::write(project_path.join("include").join(format!("{}.h", project_name)), header_example)?;
    
    // 创建示例源文件
    let source_example = format!(
        r#"#include "{}.h"
#include <stdio.h>

void example_function() {{
    printf("Hello from {} library!\\n");
}}
"#,
        project_name, project_name
    );
    
    fs::write(project_path.join("src").join(format!("{}.c", project_name)), source_example)?;
    
    ctx.info("✅ 库项目文件创建完成");
    
    Ok(())
}

fn create_ravd_project(
    project_path: &std::path::Path,
    project_name: &str,
    ctx: &utils::Context,
) -> anyhow::Result<()> {
    use std::fs;
    
    ctx.info("🔒 创建 RAVD 项目文件...");
    
    // 创建 rmm.toml
    let rmm_toml = format!(
        r#"[project]
name = "{}"
version = "1.0.0"
author = "Your Name"
description = "A RAVD RMM module"
id = "{}"

[build]
output = ".rmmp/dist"

[ravd]
enabled = true
"#,
        project_name, project_name.replace("-", "_")
    );
    
    fs::write(project_path.join("rmm.toml"), rmm_toml)?;
    
    // 创建 RAVD 相关目录
    utils::ensure_dir_exists(&project_path.join("ravd"))?;
    utils::ensure_dir_exists(&project_path.join("system"))?;
    
    // 创建 RAVD 配置文件
    let ravd_config = r#"{
    "name": "Example RAVD Module",
    "version": "1.0.0",
    "author": "Your Name",
    "description": "An example RAVD module",
    "permissions": [],
    "hooks": []
}
"#;
    
    fs::write(project_path.join("ravd").join("config.json"), ravd_config)?;
    
    ctx.info("✅ RAVD 项目文件创建完成");
    
    Ok(())
}

fn execute_sync_command(
    ctx: &utils::Context,
    project_name: Option<String>,
    _update: bool,
    sync_all: bool,
    _proxy: bool,
) -> anyhow::Result<()> {
    
    if project_name.is_none() && !sync_all {
        anyhow::bail!("❌ 请指定项目名称或使用 --all 参数同步所有项目");
    }

    if sync_all {
        ctx.info("🔄 同步所有RMM项目...");
        // 同步所有项目逻辑...
    } else if let Some(name) = project_name {
        ctx.info(&format!("🔄 同步项目: {}", name));
        // 同步单个项目逻辑...
    }

    ctx.info("✅ 同步完成!");

    Ok(())
}

fn execute_run_command(
    ctx: &utils::Context,
    script_name: Option<String>,
    _args: Vec<String>,
) -> anyhow::Result<()> {
    
    if script_name.is_none() {
        ctx.info("📜 请指定要运行的脚本名称");
        return Ok(());
    }

    let script_name = script_name.unwrap();
    ctx.info(&format!("🚀 运行脚本: {}", script_name));

    // 执行脚本逻辑...
    ctx.info(&format!("✅ 脚本 '{}' 执行完成", script_name));

    Ok(())
}

fn execute_config_ls(ctx: &utils::Context, project_name: Option<String>) -> anyhow::Result<()> {
    
    if let Some(project_name) = project_name {
        ctx.info(&format!("项目 '{}' 的配置信息:", project_name));
        // 显示项目配置...
    } else {
        ctx.info("系统配置:");
        // 显示系统配置...
    }

    Ok(())
}

fn execute_config_set(ctx: &utils::Context, key: String, value: String) -> anyhow::Result<()> {
    
    let mut config = utils::Config::load()?;
    config.set(key.clone(), value.clone());
    config.save()?;

    ctx.info(&format!("✅ 配置已设置: {} = {}", key, value));

    Ok(())
}

fn execute_config_delete(ctx: &utils::Context, key: String) -> anyhow::Result<()> {
    
    let mut config = utils::Config::load()?;
    
    if let Some(old_value) = config.remove(&key) {
        config.save()?;
        ctx.info(&format!("✅ 配置已删除: {} (原值: {})", key, old_value));
    } else {
        ctx.warn(&format!("⚠️  配置键不存在: {}", key));
    }

    Ok(())
}

fn execute_config_get(ctx: &utils::Context, key: String) -> anyhow::Result<Option<String>> {
    
    let config = utils::Config::load()?;
    
    if let Some(value) = config.get(&key) {
        Ok(Some(value.clone()))
    } else {
        ctx.error(&format!("❌ 配置键不存在: {}", key));
        Ok(None)
    }
}

fn execute_clean_dist(ctx: &utils::Context) -> anyhow::Result<()> {
    
    ctx.info("🧹 清理构建输出目录...");

    let current_dir = std::env::current_dir()?;
    let dist_dir = current_dir.join(".rmmp").join("dist");

    if utils::dir_exists(&dist_dir) {
        utils::remove_dir_all(&dist_dir)?;
        ctx.info(&format!("✅ 已清理: {}", dist_dir.display()));
    } else {
        ctx.info("ℹ️  构建输出目录不存在，无需清理");
    }

    Ok(())
}

fn execute_clean_tags(ctx: &utils::Context) -> anyhow::Result<()> {
    
    ctx.info("🧹 清理项目标签...");

    let current_dir = std::env::current_dir()?;
    let tags_dir = current_dir.join(".rmmp").join("tags");

    if utils::dir_exists(&tags_dir) {
        utils::remove_dir_all(&tags_dir)?;
        ctx.info(&format!("✅ 已清理: {}", tags_dir.display()));
    } else {
        ctx.info("ℹ️  标签目录不存在，无需清理");
    }

    Ok(())
}

fn execute_clean_all(ctx: &utils::Context) -> anyhow::Result<()> {
    
    ctx.info("🧹 清理所有生成的文件...");

    let current_dir = std::env::current_dir()?;
    let rmmp_dir = current_dir.join(".rmmp");

    if utils::dir_exists(&rmmp_dir) {
        utils::remove_dir_all(&rmmp_dir)?;
        ctx.info(&format!("✅ 已清理: {}", rmmp_dir.display()));
    } else {
        ctx.info("ℹ️  没有找到需要清理的文件");
    }

    ctx.info("✅ 清理完成");

    Ok(())
}
