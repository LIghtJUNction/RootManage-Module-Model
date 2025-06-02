ui_print "开始安装$MODID"
ui_print "模块路径: $MODPATH"

# 检查设备架构
case "$ARCH" in
    "arm")
        ui_print "设备架构为 ARM 32位"
        abort "不支持32位设备架构: $ARCH"
        ;;
    "arm64")
        ui_print "设备架构为 ARM 64位"
        ;;
    "x86")
        ui_print "设备架构为 x86 32位"
        abort "不支持32位设备架构: $ARCH"
        ;;
    "x64")
        ui_print "设备架构为 x86 64位"
        ;;
    *)
        abort "不支持的设备架构: $ARCH"
        ;;
esac

ui_print "Android API 版本: $API"

if [ "$KSU" = "true" ]; then
  ui_print "- kernelSU version: $KSU_VER ($KSU_VER_CODE)"
  touch $MODPATH/ksu
  echo $KSU_VER > $MODPATH/ksu

elif [ "$APATCH" = "true" ]; then
  APATCH_VER=$(cat "/data/adb/ap/version")
  ui_print "- APatch version: $APATCH_VER"
  ui_print "- KERNEL_VERSION: $KERNEL_VERSION"
  ui_print "- KERNELPATCH_VERSION: $KERNELPATCH_VERSION"
  touch $MODPATH/apatch
  echo $APATCH_VER > $MODPATH/apatch

else
  ui_print "- Magisk version: $MAGISK_VER ($MAGISK_VER_CODE)"
  mv $MODPATH/boot-completed.sh $MODPATH/service.sh
  touch $MODPATH/magisk
  echo $MAGISK_VER > $MODPATH/magisk
fi
# 应该很少有人同时安装两个吧

# 创建必要的目录结构
mkdir -p "$MODPATH/GOCACHE"
mkdir -p "$MODPATH/GOTELEMETRYDIR"
mkdir -p "$MODPATH/go/pkg/mod"
mkdir -p "$MODPATH/go/bin"
mkdir -p "$MODPATH/system/bin"


# 解压Go语言压缩包到GOROOT目录
ui_print "- 正在解压Go语言环境..."
GO_TAR="$MODPATH/go.tar.gz"
GOROOT_DIR="$MODPATH/GOROOT"

if [ -f "$GO_TAR" ]; then
  # 确保GOROOT目录存在
  mkdir -p "$GOROOT_DIR"
  
  # 解压缩文件到GOROOT目录
  tar -xzf "$GO_TAR" -C "$MODPATH"
  
  # 由于解压出来的是go目录，我们需要将内容移动到GOROOT
  if [ -d "$MODPATH/go" ]; then
    mv "$MODPATH/go"/* "$GOROOT_DIR/"
    rm -rf "$MODPATH/go"
    ui_print "  Go语言环境解压完成！"
  else
    ui_print "  ❌ 解压后未找到go目录，请检查压缩包"
  fi
else
  ui_print "  ❌ 未找到Go语言压缩包: $GO_TAR"
fi

# 移除刚刚的压缩包
rm -f "$GO_TAR"

ui_print "模块目录: $MODPATH "
ui_print "给你3秒,请记住模块安装目录"
sleep 3

ui_print "新增CLI命令: gogogo -- 一键构建为多平台/架构"

ui_print "gogogo -h"

ui_print "使用教程"
ui_print "一：新建go mod init github.com/user_name/repo_name" 
ui_print "二：编写go代码"
ui_print "三：使用gogogo命令进行快捷编译，支持39种平台/架构 ，ios amd/arm 暂不支持. android amd 暂不支持"

ui_print "四：实操 从源代码构建gogogo并替换现有的gogogo命令"
ui_print "gogogo -s '/data/adb/modules/gogogo/gogogo.go' -p 'android/arm64' -o './data/adb/modules/gogogo/build'"

ui_print "移动到system/bin目录下 mv /data/adb/modules/gogogo/build/gogogo_android_arm64 /system/bin/gogogo"

ui_print "五：使用gogogo命令进行交互式构建（推荐）"
ui_print "gogogo -s 'xxx.go' -i"

ui_print "六：环境变量已自动配置"
ui_print "  GOENV=/data/adb/modules/gogogo/gogogo.env"
ui_print "  GOROOT=/data/adb/modules/gogogo/GOROOT"
ui_print "  可以在任意终端中使用Go和gogogo命令"

set_perm "$MODPATH/gogogo/bin/gogogo" 0 0 0755
set_perm "$MODPATH/GOROOT/bin/go" 0 0 0755
set_perm "$MODPATH/GOROOT/bin/gofmt" 0 0 0755