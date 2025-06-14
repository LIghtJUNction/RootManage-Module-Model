#!/system/bin/sh
# KernelSU 模块自定义安装脚本

# 检查设备信息
ui_print "- 设备架构: $ARCH"
ui_print "- Android API: $API"
ui_print "- KernelSU 版本: $KSU_VER"

# 根据设备架构进行不同的处理
case $ARCH in
    arm64)
        ui_print "- 64位ARM设备"
        ;;
    arm)
        ui_print "- 32位ARM设备"
        ;;
    x64)
        ui_print "- x86_64设备"
        ;;
    x86)
        ui_print "- x86设备"
        ;;
esac

# 根据Android版本进行处理
# 示例shellcheck 自动修复 $API -> "$API"
if [ $API -lt 29 ]; then
    ui_print "- Android 10以下版本"
else
    ui_print "- Android 10及以上版本"
fi

# 设置权限（如果需要）
# set_perm_recursive $MODPATH/system/bin 0 0 0755 0755
# set_perm $MODPATH/system/etc/example.conf 0 0 0644

# 示例：删除系统文件（取消注释以使用）
# REMOVE="
# /system/app/SomeSystemApp
# /system/etc/some_config_file
# "

# 示例：替换系统目录（取消注释以使用）
# REPLACE="
# /system/app/SomeSystemApp
# "

ui_print "- 模块安装完成"
