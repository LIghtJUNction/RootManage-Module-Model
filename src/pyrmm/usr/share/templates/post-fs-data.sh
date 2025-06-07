#!/system/bin/sh
# This script will be executed in post-fs-data mode
# 该脚本将在 post-fs-data 模式下运行（阻塞启动过程）

# 获取模块目录
MODDIR=${0%/*}

# 检测运行环境
if [ "$KSU" = "true" ]; then
    echo "Post-fs-data script running in KernelSU environment"
else
    echo "Post-fs-data script running in Magisk environment"
fi

# 注意：这个阶段是阻塞的，脚本执行完成前启动过程会暂停
# 请只在必要时使用此脚本，大多数情况下应使用service.sh

# 示例：在模块挂载前进行动态调整
# 使用 resetprop -n 而不是 setprop 来避免死锁

echo "Post-fs-data script executed successfully"
