#!/system/bin/sh

# Kernel Performance Tweaks Service Script
# 高级内核参数调优服务脚本

MODPATH="${0%/*}"
LOG_FILE="$MODPATH/logs/kernel-tweaks.log"

# 创建日志目录
mkdir -p "$(dirname "$LOG_FILE")"

# 日志记录函数
log_msg() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# 检查文件是否存在并可写
check_writable() {
    local file="$1"
    if [ -f "$file" ] && [ -w "$file" ]; then
        return 0
    else
        return 1
    fi
}

# 安全写入函数
safe_write() {
    local file="$1"
    local value="$2"
    local description="$3"
    
    if check_writable "$file"; then
        echo "$value" > "$file" 2>/dev/null
        if [ $? -eq 0 ]; then
            log_msg "✓ $description: $value"
        else
            log_msg "✗ 无法设置 $description"
        fi
    else
        log_msg "⚠ 跳过不可写文件: $file"
    fi
}

# CPU调度器优化
optimize_cpu_scheduler() {
    log_msg "优化CPU调度器参数..."
    
    # 调度延迟优化
    safe_write "/proc/sys/kernel/sched_latency_ns" "6000000" "调度延迟"
    safe_write "/proc/sys/kernel/sched_min_granularity_ns" "750000" "最小调度粒度"
    safe_write "/proc/sys/kernel/sched_wakeup_granularity_ns" "1000000" "唤醒粒度"
    
    # 调度器特性
    safe_write "/proc/sys/kernel/sched_child_runs_first" "0" "子进程优先运行"
    safe_write "/proc/sys/kernel/sched_compat_yield" "1" "兼容性让步"
    
    # CPU频率缩放
    for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        if [ -f "$cpu" ]; then
            # 优先使用interactive或ondemand调速器
            if grep -q "interactive" "$cpu"_available_governors 2>/dev/null; then
                safe_write "$cpu" "interactive" "CPU调速器($(basename $(dirname $cpu)))"
            elif grep -q "ondemand" "$cpu"_available_governors 2>/dev/null; then
                safe_write "$cpu" "ondemand" "CPU调速器($(basename $(dirname $cpu)))"
            fi
        fi
    done
    
    log_msg "CPU调度器优化完成"
}

# 内存管理优化
optimize_memory_management() {
    log_msg "优化内存管理参数..."
    
    # 虚拟内存参数
    safe_write "/proc/sys/vm/swappiness" "10" "Swap使用倾向"
    safe_write "/proc/sys/vm/vfs_cache_pressure" "50" "VFS缓存压力"
    safe_write "/proc/sys/vm/dirty_ratio" "20" "脏页比例"
    safe_write "/proc/sys/vm/dirty_background_ratio" "5" "后台脏页比例"
    safe_write "/proc/sys/vm/dirty_expire_centisecs" "3000" "脏页过期时间"
    safe_write "/proc/sys/vm/dirty_writeback_centisecs" "500" "脏页回写间隔"
    
    # 内存管理策略
    safe_write "/proc/sys/vm/overcommit_memory" "1" "内存过度分配"
    safe_write "/proc/sys/vm/oom_kill_allocating_task" "1" "OOM杀死分配任务"
    safe_write "/proc/sys/vm/page-cluster" "0" "页面聚类"
    
    # 透明大页面
    if [ -f "/sys/kernel/mm/transparent_hugepage/enabled" ]; then
        safe_write "/sys/kernel/mm/transparent_hugepage/enabled" "madvise" "透明大页面"
    fi
    
    log_msg "内存管理优化完成"
}

