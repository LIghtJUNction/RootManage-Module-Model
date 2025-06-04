#这个脚本将会在 late_start 服务模式下运行
# 获取模块的基本目录路径
MODDIR=${0%/*}

# 在此处编写您的服务脚本逻辑
# 例如，您可以在此处添加需要在 late_start 服务模式下运行的命令

# 常用函数：

# 1. 等待开机完成 / 如果管理器支持boot-completed.sh 这个就没什么用
wait_for_boot_completed() {
    
}