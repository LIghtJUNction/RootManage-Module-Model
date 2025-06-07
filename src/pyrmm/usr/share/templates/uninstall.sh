#!/system/bin/sh
# This script will be executed when the module is being uninstalled
# 该脚本将在模块被卸载时运行

# 获取模块目录
MODDIR=${0%/*}

echo "Uninstalling module..."

# 在这里添加卸载时需要执行的清理操作
# 例如：删除创建的文件、恢复备份等

# 示例：删除创建的临时文件
# rm -rf /data/local/tmp/your_module_files

echo "Module uninstalled successfully"
