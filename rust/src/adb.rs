use anyhow::{Result, anyhow};
use adb_client::{ADBServer, ADBDeviceExt};
use std::path::Path;
use std::fs::File;
use std::process::Command;

/// 设备信息结构
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub model: String,
    pub android_version: String,
    pub sdk_version: u32,
    pub is_rooted: bool,
    pub root_method: Option<String>,
    pub connection_type: String,
}

/// ADB 管理器
pub struct AdbManager {
    server: ADBServer,
}

impl AdbManager {
    /// 创建新的 ADB 管理器
    pub fn new() -> Self {
        let server = ADBServer::default();
        Self { server }
    }

    /// 启动 ADB 服务器
    pub fn start_server(&mut self) -> Result<()> {
        // ADB服务器会自动启动，这里只是确保连接正常
        self.list_devices()?;
        Ok(())
    }

    /// 列出所有连接的设备
    pub fn list_devices(&mut self) -> Result<Vec<DeviceInfo>> {
        let devices = self.server.devices()
            .map_err(|e| anyhow!("获取设备列表失败: {}", e))?;

        let mut device_list = Vec::new();
          for device_info in devices {
            let serial = device_info.identifier;
            
            // 获取设备详细信息
            if let Ok(device_detail) = self.get_device_info(&serial) {
                device_list.push(device_detail);
            } else {
                // 如果无法获取详细信息，使用基本信息
                device_list.push(DeviceInfo {
                    id: serial.to_string(),
                    model: "Unknown".to_string(),
                    android_version: "Unknown".to_string(),
                    sdk_version: 0,
                    is_rooted: false,
                    root_method: None,
                    connection_type: device_info.state.to_string(),
                });
            }
        }

        Ok(device_list)
    }    /// 获取设备详细信息
    pub fn get_device_info(&mut self, device_id: &str) -> Result<DeviceInfo> {
        // 获取设备属性
        let model = self.exec_shell(device_id, &["getprop", "ro.product.model"])?
            .trim().to_string();
        let android_version = self.exec_shell(device_id, &["getprop", "ro.build.version.release"])?
            .trim().to_string();
        let sdk_version_str = self.exec_shell(device_id, &["getprop", "ro.build.version.sdk"])?
            .trim().to_string();
        let sdk_version = sdk_version_str.parse::<u32>().unwrap_or(0);

        // 检查 Root 状态
        let (is_rooted, root_method) = self.check_root_status(device_id)?;

        Ok(DeviceInfo {
            id: device_id.to_string(),
            model,
            android_version,
            sdk_version,
            is_rooted,
            root_method,
            connection_type: "device".to_string(),
        })
    }

    /// 执行 shell 命令
    pub fn exec_shell(&mut self, device_id: &str, command: &[&str]) -> Result<String> {
        let mut device = self.server.get_device_by_name(device_id)
            .map_err(|e| anyhow!("无法连接到设备 {}: {}", device_id, e))?;

        let mut output = Vec::new();
        device.shell_command(command, &mut output)
            .map_err(|e| anyhow!("执行命令失败: {}", e))?;

        Ok(String::from_utf8_lossy(&output).to_string())
    }

    /// 推送文件到设备
    pub fn push_file(&mut self, device_id: &str, local_path: &Path, remote_path: &str) -> Result<()> {
        let mut device = self.server.get_device_by_name(device_id)
            .map_err(|e| anyhow!("无法连接到设备 {}: {}", device_id, e))?;

        let mut file = File::open(local_path)
            .map_err(|e| anyhow!("无法打开本地文件: {}", e))?;

        println!("📤 推送文件: {} -> {}", local_path.display(), remote_path);
        device.push(&mut file, remote_path)
            .map_err(|e| anyhow!("推送文件失败: {}", e))?;

        println!("✅ 文件推送成功");
        Ok(())
    }

