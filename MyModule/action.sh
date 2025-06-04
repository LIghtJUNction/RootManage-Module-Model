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

echo "音量+ : 从源代码本地构建go"
echo "音量- : 检查更新-更新GOPROXY(代理加速)"
echo "电源键 : 退出"    

choice=$(Key_monitoring)

update_go() {
    echo "正在更新Go..."
    go get -u all || exit 1
    echo "Go更新完成"
    exit 0
}

build_from_src() {
    echo "正在从源代码构建Go..."
    echo "这将会启用开发者模式"
    
    if [ ! -d "$MODDIR/GOROOT" ]; then
        echo "错误：未找到Go源代码目录，请确保已正确安装Go源代码。"
        exit 1
    fi

    # 读取开发者模式标志
    if [ ! -f "$MODDIR/gogogo.dev" ]; then
        echo "错误：未找到开发者模式标志文件，请先启用开发者模式。"
        exit 1
    fi

    if [ "$(cat $MODDIR/gogogo.dev)" != "1" ]; then
        mkdir -p $GOROOT_BOOTSTRAP_DIR
        export GOROOT_BOOTSTRAP=$GOROOT_BOOTSTRAP_DIR
        # 将GOROOT 拷贝到GOROOT_BOOTSTRAP
        cp -r $GOROOT_DIR/* $GOROOT_BOOTSTRAP_DIR/
        echo "1" > $MODDIR/gogogo.dev
    fi

    if [ ! -s "$GOROOT_BOOTSTRAP_DIR" ]; then
        echo "错误: GOROOT_BOOTSTRAP目录为空或不存在，该目录意外受损，请尝试删除gogogo.dev并重试。"
        exit 1
    fi


    echo "开始构建Go源代码..." # 重定向到标准输出
    cd $MODDIR/GOROOT/src
    . make.bash > /dev/null 2>&1 || {
        echo "构建失败，请检查错误信息。"
        exit 1
    }
    echo "Go源代码构建完成"

    choice=$(Key_monitoring)
    echo "音量上：保持开发者模式（不删除自举拷贝） 音量下：退出开发者模式（删除自举拷贝） "
    if [ "$choice" = "0" ]; then
        echo "选择：保持开发者模式"
        exit 0
    elif [ "$choice" = "1" ]; then
        echo "选择：退出开发者模式"
        echo "0" > $MODDIR/gogogo.dev
        rm -rf $GOROOT_BOOTSTRAP_DIR
        echo "已删除自举拷贝"
    fi


    exit 0
}

if [ "$choice" = "0" ]; then
    build_from_src || echo "构建失败！PATH: $PATH"

elif [ "$choice" = "1" ]; then
    echo "当前Go版本:"
    which go
    go version || echo "未找到Go！"
    echo "本地记录的go版本："
    echo "$(cat $MODDIR/VERSION || echo "未记录版本")"

    echo "检查最新GO版本..."
    latest_go_version=$(curl -s https://go.dev/VERSION?m=text)
    echo "最新GO版本: $latest_go_version"

    # 检查是否需要更新
    if [ "$(go version | awk '{print $3}')" != "$latest_go_version" ]; then
        echo "有新版本可用，正在更新..."
        if update_go; then
            echo "$latest_go_version" > "$MODDIR/VERSION"
            echo "更新成功，已记录版本：$latest_go_version"
        else
            echo "更新失败，请检查网络连接或手动更新"
        fi
    else
        echo "已是最新版本"
    fi

else
    echo "退出"
    exit 0
fi