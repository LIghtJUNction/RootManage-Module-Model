#这个脚本将会在 late_start 服务模式下运行
# 获取模块的基本目录路径
MODDIR=${0%/*}

# 在此处编写您的服务脚本逻辑
# 例如，您可以在此处添加需要在 late_start 服务模式下运行的命令

# 示例：打印一条消息到日志
echo "服务脚本已启动" >> /data/local/tmp/service.log

# 示例：设置系统属性
resetprop ro.example.property "example_value"

# 示例：启动一个后台服务
nohup some_background_service &

# 示例：执行一个耗时的任务 sleep 10等待10秒
sleep 10

# 示例：打印一条消息到日志
echo "服务脚本已完成" >> /data/local/tmp/service.log

# 请注意：
# - 避免使用可能阻塞或显著延迟启动过程的命令。
# - 确保此脚本启动的任何后台任务都得到妥善管理，以避免资源泄漏。

# 有关更多信息，请参阅 KernelSU 文档中的启动脚本部分。

# 示例：检查设备的架构并执行相应的操作
if [ "$(uname -m)" = "aarch64" ]; then
    echo "设备架构为 arm64" >> /data/local/tmp/service.log
    # 在此处添加针对 arm64 架构的命令
else
    echo "设备架构为其他" >> /data/local/tmp/service.log
    # 在此处添加针对其他架构的命令
fi

# 示例：检查某个文件是否存在
if [ -f /data/local/tmp/some_file ]; then
    echo "文件存在" >> /data/local/tmp/service.log
    # 在此处添加文件存在时的处理逻辑
else
    echo "文件不存在" >> /data/local/tmp/service.log
    # 在此处添加文件不存在时的处理逻辑
fi

# 示例：设置权限
chmod 644 /data/local/tmp/service.log

# 示例：创建一个目录
mkdir -p /data/local/tmp/my_service_dir

# 示例：写入环境变量到文件
echo "MY_ENV_VAR=my_value" > /data/local/tmp/my_service_dir/env_vars

# 示例：启动另一个脚本
sh /data/local/tmp/my_service_dir/another_script.sh & Compare this snippet from MyModule/service.sh: # 这个脚本将在服务模式下运行



