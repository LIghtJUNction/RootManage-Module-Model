# KernelSU模块开发完整指南
# Complete Guide to KernelSU Module Development

## 概述 | Overview

KernelSU是一个内核级的root解决方案，为Android设备提供了强大的系统级修改能力。本指南将详细介绍如何开发KernelSU模块，从基础概念到高级技巧。

KernelSU is a kernel-level root solution that provides powerful system-level modification capabilities for Android devices. This guide provides comprehensive instructions on developing KernelSU modules, from basic concepts to advanced techniques.

## 第一章：基础知识 | Chapter 1: Fundamentals

### 1.1 什么是KernelSU模块

KernelSU模块是一个包含脚本、配置文件和资源的ZIP包，可以在Android系统启动时执行，用于修改系统行为、添加功能或优化性能。

### 1.2 模块结构

```
my_module/
├── module.prop              # 模块属性文件（必需）
├── service.sh              # 服务脚本（可选）
├── post-fs-data.sh         # post-fs-data阶段脚本（可选）
├── boot-completed.sh       # 启动完成脚本（可选）
├── uninstall.sh           # 卸载脚本（可选）
├── system.prop            # 系统属性文件（可选）
├── sepolicy.rule          # SELinux策略规则（可选）
├── system/                # 系统文件替换目录（可选）
├── vendor/                # vendor文件替换目录（可选）
├── webui/                 # Web界面文件（可选）
└── META-INF/              # 安装脚本目录（可选）
    └── com/google/android/
        ├── update-binary
        └── updater-script
```

### 1.3 模块属性文件 (module.prop)

这是每个模块必须包含的核心文件：

```properties
id=my_module_id
name=My Module Name
version=v1.0.0
versionCode=1
author=YourName
description=Description of what this module does
```

## 第二章：开发环境设置 | Chapter 2: Development Environment Setup

### 2.1 安装开发工具

```bash
# 初始化开发环境
setup-dev-env

# 验证安装
module-builder --version
module-validator --version
module-packager --version
```

### 2.2 创建第一个项目

```bash
# 创建基础模块项目
mkmodule my-first-module basic

# 进入项目目录
cd my-first-module

# 查看项目结构
ls -la
```

### 2.3 配置开发环境

编辑 `.kernelsu-project` 文件：

```bash
PROJECT_NAME="my-first-module"
PROJECT_TYPE="basic"
PROJECT_VERSION="1.0.0"
PROJECT_AUTHOR="YourName"
CREATED_DATE="2025-06-07"

MIN_API=21
TARGET_API=34
MIN_KERNELSU=10940

ENABLE_DEBUG=true
ENABLE_LINT=true
ENABLE_TEST=true
```

## 第三章：脚本开发 | Chapter 3: Script Development

### 3.1 服务脚本 (service.sh)

这是模块的主要执行脚本，在系统启动后运行：

```bash
#!/system/bin/sh
# 服务脚本在系统完全启动后执行

# 模块配置
MODULE_NAME="My Module"
MODULE_DIR="/data/adb/modules/my_module_id"

# 日志函数
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $MODULE_NAME: $*" >> /data/adb/modules_log
}

log "Starting $MODULE_NAME"

# 检查设备兼容性
check_compatibility() {
    local api_level
    api_level="$(getprop ro.build.version.sdk)"
    
    if [ "$api_level" -lt 21 ]; then
        log "ERROR: Unsupported Android version (API $api_level)"
        return 1
    fi
    
    log "Compatibility check passed (API $api_level)"
    return 0
}

# 应用模块功能
apply_module_features() {
    log "Applying module features..."
    
    # 示例：修改系统属性
    resetprop ro.debuggable 1
    
    # 示例：创建配置文件
    echo "module_active=true" > "$MODULE_DIR/status"
    
    log "Module features applied"
}

# 主函数
main() {
    if check_compatibility; then
        apply_module_features
        log "$MODULE_NAME loaded successfully"
    else
        log "ERROR: $MODULE_NAME failed to load"
        exit 1
    fi
}

# 执行主函数
main "$@"
```

### 3.2 启动早期脚本 (post-fs-data.sh)

在文件系统挂载后但在系统启动前执行：