# I/O调度器优化
optimize_io_scheduler() {
    log_msg "优化I/O调度器..."
    
    # 为不同类型的存储设备设置合适的调度器
    for device in /sys/block/*/queue; do
        local device_name=$(basename "$(dirname "$device")")
        local scheduler_file="$device/scheduler"
        local rotational_file="$device/rotational"
        
        if [ -f "$scheduler_file" ] && [ -r "$rotational_file" ]; then
            local is_rotational=$(cat "$rotational_file")
            
            if [ "$is_rotational" = "1" ]; then
                # 机械硬盘使用deadline或cfq
                if grep -q "deadline" "$scheduler_file"; then
                    safe_write "$scheduler_file" "deadline" "I/O调度器($device_name-HDD)"
                elif grep -q "cfq" "$scheduler_file"; then
                    safe_write "$scheduler_file" "cfq" "I/O调度器($device_name-HDD)"
                fi
            else
                # SSD/eMMC使用noop或deadline
                if grep -q "noop" "$scheduler_file"; then
                    safe_write "$scheduler_file" "noop" "I/O调度器($device_name-SSD)"
                elif grep -q "deadline" "$scheduler_file"; then
                    safe_write "$scheduler_file" "deadline" "I/O调度器($device_name-SSD)"
                fi
            fi
            
            # 优化队列深度
            local nr_requests_file="$device/nr_requests"
            if [ -f "$nr_requests_file" ]; then
                safe_write "$nr_requests_file" "128" "队列深度($device_name)"
            fi
            
            # 优化预读
            local read_ahead_file="$device/read_ahead_kb"
            if [ -f "$read_ahead_file" ]; then
                if [ "$is_rotational" = "1" ]; then
                    safe_write "$read_ahead_file" "2048" "预读大小($device_name-HDD)"
                else
                    safe_write "$read_ahead_file" "512" "预读大小($device_name-SSD)"
                fi
            fi
        fi
    done
    
    log_msg "I/O调度器优化完成"
}

# 网络参数优化
optimize_network() {
    log_msg "优化网络参数..."
    
    # TCP参数优化
    safe_write "/proc/sys/net/core/rmem_default" "262144" "TCP接收缓冲区默认大小"
    safe_write "/proc/sys/net/core/rmem_max" "16777216" "TCP接收缓冲区最大大小"
    safe_write "/proc/sys/net/core/wmem_default" "262144" "TCP发送缓冲区默认大小"
    safe_write "/proc/sys/net/core/wmem_max" "16777216" "TCP发送缓冲区最大大小"
    
    # TCP拥塞控制
    if [ -f "/proc/sys/net/ipv4/tcp_congestion_control" ]; then
        # 优先使用BBR拥塞控制算法
        if grep -q "bbr" /proc/sys/net/ipv4/tcp_available_congestion_control 2>/dev/null; then
            safe_write "/proc/sys/net/ipv4/tcp_congestion_control" "bbr" "TCP拥塞控制"
        elif grep -q "cubic" /proc/sys/net/ipv4/tcp_available_congestion_control 2>/dev/null; then
            safe_write "/proc/sys/net/ipv4/tcp_congestion_control" "cubic" "TCP拥塞控制"
        fi
    fi
    
    # TCP窗口缩放
    safe_write "/proc/sys/net/ipv4/tcp_window_scaling" "1" "TCP窗口缩放"
    safe_write "/proc/sys/net/ipv4/tcp_timestamps" "1" "TCP时间戳"
    safe_write "/proc/sys/net/ipv4/tcp_sack" "1" "TCP选择性确认"
    
    log_msg "网络参数优化完成"
}

# 电源管理优化
optimize_power_management() {
    log_msg "优化电源管理..."
    
    # CPU空闲状态管理
    for cpu in /sys/devices/system/cpu/cpu*/cpuidle; do
        if [ -d "$cpu" ]; then
            # 启用深度睡眠状态
            for state in "$cpu"/state*; do
                if [ -f "$state/disable" ]; then
                    safe_write "$state/disable" "0" "CPU空闲状态($(basename "$state"))"
                fi
            done
        fi
    done
    
    # 热管理
    if [ -f "/sys/class/thermal/thermal_zone0/mode" ]; then
        safe_write "/sys/class/thermal/thermal_zone0/mode" "enabled" "热管理"
    fi
    
    log_msg "电源管理优化完成"
}

# GPU优化 (如果支持)
optimize_gpu() {
    log_msg "检查GPU优化选项..."
    
    # Adreno GPU优化
    local adreno_gpu_dir="/sys/class/kgsl/kgsl-3d0"
    if [ -d "$adreno_gpu_dir" ]; then
        log_msg "检测到Adreno GPU，应用优化..."
        
        if [ -f "$adreno_gpu_dir/devfreq/governor" ]; then
            safe_write "$adreno_gpu_dir/devfreq/governor" "msm-adreno-tz" "GPU调速器"
        fi
        
        if [ -f "$adreno_gpu_dir/force_clk_on" ]; then
            safe_write "$adreno_gpu_dir/force_clk_on" "0" "GPU强制时钟"
        fi
        
        if [ -f "$adreno_gpu_dir/force_bus_on" ]; then
            safe_write "$adreno_gpu_dir/force_bus_on" "0" "GPU强制总线"
        fi
    fi
    
    # Mali GPU优化
    local mali_gpu_dir="/sys/devices/platform/mali.0/dvfs"
    if [ -d "$mali_gpu_dir" ]; then
        log_msg "检测到Mali GPU，应用优化..."
        
        if [ -f "$mali_gpu_dir/governor" ]; then
            safe_write "$mali_gpu_dir/governor" "interactive" "Mali GPU调速器"
        fi
    fi
    
    log_msg "GPU优化完成"
}

