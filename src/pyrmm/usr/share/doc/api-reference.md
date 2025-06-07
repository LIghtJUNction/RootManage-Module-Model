
# KernelSU Module Development API Reference

## 目录 | Table of Contents

1. [模块属性API | Module Properties API](#模块属性api--module-properties-api)
2. [系统属性API | System Properties API](#系统属性api--system-properties-api)
3. [文件系统API | File System API](#文件系统api--file-system-api)
4. [进程管理API | Process Management API](#进程管理api--process-management-api)
5. [WebUI API](#webui-api--webui-api)
6. [日志API | Logging API](#日志api--logging-api)
7. [工具函数API | Utility Functions API](#工具函数api--utility-functions-api)
8. [兼容性API | Compatibility API](#兼容性api--compatibility-api)

## 模块属性API | Module Properties API

### 基本属性

```properties
# 必需属性 | Required Properties
id=module_identifier          # 模块ID（唯一标识符）
name=Module Display Name      # 模块显示名称
version=v1.0.0               # 版本号
versionCode=1                # 版本代码（数字）
author=Author Name           # 作者名称
description=Module Description # 模块描述

# 可选属性 | Optional Properties
minApi=21                    # 最低API级别
maxApi=34                   # 最高API级别
minKernelSU=10940           # 最低KernelSU版本
updateJson=https://...       # 更新检查URL
support=https://...          # 支持页面URL
donate=https://...           # 捐赠页面URL
```

### 高级属性

```properties
# 模块行为控制
auto_mount=true              # 自动挂载系统文件
boot_stage=post-fs-data      # 启动阶段
replace_example=true         # 替换示例模块

# WebUI配置
webui=true                   # 启用WebUI
webui_port=8080             # WebUI端口
webui_path=/webui           # WebUI路径

# 兼容性设置
magisk_compatible=true       # Magisk兼容模式
apatch_compatible=false      # APatch兼容模式
```

## 系统属性API | System Properties API

### resetprop 命令

```bash
# 读取属性
value=$(resetprop ro.build.version.sdk)

# 设置属性（临时，重启后失效）
resetprop ro.debuggable 1

# 设置属性（持久化）
resetprop -p ro.debuggable 1

# 删除属性
resetprop --delete ro.custom.property

# 列出所有属性
resetprop --list

# 监听属性变化
resetprop --watch ro.boot.complete
```

### getprop 命令

```bash
# 读取系统属性
api_level=$(getprop ro.build.version.sdk)
device_model=$(getprop ro.product.model)
android_version=$(getprop ro.build.version.release)

# 检查属性是否存在
if getprop ro.custom.property >/dev/null 2>&1; then
    echo "Property exists"
fi
```

### setprop 命令

```bash
# 设置系统属性（需要root权限）
setprop debug.sf.nobootanimation 1
setprop persist.sys.dalvik.vm.lib.2 libart.so
```

## 文件系统API | File System API

### 文件操作函数

```bash
# 安全复制文件
safe_copy_file source_file target_file [mode] [owner] [group]

# 安全移动文件
safe_move_file source_file target_file

# 安全删除文件
safe_delete_file file_path

# 创建目录
safe_mkdir directory_path [mode] [owner] [group]

# 检查文件存在
if file_exists "/path/to/file"; then
    echo "File exists"
fi

# 检查目录存在
if dir_exists "/path/to/directory"; then
    echo "Directory exists"
fi

# 获取文件大小
size=$(get_file_size "/path/to/file")

# 获取文件权限
permissions=$(get_file_permissions "/path/to/file")
```

### 挂载操作

```bash
# 检查挂载点
if is_mounted "/system"; then
    echo "System is mounted"
fi

# 重新挂载为可写
remount_rw "/system"

# 重新挂载为只读
remount_ro "/system"

# 创建绑定挂载
bind_mount source_path target_path

# 解除挂载
umount_safe "/path/to/mountpoint"
```

### 权限管理

```bash
# 设置文件权限
set_perm file_path owner group mode

# 设置目录权限（递归）
set_perm_recursive directory_path owner group dir_mode file_mode

# 恢复SELinux上下文
restore_selinux_context "/path/to/file"

# 设置自定义SELinux上下文
set_selinux_context "/path/to/file" "u:object_r:system_file:s0"
```

## 进程管理API | Process Management API

### 进程控制

```bash
# 启动后台进程
start_daemon "/path/to/daemon" [args...]

# 停止进程
stop_process process_name

# 检查进程是否运行
if is_process_running "process_name"; then
    echo "Process is running"
fi

# 获取进程PID
pid=$(get_process_pid "process_name")

# 等待进程启动
wait_for_process "process_name" [timeout]

# 杀死进程树
kill_process_tree pid
```

### 服务管理

```bash
# 启动系统服务
start_service service_name

# 停止系统服务
stop_service service_name

# 重启系统服务
restart_service service_name

# 检查服务状态
service_status=$(get_service_status "service_name")

# 设置服务属性
set_service_property service_name property_name value
```

## WebUI API | WebUI API

### WebUI服务器管理

```bash
# 启动WebUI服务器
start_webui_server port [bind_address] [document_root]

# 停止WebUI服务器
stop_webui_server

# 检查WebUI服务器状态
if is_webui_running; then
    echo "WebUI is running"
fi

# 获取WebUI URL
webui_url=$(get_webui_url)

# 重启WebUI服务器
restart_webui_server
```

### HTTP服务器配置

```bash
# 设置HTTP服务器类型
set_http_server_type "python|busybox|lighttpd"

# 配置SSL
configure_ssl_cert cert_file key_file

# 设置认证
configure_auth username password

# 添加MIME类型
add_mime_type extension mime_type

# 设置安全头
set_security_headers
```

### WebUI模板系统

```bash
# 渲染模板
render_template template_file output_file [variables...]

# 设置模板变量
set_template_var variable_name value

# 获取模板变量
value=$(get_template_var variable_name)

# 包含模板片段
include_template template_fragment
```

## 日志API | Logging API

### 日志函数

```bash
# 基本日志级别
log_debug "Debug message"
log_info "Information message"
log_warn "Warning message"
log_error "Error message"
log_fatal "Fatal error message"

# 带时间戳的日志
log_with_timestamp "INFO" "Message with timestamp"

# 日志到文件
log_to_file "/path/to/logfile" "INFO" "Message to file"

# 结构化日志
log_structured "level" "component" "message" [key=value...]
```

### 日志配置

```bash
# 设置日志级别
set_log_level "debug|info|warn|error|fatal"

# 设置日志文件
set_log_file "/path/to/logfile"

# 设置日志格式
set_log_format "simple|detailed|json"

# 启用/禁用控制台输出
set_console_logging true|false

# 设置日志轮转
set_log_rotation max_size max_files
```

### 日志查看和分析

```bash
# 获取最近的日志
get_recent_logs [count] [level]

# 搜索日志
search_logs pattern [start_time] [end_time]

# 统计日志
get_log_stats [start_time] [end_time]

# 清理旧日志
cleanup_old_logs [days]
```

## 工具函数API | Utility Functions API

### 字符串处理

```bash
# 字符串比较
if string_equals "string1" "string2"; then
    echo "Strings are equal"
fi

# 字符串包含
if string_contains "haystack" "needle"; then
    echo "String contains substring"
fi

# 字符串替换
result=$(string_replace "original" "pattern" "replacement")

# 去除空白
trimmed=$(string_trim "  spaced string  ")

# 转换大小写
upper=$(string_upper "lowercase")
lower=$(string_lower "UPPERCASE")
```

### 数学运算

```bash
# 基本运算
result=$(math_add 10 20)
result=$(math_subtract 30 10)
result=$(math_multiply 5 6)
result=$(math_divide 100 5)

# 比较运算
if math_greater_than 10 5; then
    echo "10 > 5"
fi

# 范围检查
if math_in_range 15 10 20; then
    echo "15 is between 10 and 20"
fi
```

### 时间和日期

```bash
# 获取当前时间戳
timestamp=$(get_timestamp)

# 格式化时间
formatted=$(format_time timestamp "%Y-%m-%d %H:%M:%S")

# 计算时间差
diff=$(time_diff start_timestamp end_timestamp)

# 睡眠函数
sleep_seconds 5
sleep_milliseconds 500
```

### 网络工具

```bash
# 检查网络连接
if is_online; then
    echo "Network is available"
fi

# 检查主机可达性
if is_host_reachable "google.com"; then
    echo "Host is reachable"
fi

# 下载文件
download_file "https://example.com/file.zip" "/path/to/local/file.zip"

# 获取IP地址
ip_address=$(get_ip_address)

# 检查端口状态
if is_port_open "127.0.0.1" 8080; then
    echo "Port 8080 is open"
fi
```

### 系统信息

```bash
# 获取系统信息
api_level=$(get_api_level)
architecture=$(get_architecture)
device_model=$(get_device_model)
android_version=$(get_android_version)
kernel_version=$(get_kernel_version)

# 获取硬件信息
cpu_info=$(get_cpu_info)
memory_info=$(get_memory_info)
storage_info=$(get_storage_info)

# 获取电池信息
battery_level=$(get_battery_level)
battery_status=$(get_battery_status)
```

## 兼容性API | Compatibility API

### 环境检测

```bash
# 检测root方案
root_solution=$(detect_root_solution)  # kernelsu|magisk|apatch|supersu

# 检测KernelSU环境
if is_kernelsu; then
    echo "Running under KernelSU"
fi

# 检测Magisk环境
if is_magisk; then
    echo "Running under Magisk"
fi

# 检测APatch环境
if is_apatch; then
    echo "Running under APatch"
fi

# 获取root方案版本
version=$(get_root_version)
```

### 兼容性处理

```bash
# 兼容性模式执行
execute_compatible "command" [args...]

# 获取兼容的路径
compatible_path=$(get_compatible_path "/original/path")

# 设置兼容性标志
set_compatibility_flag "magisk_module" true

# 检查功能支持
if is_feature_supported "selinux_patch"; then
    echo "SELinux patching is supported"
fi
```

### 模块互操作

```bash
# 检查其他模块
if is_module_installed "module_id"; then
    echo "Module is installed"
fi

# 获取模块信息
module_info=$(get_module_info "module_id")

# 模块间通信
send_module_message "target_module" "message"
receive_module_message [timeout]

# 共享资源
share_resource "resource_name" "resource_path"
get_shared_resource "resource_name"
```

## 错误处理和调试 | Error Handling and Debugging

### 错误处理

```bash
# 设置错误处理器
set_error_handler custom_error_handler

# 抛出错误
throw_error "Error message" [error_code]

# 尝试执行并捕获错误
if try_execute "risky_command"; then
    echo "Command succeeded"
else
    echo "Command failed with code: $?"
fi

# 重试机制
retry_command 3 "command_that_might_fail"
```

### 调试工具

```bash
# 调试模式
enable_debug_mode
disable_debug_mode

# 跟踪执行
trace_execution command [args...]

# 性能分析
start_profiling
stop_profiling
get_profile_results

# 内存使用监控
monitor_memory_usage pid [interval]
```

### 诊断信息

```bash
# 收集诊断信息
collect_diagnostics [output_file]

# 生成系统报告
generate_system_report [output_file]

# 检查模块健康状态
check_module_health

# 验证系统完整性
verify_system_integrity
```

## 配置管理API | Configuration Management API

### 配置文件操作

```bash
# 读取配置值
value=$(config_get "section.key" [config_file])

# 设置配置值
config_set "section.key" "value" [config_file]

# 删除配置项
config_unset "section.key" [config_file]

# 检查配置项存在
if config_exists "section.key" [config_file]; then
    echo "Configuration exists"
fi
```

### 配置验证

```bash
# 验证配置文件
validate_config [config_file]

# 应用配置更改
apply_config_changes [config_file]

# 重载配置
reload_config [config_file]

# 备份配置
backup_config [config_file] [backup_path]
```

## 模块生命周期API | Module Lifecycle API

### 安装和卸载

```bash
# 模块安装钩子
on_module_install() {
    # 自定义安装逻辑
}

# 模块卸载钩子
on_module_uninstall() {
    # 自定义卸载逻辑
}

# 模块更新钩子
on_module_update() {
    # 自定义更新逻辑
}

# 模块启用钩子
on_module_enable() {
    # 模块启用时的逻辑
}

# 模块禁用钩子
on_module_disable() {
    # 模块禁用时的逻辑
}
```

### 启动阶段

```bash
# post-fs-data阶段
on_post_fs_data() {
    # 文件系统挂载后的逻辑
}

# service阶段
on_service() {
    # 系统服务启动时的逻辑
}

# boot-completed阶段
on_boot_completed() {
    # 系统完全启动后的逻辑
}
```

## 安全API | Security API

### 权限检查

```bash
# 检查root权限
if has_root_permission; then
    echo "Has root permission"
fi

# 检查文件权限
if has_file_permission "/path/to/file" "read"; then
    echo "Has read permission"
fi

# 检查SELinux权限
if has_selinux_permission "domain" "type" "class" "permission"; then
    echo "Has SELinux permission"
fi
```

### 签名验证

```bash
# 验证模块签名
if verify_module_signature "/path/to/module.zip"; then
    echo "Module signature is valid"
fi

# 验证APK签名
if verify_apk_signature "/path/to/app.apk"; then
    echo "APK signature is valid"
fi

# 生成文件哈希
hash=$(generate_file_hash "/path/to/file" "sha256")
```

### 加密和解密

```bash
# 加密数据
encrypted=$(encrypt_data "plain_text" "password")

# 解密数据
decrypted=$(decrypt_data "encrypted_text" "password")

# 生成随机密钥
key=$(generate_random_key 32)

# 安全删除文件
secure_delete_file "/path/to/sensitive/file"
```

## 示例用法 | Usage Examples

### 基础模块结构

```bash
#!/system/bin/sh
# 模块主脚本示例

# 导入通用函数库
. /usr/lib/common-functions.sh

# 设置日志
set_log_file "/data/adb/modules/my_module/module.log"
set_log_level "info"

# 模块初始化
log_info "模块初始化开始"

# 检查环境
if ! check_kernelsu_environment; then
    log_error "KernelSU环境检查失败"
    exit 1
fi

# 执行模块逻辑
log_info "执行模块主要逻辑"

# 模块完成
log_info "模块初始化完成"
```

### WebUI模块示例

```bash
#!/system/bin/sh
# WebUI模块示例

# 导入WebUI函数库
. /usr/lib/webui-helpers.sh

# 配置WebUI
setup_webui_structure "/data/adb/modules/my_webui_module"
start_webui_server 8080 "127.0.0.1" "/data/adb/modules/my_webui_module/webroot"

# 检查WebUI状态
if is_webui_running; then
    log_info "WebUI已启动: $(get_webui_url)"
else
    log_error "WebUI启动失败"
fi
```

### 配置管理示例

```bash
#!/system/bin/sh
# 配置管理示例

# 读取配置
debug_mode=$(config_get "general.debug_mode" "/data/adb/modules/my_module/config.ini")
log_level=$(config_get "logging.level" "/data/adb/modules/my_module/config.ini")

# 应用配置
if [ "$debug_mode" = "true" ]; then
    enable_debug_mode
fi

set_log_level "$log_level"

# 保存新配置
config_set "runtime.start_time" "$(date)" "/data/adb/modules/my_module/config.ini"
```

---

## API参考索引 | API Reference Index

### 核心函数

- `check_kernelsu_environment()` - 检查KernelSU环境
- `get_api_level()` - 获取API级别
- `get_architecture()` - 获取设备架构
- `is_64bit()` - 检查是否为64位系统

### 文件系统函数

- `safe_copy_file()` - 安全文件复制
- `set_perm()` - 设置文件权限
- `is_mounted()` - 检查挂载状态
- `bind_mount()` - 创建绑定挂载

### 进程管理函数

- `start_daemon()` - 启动守护进程
- `is_process_running()` - 检查进程状态
- `kill_process_tree()` - 终止进程树

### WebUI函数

- `start_webui_server()` - 启动WebUI服务器
- `setup_webui_structure()` - 设置WebUI结构
- `render_template()` - 渲染模板

### 日志相关函数

- `log_info()` - 信息日志
- `log_error()` - 错误日志
- `set_log_level()` - 设置日志级别

### 工具函数

- `string_contains()` - 字符串包含检查
- `download_file()` - 文件下载
- `is_online()` - 网络连接检查

### 兼容性函数

- `detect_root_solution()` - 检测root方案
- `execute_compatible()` - 兼容性执行

### 配置函数

- `config_get()` - 读取配置
- `config_set()` - 设置配置
- `validate_config()` - 验证配置

---

此API参考文档涵盖了KernelSU模块开发中常用的函数和接口。更多详细信息请参考具体的函数实现和示例代码。
