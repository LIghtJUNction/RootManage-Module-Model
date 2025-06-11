#!/system/bin/sh

# RMM 模块自定义脚本
# 此脚本在模块安装时执行，用于进行必要的设置和配置

MODDIR=${0%/*}

# 打印安装信息
ui_print "- 正在安装 RMM 模块..."
ui_print "- 模块目录: $MODDIR"

# 设置权限
set_perm_recursive "$MODDIR" 0 0 0755 0644

# 自定义安装逻辑
# 在这里添加您的安装步骤

ui_print "- 模块安装完成"
