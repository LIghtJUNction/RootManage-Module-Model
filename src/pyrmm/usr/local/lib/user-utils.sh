#!/system/bin/sh
# 用户自定义工具库
# 版本: 1.0
# 作者: RootManage Module System

# ===========================================
# 颜色定义
# ===========================================
export RED='\033[0;31m'
export GREEN='\033[0;32m'
export YELLOW='\033[1;33m'
export BLUE='\033[0;34m'
export PURPLE='\033[0;35m'
export CYAN='\033[0;36m'
export WHITE='\033[1;37m'
export NC='\033[0m' # No Color

# 背景色
export BG_RED='\033[41m'
export BG_GREEN='\033[42m'
export BG_YELLOW='\033[43m'
export BG_BLUE='\033[44m'

# 文本样式
export BOLD='\033[1m'
export DIM='\033[2m'
export UNDERLINE='\033[4m'
export BLINK='\033[5m'
export REVERSE='\033[7m'

# ===========================================
# 日志函数
# ===========================================

# 初始化日志
init_log() {
    local log_file="${1:-/data/local/tmp/user-tools.log}"
    export USER_LOG_FILE="$log_file"
    export USER_LOG_LEVEL="${2:-INFO}"
    
    # 创建日志目录
    mkdir -p "$(dirname "$log_file")"
    
    # 日志轮转
    if [ -f "$log_file" ] && [ $(stat -c%s "$log_file" 2>/dev/null || echo 0) -gt 1048576 ]; then
        mv "$log_file" "${log_file}.old"
    fi
}

# 记录日志
log_message() {
    local level="$1"
    local message="$2"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    if [ -n "$USER_LOG_FILE" ]; then
        echo "[$timestamp] [$level] $message" >> "$USER_LOG_FILE"
    fi
}

# 调试日志
log_debug() {
    [ "$USER_LOG_LEVEL" = "DEBUG" ] && log_message "DEBUG" "$1"
    [ "$USER_LOG_LEVEL" = "DEBUG" ] && echo -e "${DIM}[DEBUG] $1${NC}" >&2
}

# 信息日志
log_info() {
    log_message "INFO" "$1"
    echo -e "${BLUE}[INFO]${NC} $1"
}

# 警告日志
log_warn() {
    log_message "WARN" "$1"
    echo -e "${YELLOW}[WARN]${NC} $1" >&2
}

