#!/system/bin/sh
#
# 简单系统调优模块卸载脚本
# Simple System Tweaks Module Uninstall Script
#

# 模块信息
MODULE_NAME="Simple System Tweaks"
MODULE_DIR="/data/adb/modules/simple_tweaks"

# 日志函数
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $MODULE_NAME Uninstall: $*" >> /data/adb/modules_log
}

log "Starting $MODULE_NAME uninstallation"

# 恢复原始设置
restore_original_settings() {
    log "Restoring original system settings..."
    
    # 恢复虚拟内存参数到默认值
    echo 60 > /proc/sys/vm/swappiness 2>/dev/null || true
    echo 3000 > /proc/sys/vm/dirty_expire_centisecs 2>/dev/null || true
    echo 500 > /proc/sys/vm/dirty_writeback_centisecs 2>/dev/null || true
    
    # 恢复网络参数
    echo 0 > /proc/sys/net/ipv4/tcp_low_latency 2>/dev/null || true
    echo 2 > /proc/sys/net/ipv4/tcp_frto 2>/dev/null || true
    
    # 恢复文件系统参数
    echo 128 > /sys/block/mmcblk0/queue/read_ahead_kb 2>/dev/null || true
    
    log "Original settings restored"
}

# 恢复CPU调速器设置
restore_cpu_governor() {
    log "Restoring CPU governor settings..."
    
    for cpu in /sys/devices/system/cpu/cpu*/cpufreq/; do
        if [ -d "$cpu" ]; then
            # 恢复到默认的interactive调速器
            echo "interactive" > "${cpu}scaling_governor" 2>/dev/null || true
        fi
    done
    
    log "CPU governor settings restored"
}

# 清理模块文件
cleanup_module_files() {
    log "Cleaning up module files..."
    
    # 删除状态文件
    rm -f "$MODULE_DIR/tweaks_applied"
    
    # 删除临时配置文件
    rm -f "$MODULE_DIR/temp_config"
    rm -f "$MODULE_DIR/backup_*"
    
    log "Module files cleaned up"
}

# 清理系统修改
cleanup_system_modifications() {
    log "Cleaning up system modifications..."
    
    # 如果修改了系统配置文件，在这里恢复
    # 注意：这个示例模块没有修改系统文件，所以这里只是示例
    
    # 示例：恢复limits.conf（如果有备份）
    if [ -f "$MODULE_DIR/backup_limits.conf" ]; then
        cp "$MODULE_DIR/backup_limits.conf" /system/etc/security/limits.conf
        log "Restored limits.conf from backup"
    fi
    
    log "System modifications cleaned up"
}

# 主卸载函数
main() {
    log "Beginning uninstallation process..."
    
    # 执行卸载步骤
    restore_original_settings
    restore_cpu_governor
    cleanup_system_modifications
    cleanup_module_files
    
    log "$MODULE_NAME uninstalled successfully"
    log "Changes will take effect after reboot"
}

# 执行主函数
main "$@"
