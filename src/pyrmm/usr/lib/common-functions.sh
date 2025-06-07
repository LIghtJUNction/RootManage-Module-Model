#!/bin/bash
# KernelSU Module Common Functions Library
# 通用函数库，可在模块脚本中引用

# 防止重复加载
if [ "${COMMON_FUNCTIONS_LOADED:-0}" = "1" ]; then
    return 0
fi
readonly COMMON_FUNCTIONS_LOADED=1

# 颜色定义
readonly COLOR_RED='\033[0;31m'
readonly COLOR_GREEN='\033[0;32m'
readonly COLOR_YELLOW='\033[1;33m'
readonly COLOR_BLUE='\033[0;34m'
readonly COLOR_PURPLE='\033[0;35m'
readonly COLOR_CYAN='\033[0;36m'
readonly COLOR_WHITE='\033[1;37m'
readonly COLOR_GRAY='\033[0;37m'
readonly COLOR_NC='\033[0m'

# 日志级别
readonly LOG_LEVEL_DEBUG=0
readonly LOG_LEVEL_INFO=1
readonly LOG_LEVEL_WARNING=2
readonly LOG_LEVEL_ERROR=3

# 当前日志级别 (可通过环境变量设置)
CURRENT_LOG_LEVEL="${LOG_LEVEL:-$LOG_LEVEL_INFO}"

# 日志文件路径
LOG_FILE="${LOG_FILE:-/data/local/tmp/kernelsu_modules.log}"

# 通用日志函数
write_log() {
    local level="$1"
    local message="$2"
    local timestamp="$(date '+%Y-%m-%d %H:%M:%S')"
    local module_id="${MODULE_ID:-unknown}"
    
    # 写入日志文件
    if [ -w "$(dirname "$LOG_FILE")" ]; then
        echo "[$timestamp] [$level] [$module_id] $message" >> "$LOG_FILE"
    fi
}

# 日志函数
log_debug() {
    if [ "$CURRENT_LOG_LEVEL" -le "$LOG_LEVEL_DEBUG" ]; then
        echo -e "${COLOR_GRAY}[DEBUG]${COLOR_NC} $1" >&2
        write_log "DEBUG" "$1"
    fi
}

log_info() {
    if [ "$CURRENT_LOG_LEVEL" -le "$LOG_LEVEL_INFO" ]; then
        echo -e "${COLOR_BLUE}[INFO]${COLOR_NC} $1"
        write_log "INFO" "$1"
    fi
}

log_success() {
    if [ "$CURRENT_LOG_LEVEL" -le "$LOG_LEVEL_INFO" ]; then
        echo -e "${COLOR_GREEN}[SUCCESS]${COLOR_NC} $1"
        write_log "SUCCESS" "$1"
    fi
}

log_warning() {
    if [ "$CURRENT_LOG_LEVEL" -le "$LOG_LEVEL_WARNING" ]; then
        echo -e "${COLOR_YELLOW}[WARNING]${COLOR_NC} $1" >&2
        write_log "WARNING" "$1"
    fi
}

log_error() {
    if [ "$CURRENT_LOG_LEVEL" -le "$LOG_LEVEL_ERROR" ]; then
        echo -e "${COLOR_RED}[ERROR]${COLOR_NC} $1" >&2
        write_log "ERROR" "$1"
    fi
}

log_debug() {
    if [[ "${DEBUG:-0}" == "1" ]]; then
        echo -e "${COLOR_PURPLE}[DEBUG]${COLOR_NC} $1"
    fi
}

# 环境检查函数
check_root() {
    if [[ "$(id -u)" != "0" ]]; then
        log_error "此脚本必须以root权限运行"
        exit 1
    fi
}

check_kernelsu() {
    if [[ "$KSU" != "true" ]]; then
        log_error "此脚本需要KernelSU环境"
        exit 1
    fi
}

check_magisk() {
    if [[ -z "$MAGISK_VER_CODE" ]]; then
        log_error "此脚本需要Magisk环境"
        exit 1
    fi
}

detect_root_solution() {
    if [[ "$KSU" == "true" ]]; then
        echo "kernelsu"
    elif [[ -n "$MAGISK_VER_CODE" ]]; then
        echo "magisk"
    elif [[ "$APATCH" == "true" ]]; then
        echo "apatch"
    else
        echo "unknown"
    fi
}

