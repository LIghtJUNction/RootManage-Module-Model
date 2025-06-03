VERSION = "0.2.0"
AUTHOR = "LIghtJUNction"

ui_print "========================================"
ui_print "           GoGogo 模块安装程序           "
ui_print "========================================"
ui_print "模块ID: $MODID"
ui_print "模块路径: /data/adb/modules/$MODID"
# $MODPATH ： modules_update/$MODID
# MODID: gogogo

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

#  ..%%%%....%%%%....%%%%....%%%%....%%%%....%%%%..
# .%%......%%..%%..%%......%%..%%..%%......%%..%%.
# .%%.%%%..%%..%%..%%.%%%..%%..%%..%%.%%%..%%..%%.
# .%%..%%..%%..%%..%%..%%..%%..%%..%%..%%..%%..%%.
# ..%%%%....%%%%....%%%%....%%%%....%%%%....%%%%..
# ................................................


# 创建必要的目录结构
ui_print "- 创建目录结构..."
mkdir -p "$MODPATH/GOCACHE"
mkdir -p "$MODPATH/GOTELEMETRYDIR"
mkdir -p "$MODPATH/go/pkg/mod"
mkdir -p "$MODPATH/go/bin"
mkdir -p "$MODPATH/system/bin"
mkdir -p "$MODPATH/system/etc/profile.d"

# 解压Go语言压缩包到GOROOT和GOROOT_BOOTSTRAP目录
ui_print "- 正在解压Go语言环境..."
GO_TAR="$MODPATH/go.tar.gz"
GOROOT_DIR="$MODPATH/GOROOT"
GOROOT_BOOTSTRAP_DIR="$MODPATH/GOROOT_BOOTSTRAP"

