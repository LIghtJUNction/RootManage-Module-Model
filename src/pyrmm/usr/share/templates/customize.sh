#!/sbin/sh
# KernelSU/Magisk 模块安装脚本
# 这个脚本在模块安装时会被调用

# 检测运行环境
if [ "$KSU" = "true" ]; then
    ui_print "- KernelSU environment detected"
else
    ui_print "- Magisk environment detected"
fi

ui_print "- Installing module..."

# 可用的环境变量：
# KSU: 如果运行在 KernelSU 中，此值为 true
# MODPATH: 模块安装路径
# MODDIR: 等同于 $MODPATH
# TMPDIR: 临时目录
# ZIPFILE: 模块安装包路径
# ARCH: 设备架构 (arm, arm64, x86, x64)
# API: Android API 级别

# 设置权限示例
# set_perm_recursive $MODPATH/system/bin 0 0 0755 0755

# KernelSU 特有功能：
# 删除系统文件/目录（使用 REMOVE 变量）
# REMOVE="
# /system/app/UnwantedApp
# /system/priv-app/Bloatware
# "

# 替换系统目录（使用 REPLACE 变量） 
# REPLACE="
# /system/app/SystemUI
# /system/framework
# "

# 手动设置目录为不透明（完全替换）
# setfattr -n trusted.overlay.opaque -v y $MODPATH/system/app/YourApp

ui_print "- Module installed successfully!"