# 环境检测函数
detect_environment() {
    # 检测KernelSU
    if [ -n "$KSU" ] || [ -f "/data/adb/ksu/bin/busybox" ]; then
        echo "kernelsu"
        return 0
    fi
    
    # 检测Magisk
    if [ -n "$MAGISK_VER" ] || [ -f "/data/adb/magisk/busybox" ]; then
        echo "magisk"
        return 0
    fi
    
    # 检测APatch
    if [ -f "/data/adb/ap/bin/busybox" ]; then
        echo "apatch"
        return 0
    fi
    
    echo "unknown"
    return 1
}

# 系统信息函数
get_android_version() {
    getprop ro.build.version.release
}

get_api_level() {
    getprop ro.build.version.sdk
}

get_device_arch() {
    getprop ro.product.cpu.abi
}

get_device_model() {
    getprop ro.product.model
}

get_device_brand() {
    getprop ro.product.brand
}

# 启动状态检查
is_boot_completed() {
    [[ "$(getprop sys.boot_completed)" == "1" ]]
}

wait_for_boot() {
    log_info "等待系统启动完成..."
    while ! is_boot_completed; do
        sleep 1
    done
    log_success "系统启动完成"
}

# 网络检查
check_internet() {
    ping -c 1 -W 3 8.8.8.8 >/dev/null 2>&1
}

wait_for_network() {
    log_info "等待网络连接..."
    local timeout=30
    local count=0
    
    while ! check_internet && [[ $count -lt $timeout ]]; do
        sleep 1
        ((count++))
    done
    
    if check_internet; then
        log_success "网络连接正常"
        return 0
    else
        log_warning "网络连接超时"
        return 1
    fi
}

# 文件操作函数
backup_file() {
    local file="$1"
    local backup_dir="${2:-/data/backup}"
    
    if [[ -f "$file" ]]; then
        mkdir -p "$backup_dir"
        local backup_name="$(basename "$file").$(date +%Y%m%d_%H%M%S).bak"
        cp "$file" "$backup_dir/$backup_name"
        log_info "已备份文件: $file -> $backup_dir/$backup_name"
        return 0
    else
        log_warning "文件不存在，无需备份: $file"
        return 1
    fi
}

restore_file() {
    local backup_file="$1"
    local target_file="$2"
    
    if [[ -f "$backup_file" ]]; then
        cp "$backup_file" "$target_file"
        log_info "已恢复文件: $backup_file -> $target_file"
        return 0
    else
        log_error "备份文件不存在: $backup_file"
        return 1
    fi
}

# 权限设置函数
set_module_permissions() {
    local module_dir="$1"
    
    if [[ ! -d "$module_dir" ]]; then
        log_error "模块目录不存在: $module_dir"
        return 1
    fi
    
    # 设置目录权限
    find "$module_dir" -type d -exec chmod 755 {} \;
    
    # 设置文件权限
    find "$module_dir" -type f -name "*.sh" -exec chmod 755 {} \;
    find "$module_dir" -type f -name "update-binary" -exec chmod 755 {} \;
    find "$module_dir" -type f ! -name "*.sh" ! -name "update-binary" -exec chmod 644 {} \;
    
    log_success "已设置模块权限: $module_dir"
}

# 属性操作函数
get_prop_safe() {
    local prop="$1"
    local default="$2"
    
    local value=$(getprop "$prop")
    echo "${value:-$default}"
}

set_prop_safe() {
    local prop="$1"
    local value="$2"
    
    if command -v resetprop >/dev/null 2>&1; then
        resetprop "$prop" "$value"
    else
        setprop "$prop" "$value"
    fi
}

# 服务管理函数
is_service_running() {
    local service="$1"
    getprop "init.svc.$service" | grep -q "running"
}

start_service() {
    local service="$1"
    if ! is_service_running "$service"; then
        setprop "ctl.start" "$service"
        log_info "已启动服务: $service"
    else
        log_info "服务已在运行: $service"
    fi
}

stop_service() {
    local service="$1"
    if is_service_running "$service"; then
        setprop "ctl.stop" "$service"
        log_info "已停止服务: $service"
    else
        log_info "服务未在运行: $service"
    fi
}