# 错误日志
log_error() {
    log_message "ERROR" "$1"
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

# 成功日志
log_success() {
    log_message "SUCCESS" "$1"
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# ===========================================
# 文件操作函数
# ===========================================

# 安全复制文件
safe_copy() {
    local src="$1"
    local dst="$2"
    local backup="${3:-true}"
    
    if [ ! -f "$src" ]; then
        log_error "源文件不存在: $src"
        return 1
    fi
    
    # 创建目标目录
    mkdir -p "$(dirname "$dst")"
    
    # 备份原文件
    if [ "$backup" = "true" ] && [ -f "$dst" ]; then
        local backup_file="${dst}.backup.$(date +%s)"
        cp "$dst" "$backup_file"
        log_info "已备份原文件: $backup_file"
    fi
    
    # 复制文件
    if cp "$src" "$dst"; then
        log_success "文件复制成功: $src -> $dst"
        return 0
    else
        log_error "文件复制失败: $src -> $dst"
        return 1
    fi
}

# 安全移动文件
safe_move() {
    local src="$1"
    local dst="$2"
    local backup="${3:-true}"
    
    if [ ! -f "$src" ]; then
        log_error "源文件不存在: $src"
        return 1
    fi
    
    # 创建目标目录
    mkdir -p "$(dirname "$dst")"
    
    # 备份原文件
    if [ "$backup" = "true" ] && [ -f "$dst" ]; then
        local backup_file="${dst}.backup.$(date +%s)"
        cp "$dst" "$backup_file"
        log_info "已备份原文件: $backup_file"
    fi
    
    # 移动文件
    if mv "$src" "$dst"; then
        log_success "文件移动成功: $src -> $dst"
        return 0
    else
        log_error "文件移动失败: $src -> $dst"
        return 1
    fi
}

# 安全删除文件
safe_remove() {
    local file="$1"
    local backup="${2:-true}"
    
    if [ ! -f "$file" ]; then
        log_warn "文件不存在: $file"
        return 0
    fi
    
    # 备份文件
    if [ "$backup" = "true" ]; then
        local backup_file="${file}.deleted.$(date +%s)"
        cp "$file" "$backup_file"
        log_info "已备份删除的文件: $backup_file"
    fi
    
    # 删除文件
    if rm "$file"; then
        log_success "文件删除成功: $file"
        return 0
    else
        log_error "文件删除失败: $file"
        return 1
    fi
}

# ===========================================
# 系统信息函数
# ===========================================

# 获取系统信息
get_system_info() {
    echo "=== 系统信息 ==="
    echo "内核版本: $(uname -r)"
    echo "Android版本: $(getprop ro.build.version.release)"
    echo "API级别: $(getprop ro.build.version.sdk)"
    echo "设备型号: $(getprop ro.product.model)"
    echo "设备品牌: $(getprop ro.product.brand)"
    echo "架构: $(uname -m)"
    echo "时间: $(date)"
    echo "运行时间: $(uptime)"
}

# 获取内存信息
get_memory_info() {
    echo "=== 内存信息 ==="
    if [ -f /proc/meminfo ]; then
        grep -E "(MemTotal|MemFree|MemAvailable|Buffers|Cached)" /proc/meminfo
    else
        echo "无法获取内存信息"
    fi
}

# 获取存储信息
get_storage_info() {
    echo "=== 存储信息 ==="
    df -h 2>/dev/null | grep -E "(Filesystem|/data|/system|/cache)"
}

# 获取CPU信息
get_cpu_info() {
    echo "=== CPU信息 ==="
    if [ -f /proc/cpuinfo ]; then
        grep -E "(processor|model name|cpu cores|Hardware)" /proc/cpuinfo | head -10
    else
        echo "无法获取CPU信息"
    fi
}

# 获取网络信息
get_network_info() {
    echo "=== 网络信息 ==="
    if command -v ip >/dev/null 2>&1; then
        ip addr show 2>/dev/null | grep -E "(inet |inet6 )" | head -5
    elif command -v ifconfig >/dev/null 2>&1; then
        ifconfig 2>/dev/null | grep -E "(inet |inet6 )" | head -5
    else
        echo "无法获取网络信息"
    fi
}

# ===========================================
# 进程管理函数
# ===========================================

# 查找进程
find_process() {
    local name="$1"
    if [ -z "$name" ]; then
        log_error "请提供进程名称"
        return 1
    fi
    
    ps aux 2>/dev/null | grep "$name" | grep -v grep
}

# 杀死进程
kill_process() {
    local name="$1"
    local signal="${2:-TERM}"
    
    if [ -z "$name" ]; then
        log_error "请提供进程名称"
        return 1
    fi
    
    local pids=$(pgrep "$name")
    if [ -z "$pids" ]; then
        log_warn "未找到进程: $name"
        return 0
    fi
    
    for pid in $pids; do
        if kill -"$signal" "$pid" 2>/dev/null; then
            log_success "已终止进程: $name (PID: $pid)"
        else
            log_error "无法终止进程: $name (PID: $pid)"
        fi
    done
}

# 检查进程是否运行
is_process_running() {
    local name="$1"
    pgrep "$name" >/dev/null 2>&1
}

# ===========================================
# 网络工具函数
# ===========================================

# 检查网络连接
check_network() {
    local host="${1:-8.8.8.8}"
    local timeout="${2:-5}"
    
    if ping -c 1 -W "$timeout" "$host" >/dev/null 2>&1; then
        log_success "网络连接正常"
        return 0
    else
        log_error "网络连接失败"
        return 1
    fi
}

# 检查端口是否开放
check_port() {
    local host="$1"
    local port="$2"
    local timeout="${3:-5}"
    
    if [ -z "$host" ] || [ -z "$port" ]; then
        log_error "请提供主机和端口"
        return 1
    fi
    
    if command -v nc >/dev/null 2>&1; then
        if nc -z -w "$timeout" "$host" "$port" 2>/dev/null; then
            log_success "端口 $host:$port 开放"
            return 0
        else
            log_error "端口 $host:$port 关闭或无法访问"
            return 1
        fi
    else
        log_warn "nc命令不可用，无法检查端口"
        return 1
    fi
}

# 下载文件
download_file() {
    local url="$1"
    local output="$2"
    local timeout="${3:-30}"
    
    if [ -z "$url" ]; then
        log_error "请提供下载URL"
        return 1
    fi
    
    if [ -z "$output" ]; then
        output=$(basename "$url")
    fi
    
    log_info "开始下载: $url"
    
    if command -v curl >/dev/null 2>&1; then
        if curl -L --connect-timeout "$timeout" -o "$output" "$url"; then
            log_success "下载完成: $output"
            return 0
        else
            log_error "下载失败: $url"
            return 1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if wget -T "$timeout" -O "$output" "$url"; then
            log_success "下载完成: $output"
            return 0
        else
            log_error "下载失败: $url"
            return 1
        fi
    else
        log_error "curl和wget都不可用，无法下载文件"
        return 1
    fi
}

# ===========================================
# 字符串处理函数
# ===========================================

# 去除字符串两端空白
trim() {
    local str="$1"
    # 去除前导空白
    str="${str#"${str%%[![:space:]]*}"}"
    # 去除尾随空白
    str="${str%"${str##*[![:space:]]}"}"
    echo "$str"
}

# 字符串是否为空
is_empty() {
    local str="$1"
    [ -z "$(trim "$str")" ]
}

# 字符串包含检查
contains() {
    local string="$1"
    local substring="$2"
    [ "${string#*$substring}" != "$string" ]
}

# 字符串开头检查
starts_with() {
    local string="$1"
    local prefix="$2"
    [ "${string#$prefix}" != "$string" ]
}

# 字符串结尾检查
ends_with() {
    local string="$1"
    local suffix="$2"
    [ "${string%$suffix}" != "$string" ]
}

# ===========================================
# 用户交互函数
# ===========================================

# 询问用户确认
ask_confirm() {
    local message="$1"
    local default="${2:-n}"
    
    local prompt="$message"
    if [ "$default" = "y" ]; then
        prompt="$prompt [Y/n]: "
    else
        prompt="$prompt [y/N]: "
    fi
    
    echo -n "$prompt"
    read -r response
    
    if [ -z "$response" ]; then
        response="$default"
    fi
    
    case "$response" in
        [Yy]|[Yy][Ee][Ss])
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

# 选择菜单
show_menu() {
    local title="$1"
    shift
    local options=("$@")
    
    echo -e "${CYAN}$title${NC}"
    echo "=================="
    
    local i=1
    for option in "${options[@]}"; do
        echo "$i) $option"
        i=$((i + 1))
    done
    
    echo -n "请选择 [1-${#options[@]}]: "
    read -r choice
    
    if [ "$choice" -ge 1 ] && [ "$choice" -le "${#options[@]}" ]; then
        return $((choice - 1))
    else
        log_error "无效选择: $choice"
        return 255
    fi
}

# 进度条
show_progress() {
    local current="$1"
    local total="$2"
    local width="${3:-50}"
    
    local percent=$((current * 100 / total))
    local filled=$((current * width / total))
    local empty=$((width - filled))
    
    printf "\r["
    printf "%${filled}s" | tr ' ' '='
    printf "%${empty}s" | tr ' ' '-'
    printf "] %d%%" "$percent"
    
    if [ "$current" -eq "$total" ]; then
        echo ""
    fi
}

# ===========================================
# 初始化
# ===========================================

# 检查运行环境
check_environment() {
    # 检查是否为root用户
    if [ "$(id -u)" -ne 0 ]; then
        log_warn "建议以root权限运行"
    fi
    
    # 检查基本命令
    local missing_commands=""
    for cmd in ps grep awk sed cut; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            missing_commands="$missing_commands $cmd"
        fi
    done
    
    if [ -n "$missing_commands" ]; then
        log_warn "缺少命令:$missing_commands"
    fi
}

# 库初始化
if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
    # 直接执行脚本时的行为
    echo "用户工具库已加载"
    echo "使用方法: source $0"
else
    # 被source时的行为
    init_log
    check_environment
    log_debug "用户工具库初始化完成"
fi