```bash
#!/system/bin/sh
# post-fs-data脚本在文件系统准备就绪后立即执行

# 这里可以进行需要在系统启动前完成的操作
# 例如：修改系统文件、设置权限等

# 创建必要的目录
mkdir -p /data/adb/modules/my_module_id/logs

# 设置文件权限
chmod 755 /data/adb/modules/my_module_id/bin/*

# 备份重要的系统文件
if [ ! -f "/data/adb/modules/my_module_id/backup/build.prop" ]; then
    mkdir -p /data/adb/modules/my_module_id/backup
    cp /system/build.prop /data/adb/modules/my_module_id/backup/
fi

echo "post-fs-data stage completed" >> /data/adb/modules_log
```

### 3.3 启动完成脚本 (boot-completed.sh)

在系统完全启动后执行：

```bash
#!/system/bin/sh
# boot-completed脚本在系统启动完成后执行

# 这里可以进行需要在系统完全启动后执行的操作
# 例如：启动服务、发送通知等

# 等待系统完全就绪
while [ "$(getprop sys.boot_completed)" != "1" ]; do
    sleep 1
done

# 启动模块的后台服务
if [ -f "/data/adb/modules/my_module_id/daemon.sh" ]; then
    nohup /data/adb/modules/my_module_id/daemon.sh &
fi

# 发送模块就绪通知
am broadcast -a com.kernelsu.module.ready \
    --es module_id "my_module_id" \
    --es module_name "My Module"

echo "boot-completed stage finished" >> /data/adb/modules_log
```

### 3.4 卸载脚本 (uninstall.sh)

提供清理和恢复功能：

```bash
#!/system/bin/sh
# 卸载脚本用于清理模块留下的修改

MODULE_DIR="/data/adb/modules/my_module_id"

# 恢复备份的文件
if [ -f "$MODULE_DIR/backup/build.prop" ]; then
    cp "$MODULE_DIR/backup/build.prop" /system/build.prop
fi

# 停止模块的后台进程
pkill -f "my_module_daemon"

# 清理创建的文件
rm -rf /data/my_module_data
rm -f /data/adb/my_module_config

# 恢复系统属性（需要重启生效）
resetprop --delete ro.my_module.enabled

echo "Module uninstalled successfully" >> /data/adb/modules_log
```

## 第四章：系统集成 | Chapter 4: System Integration

### 4.1 系统属性修改

```bash
# 临时修改属性（重启后失效）
resetprop ro.debuggable 1

# 持久化属性修改
resetprop -p persist.sys.timezone Asia/Shanghai

# 在system.prop文件中定义属性
echo "ro.my_module.version=1.0.0" >> system.prop
echo "persist.my_module.enabled=true" >> system.prop
```

### 4.2 文件系统修改

```bash
# 创建系统文件替换
mkdir -p system/bin
cp my_binary system/bin/

mkdir -p system/etc
cp my_config.conf system/etc/

# 设置正确的权限
chmod 755 system/bin/my_binary
chmod 644 system/etc/my_config.conf
```

### 4.3 SELinux策略修改

创建 `sepolicy.rule` 文件：

```
# 允许模块的二进制文件执行
allow untrusted_app my_module_exec:file execute;

# 允许模块访问特定目录
allow my_module_domain system_data_file:dir { read write };

# 允许模块修改系统属性
allow my_module_domain system_prop:property_service set;
```

## 第五章：WebUI开发 | Chapter 5: WebUI Development

### 5.1 启用WebUI

在 `module.prop` 中启用：

```properties
webui=true
webui_port=8080
webui_path=/webui
```

### 5.2 创建WebUI界面

创建 `webui/index.html`：

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>My Module WebUI</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <div class="container">
        <h1>My Module Control Panel</h1>
        
        <div class="status-section">
            <h2>Status</h2>
            <div id="module-status">Loading...</div>
            <button onclick="refreshStatus()">Refresh</button>
        </div>
        
        <div class="settings-section">
            <h2>Settings</h2>
            <form id="settings-form">
                <label>
                    Enable Feature:
                    <input type="checkbox" id="feature-enabled">
                </label>
                <button type="submit">Save</button>
            </form>
        </div>
    </div>
    
    <script src="script.js"></script>
</body>
</html>
```

### 5.3 添加JavaScript功能

创建 `webui/script.js`：

```javascript
// WebUI功能脚本
class ModuleWebUI {
    constructor() {
        this.init();
    }
    
    init() {
        this.setupEventListeners();
        this.refreshStatus();
    }
    
    setupEventListeners() {
        document.getElementById('settings-form').addEventListener('submit', (e) => {
            e.preventDefault();
            this.saveSettings();
        });
    }
    
