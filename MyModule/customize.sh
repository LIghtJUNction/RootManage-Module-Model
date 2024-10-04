#  customize.sh 脚本说明
#
# 脚本功能：
# 1. 打印自定义安装过程的开始信息。
# 2. 检查设备架构，并根据架构类型打印相应信息或终止安装。
# 3. 检查 Android API 版本，确保版本在支持范围内，否则终止安装。
# 4. 设置指定文件和目录的权限。
# 5. 打印自定义安装过程的完成信息。
#
# 脚本详细说明：
# - ui_print: 用于在安装过程中打印信息到控制台。
# - case "$ARCH" in ... esac: 检查设备架构，支持 "arm", "arm64", "x86", "x64" 四种架构。
# - abort: 用于终止安装过程并打印错误信息。
# - if [ "$API" -lt 23 ]; then ... fi: 检查 Android API 版本，要求版本不低于 23。
# - set_perm: 设置单个文件的权限。
# - set_perm_recursive: 递归设置目录及其内容的权限。




# 示例 customize.sh

# 打印信息到控制台
ui_print "开始自定义安装过程..."

# 检查设备架构
case "$ARCH" in
    "arm" | "arm64")
        ui_print "设备架构为 ARM"
        ;;
    "x86" | "x64")
        ui_print "设备架构为 x86"
        ;;
    *)
        abort "不支持的设备架构: $ARCH"
        ;;
esac

# 检查 Android API 版本
if [ "$API" -lt 23 ]; then
    abort "不支持的 Android 版本: $API"
fi

# 设置文件权限
set_perm "$MODPATH/somefile" 0 0 0644
set_perm_recursive "$MODPATH/somedir" 0 0 0755 0644

ui_print "自定义安装过程完成"