if [ -f "$GO_TAR" ]; then
    # 创建目标目录
    mkdir -p "$GOROOT_DIR"
    mkdir -p "$GOROOT_BOOTSTRAP_DIR"
    mkdir -p "$MODPATH/temp_go"
  
    # 解压到临时目录
    ui_print "  解压缩中，请稍候..."
    tar -xzf "$GO_TAR" -C "$MODPATH/temp_go"
  
    if [ -d "$MODPATH/temp_go/go" ]; then
        # 复制到GOROOT
        ui_print "  复制到GOROOT目录..."
        cp -rf "$MODPATH/temp_go/go"/* "$GOROOT_DIR/"
    
        # 复制到GOROOT_BOOTSTRAP
        ui_print "  复制到GOROOT_BOOTSTRAP目录..."
        cp -rf "$MODPATH/temp_go/go"/* "$GOROOT_BOOTSTRAP_DIR/"
    
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

# 获取当前时间
CURRENT_TIME=$(date +"%Y-%m-%d %H:%M:%S")

# 创建环境变量配置文件
ui_print "- 创建环境变量配置..."
cat > "$MODPATH/gogogo.env" << EOF
# lience MIT
# Magisk GoGogo 模块环境变量配置文件
# 该文件由 Magisk 模块安卓脚本生成
# 时间 : $CURRENT_TIME
# 版本 : $VERSION
# 作者 : $AUTHOR
GOPROXY=https://goproxy.cn,direct
GOSUMDB=sum.golang.google.cn
GOTOOLCHAIN=auto

EOF

# 创建gogogorc文件 (会被go命令源文件调用)
cat > "$MODPATH/gogogorc" << 'EOF'
#!/system/bin/sh
MODDIR=${0%/*}
# lience MIT
# Magisk GoGogo 模块环境变量配置文件
# 该文件由 Magisk 模块安卓脚本生成
# 作者 : $LIghtJUNction

echo "正在加载 必需 环境变量..."
export GOROOT=$MODDIR/GOROOT
export GOPATH=$MODDIR/go
export GOCACHE=$MODDIR/GOCACHE
export GOENV=$MODDIR/gogogo.env
export GOTELEMETRYDIR=$MODDIR/GOTELEMETRYDIR
export GOTMPDIR=$MODDIR/tmp
export GOMODCACHE=$MODDIR/go/pkg/mod
export GO111MODULE=on
echo "原始PATH: $PATH"

setup_path() {
    local old_path="$1"
    local add_paths="$2"
    local new_path=""
    local seen_paths=""
    local system_bin_path=""
    local zero_paths=""
    local normal_paths=""
    
    # 分割PATH为数组并处理
    IFS=':'
    for p in ${old_path}; do
        # 跳过空路径
        [ -z "$p" ] && continue
        
        # 检查是否已存在此路径
        echo "$seen_paths" | grep -q ":$p:"
        if [ $? -ne 0 ]; then
            # 将路径添加到已见列表
            seen_paths="$seen_paths:$p:"
            
            # 分类处理路径
            if [ "$p" = "/system/bin" ]; then
                system_bin_path="/system/bin"
            elif echo "$p" | grep -q "/0/"; then
                zero_paths="$zero_paths:$p"
            else
                normal_paths="$normal_paths:$p"
            fi
        else
            echo "跳过重复路径: $p"
        fi
    done
    
    # 添加新路径并检查重复
    for p in ${add_paths//:/ }; do
        echo "$seen_paths" | grep -q ":$p:"
        if [ $? -ne 0 ]; then
            seen_paths="$seen_paths:$p:"
            normal_paths="$normal_paths:$p"
        else
            echo "跳过重复的新路径: $p"
        fi
    done
    
    # 构建新PATH - 优先/system/bin
    if [ -n "$system_bin_path" ]; then
        new_path="/system/bin"
    fi
    
    # 添加普通路径
    for p in ${normal_paths//:/ }; do
        [ -n "$p" ] && new_path="$new_path:$p"
    done
    
    # 添加包含/0/的路径到末尾
    for p in ${zero_paths//:/ }; do
        [ -n "$p" ] && new_path="$new_path:$p"
    done
    
    # 移除开头的冒号(如果有)
    new_path="${new_path#:}"
    
    echo "$new_path"
}

# 添加Go相关路径
GO_PATHS="$MODDIR/GOROOT/bin:$MODDIR/go/bin"
echo "正在设置PATH..."
export PATH=$(setup_path "$PATH" "$GO_PATHS")
echo "优化后PATH: $PATH"
EOF

# 自举构建需要/system/bin/提前。不然会报错

# 创建/system/bin/go脚本
cat > "$MODPATH/system/bin/go" << 'EOF'
#!/system/bin/sh
# 加载环境变量
. ../../gogogorc || echo "备用加载路径" && . /data/adb/modules/gogogo/gogogorc 
# 执行真正的go命令
exec ../../GOROOT/bin/go "$@" || echo "备用加载路径" && exec /data/adb/modules/gogogo/GOROOT/bin/go "$@"
EOF

# 创建/system/bin/gofmt脚本
cat > "$MODPATH/system/bin/gofmt" << 'EOF'
#!/system/bin/sh
# 加载环境变量
. ../../gogogorc || echo "备用加载路径" && . /data/adb/modules/gogogo/gogogorc 
# 执行gofmt命令
exec ../../GOROOT/bin/gofmt "$@" || echo "备用加载路径" && exec /data/adb/modules/gogogo/GOROOT/bin/gofmt "$@"
EOF


# 创建/system/bin/gogogo脚本
cat > "$MODPATH/system/bin/gogogo" << 'EOF'
#!/system/bin/sh
# 加载环境变量
. ../../gogogorc || echo "备用加载路径" && . /data/adb/modules/gogogo/gogogorc 
# 执行gogogo命令
exec ../../gogogo/bin/gogogo "$@" || echo "备用加载路径" && exec /data/adb/modules/gogogo/gogogo/bin/gogogo "$@"
EOF


# 设置权限
ui_print "- 设置文件权限..."
chmod 755 "$MODPATH/system/bin/go"
chmod 755 "$MODPATH/system/bin/gogogo"
chmod 644 "$MODPATH/gogogo.env"
chmod 755 "$MODPATH/gogogorc"

# 设置二进制文件权限
set_perm "$MODPATH/gogogorc" 0 0 0755
set_perm_recursive "$MODPATH/GOROOT/bin" 0 0 0755 0755
set_perm_recursive "$MODPATH/go/bin" 0 0 0755 0755

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