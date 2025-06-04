VERSION = "0.3.0"
AUTHOR = "LIghtJUNction"

ui_print "========================================"
ui_print "           GoGogo 模块安装程序           "
ui_print "========================================"
ui_print "模块ID: $MODID"
ui_print "模块路径: /data/adb/modules/$MODID"
# $MODPATH ： modules_update/$MODID
# MODID: gogogo
GO_TAR="$MODPATH/go.tar.gz"
GOROOT="$MODPATH/GOROOT"
GOTMP_DIR="$MODPATH/GOTMP"
GOCACHE_DIR="$MODPATH/GOCACHE"
GOBIN="$MODPATH/GOBIN"
GOROOT_BOOTSTRAP_DIR="$MODPATH/GOROOT_BOOTSTRAP"


basic_check(){
    # 检查设备架构
    ui_print "- 检查设备架构..."
    case "$ARCH" in
        "arm")
            ui_print "设备架构为 ARM 32位"
            abort "不支持32位设备架构: $ARCH"
            ;;
        "arm64")
            ui_print "✅ 设备架构为 ARM 64位，支持"
            ;;
        "x86")
            ui_print "设备架构为 x86 32位"
            abort "不支持32位设备架构: $ARCH"
            ;;
        "x64")
            ui_print "✅ 设备架构为 x86 64位，支持"
            ;;
        *)
            ui_print "未知架构: $ARCH"
            abort "不支持的设备架构: $ARCH"
            ;;
    esac

    ui_print "- Android API 版本: $API"

    # 检测ROOT方案
    ui_print "- 检测ROOT环境..."
    if [ "$KSU" = "true" ]; then
        ui_print "检测到 KernelSU: v$KSU_VER ($KSU_VER_CODE)"
        touch $MODPATH/ksu
        echo $KSU_VER > $MODPATH/ksu

    elif [ "$APATCH" = "true" ]; then
        APATCH_VER=$(cat "/data/adb/ap/version")
        ui_print "检测到 APatch: v$APATCH_VER"
        ui_print "  KERNEL_VERSION: $KERNEL_VERSION"
        ui_print "  KERNELPATCH_VERSION: $KERNELPATCH_VERSION"
        touch $MODPATH/apatch
        echo $APATCH_VER > $MODPATH/apatch

    else
        ui_print "检测到 Magisk: v$MAGISK_VER ($MAGISK_VER_CODE)"
        mv $MODPATH/boot-completed.sh $MODPATH/service.sh
        touch $MODPATH/magisk
        echo $MAGISK_VER > $MODPATH/magisk
    fi
}

basic_check

#  ..%%%%....%%%%....%%%%....%%%%....%%%%....%%%%..
# .%%......%%..%%..%%......%%..%%..%%......%%..%%.
# .%%.%%%..%%..%%..%%.%%%..%%..%%..%%.%%%..%%..%%.
# .%%..%%..%%..%%..%%..%%..%%..%%..%%..%%..%%..%%.
# ..%%%%....%%%%....%%%%....%%%%....%%%%....%%%%..
# ................................................


# 创建必要的目录结构
ui_print "- 创建目录结构..."
mkdir -p "$MODPATH/dist"
mkdir -p "$GOCACHE_DIR"
mkdir -P "$GOBIN"
mkdir -p "$MODPATH/system/bin"
mkdir -p "$GOTMP_DIR" # 临时目录
mkdir -p "$GOROOT" # 当前Go
# mkdir -p "$GOROOT_BOOTSTRAP_DIR" # 开发者用于自举编译新版 用到的旧版或者拷贝的Go  
mkdir -p "$MODPATH/temp_go"
# 设置权限
ui_print "- 设置文件权限..."
chmod 644 "$MODPATH/gogogo.env"

# 设置二进制文件权限
set_perm_recursive "$MODPATH/GOBIN" 0 0 0755 0755
set_perm_recursive "$MODPATH/GOROOT/bin" 0 0 0755 0755
set_perm_recursive "$MODPATH/dist" 0 0 0755 0755


# 解压Go语言压缩包到GOROOT和GOROOT_BOOTSTRAP目录
ui_print "- 正在解压Go语言环境..."
if [ -f "$GO_TAR" ]; then
    # 创建目标目录
  
    # 解压到临时目录
    ui_print "  解压缩中，请稍候..."
    tar -xzf "$GO_TAR" -C "$MODPATH/temp_go"
  
    if [ -d "$MODPATH/temp_go/go" ]; then
        # 复制到GOROOT
        ui_print "  复制到GOROOT目录..."
        cp -rf "$MODPATH/temp_go/go"/* "$GOROOT/"
    
        # # 复制到GOROOT_BOOTSTRAP # 开发者可选
        # ui_print "  复制到GOROOT_BOOTSTRAP目录..."
        # cp -rf "$MODPATH/temp_go/go"/* "$GOROOT_BOOTSTRAP_DIR/"
    
        ui_print "移动版本文件"
        cp -f "$MODPATH/temp_go/go/VERSION" "$MODPATH/VERSION"

        # 清理临时文件
        rm -rf "$MODPATH/temp_go"
        ui_print "  ✓ Go语言环境安装完成！"
    else
        ui_print "  ❌ 解压后未找到go目录，请检查压缩包"
        rm -rf "$MODPATH/temp_go"
    fi
    
    # 移除压缩包
    rm -f "$GO_TAR"
else
    ui_print "  ❌ 未找到Go语言压缩包: $GO_TAR"
fi

# 备份PATH 
echo $PATH > $MODPATH/PATH.bak
ui_print "已备份当前系统环境变量：$PATH"

# 使用说明
ui_print "========================================"
ui_print "            使用说明                    "
ui_print "========================================"
ui_print "模块目录: /data/adb/modules/$MODID"
ui_print "MODPATH: $MODPATH"
ui_print "MODID: $MODID"


ui_print "新增CLI命令: gogogo -- 一键构建为多平台/架构"
ui_print "新增CLI命令: go -- Go编译器命令"
ui_print "新增CLI命令: gofmt -- Go代码格式化工具"

ui_print "运行 gogogo -h 查看帮助"

ui_print "使用教程:"
ui_print "1. 新建项目: go mod init github.com/user_name/repo_name" 
sleep 0.2
ui_print "2. 编写go代码"
sleep 0.3
ui_print "3. 使用gogogo命令进行快捷编译，支持39种平台/架构"
sleep 0.4
ui_print "   (ios amd/arm 暂不支持. android amd 暂不支持)"
ui_print ""
sleep 2
ui_print "4. 从源代码构建gogogo并替换现有命令示例:"
ui_print "   gogogo -s '/data/adb/modules/$MODID/gogogo.go' -p 'android/arm64' -o '/data/adb/modules/$MODID/build'"
sleep 1
ui_print "   移动: mv /data/adb/modules/$MODID/build/gogogo_android_arm64 /system/bin/gogogo"
sleep 2
ui_print ""
ui_print "5. 使用交互式构建（推荐）:"
ui_print "   gogogo -s 'xxx.go' -i"
sleep 4
ui_print ""
ui_print "6. 环境变量已自动配置:"
ui_print "   GOENV=/data/adb/modules/$MODID/gogogo.env"
ui_print "   GOROOT=/data/adb/modules/$MODID/GOROOT"
sleep 0.2
ui_print "   ..."
ui_print "   可以在任意终端中使用Go和gogogo命令"
ui_print ""
ui_print "更新GO,运行action.sh即可，如果使用新版管理器，你会看到一个按钮"
ui_print "安装完成！请重启设备以激活所有功能"
ui_print "========================================"