    async refreshStatus() {
        try {
            const response = await fetch('/api/status');
            const data = await response.json();
            document.getElementById('module-status').textContent = data.status;
        } catch (error) {
            console.error('Failed to refresh status:', error);
        }
    }
    
    async saveSettings() {
        const settings = {
            feature_enabled: document.getElementById('feature-enabled').checked
        };
        
        try {
            const response = await fetch('/api/settings', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(settings)
            });
            
            if (response.ok) {
                alert('Settings saved!');
            }
        } catch (error) {
            console.error('Failed to save settings:', error);
        }
    }
}

// 初始化WebUI
const webui = new ModuleWebUI();

// 全局函数
function refreshStatus() {
    webui.refreshStatus();
}
```

### 5.4 WebUI服务器设置

在 `service.sh` 中添加WebUI服务器：

```bash
# 启动WebUI服务器
start_webui() {
    local webui_dir="$MODULE_DIR/webui"
    local webui_port=8080
    
    # 检查Python是否可用
    if command -v python3 >/dev/null 2>&1; then
        cd "$webui_dir"
        python3 -m http.server "$webui_port" &
        echo $! > "$MODULE_DIR/webui_server.pid"
        log "WebUI server started on port $webui_port"
    else
        log "ERROR: Python not available for WebUI"
    fi
}

# 在主函数中调用
start_webui
```

## 第六章：测试和调试 | Chapter 6: Testing and Debugging

### 6.1 模块验证

```bash
# 验证模块结构
module-validator .

# 检查脚本语法
shellcheck *.sh

# 验证权限设置
find . -type f -name "*.sh" -exec ls -la {} \;
```

### 6.2 构建和打包

```bash
# 构建模块
module-builder .

# 打包模块
module-packager .

# 验证打包结果
unzip -l my-module-v1.0.0.zip
```

### 6.3 设备测试

```bash
# 部署到设备
deploy-module --module my-module-v1.0.0.zip

# 查看日志
adb shell cat /data/adb/modules_log | grep "My Module"

# 检查模块状态
adb shell ls -la /data/adb/modules/my_module_id/
```

### 6.4 调试技巧

```bash
# 启用调试模式
export DEBUG_MODE=true

# 详细日志输出
set -x  # 在脚本开头添加

# 错误跟踪
set -e  # 遇到错误立即退出
set -u  # 使用未定义变量时报错
set -o pipefail  # 管道命令失败时报错
```

## 第七章：高级功能 | Chapter 7: Advanced Features

### 7.1 模块间通信

```bash
# 创建模块通信接口
create_module_interface() {
    local interface_dir="/data/adb/module_interfaces"
    mkdir -p "$interface_dir"
    
    # 创建接口文件
    cat > "$interface_dir/my_module.interface" << EOF
module_id=my_module_id
module_name=My Module
api_version=1.0
interface_methods=get_status,set_config,get_logs
EOF
}

# 调用其他模块的接口
call_module_method() {
    local target_module="$1"
    local method="$2"
    local params="$3"
    
    local interface_file="/data/adb/module_interfaces/$target_module.interface"
    if [ -f "$interface_file" ]; then
        # 实现模块间调用逻辑
        echo "Calling $method on $target_module with params: $params"
    fi
}
```

### 7.2 动态配置管理

```bash
# 配置文件监听
monitor_config_changes() {
    local config_file="$MODULE_DIR/config.conf"
    local last_mtime=""
    
    while true; do
        if [ -f "$config_file" ]; then
            local current_mtime
            current_mtime=$(stat -c %Y "$config_file")
            
            if [ "$current_mtime" != "$last_mtime" ]; then
                log "Config file changed, reloading..."
                reload_config
                last_mtime="$current_mtime"
            fi
        fi
        
        sleep 5
    done
}

