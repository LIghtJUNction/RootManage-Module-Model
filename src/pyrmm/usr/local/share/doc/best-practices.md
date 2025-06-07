# KernelSU模块最佳实践指南
# KernelSU Module Best Practices Guide

## 目录 | Table of Contents

1. [模块设计原则](#模块设计原则)
2. [代码质量标准](#代码质量标准)
3. [性能优化](#性能优化)
4. [安全考虑](#安全考虑)
5. [兼容性保证](#兼容性保证)
6. [测试策略](#测试策略)
7. [文档规范](#文档规范)
8. [发布流程](#发布流程)

## 模块设计原则 | Module Design Principles

### 1. 单一职责原则

每个模块应该只做一件事，并且把它做好：

```bash
# 好的例子：专门的音频增强模块
id=audio_enhancer
name=Audio Enhancer
description=Enhance audio quality and volume

# 避免：功能过于复杂的模块
id=system_tweaker_all_in_one
name=System Tweaker All-in-One
description=CPU, GPU, Audio, Network, Memory tweaks all in one
```

### 2. 最小影响原则

只修改必要的系统组件：

```bash
#!/system/bin/sh
# 好的例子：只修改特定属性
resetprop ro.audio.enhancement 1

# 避免：大范围修改
# for prop in $(getprop | grep ro. | cut -d: -f1); do
#     resetprop $prop "modified"
# done
```

### 3. 可逆性原则

确保模块可以被安全卸载：

```bash
# uninstall.sh - 提供完整的卸载逻辑
#!/system/bin/sh

# 恢复备份的文件
if [ -f "/data/adb/modules/mymodule/backup/original_file" ]; then
    cp "/data/adb/modules/mymodule/backup/original_file" "/system/original_file"
fi

# 清理创建的文件
rm -f /data/mymodule_config
rm -f /data/mymodule_cache

# 恢复属性（在下次重启时生效）
echo "Module uninstalled successfully"
```

## 代码质量标准 | Code Quality Standards

### 1. Shell脚本规范

使用ShellCheck验证脚本：

```bash
#!/system/bin/sh
# 总是使用严格模式
set -euo pipefail

# 使用引号包围变量
MODULE_PATH="/data/adb/modules/${MODDIR##*/}"

# 检查命令是否存在
if command -v resetprop >/dev/null 2>&1; then
    resetprop ro.debuggable 1
else
    echo "Warning: resetprop not found"
fi

# 使用适当的错误处理
copy_file() {
    local src="$1"
    local dest="$2"
    
    if [ ! -f "$src" ]; then
        echo "Error: Source file not found: $src"
        return 1
    fi
    
    if ! cp "$src" "$dest"; then
        echo "Error: Failed to copy $src to $dest"
        return 1
    fi
    
    echo "Successfully copied $src to $dest"
}
```

### 2. 错误处理

实现健壮的错误处理：

```bash
# 定义错误处理函数
handle_error() {
    local line_no="$1"
    local error_code="$2"
    echo "Error on line $line_no: Exit code $error_code"
    # 清理临时文件
    cleanup_on_error
    exit "$error_code"
}

# 设置错误陷阱
trap 'handle_error ${LINENO} $?' ERR

# 定义清理函数
cleanup_on_error() {
    rm -f /tmp/module_temp_*
    # 恢复可能的更改
    if [ -f "/data/backup/original_config" ]; then
        cp "/data/backup/original_config" "/system/config"
    fi
}
```

### 3. 日志记录

实现结构化的日志记录：

```bash
# 定义日志函数
LOG_FILE="/data/adb/modules/${MODDIR##*/}/module.log"

log_info() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [INFO] $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [ERROR] $*" | tee -a "$LOG_FILE" >&2
}

log_debug() {
    if [ "$DEBUG_MODE" = "true" ]; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] [DEBUG] $*" | tee -a "$LOG_FILE"
    fi
}

# 在脚本中使用
log_info "Module service started"
log_debug "Current PATH: $PATH"
```

## 性能优化 | Performance Optimization

### 1. 启动时间优化

减少模块加载时间：

```bash
#!/system/bin/sh
# 使用后台任务处理耗时操作
{
    # 耗时的初始化操作
    initialize_heavy_components
    log_info "Heavy initialization completed"
} &

# 快速的核心功能立即执行
apply_critical_tweaks
log_info "Critical tweaks applied"

# 等待后台任务完成（如果需要）
wait
```

### 2. 内存使用优化

避免内存泄漏和过度使用：

```bash
# 使用合适的数据结构
declare -A config_cache

# 及时清理临时文件
cleanup_temp_files() {
    find /tmp -name "module_temp_*" -mtime +1 -delete
}

# 在脚本结束时清理
trap cleanup_temp_files EXIT
```

### 3. 磁盘I/O优化

减少不必要的文件操作：

```bash
# 批量操作而不是逐个处理
{
    echo "config1=value1"
    echo "config2=value2"
    echo "config3=value3"
} > /data/module_config

# 使用内存缓存
cache_file_content() {
    local file="$1"
    if [ -z "${file_cache[$file]:-}" ]; then
        file_cache[$file]="$(cat "$file")"
    fi
    echo "${file_cache[$file]}"
}
```

## 安全考虑 | Security Considerations

### 1. 权限最小化

只请求必要的权限：

```bash
# 检查权限而不是假设拥有权限
check_permission() {
    local file="$1"
    local perm="$2"
    
    if [ ! -"$perm" "$file" ]; then
        log_error "Insufficient permission for $file"
        return 1
    fi
}

# 临时提升权限
run_with_minimal_privilege() {
    # 只在需要时使用su
    if [ "$(id -u)" -ne 0 ]; then
        su -c "$*"
    else
        "$@"
    fi
}
```

### 2. 输入验证

验证所有外部输入：

```bash
validate_module_id() {
    local id="$1"
    
    # 检查格式
    if ! echo "$id" | grep -q '^[a-z][a-z0-9_]*$'; then
        log_error "Invalid module ID format: $id"
        return 1
    fi
    
    # 检查长度
    if [ ${#id} -gt 32 ]; then
        log_error "Module ID too long: $id"
        return 1
    fi
    
    return 0
}

validate_file_path() {
    local path="$1"
    
    # 防止路径遍历攻击
    case "$path" in
        *../*|*/../*|../*|*/..)
            log_error "Path traversal detected: $path"
            return 1
            ;;
    esac
    
    return 0
}
```

### 3. 安全的文件操作

```bash
safe_copy() {
    local src="$1"
    local dest="$2"
    
    # 验证源文件
    if [ ! -f "$src" ]; then
        log_error "Source file does not exist: $src"
        return 1
    fi
    
    # 创建安全的临时文件
    local temp_file
    temp_file="$(mktemp)"
    
    # 复制到临时位置
    if ! cp "$src" "$temp_file"; then
        rm -f "$temp_file"
        log_error "Failed to copy to temporary location"
        return 1
    fi
    
    # 验证复制的完整性
    if ! cmp -s "$src" "$temp_file"; then
        rm -f "$temp_file"
        log_error "File integrity check failed"
        return 1
    fi
    
    # 移动到最终位置
    if ! mv "$temp_file" "$dest"; then
        rm -f "$temp_file"
        log_error "Failed to move to final location"
        return 1
    fi
    
    log_info "File safely copied: $src -> $dest"
}
```

## 兼容性保证 | Compatibility Assurance

### 1. Android版本兼容性

```bash
check_android_version() {
    local min_api="$1"
    local current_api
    current_api="$(getprop ro.build.version.sdk)"
    
    if [ "$current_api" -lt "$min_api" ]; then
        log_error "Unsupported Android version. Required: API $min_api, Current: API $current_api"
        return 1
    fi
    
    log_info "Android version check passed: API $current_api"
}

# 版本特定的处理
handle_version_differences() {
    local api_level
    api_level="$(getprop ro.build.version.sdk)"
    
    case "$api_level" in
        2[1-3])  # Android 5.0-6.0
            apply_legacy_tweaks
            ;;
        2[4-8])  # Android 7.0-9.0
            apply_modern_tweaks
            ;;
        [23][0-9]) # Android 10+
            apply_latest_tweaks
            ;;
        *)
            log_warn "Unknown Android version: API $api_level"
            apply_safe_tweaks
            ;;
    esac
}
```

### 2. 架构兼容性

```bash
detect_architecture() {
    local arch
    arch="$(getprop ro.product.cpu.abi)"
    
    case "$arch" in
        arm64-v8a)
            export ARCH="arm64"
            export LIB_DIR="lib64"
            ;;
        armeabi-v7a)
            export ARCH="arm"
            export LIB_DIR="lib"
            ;;
        x86_64)
            export ARCH="x86_64"
            export LIB_DIR="lib64"
            ;;
        x86)
            export ARCH="x86"
            export LIB_DIR="lib"
            ;;
        *)
            log_error "Unsupported architecture: $arch"
            return 1
            ;;
    esac
    
    log_info "Architecture detected: $ARCH"
}
```

### 3. KernelSU版本兼容性

```bash
check_kernelsu_version() {
    local min_version="$1"
    local current_version
    
    # 检查KernelSU是否存在
    if [ ! -f "/data/adb/ksu/version" ]; then
        log_error "KernelSU not detected"
        return 1
    fi
    
    current_version="$(cat /data/adb/ksu/version 2>/dev/null || echo "0")"
    
    if [ "$current_version" -lt "$min_version" ]; then
        log_error "KernelSU version too old. Required: $min_version, Current: $current_version"
        return 1
    fi
    
    log_info "KernelSU version check passed: $current_version"
}
```

## 测试策略 | Testing Strategy

### 1. 单元测试

```bash
# test_module_functions.sh
#!/bin/bash

# 测试配置解析
test_config_parsing() {
    local test_config="/tmp/test_config"
    echo "key=value" > "$test_config"
    echo "number=123" >> "$test_config"
    
    source "$test_config"
    
    [ "$key" = "value" ] || return 1
    [ "$number" = "123" ] || return 1
    
    rm -f "$test_config"
    echo "Config parsing test passed"
}

# 测试文件操作
test_file_operations() {
    local test_file="/tmp/test_file"
    local test_content="Hello, World!"
    
    echo "$test_content" > "$test_file"
    
    [ -f "$test_file" ] || return 1
    [ "$(cat "$test_file")" = "$test_content" ] || return 1
    
    rm -f "$test_file"
    echo "File operations test passed"
}

# 运行所有测试
run_tests() {
    test_config_parsing || exit 1
    test_file_operations || exit 1
    echo "All tests passed!"
}
```

### 2. 集成测试

```bash
# test_integration.sh
#!/bin/bash

# 测试模块安装
test_module_installation() {
    local test_module="test_module.zip"
    local test_id="test_module"
    
    # 创建测试模块
    create_test_module "$test_module" "$test_id"
    
    # 安装模块
    install_module "$test_module"
    
    # 验证安装
    [ -d "/data/adb/modules/$test_id" ] || return 1
    
    # 清理
    uninstall_module "$test_id"
    rm -f "$test_module"
    
    echo "Module installation test passed"
}
```

### 3. 设备测试

```bash
# device_test.sh - 在真实设备上运行的测试
#!/system/bin/sh

test_on_device() {
    # 测试系统属性
    if ! getprop ro.build.version.sdk >/dev/null; then
        echo "FAIL: Cannot read system properties"
        return 1
    fi
    
    # 测试文件系统访问
    if ! touch /data/test_write_access 2>/dev/null; then
        echo "FAIL: Cannot write to /data"
        return 1
    fi
    rm -f /data/test_write_access
    
    # 测试root权限
    if [ "$(id -u)" -ne 0 ]; then
        echo "FAIL: No root privileges"
        return 1
    fi
    
    echo "PASS: Device tests completed successfully"
}
```

## 文档规范 | Documentation Standards

### 1. README模板

```markdown
# Module Name

Brief description of what this module does.

## Features

- Feature 1
- Feature 2
- Feature 3

## Requirements

- Android 5.0+ (API 21)
- KernelSU v0.6.0+
- Root access

## Installation

1. Download the latest release
2. Install via KernelSU Manager
3. Reboot device

## Configuration

Edit `/data/adb/modules/module_id/config.conf`:

```bash
# Configuration options
enable_feature1=true
feature2_level=high
```

## Troubleshooting

### Common Issues

**Issue**: Module not working after installation
**Solution**: Check KernelSU version compatibility

## License

This project is licensed under the MIT License.
```

### 2. 代码注释标准

```bash
#!/system/bin/sh
#
# Module Name: Audio Enhancer
# Description: Enhances audio quality and volume
# Author: Your Name
# Version: 1.0.0
# License: MIT
#
# This script is executed when the module is loaded
#

# Configuration section
readonly MODULE_DIR="/data/adb/modules/${0##*/}"
readonly CONFIG_FILE="$MODULE_DIR/config.conf"
readonly LOG_FILE="$MODULE_DIR/module.log"

# Load configuration if exists
if [ -f "$CONFIG_FILE" ]; then
    # Source configuration file
    # shellcheck source=/dev/null
    . "$CONFIG_FILE"
fi

#
# Function: enhance_audio
# Description: Apply audio enhancement settings
# Parameters: None
# Returns: 0 on success, 1 on failure
#
enhance_audio() {
    log_info "Applying audio enhancements..."
    
    # Set audio properties
    resetprop ro.audio.enhancement 1
    resetprop ro.audio.quality high
    
    log_info "Audio enhancement applied successfully"
    return 0
}
```

## 发布流程 | Release Process

### 1. 版本管理

```bash
# 更新版本号
update_version() {
    local new_version="$1"
    local version_code="$2"
    
    # 更新module.prop
    sed -i "s/^version=.*/version=$new_version/" module.prop
    sed -i "s/^versionCode=.*/versionCode=$version_code/" module.prop
    
    # 更新项目配置
    sed -i "s/^PROJECT_VERSION=.*/PROJECT_VERSION=\"$new_version\"/" .kernelsu-project
    
    echo "Updated version to $new_version (code: $version_code)"
}
```

### 2. 构建验证

```bash
# 完整的构建和验证流程
pre_release_check() {
    echo "Starting pre-release checks..."
    
    # 代码质量检查
    if ! shellcheck ./*.sh; then
        echo "FAIL: ShellCheck validation failed"
        return 1
    fi
    
    # 模块验证
    if ! module-validator .; then
        echo "FAIL: Module validation failed"
        return 1
    fi
    
    # 构建测试
    if ! module-builder .; then
        echo "FAIL: Build test failed"
        return 1
    fi
    
    # 运行测试套件
    if ! bash test_module_functions.sh; then
        echo "FAIL: Unit tests failed"
        return 1
    fi
    
    echo "PASS: All pre-release checks completed"
    return 0
}
```

### 3. 发布清单

- [ ] 代码审查完成
- [ ] 所有测试通过
- [ ] 文档更新
- [ ] 版本号更新
- [ ] 变更日志更新
- [ ] 构建验证通过
- [ ] 在测试设备上验证
- [ ] 创建发布标签
- [ ] 发布到分发平台

---

遵循这些最佳实践将帮助您创建高质量、可靠和可维护的KernelSU模块。