restart_service() {
    local service="$1"
    stop_service "$service"
    sleep 1
    start_service "$service"
}

# 应用包管理
is_app_installed() {
    local package="$1"
    pm list packages | grep -q "package:$package"
}

get_app_version() {
    local package="$1"
    dumpsys package "$package" | grep "versionName" | head -n1 | cut -d'=' -f2
}

force_stop_app() {
    local package="$1"
    am force-stop "$package"
    log_info "已强制停止应用: $package"
}

# 挂载操作
is_mounted() {
    local path="$1"
    mount | grep -q " $path "
}

remount_rw() {
    local path="$1"
    mount -o remount,rw "$path" 2>/dev/null
}

remount_ro() {
    local path="$1"
    mount -o remount,ro "$path" 2>/dev/null
}

# 下载函数
download_file() {
    local url="$1"
    local output="$2"
    local timeout="${3:-30}"
    
    log_info "下载文件: $url"
    
    if command_exists wget; then
        wget -q --timeout="$timeout" -O "$output" "$url"
    elif command_exists curl; then
        curl -s --connect-timeout "$timeout" -o "$output" "$url"
    else
        log_error "无法下载文件：缺少下载工具"
        return 1
    fi
    
    if [ $? -eq 0 ] && [ -f "$output" ]; then
        log_success "文件下载完成: $output"
        return 0
    else
        log_error "文件下载失败: $url"
        return 1
    fi
}

# 版本比较
version_compare() {
    local version1="$1"
    local version2="$2"
    
    # 移除v前缀
    version1="${version1#v}"
    version2="${version2#v}"
    
    # 使用sort进行版本比较
    if [[ "$(echo -e "$version1\n$version2" | sort -V | head -n1)" == "$version1" ]]; then
        if [[ "$version1" == "$version2" ]]; then
            echo "0"  # 相等
        else
            echo "-1" # version1 < version2
        fi
    else
        echo "1"      # version1 > version2
    fi
}

# 清理函数
cleanup_temp_files() {
    local temp_dir="${1:-/data/local/tmp}"
    
    if [[ -d "$temp_dir" ]]; then
        find "$temp_dir" -name "*.tmp" -type f -mtime +1 -delete 2>/dev/null
        find "$temp_dir" -name "tmp.*" -type f -mtime +1 -delete 2>/dev/null
        log_info "已清理临时文件: $temp_dir"
    fi
}

# 错误处理
trap_error() {
    local line_number="$1"
    local error_code="$2"
    log_error "脚本执行失败 (行号: $line_number, 错误代码: $error_code)"
}

# 设置错误陷阱
set_error_trap() {
    set -e
    trap 'trap_error ${LINENO} $?' ERR
}

# 模块信息获取
get_module_info() {
    local module_dir="$1"
    local key="$2"
    
    if [[ -f "$module_dir/module.prop" ]]; then
        grep "^$key=" "$module_dir/module.prop" | cut -d'=' -f2- | head -n1
    fi
}

get_module_id() {
    get_module_info "$1" "id"
}

get_module_name() {
    get_module_info "$1" "name"
}

get_module_version() {
    get_module_info "$1" "version"
}

get_module_author() {
    get_module_info "$1" "author"
}

# 初始化函数
init_module_env() {
    # 设置模块目录
    MODDIR="${MODDIR:-${0%/*}}"
    
    # 设置BusyBox环境
    export PATH="/data/adb/ksu/bin:$PATH"
    export ASH_STANDALONE=1
    
    # 记录模块信息
    if [[ -f "$MODDIR/module.prop" ]]; then
        MODULE_ID=$(get_module_id "$MODDIR")
        MODULE_NAME=$(get_module_name "$MODDIR")
        MODULE_VERSION=$(get_module_version "$MODDIR")
        
        log_info "模块: $MODULE_NAME ($MODULE_ID) v$MODULE_VERSION"
    fi
}

# 退出清理
cleanup_and_exit() {
    local exit_code="${1:-0}"
    cleanup_temp_files
    exit "$exit_code"
}

# 库初始化
if [[ "${BASH_SOURCE[0]}" != "${0}" ]]; then
    # 被其他脚本引用时自动初始化
    init_module_env
fi
