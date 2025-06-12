use anyhow::Result;
use std::process::Command;

/// ADB 管理器
pub struct AdbManager {
    pub adb_path: String,
}

impl AdbManager {
    /// 创建新的 ADB 管理器
    pub fn new() -> Self {
        Self {
            adb_path: "adb".to_string(),
        }
    }
    
    /// 获取设备列表
    pub fn get_devices(&mut self) -> Result<Vec<String>> {
        let output = Command::new(&self.adb_path)
            .args(&["devices"])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("ADB 命令执行失败"));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let devices: Vec<String> = stdout
            .lines()
            .skip(1) // 跳过第一行 "List of devices attached"
            .filter_map(|line| {
                if line.trim().is_empty() {
                    None
                } else {
                    Some(line.split_whitespace().next().unwrap_or("").to_string())
                }
            })
            .filter(|device| !device.is_empty())
            .collect();
        
        Ok(devices)
    }
    
    /// 启动 ADB 服务器
    pub fn start_server(&mut self) -> Result<()> {
        let output = Command::new(&self.adb_path)
            .args(&["start-server"])
            .output()?;
        
        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to start ADB server"))
        }
    }

    /// 重启设备
    pub fn reboot_device(&mut self, device_id: &str) -> Result<()> {
        let output = Command::new(&self.adb_path)
            .args(&["-s", device_id, "reboot"])
            .output()?;
        
        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to reboot device"))
        }
    }

    /// 执行 shell 命令
    pub fn shell(&mut self, device_id: &str, command: &str) -> Result<String> {
        let output = Command::new(&self.adb_path)
            .args(&["-s", device_id, "shell", command])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "ADB shell 命令执行失败: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    /// 推送文件到设备
    pub fn push(&mut self, device_id: &str, local_path: &str, remote_path: &str) -> Result<()> {
        let output = Command::new(&self.adb_path)
            .args(&["-s", device_id, "push", local_path, remote_path])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "ADB push 命令执行失败: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        Ok(())
    }
    
    /// 从设备拉取文件
    pub fn pull(&mut self, device_id: &str, remote_path: &str, local_path: &str) -> Result<()> {
        let output = Command::new(&self.adb_path)
            .args(&["-s", device_id, "pull", remote_path, local_path])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "ADB pull 命令执行失败: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        Ok(())
    }

    /// 推送文件到设备
    pub fn push_file(&mut self, device_id: &str, local_path: &str, remote_path: &str) -> Result<()> {
        self.push(device_id, local_path, remote_path)
    }

    /// 从设备拉取文件
    pub fn pull_file(&mut self, device_id: &str, remote_path: &str, local_path: &str) -> Result<()> {
        self.pull(device_id, remote_path, local_path)
    }

    /// 安装模块（简化版）
    pub fn install_module(&mut self, device_id: &str, module_path: &str) -> Result<()> {
        // 简化版本，推送模块文件并安装
        self.push_file(device_id, module_path, "/data/local/tmp/module.zip")?;
        let cmd = "su -c 'cd /data/local/tmp && unzip -o module.zip'";
        self.shell(device_id, cmd)?;
        Ok(())
    }

    /// 获取设备日志
    pub fn get_device_logs(&mut self, device_id: &str, _filter: Option<&str>) -> Result<Vec<String>> {
        let output = self.shell(device_id, "logcat -d -t 100")?;
        Ok(output.lines().map(|s| s.to_string()).collect())
    }

    /// 检查模块状态
    pub fn check_module_status(&mut self, device_id: &str, _module_id: &str) -> Result<bool> {
        let output = self.shell(device_id, "su -c 'ls /data/adb/modules/'")?;
        Ok(!output.trim().is_empty())
    }

    /// 获取设备信息
    pub fn get_device_info(&mut self, device_id: &str) -> Result<String> {
        let model = self.shell(device_id, "getprop ro.product.model")?;
        let android_version = self.shell(device_id, "getprop ro.build.version.release")?;
        
        Ok(format!(
            "Device: {} (Android {})", 
            model.trim(), 
            android_version.trim()
        ))
    }

    /// 执行 shell 命令（接受字符串数组）
    pub fn exec_shell(&mut self, device_id: &str, cmd: &[&str]) -> Result<String> {
        let command = cmd.join(" ");
        self.shell(device_id, &command)
    }

    /// 列出设备（别名）
    pub fn list_devices(&mut self) -> Result<Vec<String>> {
        self.get_devices()
    }
}