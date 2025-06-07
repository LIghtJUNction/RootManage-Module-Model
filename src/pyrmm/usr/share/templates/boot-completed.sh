#!/system/bin/sh
# This script will be executed after Android boot completed
# 该脚本将在 Android 系统启动完毕后运行

# 获取模块目录
MODDIR=${0%/*}

# 检测运行环境
if [ "$KSU" = "true" ]; then
    echo "Boot-completed script running in KernelSU environment"
else
    echo "Boot-completed script running in Magisk environment"
fi

# 这个脚本在系统完全启动后运行，适合进行一些初始化操作
# 例如：启动需要完整系统环境的服务

echo "Boot-completed script executed successfully"
