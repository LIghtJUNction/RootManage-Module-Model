/// 选择设备
use anyhow::Result;
use crate::commands::utils::core::adb::AdbManager;
use std::path::Path;

pub fn select_device(adb: &mut AdbManager) -> Result<String> {
    use std::io::{self, Write};
    
    let devices = adb.list_devices()?;
    
    if devices.is_empty() {
        return Err(anyhow::anyhow!("❌ 未发现连接的设备"));
    }
    
    if devices.len() == 1 {
        println!("📱 自动选择唯一设备: {}", devices[0]);
        return Ok(devices[0].clone());
    }
    
    println!("📱 发现多个设备，请选择:");
    println!("{:<5} {:<20} {:<15}", "序号", "设备ID", "状态");
    println!("{:-<50}", "");
    
    for (idx, device) in devices.iter().enumerate() {
        println!("{:<5} {:<20} {:<15}", idx + 1, device, "连接");
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
    
    Ok(devices[choice - 1].clone())
}

/// 检测 Root 管理器
pub fn detect_root_manager(adb: &mut AdbManager, device_id: &str) -> Result<String> {
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

/// 选择模块 ZIP 文件
pub fn select_module_zip(interactive: bool) -> Result<std::path::PathBuf> {
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
        let size = format!("{:.1} KB", metadata.len() as f64 / 1024.0);
        let modified = metadata.modified()
            .map(|time| {
                use std::time::UNIX_EPOCH;
                let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
                let secs = duration.as_secs();
                format!("{} ago", humantime::format_duration(std::time::Duration::from_secs(
                    std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() - secs
                )))
            })
            .unwrap_or_else(|_| "Unknown".to_string());
        
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
pub fn install_module_with_manager(adb: &mut AdbManager, device_id: &str, module_path: &Path, root_manager: &str) -> Result<()> {
    // 先推送模块文件
    adb.push_file(device_id, &module_path.to_string_lossy(), "/data/local/tmp/test_module.zip")?;
    
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
pub fn get_installation_logs(adb: &mut AdbManager, device_id: &str, root_manager: &str) -> Result<Vec<String>> {
    let mut log_paths = Vec::new();
    
    // 1. 首先获取各 Root 管理器的安装日志
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
pub fn verify_installation(adb: &mut AdbManager, device_id: &str, root_manager: &str) -> Result<()> {
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
                
                // 列出具体模块
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
        },
        "KernelSU" => {
            if let Ok(output) = adb.exec_shell(device_id, &["su", "-c", "ksud module list"]) {
                println!("🛡️ KernelSU 模块列表:\n{}", output);
            }
        },
        "APatch" => {
            if let Ok(output) = adb.exec_shell(device_id, &["su", "-c", "ls -la /data/adb/ap/modules/"]) {
                println!("🔧 APatch 模块目录:\n{}", output);
            }
        },
        _ => {}
    }
    
    println!("✅ 模块验证完成");
    Ok(())
}

/// 询问是否下载日志
pub fn ask_download_logs() -> bool {
    use std::io::{self, Write};
    
    print!("📥 是否下载安装日志到本地? (y/N): ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes" | "是")
}

/// 下载日志到本地
pub fn download_logs_to_local(adb: &mut AdbManager, device_id: &str, log_paths: &[String]) -> Result<()> {
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
        
        match adb.pull_file(device_id, log_path, &local_path.to_string_lossy()) {
            Ok(_) => println!("✅ 下载成功: {}", filename),
            Err(e) => println!("❌ 下载失败 {}: {}", filename, e),
        }
    }
    
    println!("📁 日志文件保存在: {}", logs_dir.display());
    Ok(())
}

/// 获取已安装的模块列表
pub fn get_installed_modules(adb: &mut AdbManager, device_id: &str, root_manager: &str) -> Result<Vec<String>> {
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
pub fn select_module_to_uninstall(modules: &[String]) -> Result<String> {
    use std::io::{self, Write};
    
    println!("📋 已安装的模块列表:");
    println!("{:<5} {:<30}", "序号", "模块ID");
    println!("{:-<40}", "");
    
    for (idx, module) in modules.iter().enumerate() {
        println!("{:<5} {:<30}", idx + 1, module);
    }
    
    print!("\n请输入模块序号 (1-{}): ", modules.len());
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
pub fn uninstall_module_with_manager(adb: &mut AdbManager, device_id: &str, module_id: &str, root_manager: &str) -> Result<()> {
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
pub fn verify_uninstall(adb: &mut AdbManager, device_id: &str, module_id: &str, root_manager: &str) -> Result<()> {
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

/// 显示日志内容预览
pub fn display_log_preview(adb: &mut AdbManager, device_id: &str, log_paths: &[String]) -> Result<()> {
    use std::io::{self, Write};
    
    println!("\n📋 安装日志预览:");
    
    for (idx, log_path) in log_paths.iter().enumerate() {
        println!("\n--- 日志文件 {} ({}) ---", idx + 1, log_path);
        
        if let Ok(content) = adb.exec_shell(device_id, &["su", "-c", &format!("tail -10 {}", log_path)]) {
            if content.trim().is_empty() {
                println!("(文件为空或无法读取)");
            } else {
                println!("{}", content);
            }
        } else {
            println!("(无法读取文件)");
        }
        
        if idx < log_paths.len() - 1 {
            print!("按回车查看下一个日志文件...");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
        }
    }
    
    Ok(())
}

