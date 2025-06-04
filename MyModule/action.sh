#!system/bin/sh
MODDIR=${0%/*}
GOROOT_BOOTSTRAP_DIR="$MODDIR/GOROOT_BOOTSTRAP"
GOROOT_DIR="$MODDIR/GOROOT"


Key_monitoring() {
    while :; do
        event_info=$(getevent -qlc 1)
        case "$event_info" in
            *KEY_VOLUMEUP*) 
                echo "0"  
                break
                ;;
            *KEY_VOLUMEDOWN*) 
                echo "1"  
                break
                ;;
            *KEY_POWER*)
                echo "exit" 
                break
                ;;
            *)
                continue
                ;;
        esac
    done
}

ui_print "音量+ : 从源代码本地构建go && 启动开发者模式"
ui_print "音量- : 检查更新-更新GOPROXY(代理加速)"
ui_print "电源键 : 按键退出 && 关闭开发者模式"    

choice=$(Key_monitoring)

update_go() {
    ui_print "正在更新Go..."
    go get -u all || exit 1
    ui_print "Go更新完成"
    exit 0
}

build_from_src() {
    ui_print "正在从源代码构建Go..."
    ui_print "这将会启用开发者模式"
    
    if [ ! -d "$MODDIR/GOROOT" ]; then
        ui_print "错误：未找到Go源代码目录，请确保已正确安装Go源代码。"
        exit 1
    fi

    # 读取开发者模式标志
    if [ ! -f "$MODDIR/gogogo.dev" ]; then
        ui_print "开启开发者模式..."
        echo "1" > $MODDIR/gogogo.dev
    fi


    mkdir -p $GOROOT_BOOTSTRAP_DIR
    export GOROOT_BOOTSTRAP=$GOROOT_BOOTSTRAP_DIR
    # 将GOROOT 拷贝到GOROOT_BOOTSTRAP
    cp -r $GOROOT_DIR/* $GOROOT_BOOTSTRAP_DIR/
    echo "1" > $MODDIR/gogogo.dev


    if [ ! -s "$GOROOT_BOOTSTRAP_DIR" ]; then
        ui_print "错误: GOROOT_BOOTSTRAP目录为空或不存在,正在复制GOROOT。"
        cp -r $GOROOT_DIR/* $GOROOT_BOOTSTRAP_DIR/
    fi


    ui_print "开始构建Go源代码..." # 重定向到标准输出
    cd $MODDIR/GOROOT/src
    . make.bash
    ui_print "Go源代码构建完成"

    choice=$(Key_monitoring)
    ui_print "音量上：保持开发者模式（不删除自举拷贝） 音量下：退出开发者模式（删除自举拷贝） "
    if [ "$choice" = "0" ]; then
        ui_print "选择：保持开发者模式"
        exit 0
    elif [ "$choice" = "1" ]; then
        ui_print "选择：退出开发者模式"
        echo "0" > $MODDIR/gogogo.dev
        rm -rf $GOROOT_BOOTSTRAP_DIR
        ui_print "已删除自举拷贝"
        exit 0
    else
        ui_print "无效选择，退出脚本"
        exit 0
    fi

}

if [ "$choice" = "0" ]; then
    build_from_src || ui_print "构建失败！PATH: $PATH"

elif [ "$choice" = "1" ]; then
    ui_print "当前Go版本:"
    which go
    go version || ui_print "未找到Go！"
    ui_print "本地记录的go版本："
    ui_print "$(cat $MODDIR/VERSION || ui_print "未记录版本")"

    ui_print "检查最新GO版本..."
    latest_go_version=$(curl -s https://go.dev/VERSION?m=text)
    ui_print "最新GO版本: $latest_go_version"

    # 检查是否需要更新
    if [ "$(go version | awk '{print $3}')" != "$latest_go_version" ]; then
        ui_print "有新版本可用，正在更新..."
        if update_go; then
            echo "$latest_go_version" > "$MODDIR/VERSION"
            ui_print "更新成功，已记录版本：$latest_go_version"
        else
            ui_print "更新失败，请检查网络连接或手动更新"
        fi
    else
        ui_print "已是最新版本"
    fi

else
    ui_print "退出 && 关闭开发者模式"
    echo "0" > $MODDIR/gogogo.dev
    exit 0
fi