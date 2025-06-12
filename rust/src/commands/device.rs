use anyhow::Result;
use clap::{Arg, ArgAction, ArgMatches, Command};
use crate::commands::utils::core::config::RmmConfig;
use crate::commands::utils::core::adb::AdbManager;
use crate::commands::utils::core::executor::DeviceManager;
use crate::commands::utils::device_utils::*;
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
        )
        .subcommand(
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
        )
        .subcommand(
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
    if !DeviceManager::check_adb_available() {
        println!("❌ ADB 不可用");
        println!("💡 请确保:");
        println!("  1. 已安装 Android SDK Platform Tools");
        println!("  2. ADB 已添加到系统 PATH");
        println!("  3. 运行 'adb version' 确认安装");
        return Ok("ADB 不可用".to_string());
    }

    let mut adb = AdbManager::new();
    adb.start_server()?;

    match matches.subcommand() {
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
            let device_id = sub_matches.get_one::<String>("device_id").unwrap();
            let module_path_str = sub_matches.get_one::<String>("module_path").unwrap();
            let module_path = Path::new(module_path_str);
            
            println!("📱 安装模块到设备: {}", device_id);
            match DeviceManager::install_module_to_device(device_id, module_path) {
                Ok(result) => {
                    println!("{}", result);
                    Ok("模块安装成功".to_string())
                }
                Err(e) => {
                    println!("❌ 安装失败: {}", e);
                    Err(e)
                }
            }
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


/// 处理 KernelSU 特殊选项
pub fn handle_kernelsu_options(adb: &mut AdbManager, device_id: &str) -> Result<()> {
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
            println!("🔄 重启设备...");
            adb.reboot_device(device_id)?;
        },
        "2" => {
            println!("⚡ 发送模拟开机事件...");
            let output = adb.exec_shell(device_id, &["su", "-c", "ksud trigger post-fs-data"])?;
            println!("📋 输出: {}", output);
        },
        _ => {
            println!("⏭️ 跳过特殊选项");
        }
    }
    
    Ok(())
}

/// 列出连接的设备
fn handle_list_devices(adb: &mut AdbManager) -> Result<()> {
    let devices = adb.list_devices()?;
    
    if devices.is_empty() {
        println!("❌ 未发现连接的设备");
        return Ok(());
    }

    println!("\n📱 连接的设备列表:");
    println!("{:<20} {:<15}", "设备ID", "状态");
    println!("{:-<40}", "");
    
    for device in devices {
        println!("{:<20} {:<15}", device, "连接");
    }
    
    Ok(())
}

/// 显示设备详细信息
fn handle_device_info(adb: &mut AdbManager, matches: &ArgMatches) -> Result<()> {
    let device_id = matches.get_one::<String>("device_id").unwrap();
    
    // 简单显示设备信息，因为AdbManager返回的是基本字符串列表
    println!("\n📱 设备信息:");
    println!("设备ID: {}", device_id);
    
    // 尝试获取更多信息通过shell命令
    match adb.shell(device_id, "getprop ro.product.model") {
        Ok(model) => println!("型号: {}", model.trim()),
        Err(_) => println!("型号: 无法获取"),
    }
    
    match adb.shell(device_id, "getprop ro.build.version.release") {
        Ok(version) => println!("Android版本: {}", version.trim()),
        Err(_) => println!("Android版本: 无法获取"),
    }
    
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
    
    adb.install_module(device_id, module_path)?;
    
    Ok(())
}

/// 推送文件
fn handle_push_file(adb: &mut AdbManager, matches: &ArgMatches) -> Result<()> {
    let device_id = matches.get_one::<String>("device_id").unwrap();
    let local_path = matches.get_one::<String>("local_path").unwrap();
    let remote_path = matches.get_one::<String>("remote_path").unwrap();
    
    adb.push_file(device_id, local_path, remote_path)?;
    
    Ok(())
}

/// 拉取文件
fn handle_pull_file(adb: &mut AdbManager, matches: &ArgMatches) -> Result<()> {
    let device_id = matches.get_one::<String>("device_id").unwrap();
    let remote_path = matches.get_one::<String>("remote_path").unwrap();
    let local_path = matches.get_one::<String>("local_path").unwrap();
    
    adb.pull_file(device_id, remote_path, local_path)?;
    
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
    
    let logs = adb.get_device_logs(device_id, filter.map(|s| s.as_str()))?;
    println!("📋 设备日志:");
    for log_line in logs {
        println!("{}", log_line);
    }
    
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
        println!("✅ 模块构建成功");
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