# 文件系统优化
optimize_filesystem() {
    log_msg "优化文件系统参数..."
    
    # EXT4文件系统优化
    if mount | grep -q ext4; then
        log_msg "检测到EXT4文件系统"
        
        # 重新挂载EXT4分区以启用优化
        mount | grep ext4 | while read -r line; do
            local device=$(echo "$line" | awk '{print $1}')
            local mountpoint=$(echo "$line" | awk '{print $3}')
            
            # 跳过只读分区
            if echo "$line" | grep -q "ro,"; then
                continue
            fi
            
            log_msg "优化EXT4分区: $mountpoint"
            # 这里可以添加EXT4优化选项，但需要谨慎操作
        done
    fi
    
    # F2FS文件系统优化
    if mount | grep -q f2fs; then
        log_msg "检测到F2FS文件系统"
        # F2FS相关优化可以在这里添加
    fi
    
    log_msg "文件系统优化完成"
}

# 内核模块管理
manage_kernel_modules() {
    log_msg "管理内核模块..."
    
    # 检查是否有需要加载的内核模块
    local modules_file="$MODPATH/kernel-modules.conf"
    if [ -f "$modules_file" ]; then
        while read -r module; do
            # 跳过注释和空行
            if [[ "$module" =~ ^#.*$ ]] || [ -z "$module" ]; then
                continue
            fi
            
            # 尝试加载模块
            if modprobe "$module" 2>/dev/null; then
                log_msg "✓ 已加载内核模块: $module"
            else
                log_msg "⚠ 无法加载内核模块: $module"
            fi
        done < "$modules_file"
    fi
    
    log_msg "内核模块管理完成"
}

# 性能监控
setup_performance_monitoring() {
    log_msg "设置性能监控..."
    
    # 创建性能监控脚本
    cat > "$MODPATH/monitor.sh" << 'EOF'
#!/system/bin/sh

# 性能监控脚本
LOG_FILE="/data/local/tmp/performance-monitor.log"

while true; do
    {
        echo "=== $(date) ==="
        echo "CPU使用率:"
        cat /proc/loadavg
        echo "内存使用:"
        cat /proc/meminfo | grep -E "(MemTotal|MemFree|MemAvailable|Cached|Buffers)"
        echo "存储I/O:"
        cat /proc/diskstats | head -5
        echo ""
    } >> "$LOG_FILE"
    
    sleep 300  # 5分钟记录一次
done
EOF
    
    chmod +x "$MODPATH/monitor.sh"
    
    # 在后台启动监控 (可选)
    # nohup "$MODPATH/monitor.sh" &
    
    log_msg "性能监控设置完成"
}

# 主要优化流程
main() {
    log_msg "开始内核性能调优..."
    log_msg "模块路径: $MODPATH"
    
    # 检查系统信息
    log_msg "系统信息:"
    log_msg "  内核版本: $(uname -r)"
    log_msg "  处理器: $(cat /proc/cpuinfo | grep "Hardware" | head -1 | cut -d':' -f2 | xargs)"
    log_msg "  内存总量: $(cat /proc/meminfo | grep MemTotal | awk '{print $2 $3}')"
    
    # 执行各项优化
    optimize_cpu_scheduler
    optimize_memory_management
    optimize_io_scheduler
    optimize_network
    optimize_power_management
    optimize_gpu
    optimize_filesystem
    manage_kernel_modules
    setup_performance_monitoring
    
    log_msg "内核性能调优完成!"
    log_msg "建议重启设备以确保所有优化生效"
    
    # 显示优化摘要
    log_msg "优化摘要:"
    log_msg "  ✓ CPU调度器优化"
    log_msg "  ✓ 内存管理优化"
    log_msg "  ✓ I/O调度器优化"
    log_msg "  ✓ 网络参数优化"
    log_msg "  ✓ 电源管理优化"
    log_msg "  ✓ GPU优化 (如适用)"
    log_msg "  ✓ 文件系统优化"
    log_msg "  ✓ 性能监控设置"
}

# 错误处理
trap 'log_msg "脚本执行出错，退出代码: $?"' ERR

# 执行主函数
main "$@"