# 热重载配置
reload_config() {
    source "$MODULE_DIR/config.conf"
    apply_config_changes
    log "Configuration reloaded"
}
```

### 7.3 性能监控

```bash
# 性能监控函数
monitor_performance() {
    local cpu_usage
    local memory_usage
    local io_usage
    
    while true; do
        # CPU使用率
        cpu_usage=$(top -n 1 | grep "my_module" | awk '{print $9}')
        
        # 内存使用
        memory_usage=$(ps -o pid,vsz,rss,comm | grep "my_module" | awk '{print $2}')
        
        # I/O统计
        io_usage=$(iostat -x 1 1 | tail -1 | awk '{print $10}')
        
        # 记录性能数据
        echo "$(date '+%Y-%m-%d %H:%M:%S') CPU:$cpu_usage% MEM:${memory_usage}KB IO:$io_usage%" >> "$MODULE_DIR/performance.log"
        
        sleep 60
    done
}
```

## 第八章：部署和分发 | Chapter 8: Deployment and Distribution

### 8.1 版本管理

```bash
# 更新版本号
update_version() {
    local new_version="$1"
    local new_version_code="$2"
    
    sed -i "s/^version=.*/version=$new_version/" module.prop
    sed -i "s/^versionCode=.*/versionCode=$new_version_code/" module.prop
    
    echo "Updated to version $new_version (code: $new_version_code)"
}
```

### 8.2 自动更新

创建 `update.json`：

```json
{
    "version": "v1.0.0",
    "versionCode": 1,
    "zipUrl": "https://github.com/user/repo/releases/download/v1.0.0/module.zip",
    "changelog": "https://github.com/user/repo/releases/tag/v1.0.0",
    "minKernelSU": 10940,
    "minApi": 21,
    "maxApi": 34
}
```

### 8.3 签名和验证

```bash
# 生成模块签名
sign_module() {
    local module_zip="$1"
    local private_key="$2"
    
    # 计算文件哈希
    local file_hash
    file_hash=$(sha256sum "$module_zip" | cut -d' ' -f1)
    
    # 创建签名文件
    echo "$file_hash" | openssl dgst -sha256 -sign "$private_key" -out "${module_zip}.sig"
    
    echo "Module signed: ${module_zip}.sig"
}

# 验证模块签名
verify_module() {
    local module_zip="$1"
    local public_key="$2"
    local signature="${module_zip}.sig"
    
    if [ -f "$signature" ]; then
        local file_hash
        file_hash=$(sha256sum "$module_zip" | cut -d' ' -f1)
        
        if echo "$file_hash" | openssl dgst -sha256 -verify "$public_key" -signature "$signature"; then
            echo "Module signature valid"
            return 0
        else
            echo "Module signature invalid"
            return 1
        fi
    else
        echo "No signature found"
        return 1
    fi
}
```

## 第九章：最佳实践 | Chapter 9: Best Practices

### 9.1 代码质量

- 使用ShellCheck验证脚本语法
- 添加详细的注释和文档
- 实现完整的错误处理
- 遵循Shell脚本编程规范

### 9.2 性能优化

- 减少启动时间
- 优化内存使用
- 避免不必要的文件I/O
- 使用后台任务处理耗时操作

### 9.3 安全考虑

- 最小权限原则
- 输入验证
- 安全的文件操作
- 防止路径遍历攻击

### 9.4 兼容性

- 支持多个Android版本
- 检查设备架构
- 测试不同的设备类型
- 提供降级兼容方案

## 第十章：故障排除 | Chapter 10: Troubleshooting

### 10.1 常见问题

**模块无法安装**
- 检查module.prop格式
- 验证ZIP文件结构
- 确认KernelSU版本兼容性

**脚本执行失败**
- 检查文件权限
- 验证脚本语法
- 查看错误日志

**功能不正常**
- 检查SELinux策略
- 验证文件路径
- 确认系统权限

### 10.2 调试工具

```bash
# 脚本调试
bash -x service.sh

# 权限检查
ls -laZ /data/adb/modules/my_module_id/

# 进程监控
ps aux | grep my_module

# 日志查看
logcat | grep KernelSU
```

### 10.3 日志分析

```bash
# 提取模块相关日志
grep "My Module" /data/adb/modules_log

# 分析错误信息
grep -i "error\|fail\|exception" /data/adb/modules_log

# 性能分析
awk '/My Module/ {print $1, $2, $NF}' /data/adb/modules_log
```

## 附录 | Appendix

### A. API参考

详见 `/usr/share/doc/api-reference.md`

### B. 示例模块

参考 `/usr/local/share/examples/` 目录下的示例

### C. 开发工具

- `module-builder`: 模块构建工具
- `module-validator`: 模块验证工具
- `module-packager`: 模块打包工具
- `deploy-module`: 模块部署工具

### D. 相关资源

- [KernelSU官方文档](https://kernelsu.org/)
- [Android开发者文档](https://developer.android.com/)
- [Shell脚本编程指南](https://www.gnu.org/software/bash/manual/)

---

本指南涵盖了KernelSU模块开发的所有重要方面，从基础概念到高级技巧。通过遵循本指南，您将能够开发出高质量、可靠且功能丰富的KernelSU模块。
