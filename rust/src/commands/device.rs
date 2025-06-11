use clap::{Arg, ArgAction, ArgMatches, Command};
use anyhow::Result;
use crate::config::RmmConfig;
use crate::adb::{AdbManager, check_adb_available};
use std::path::Path;

/// 构建 device 命令
pub fn build_command() -> Command {
    Command::new("device")
        .alias("devices")  // 添加 devices 别名
        .about("管理 ADB 设备和模块安装")
        .long_about("通过 ADB 管理连接的 Android 设备，包括模块安装、设备信息查看等")
        .subcommand(
            Command::new("list")
                .about("列出连接的设备")
                .alias("ls")
        )
        .subcommand(
            Command::new("info")
                .about("显示设备详细信息")
                .arg(
                    Arg::new("device_id")
                        .help("设备ID")
                        .value_name("DEVICE_ID")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("shell")
                .about("在设备上执行shell命令")
                .arg(
                    Arg::new("device_id")
                        .help("设备ID")
                        .value_name("DEVICE_ID")
                        .required(true)
                )
                .arg(
                    Arg::new("command")
                        .help("要执行的命令")
                        .value_name("COMMAND")
                        .required(true)
                        .action(ArgAction::Append)
                )
        )        
        .subcommand(
            Command::new("install")
                .about("安装模块到设备")
                .arg(
                    Arg::new("device_id")
                        .help("设备ID")
                        .value_name("DEVICE_ID")
                        .required(true)
                )
                .arg(
                    Arg::new("module_path")
                        .help("模块文件路径")
                        .value_name("MODULE_PATH")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("uninstall")
                .about("从设备卸载模块")
                .arg(
                    Arg::new("device_id")
                        .help("设备ID (可选，留空自动选择)")
                        .value_name("DEVICE_ID")
                        .required(false)
                )
                .arg(
                    Arg::new("module_id")
                        .help("模块ID (可选，留空显示已安装模块列表)")
                        .value_name("MODULE_ID")
                        .required(false)
                )
                .arg(
                    Arg::new("force")
                        .help("强制卸载，不进行确认")
                        .long("force")
                        .short('f')
                        .action(ArgAction::SetTrue)
                )
        )
        .subcommand(
            Command::new("push")
                .about("推送文件到设备")
                .arg(
                    Arg::new("device_id")
                        .help("设备ID")
                        .value_name("DEVICE_ID")
                        .required(true)
                )
                .arg(
                    Arg::new("local_path")
                        .help("本地文件路径")
                        .value_name("LOCAL_PATH")
                        .required(true)
                )
                .arg(
                    Arg::new("remote_path")
                        .help("设备上的目标路径")
                        .value_name("REMOTE_PATH")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("pull")
                .about("从设备拉取文件")
                .arg(
                    Arg::new("device_id")
                        .help("设备ID")
                        .value_name("DEVICE_ID")
                        .required(true)
                )
                .arg(
                    Arg::new("remote_path")
                        .help("设备上的文件路径")
                        .value_name("REMOTE_PATH")
                        .required(true)
                )
                .arg(
                    Arg::new("local_path")
                        .help("本地保存路径")
                        .value_name("LOCAL_PATH")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("reboot")
                .about("重启设备")
                .arg(
                    Arg::new("device_id")
                        .help("设备ID")
                        .value_name("DEVICE_ID")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("logs")
                .about("获取设备日志")
                .arg(
                    Arg::new("device_id")
                        .help("设备ID")
                        .value_name("DEVICE_ID")
                        .required(true)
                )
                .arg(
                    Arg::new("filter")
                        .help("日志过滤器")
                        .value_name("FILTER")
                        .short('f')
                        .long("filter")
                )
        )        .subcommand(
            Command::new("check")
                .about("检查模块安装状态")
                .arg(
                    Arg::new("device_id")
                        .help("设备ID")
                        .value_name("DEVICE_ID")
                        .required(true)
                )
                .arg(
                    Arg::new("module_id")
                        .help("模块ID")
                        .value_name("MODULE_ID")
                        .required(true)
                )
        )        .subcommand(
            Command::new("test")
                .about("完整测试模块安装和功能")
                .arg(
                    Arg::new("device_id")
                        .help("设备ID (可选，留空自动选择)")
                        .value_name("DEVICE_ID")
                        .required(false)
                )
                .arg(
                    Arg::new("module_path")
                        .help("模块文件路径 (可选，默认使用当前项目构建的模块)")
                        .value_name("MODULE_PATH")
                        .required(false)
                )
                .arg(
                    Arg::new("download_logs")
                        .help("自动下载日志文件")
                        .long("download-logs")
                        .short('d')
                        .action(ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("interactive")
                        .help("交互式选择模块文件")
                        .long("interactive")
                        .short('i')
                        .action(ArgAction::SetTrue)
                )
        )
}

/// 处理 device 命令
pub fn handle_device(_config: &RmmConfig, matches: &ArgMatches) -> Result<String> {
    // 检查 ADB 是否可用
    if !check_adb_available() {
        println!("❌ ADB 不可用");        println!("💡 请确保:");
        println!("  1. 已安装 Android SDK Platform Tools");
        println!("  2. ADB 已添加到系统 PATH");
        println!("  3. 运行 'adb version' 确认安装");
        return Ok("ADB 不可用".to_string());
    }

    let mut adb = AdbManager::new();
    adb.start_server()?;    match matches.subcommand() {
        Some(("list", _)) => {
            handle_list_devices(&mut adb)?;
            Ok("设备列表获取成功".to_string())
        },
        Some(("info", sub_matches)) => {
            handle_device_info(&mut adb, sub_matches)?;
            Ok("设备信息获取成功".to_string())
        },
        Some(("shell", sub_matches)) => {
            handle_shell_command(&mut adb, sub_matches)?;
            Ok("命令执行成功".to_string())
        },
        Some(("install", sub_matches)) => {
            handle_install_module(&mut adb, sub_matches)?;
            Ok("模块安装成功".to_string())
        },
        Some(("uninstall", sub_matches)) => {
            handle_uninstall_module(&mut adb, sub_matches)?;
            Ok("模块卸载成功".to_string())
        },
        Some(("push", sub_matches)) => {
            handle_push_file(&mut adb, sub_matches)?;
            Ok("文件推送成功".to_string())
        },
        Some(("pull", sub_matches)) => {
            handle_pull_file(&mut adb, sub_matches)?;
            Ok("文件拉取成功".to_string())
        },
        Some(("reboot", sub_matches)) => {
            handle_reboot_device(&mut adb, sub_matches)?;
            Ok("设备重启成功".to_string())
        },
        Some(("logs", sub_matches)) => {
            handle_get_logs(&mut adb, sub_matches)?;
            Ok("日志获取成功".to_string())
        },
        Some(("check", sub_matches)) => {
            handle_check_module(&mut adb, sub_matches)?;
            Ok("模块检查完成".to_string())
        },
        Some(("test", sub_matches)) => {
            handle_test_module(&mut adb, sub_matches)?;
            Ok("模块测试完成".to_string())
        },
        _ => {
            println!("使用 'rmm device --help' 查看可用命令");
            Ok("设备命令执行完成".to_string())
        }
    }
}

/// 列出连接的设备
fn handle_list_devices(adb: &mut AdbManager) -> Result<()> {
    let devices = adb.list_devices()?;
    
    if devices.is_empty() {
        println!("❌ 未发现连接的设备");
        return Ok(());
    }

    println!("\n📱 连接的设备列表:");
    println!("{:<20} {:<15} {:<12} {:<15} {:<10}", "设备ID", "型号", "Android版本", "Root状态", "连接类型");
    println!("{:-<80}", "");
    
    for device in devices {
        let root_status = if device.is_rooted {
            device.root_method.as_deref().unwrap_or("Unknown")
        } else {
            "未Root"
        };
        
        println!("{:<20} {:<15} {:<12} {:<15} {:<10}", 
                 device.id, device.model, device.android_version, root_status, device.connection_type);
    }
    
    Ok(())
}

/// 显示设备详细信息
fn handle_device_info(adb: &mut AdbManager, matches: &ArgMatches) -> Result<()> {
    let device_id = matches.get_one::<String>("device_id").unwrap();
    
    let device = adb.get_device_info(device_id)?;
    
    println!("\n📱 设备详细信息:");
    println!("设备ID: {}", device.id);
    println!("型号: {}", device.model);
    println!("Android版本: {}", device.android_version);
    println!("SDK版本: {}", device.sdk_version);
    println!("Root状态: {}", if device.is_rooted { "已Root" } else { "未Root" });
    if let Some(root_method) = device.root_method {
        println!("Root方法: {}", root_method);
    }
    println!("连接类型: {}", device.connection_type);
    
    Ok(())
}

/// 执行shell命令
fn handle_shell_command(adb: &mut AdbManager, matches: &ArgMatches) -> Result<()> {
    let device_id = matches.get_one::<String>("device_id").unwrap();
    let command: Vec<&String> = matches.get_many::<String>("command").unwrap_or_default().collect();
    
    if command.is_empty() {
        println!("❌ 请提供要执行的命令");
        return Ok(());
    }
    
    let cmd_args: Vec<&str> = command.iter().map(|s| s.as_str()).collect();
    println!("🔧 执行命令: {}", cmd_args.join(" "));
    
    let result = adb.exec_shell(device_id, &cmd_args)?;
    println!("📤 命令输出:");
    println!("{}", result);
    
    Ok(())
}

/// 安装模块
fn handle_install_module(adb: &mut AdbManager, matches: &ArgMatches) -> Result<()> {
    let device_id = matches.get_one::<String>("device_id").unwrap();
    let module_path = matches.get_one::<String>("module_path").unwrap();
    
    let path = Path::new(module_path);
    if !path.exists() {
        println!("❌ 模块文件不存在: {}", module_path);
        return Ok(());
    }
    
    adb.install_module(device_id, path)?;
    
    Ok(())
}

/// 推送文件
fn handle_push_file(adb: &mut AdbManager, matches: &ArgMatches) -> Result<()> {
    let device_id = matches.get_one::<String>("device_id").unwrap();
    let local_path = matches.get_one::<String>("local_path").unwrap();
    let remote_path = matches.get_one::<String>("remote_path").unwrap();
    
    let path = Path::new(local_path);
    if !path.exists() {
        println!("❌ 本地文件不存在: {}", local_path);
        return Ok(());
    }
    
    adb.push_file(device_id, path, remote_path)?;
    
    Ok(())
}

/// 拉取文件
fn handle_pull_file(adb: &mut AdbManager, matches: &ArgMatches) -> Result<()> {
    let device_id = matches.get_one::<String>("device_id").unwrap();
    let remote_path = matches.get_one::<String>("remote_path").unwrap();
    let local_path = matches.get_one::<String>("local_path").unwrap();
    
    let path = Path::new(local_path);
    adb.pull_file(device_id, remote_path, path)?;
    
    Ok(())
}

/// 重启设备
fn handle_reboot_device(adb: &mut AdbManager, matches: &ArgMatches) -> Result<()> {
    let device_id = matches.get_one::<String>("device_id").unwrap();
    
    adb.reboot_device(device_id)?;
    
    Ok(())
}

/// 获取设备日志
fn handle_get_logs(adb: &mut AdbManager, matches: &ArgMatches) -> Result<()> {
    let device_id = matches.get_one::<String>("device_id").unwrap();
    let filter = matches.get_one::<String>("filter");
    
    println!("📋 获取设备日志...");
    let logs = adb.get_device_logs(device_id, filter.map(|s| s.as_str()))?;
    
    println!("📝 设备日志:");
    println!("{}", logs);
    
    Ok(())
}

/// 检查模块状态
fn handle_check_module(adb: &mut AdbManager, matches: &ArgMatches) -> Result<()> {
    let device_id = matches.get_one::<String>("device_id").unwrap();
    let module_id = matches.get_one::<String>("module_id").unwrap();
    
    let is_installed = adb.check_module_status(device_id, module_id)?;
    
    if is_installed {
        println!("✅ 模块 {} 已安装", module_id);
    } else {
        println!("❌ 模块 {} 未安装", module_id);
    }
    
    Ok(())
}

/// 完整测试模块
fn handle_test_module(adb: &mut AdbManager, matches: &ArgMatches) -> Result<()> {
    println!("🧪 开始 RMM 模块完整测试流程...\n");
    
    // 1. 设备选择
    let device_id = if let Some(id) = matches.get_one::<String>("device_id") {
        id.clone()
    } else {
        select_device(adb)?
    };
    
    println!("📱 选择的设备: {}", device_id);
    
    // 2. 检测 Root 管理器
    let root_manager = detect_root_manager(adb, &device_id)?;
    println!("🔑 检测到 Root 管理器: {}", root_manager);
      // 3. 确定模块路径
    let module_path = if let Some(path) = matches.get_one::<String>("module_path") {
        Path::new(path).to_path_buf()
    } else {
        // 根据交互式参数选择模块
        let interactive = matches.get_flag("interactive");
        select_module_zip(interactive)?
    };
    
    println!("📦 模块路径: {}", module_path.display());
    
    // 4. 检查模块是否存在
    if !module_path.exists() {
        println!("❌ 模块文件不存在，正在自动构建...");
        // 自动构建模块
        std::process::Command::new("rmm")
            .arg("build")
            .status()?;
        
        if !module_path.exists() {
            println!("❌ 构建失败，请手动构建模块");
            return Ok(());
        }
    }
    
    // 5. 安装模块
    println!("\n🚀 开始安装模块...");
    install_module_with_manager(adb, &device_id, &module_path, &root_manager)?;
    
    // 6. 获取安装日志
    let log_paths = get_installation_logs(adb, &device_id, &root_manager)?;
    
    // 7. 验证安装
    verify_installation(adb, &device_id, &root_manager)?;
    
    // 8. 询问是否下载日志
    let download_logs = matches.get_flag("download_logs") || ask_download_logs();
    
    if download_logs && !log_paths.is_empty() {
        download_logs_to_local(adb, &device_id, &log_paths)?;
    }
    
    // 9. KernelSU 特殊处理
    if root_manager == "KernelSU" {
        handle_kernelsu_options(adb, &device_id)?;
    }
    
    println!("\n✅ 模块测试流程完成！");
    
    Ok(())
}

/// 选择设备
fn select_device(adb: &mut AdbManager) -> Result<String> {
    use std::io::{self, Write};
    
    let devices = adb.list_devices()?;
    
    if devices.is_empty() {
        return Err(anyhow::anyhow!("❌ 未发现连接的设备"));
    }
    
    if devices.len() == 1 {
        println!("📱 自动选择唯一设备: {}", devices[0].id);
        return Ok(devices[0].id.clone());
    }
    
    println!("📱 发现多个设备，请选择:");
    println!("{:<5} {:<20} {:<15} {:<12} {:<15}", "序号", "设备ID", "型号", "Android版本", "Root状态");
    println!("{:-<80}", "");
    
    for (idx, device) in devices.iter().enumerate() {
        let root_status = if device.is_rooted {
            device.root_method.as_deref().unwrap_or("Unknown")
        } else {
            "未Root"
        };
        
        println!("{:<5} {:<20} {:<15} {:<12} {:<15}", 
                 idx + 1, device.id, device.model, device.android_version, root_status);
    }
    
    print!("\n请输入设备序号 (1-{}): ", devices.len());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let choice: usize = input.trim().parse()
        .map_err(|_| anyhow::anyhow!("❌ 无效的选择"))?;
    
    if choice == 0 || choice > devices.len() {
        return Err(anyhow::anyhow!("❌ 选择超出范围"));
    }
    
    Ok(devices[choice - 1].id.clone())
}

/// 检测 Root 管理器
fn detect_root_manager(adb: &mut AdbManager, device_id: &str) -> Result<String> {
    // 检测 Magisk
    if let Ok(output) = adb.exec_shell(device_id, &["su", "-c", "which magisk"]) {
        if !output.trim().is_empty() && !output.contains("not found") {
            return Ok("Magisk".to_string());
        }
    }
    
    // 检测 KernelSU
    if let Ok(output) = adb.exec_shell(device_id, &["su", "-c", "which ksud"]) {
        if !output.trim().is_empty() && !output.contains("not found") {
            return Ok("KernelSU".to_string());
        }
    }
    
    // 检测 APatch
    if let Ok(output) = adb.exec_shell(device_id, &["su", "-c", "which apd"]) {
        if !output.trim().is_empty() && !output.contains("not found") {
            return Ok("APatch".to_string());
        }
    }
    
    // 检查通用路径
    if let Ok(_) = adb.exec_shell(device_id, &["su", "-c", "test -d /data/adb/magisk"]) {
        return Ok("Magisk".to_string());
    }
    
    if let Ok(_) = adb.exec_shell(device_id, &["su", "-c", "test -d /data/adb/ksu"]) {
        return Ok("KernelSU".to_string());
    }
    
    if let Ok(_) = adb.exec_shell(device_id, &["su", "-c", "test -d /data/adb/ap"]) {
        return Ok("APatch".to_string());
    }
    
    Ok("Unknown".to_string())
}

/// 查找最新的模块 ZIP 文件
fn find_latest_module_zip() -> Result<std::path::PathBuf> {
    select_module_zip(false)
}

/// 选择模块 ZIP 文件
fn select_module_zip(interactive: bool) -> Result<std::path::PathBuf> {
    use std::fs;
    use std::io::{self, Write};
    
    let dist_dir = Path::new(".rmmp/dist");
    if !dist_dir.exists() {
        return Err(anyhow::anyhow!("❌ 构建目录不存在，请先运行 'rmm build'"));
    }
    
    let mut zip_files: Vec<_> = fs::read_dir(dist_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("zip"))
                .unwrap_or(false)
        })
        .collect();
    
    if zip_files.is_empty() {
        return Err(anyhow::anyhow!("❌ 未找到模块 ZIP 文件"));
    }
    
    // 按修改时间排序，最新的在最后
    zip_files.sort_by_key(|entry| {
        entry.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    
    // 如果只有一个文件或非交互式模式，直接返回最新的
    if zip_files.len() == 1 || !interactive {
        let latest = zip_files.last().unwrap().path();
        if !interactive {
            let filename = latest.file_name().unwrap_or_default().to_string_lossy();
            println!("📦 自动选择最新模块: {}", filename);
        }
        return Ok(latest);
    }
    
    // 交互式选择
    println!("📦 发现多个模块文件，请选择:");
    println!("{:<5} {:<30} {:<15} {:<20}", "序号", "文件名", "大小", "修改时间");
    println!("{:-<80}", "");
    
    for (idx, entry) in zip_files.iter().enumerate() {
        let metadata = entry.metadata().unwrap();
        let size = format!("{:.1} KB", metadata.len() as f64 / 1024.0);        let modified = metadata.modified()
            .map(|time| {
                use std::time::UNIX_EPOCH;
                let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
                let secs = duration.as_secs();
                format!("{} ago", humantime::format_duration(std::time::Duration::from_secs(
                    std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() - secs
                )))
            })            .unwrap_or_else(|_| "Unknown".to_string());
        
        let entry_path = entry.path();
        let filename = entry_path.file_name()
            .unwrap_or_default()
            .to_string_lossy();
        
        let marker = if idx == zip_files.len() - 1 { " (最新)" } else { "" };
        
        println!("{:<5} {:<30} {:<15} {:<20}{}", 
                 idx + 1, filename, size, modified, marker);
    }
    
    print!("\n请输入文件序号 (1-{}, 直接回车选择最新): ", zip_files.len());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let choice = if input.trim().is_empty() {
        // 直接回车，选择最新的
        zip_files.len()
    } else {
        input.trim().parse::<usize>()
            .map_err(|_| anyhow::anyhow!("❌ 无效的选择"))?
    };
    
    if choice == 0 || choice > zip_files.len() {
        return Err(anyhow::anyhow!("❌ 选择超出范围"));
    }
    
    Ok(zip_files[choice - 1].path())
}

/// 使用对应的 Root 管理器安装模块
fn install_module_with_manager(adb: &mut AdbManager, device_id: &str, module_path: &Path, root_manager: &str) -> Result<()> {
    // 先推送模块文件
    adb.push_file(device_id, module_path, "/data/local/tmp/test_module.zip")?;
    
    match root_manager {        
        "Magisk" => {
            println!("🎭 使用 Magisk 安装模块");
            let output = adb.exec_shell(device_id, &[
                "su", "-c", "cd /data/local/tmp && magisk --install-module test_module.zip 2>&1"
            ])?;
            println!("📋 安装输出:");
            if output.trim().is_empty() || output.contains("Run this command with root") {
                // 尝试直接使用 su 执行
                let retry_output = adb.exec_shell(device_id, &[
                    "su", "-c", "magisk --install-module /data/local/tmp/test_module.zip"
                ])?;
                println!("{}", retry_output);
            } else {
                println!("{}", output);
            }
        },
        "KernelSU" => {
            println!("🛡️ 使用 KernelSU 安装模块");
            let output = adb.exec_shell(device_id, &[
                "su", "-c", "cd /data/local/tmp && ksud module install test_module.zip"
            ])?;
            println!("📋 安装输出:\n{}", output);
        },
        "APatch" => {
            println!("🔧 使用 APatch 安装模块");
            let output = adb.exec_shell(device_id, &[
                "su", "-c", "cd /data/local/tmp && apd module install test_module.zip"
            ])?;
            println!("📋 安装输出:\n{}", output);
        },
        _ => {
            println!("⚠️ 未知的 Root 管理器，尝试通用安装方法");
            // 通用方法：解压到模块目录
            let output = adb.exec_shell(device_id, &[
                "su", "-c", 
                "cd /data/local/tmp && unzip -o test_module.zip -d /data/adb/modules_update/test_module/"
            ])?;
            println!("📋 安装输出:\n{}", output);
        }
    }
    
    Ok(())
}

/// 获取安装日志路径
fn get_installation_logs(adb: &mut AdbManager, device_id: &str, root_manager: &str) -> Result<Vec<String>> {
    let mut log_paths = Vec::new();
    
    // 1. 首先获取 Magisk 的最新安装日志
    match root_manager {
        "Magisk" => {
            // 尝试获取 Magisk 的实时日志
            if let Ok(magisk_path) = adb.exec_shell(device_id, &["su", "-c", "magisk --path"]) {
                let magisk_path = magisk_path.trim();
                if !magisk_path.is_empty() {
                    // 检查 Magisk 临时日志目录
                    let temp_log_paths = vec![
                        format!("{}/install.log", magisk_path),
                        "/data/local/tmp/magisk_install.log".to_string(),
                        "/tmp/magisk_install.log".to_string(),
                    ];
                    
                    for path in temp_log_paths {
                        if let Ok(_) = adb.exec_shell(device_id, &["su", "-c", &format!("test -f {}", path)]) {
                            log_paths.push(path);
                        }
                    }
                }
            }
            
            // 检查传统 Magisk 日志位置
            let traditional_paths = vec![
                "/cache/magisk.log",
                "/data/adb/magisk.log",
                "/data/adb/magisk_install.log",
            ];
            
            for path in traditional_paths {
                if let Ok(_) = adb.exec_shell(device_id, &["su", "-c", &format!("test -f {}", path)]) {
                    log_paths.push(path.to_string());
                }
            }
            
            // 从 logcat 获取 Magisk 安装日志
            if let Ok(logcat_output) = adb.exec_shell(device_id, &[
                "su", "-c", "logcat -d | grep -i 'magisk.*install\\|module.*install' | tail -50"
            ]) {
                if !logcat_output.trim().is_empty() {
                    // 创建临时文件保存 logcat 输出
                    let _ = adb.exec_shell(device_id, &[
                        "su", "-c", &format!("echo '{}' > /data/local/tmp/magisk_logcat.log", logcat_output.replace("'", "\\'"))
                    ]);
                    log_paths.push("/data/local/tmp/magisk_logcat.log".to_string());
                }
            }
        },
        "KernelSU" => {
            // KernelSU 日志
            let ksu_paths = vec![
                "/data/adb/ksu/log",
                "/data/adb/ksu/install.log",
            ];
            
            for path in ksu_paths {
                if let Ok(_) = adb.exec_shell(device_id, &["su", "-c", &format!("test -f {}", path)]) {
                    log_paths.push(path.to_string());
                }
            }
            
            // 从 logcat 获取 KernelSU 日志
            if let Ok(logcat_output) = adb.exec_shell(device_id, &[
                "su", "-c", "logcat -d | grep -i 'kernelsu\\|ksu.*install' | tail -50"
            ]) {
                if !logcat_output.trim().is_empty() {
                    let _ = adb.exec_shell(device_id, &[
                        "su", "-c", &format!("echo '{}' > /data/local/tmp/ksu_logcat.log", logcat_output.replace("'", "\\'"))
                    ]);
                    log_paths.push("/data/local/tmp/ksu_logcat.log".to_string());
                }
            }
        },
        "APatch" => {
            // APatch 日志
            let ap_paths = vec![
                "/data/adb/ap/log",
                "/data/adb/ap/install.log",
            ];
            
            for path in ap_paths {
                if let Ok(_) = adb.exec_shell(device_id, &["su", "-c", &format!("test -f {}", path)]) {
                    log_paths.push(path.to_string());
                }
            }
        },
        _ => {}
    }
    
    // 2. 检查通用的安装日志（我们自己创建的）
    if let Ok(_) = adb.exec_shell(device_id, &["su", "-c", "test -f /data/local/tmp/rmm_install.log"]) {
        log_paths.push("/data/local/tmp/rmm_install.log".to_string());
    }
    
    // 3. 获取最新的系统日志中与模块安装相关的内容
    if let Ok(system_log) = adb.exec_shell(device_id, &[
        "su", "-c", "dmesg | grep -i 'module\\|install' | tail -20"
    ]) {
        if !system_log.trim().is_empty() {
            let _ = adb.exec_shell(device_id, &[
                "su", "-c", &format!("echo '{}' > /data/local/tmp/system_install.log", system_log.replace("'", "\\'"))
            ]);
            log_paths.push("/data/local/tmp/system_install.log".to_string());
        }
    }
    
    println!("📋 发现 {} 个日志文件: {:?}", log_paths.len(), log_paths);
    
    // 显示日志内容预览
    if !log_paths.is_empty() {
        display_log_preview(adb, device_id, &log_paths)?;
    }
    
    Ok(log_paths)
}

/// 验证安装
fn verify_installation(adb: &mut AdbManager, device_id: &str, root_manager: &str) -> Result<()> {
    println!("\n🔍 验证模块安装状态...");
    
    // 检查模块目录
    let module_dirs = vec![
        "/data/adb/modules/test",
        "/data/adb/modules_update/test",
        "/data/adb/ksu/modules/test",
        "/data/adb/ap/modules/test",
    ];
    
    let mut found = false;
    for dir in module_dirs {
        if let Ok(_) = adb.exec_shell(device_id, &["su", "-c", &format!("test -d {}", dir)]) {
            println!("✅ 找到模块目录: {}", dir);
            
            // 显示模块信息
            if let Ok(prop_content) = adb.exec_shell(device_id, &[
                "su", "-c", &format!("cat {}/module.prop", dir)
            ]) {
                println!("📄 模块属性:\n{}", prop_content);
            }
            
            found = true;
            break;
        }
    }
    
    if !found {
        println!("❌ 未找到已安装的模块");
        return Ok(());
    }
      // Root 管理器特定验证
    match root_manager {        
        "Magisk" => {
            // 显示已安装的 Magisk 模块
            if let Ok(output) = adb.exec_shell(device_id, &["su", "-c", "ls -la /data/adb/modules/"]) {
                println!("🎭 Magisk 已安装模块目录:");
                println!("{}", output);
                
                // 列出具体模块 - 修复命令兼容性
                if let Ok(modules) = adb.exec_shell(device_id, &["su", "-c", "find /data/adb/modules -maxdepth 1 -type d ! -path /data/adb/modules"]) {
                    if !modules.trim().is_empty() {
                        println!("📋 已安装的模块:");
                        for module_path in modules.lines() {
                            if !module_path.trim().is_empty() {
                                let module_name = std::path::Path::new(module_path.trim())
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown");
                                if module_name != "modules" {
                                    println!("  - {}", module_name);
                                }
                            }
                        }
                    }
                } else {
                    // 备用方法
                    if let Ok(simple_list) = adb.exec_shell(device_id, &["su", "-c", "ls /data/adb/modules/"]) {
                        println!("📋 已安装的模块:");
                        for module in simple_list.lines() {
                            let module = module.trim();
                            if !module.is_empty() && module != "." && module != ".." {
                                println!("  - {}", module);
                            }
                        }
                    }
                }
            }
        },
        "KernelSU" => {
            if let Ok(output) = adb.exec_shell(device_id, &["su", "-c", "ksud module list"]) {
                println!("🛡️ KernelSU 模块列表:\n{}", output);
            }
        },
        "APatch" => {
            if let Ok(output) = adb.exec_shell(device_id, &["su", "-c", "apd module list"]) {
                println!("🔧 APatch 模块列表:\n{}", output);
            }
        },
        _ => {}
    }
    
    println!("✅ 模块验证完成");
    Ok(())
}

/// 询问是否下载日志
fn ask_download_logs() -> bool {
    use std::io::{self, Write};
    
    print!("📥 是否下载安装日志到本地? (y/N): ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes" | "是")
}

/// 下载日志到本地
fn download_logs_to_local(adb: &mut AdbManager, device_id: &str, log_paths: &[String]) -> Result<()> {
    use std::fs;
    
    let logs_dir = Path::new("logs");
    fs::create_dir_all(logs_dir)?;
    
    println!("📥 正在下载日志文件...");
    
    for log_path in log_paths {
        let filename = Path::new(log_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        
        let local_path = logs_dir.join(&*filename);
        
        match adb.pull_file(device_id, log_path, &local_path) {
            Ok(_) => println!("✅ 已下载: {} -> {}", log_path, local_path.display()),
            Err(e) => println!("❌ 下载失败 {}: {}", log_path, e),
        }
    }
    
    println!("📁 日志文件保存在: {}", logs_dir.display());
    Ok(())
}

/// 处理 KernelSU 特殊选项
fn handle_kernelsu_options(adb: &mut AdbManager, device_id: &str) -> Result<()> {
    use std::io::{self, Write};
    
    println!("\n🛡️ KernelSU 特殊选项:");
    println!("1. 重启设备");
    println!("2. 发送模拟开机事件 (ksud trigger)");
    println!("3. 跳过");
    
    print!("请选择 (1-3): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    match input.trim() {
        "1" => {
            println!("🔄 正在重启设备...");
            adb.reboot_device(device_id)?;
            println!("✅ 重启命令已发送");
                },
        "2" => {
            println!("🚀 发送模拟开机事件...");
            let output = adb.exec_shell(device_id, &[
                "su", "-c", "ksud trigger post-fs-data && ksud trigger service && ksud trigger boot-complete"
            ])?;
            println!("📋 触发输出:\n{}", output);
            println!("✅ 模拟开机事件已发送");
        },
        "3" | _ => {
            println!("⏭️ 跳过特殊选项");
        }
    }
    
    Ok(())
}

/// 显示日志内容预览
fn display_log_preview(adb: &mut AdbManager, device_id: &str, log_paths: &[String]) -> Result<()> {
    use std::io::{self, Write};
    
    println!("\n📋 安装日志预览:");
    
    for (idx, log_path) in log_paths.iter().enumerate() {
        println!("\n{}. {} :", idx + 1, log_path);
        println!("{:-<60}", "");
        
        if let Ok(content) = adb.exec_shell(device_id, &["su", "-c", &format!("cat {}", log_path)]) {
            if content.trim().is_empty() {
                println!("(空文件)");
                continue;
            }
            
            let lines: Vec<&str> = content.lines().collect();
            let total_lines = lines.len();
            
            if total_lines > 20 {
                println!("📝 日志文件共 {} 行", total_lines);
                print!("显示选项: (a)全部 / (l)最后20行 / (f)前20行 / (s)跳过 [默认:l]: ");
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                
                match input.trim().to_lowercase().as_str() {
                    "a" | "all" | "全部" => {
                        for line in &lines {
                            println!("{}", line);
                        }
                    },
                    "f" | "first" | "前" => {
                        println!("... (显示前20行，共{}行) ...", total_lines);
                        for line in &lines[..20.min(total_lines)] {
                            println!("{}", line);
                        }
                    },
                    "s" | "skip" | "跳过" => {
                        println!("⏭️ 跳过显示");
                        continue;
                    },
                    _ => { // 默认显示最后20行
                        println!("... (显示最后20行，共{}行) ...", total_lines);
                        let start_idx = if total_lines > 20 { total_lines - 20 } else { 0 };
                        for line in &lines[start_idx..] {
                            println!("{}", line);
                        }
                    }
                }
            } else {
                // 少于20行直接全部显示
                for line in &lines {
                    println!("{}", line);
                }
            }        } else {
            println!("❌ 无法读取日志文件");
        }
    }
    
    Ok(())
}

/// 卸载模块
fn handle_uninstall_module(adb: &mut AdbManager, matches: &ArgMatches) -> Result<()> {
    use std::io::{self, Write};
    
    println!("🗑️ 开始模块卸载流程...\n");
    
    // 1. 设备选择
    let device_id = if let Some(id) = matches.get_one::<String>("device_id") {
        id.clone()
    } else {
        select_device(adb)?
    };
    
    println!("📱 选择的设备: {}", device_id);
    
    // 2. 检测 Root 管理器
    let root_manager = detect_root_manager(adb, &device_id)?;
    println!("🔑 检测到 Root 管理器: {}", root_manager);
    
    // 3. 获取已安装的模块列表
    let installed_modules = get_installed_modules(adb, &device_id, &root_manager)?;
    
    if installed_modules.is_empty() {
        println!("📋 未发现已安装的模块");
        return Ok(());
    }
    
    // 4. 选择要卸载的模块
    let module_id = if let Some(id) = matches.get_one::<String>("module_id") {
        if installed_modules.contains(&id.to_string()) {
            id.clone()
        } else {
            println!("❌ 模块 '{}' 未找到", id);
            println!("📋 已安装的模块: {:?}", installed_modules);
            return Ok(());
        }
    } else {
        // 显示模块列表供用户选择
        select_module_to_uninstall(&installed_modules)?
    };
    
    println!("🎯 准备卸载模块: {}", module_id);
    
    // 5. 确认卸载
    let force = matches.get_flag("force");
    if !force {
        print!("⚠️  确定要卸载模块 '{}' 吗? (y/N): ", module_id);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes" | "是") {
            println!("❌ 已取消卸载");
            return Ok(());
        }
    }
    
    // 6. 执行卸载
    uninstall_module_with_manager(adb, &device_id, &module_id, &root_manager)?;
    
    // 7. 验证卸载结果
    verify_uninstall(adb, &device_id, &module_id, &root_manager)?;
    
    println!("\n✅ 模块卸载流程完成！");
    
    Ok(())
}

/// 获取已安装的模块列表
fn get_installed_modules(adb: &mut AdbManager, device_id: &str, root_manager: &str) -> Result<Vec<String>> {
    let mut modules = Vec::new();
    
    match root_manager {
        "Magisk" => {
            // 检查 /data/adb/modules 目录
            if let Ok(output) = adb.exec_shell(device_id, &["su", "-c", "ls /data/adb/modules/"]) {
                for module in output.lines() {
                    let module = module.trim();
                    if !module.is_empty() && module != "." && module != ".." && !module.starts_with("lost+found") {
                        modules.push(module.to_string());
                    }
                }
            }
        },
        "KernelSU" => {
            // 使用 ksud 命令获取模块列表
            if let Ok(output) = adb.exec_shell(device_id, &["su", "-c", "ksud module list"]) {
                for line in output.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.contains("No modules") {
                        // 提取模块ID（可能需要根据 ksud 输出格式调整）
                        if let Some(module_id) = line.split_whitespace().next() {
                            modules.push(module_id.to_string());
                        }
                    }
                }
            }
            
            // 备用方法：检查 KernelSU 模块目录
            if modules.is_empty() {
                if let Ok(output) = adb.exec_shell(device_id, &["su", "-c", "ls /data/adb/ksu/modules/ 2>/dev/null || true"]) {
                    for module in output.lines() {
                        let module = module.trim();
                        if !module.is_empty() && module != "." && module != ".." {
                            modules.push(module.to_string());
                        }
                    }
                }
            }
        },
        "APatch" => {
            // 使用 apd 命令获取模块列表
            if let Ok(output) = adb.exec_shell(device_id, &["su", "-c", "apd module list"]) {
                for line in output.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.contains("No modules") {
                        if let Some(module_id) = line.split_whitespace().next() {
                            modules.push(module_id.to_string());
                        }
                    }
                }
            }
            
            // 备用方法：检查 APatch 模块目录
            if modules.is_empty() {
                if let Ok(output) = adb.exec_shell(device_id, &["su", "-c", "ls /data/adb/ap/modules/ 2>/dev/null || true"]) {
                    for module in output.lines() {
                        let module = module.trim();
                        if !module.is_empty() && module != "." && module != ".." {
                            modules.push(module.to_string());
                        }
                    }
                }
            }
        },
        _ => {
            // 通用方法：检查所有可能的模块目录
            let dirs = vec![
                "/data/adb/modules/",
            ];
            
            for dir in dirs {
                if let Ok(output) = adb.exec_shell(device_id, &["su", "-c", &format!("ls {} 2>/dev/null || true", dir)]) {
                    for module in output.lines() {
                        let module = module.trim();
                        if !module.is_empty() && module != "." && module != ".." && !modules.contains(&module.to_string()) {
                            modules.push(module.to_string());
                        }
                    }
                }
            }
        }
    }
    
    println!("📋 发现 {} 个已安装的模块: {:?}", modules.len(), modules);
    Ok(modules)
}

/// 选择要卸载的模块
fn select_module_to_uninstall(modules: &[String]) -> Result<String> {
    use std::io::{self, Write};
    
    println!("📋 已安装的模块列表:");
    println!("{:<5} {:<20}", "序号", "模块ID");
    println!("{:-<30}", "");
    
    for (idx, module) in modules.iter().enumerate() {
        println!("{:<5} {:<20}", idx + 1, module);
    }
    
    print!("\n请输入要卸载的模块序号 (1-{}): ", modules.len());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let choice: usize = input.trim().parse()
        .map_err(|_| anyhow::anyhow!("❌ 无效的选择"))?;
    
    if choice == 0 || choice > modules.len() {
        return Err(anyhow::anyhow!("❌ 选择超出范围"));
    }
    
    Ok(modules[choice - 1].clone())
}

/// 使用对应的 Root 管理器卸载模块
fn uninstall_module_with_manager(adb: &mut AdbManager, device_id: &str, module_id: &str, root_manager: &str) -> Result<()> {
    println!("\n🗑️ 开始卸载模块: {}", module_id);
    
    match root_manager {
        "Magisk" => {
            println!("🎭 使用 Magisk 卸载模块");
            
            // Magisk 卸载方法：删除模块目录或创建 remove 文件
            let output = adb.exec_shell(device_id, &[
                "su", "-c", &format!("touch /data/adb/modules/{}/remove", module_id)
            ])?;
            
            if output.contains("No such file or directory") {
                println!("❌ 模块目录不存在");
                return Ok(());
            }
            
            println!("📋 卸载输出: 已标记模块为删除状态");
            println!("⚠️  需要重启设备才能完全卸载模块");
            
            // 也可以尝试直接删除（立即生效）
            let _ = adb.exec_shell(device_id, &[
                "su", "-c", &format!("rm -rf /data/adb/modules/{}", module_id)
            ]);
        },
        "KernelSU" => {
            println!("🛡️ 使用 KernelSU 卸载模块");
            let output = adb.exec_shell(device_id, &[
                "su", "-c", &format!("ksud module uninstall {}", module_id)
            ])?;
            println!("📋 卸载输出:\n{}", output);
            
            // 如果 ksud 命令不存在，尝试手动删除
            if output.contains("not found") || output.contains("No such file") {
                let _ = adb.exec_shell(device_id, &[
                    "su", "-c", &format!("rm -rf /data/adb/ksu/modules/{}", module_id)
                ]);
                println!("📋 已手动删除模块目录");
            }
        },
        "APatch" => {
            println!("🔧 使用 APatch 卸载模块");
            let output = adb.exec_shell(device_id, &[
                "su", "-c", &format!("apd module uninstall {}", module_id)
            ])?;
            println!("📋 卸载输出:\n{}", output);
            
            // 如果 apd 命令不存在，尝试手动删除
            if output.contains("not found") || output.contains("No such file") {
                let _ = adb.exec_shell(device_id, &[
                    "su", "-c", &format!("rm -rf /data/adb/ap/modules/{}", module_id)
                ]);
                println!("📋 已手动删除模块目录");
            }
        },
        _ => {
            println!("⚠️ 未知的 Root 管理器，尝试通用卸载方法");
            // 通用方法：直接删除模块目录
            let dirs = vec![
                format!("/data/adb/modules/{}", module_id),
                format!("/data/adb/ksu/modules/{}", module_id),
                format!("/data/adb/ap/modules/{}", module_id),
            ];
            
            for dir in dirs {
                let output = adb.exec_shell(device_id, &[
                    "su", "-c", &format!("rm -rf {}", dir)
                ])?;
                if !output.trim().is_empty() {
                    println!("📋 删除 {}: {}", dir, output);
                }
            }
        }
    }
    
    Ok(())
}

/// 验证卸载结果
fn verify_uninstall(adb: &mut AdbManager, device_id: &str, module_id: &str, root_manager: &str) -> Result<()> {
    println!("\n🔍 验证模块卸载状态...");
    
    let mut found = false;
    
    // 检查模块目录是否仍然存在
    let module_dirs = vec![
        format!("/data/adb/modules/{}", module_id),
        format!("/data/adb/modules_update/{}", module_id),
        format!("/data/adb/ksu/modules/{}", module_id),
        format!("/data/adb/ap/modules/{}", module_id),
    ];
    
    for dir in module_dirs {
        if let Ok(_) = adb.exec_shell(device_id, &["su", "-c", &format!("test -d {}", dir)]) {
            // 检查是否有 remove 标记
            if let Ok(_) = adb.exec_shell(device_id, &["su", "-c", &format!("test -f {}/remove", dir)]) {
                println!("⚠️  模块目录仍存在但已标记删除: {}", dir);
                println!("🔄 需要重启设备以完成卸载");
                found = true;
            } else {
                println!("❌ 模块目录仍然存在: {}", dir);
                found = true;
            }
        }
    }
    
    if !found {
        println!("✅ 模块 '{}' 已成功卸载", module_id);
        
        // 验证模块确实从列表中消失
        let remaining_modules = get_installed_modules(adb, device_id, root_manager)?;
        if !remaining_modules.contains(&module_id.to_string()) {
            println!("✅ 模块已从已安装列表中移除");
        }
    } else {
        println!("⚠️  模块可能需要重启后才能完全卸载");
    }
    
    Ok(())
}