    /// 从设备拉取文件
    pub fn pull_file(&mut self, device_id: &str, remote_path: &str, local_path: &Path) -> Result<()> {
        let mut device = self.server.get_device_by_name(device_id)
            .map_err(|e| anyhow!("无法连接到设备 {}: {}", device_id, e))?;

        println!("📥 拉取文件: {} -> {}", remote_path, local_path.display());
        
        // 创建本地目录
        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow!("创建本地目录失败: {}", e))?;
        }

        let mut file = File::create(local_path)
            .map_err(|e| anyhow!("无法创建本地文件: {}", e))?;

        device.pull(&remote_path, &mut file)
            .map_err(|e| anyhow!("拉取文件失败: {}", e))?;

        println!("✅ 文件拉取成功");
        Ok(())
    }

    /// 重启设备
    pub fn reboot_device(&mut self, device_id: &str) -> Result<()> {
        println!("🔄 重启设备: {}", device_id);
        self.exec_shell(device_id, &["reboot"])?;
        println!("✅ 重启命令已发送");
        Ok(())
    }

    /// 检查 Root 状态和方法
    fn check_root_status(&mut self, device_id: &str) -> Result<(bool, Option<String>)> {
        // 检查 su 命令
        if let Ok(output) = self.exec_shell(device_id, &["su", "-c", "id"]) {
            if output.contains("uid=0") {
                // 进一步检查 Root 方法
                let root_method = self.detect_root_method(device_id)?;
                return Ok((true, Some(root_method)));
            }
        }

        // 检查 which su
        if let Ok(output) = self.exec_shell(device_id, &["which", "su"]) {
            if !output.trim().is_empty() {
                return Ok((true, Some("Unknown".to_string())));
            }
        }

        Ok((false, None))
    }

    /// 检测 Root 方法
    fn detect_root_method(&mut self, device_id: &str) -> Result<String> {
        // 检查 Magisk
        if self.exec_shell(device_id, &["which", "magisk"]).is_ok() {
            return Ok("Magisk".to_string());
        }

        // 检查 KernelSU
        if self.exec_shell(device_id, &["ls", "/data/adb/ksud"]).is_ok() {
            return Ok("KernelSU".to_string());
        }

        // 检查 APatch
        if self.exec_shell(device_id, &["ls", "/data/adb/ap"]).is_ok() {
            return Ok("APatch".to_string());
        }

        // 检查 SuperSU
        if self.exec_shell(device_id, &["which", "daemonsu"]).is_ok() {
            return Ok("SuperSU".to_string());
        }

        Ok("Unknown".to_string())
    }

    /// 获取设备日志
    pub fn get_device_logs(&mut self, device_id: &str, filter: Option<&str>) -> Result<String> {
        let command = if let Some(filter) = filter {
            vec!["logcat", "-d", "-s", filter]
        } else {
            vec!["logcat", "-d", "-t", "100"]
        };

        self.exec_shell(device_id, &command)
    }

    /// 安装模块
    pub fn install_module(&mut self, device_id: &str, module_path: &Path) -> Result<()> {
        println!("📦 安装模块: {}", module_path.display());

        // 获取设备信息以确定 Root 方法
        let device_info = self.get_device_info(device_id)?;
        
        if !device_info.is_rooted {
            return Err(anyhow!("设备未Root，无法安装模块"));
        }

        let remote_path = "/data/local/tmp/module.zip";
        
        // 推送模块文件
        self.push_file(device_id, module_path, remote_path)?;

        // 根据 Root 方法执行安装
        match device_info.root_method.as_deref() {
            Some("Magisk") => self.install_magisk_module(device_id, remote_path),
            Some("KernelSU") => self.install_kernelsu_module(device_id, remote_path),
            Some("APatch") => self.install_apatch_module(device_id, remote_path),
            _ => {
                println!("⚠️  未知的Root方法，尝试通用安装");
                self.install_generic_module(device_id, remote_path)
            }
        }
    }

    /// 安装 Magisk 模块
    fn install_magisk_module(&mut self, device_id: &str, module_path: &str) -> Result<()> {
        println!("🎭 使用 Magisk 安装模块");
        
        // 使用 Magisk 命令安装
        let output = self.exec_shell(device_id, &["su", "-c", &format!("magisk --install-module {}", module_path)])?;
        
        if output.contains("Success") || output.contains("installed") {
            println!("✅ Magisk 模块安装成功");
        } else {
            println!("ℹ️  安装输出: {}", output);
        }
        
        Ok(())
    }    /// 安装 KernelSU 模块  
    fn install_kernelsu_module(&mut self, device_id: &str, module_path: &str) -> Result<()> {
        println!("🔧 使用 KernelSU 安装模块");
        
        // 使用 KernelSU 的 ksud 命令安装模块
        let output = self.exec_shell(device_id, &["su", "-c", &format!("ksud module install {}", module_path)])?;
        
        if output.contains("Success") || output.contains("installed") || output.contains("done") {
            println!("✅ KernelSU 模块安装成功");
        } else {
            println!("ℹ️  安装输出: {}", output);
        }
        
        Ok(())
    }

    /// 安装 APatch 模块
    fn install_apatch_module(&mut self, device_id: &str, module_path: &str) -> Result<()> {
        println!("🔨 使用 APatch 安装模块");
        
        // 使用 APatch 的 apd 命令安装模块
        let output = self.exec_shell(device_id, &["su", "-c", &format!("apd module install {}", module_path)])?;
        
        if output.contains("Success") || output.contains("installed") || output.contains("done") {
            println!("✅ APatch 模块安装成功");
        } else {
            println!("ℹ️  安装输出: {}", output);
        }
        
        Ok(())
    }

    /// 通用模块安装
    fn install_generic_module(&mut self, device_id: &str, module_path: &str) -> Result<()> {
        println!("📋 使用通用方法安装模块");
        
        let modules_dir = "/data/adb/modules";
        let output = self.exec_shell(device_id, &["su", "-c", &format!("mkdir -p {} && cd {} && unzip {}", modules_dir, modules_dir, module_path)])?;
        
        println!("ℹ️  安装输出: {}", output);
        println!("✅ 模块安装完成");
        
        Ok(())
    }    /// 检查模块状态
    pub fn check_module_status(&mut self, device_id: &str, module_id: &str) -> Result<bool> {
        let module_path = format!("/data/adb/modules/{}", module_id);
        let output = self.exec_shell(device_id, &["su", "-c", &format!("test -d {} && echo exists", module_path)])?;
        
        Ok(output.trim() == "exists")
    }
}

/// 检查 ADB 是否可用
pub fn check_adb_available() -> bool {
    Command::new("adb")
        .arg("version")
        .output()
        .is_ok()
}