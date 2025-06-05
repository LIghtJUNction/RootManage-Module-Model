#!system/bin/sh
MODDIR=${0%/*}
GOROOT_BOOTSTRAP="$MODDIR/GOROOT_BOOTSTRAP"
GOROOT="$MODDIR/GOROOT"

MAKEBASH="$GOROOT/src/make.bash"


whoami

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

echo "音量+ : 从源代码本地构建go && 启动开发者模式"
echo "音量- : 检查更新-更新GOPROXY(代理加速)"
echo "电源键 : 按键退出 && 关闭开发者模式"    

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
        echo "开启开发者模式..."
        echo "1" > $MODDIR/gogogo.dev
    fi


    mkdir -p $GOROOT_BOOTSTRAP
    export GOROOT_BOOTSTRAP=$GOROOT_BOOTSTRAP
    # 将GOROOT 拷贝到GOROOT_BOOTSTRAP
    cp -r $GOROOT/* $GOROOT_BOOTSTRAP/
    echo "1" > $MODDIR/gogogo.dev


    if [ ! -s "$GOROOT_BOOTSTRAP" ]; then
        echo "错误: GOROOT_BOOTSTRAP目录为空或不存在,正在复制GOROOT。"
        cp -r $GOROOT/* $GOROOT_BOOTSTRAP/
        chmod -R 755 $GOROOT_BOOTSTRAP/bin/
    fi


    echo "开始构建Go源代码..." # 重定向到标准输出
    echo $PATH
    echo $GOROOT_BOOTSTRAP
    cd $GOROOT/src || { echo "错误：无法进入Go源代码目录"; exit 1; }
    . $MAKEBASH | tee -a $MODDIR/build.log 2>&1
    cat $MODDIR/build.log

    echo "Go源代码构建完成"

    cat $GOROOT/src/build.log

    echo "音量上：保持开发者模式（不删除自举拷贝） 音量下：退出开发者模式（删除自举拷贝） "
    
    choice=$(Key_monitoring)
    
    if [ "$choice" = "0" ]; then
        echo "选择：保持开发者模式"
        exit 0
    elif [ "$choice" = "1" ]; then
        echo "选择：退出开发者模式"
        echo "0" > $MODDIR/gogogo.dev
        rm -rf $GOROOT_BOOTSTRAP
        echo "已删除自举拷贝"
        exit 0
    else
        echo "无效选择，退出脚本"
        exit 0
    fi

}

if [ "$choice" = "0" ]; then
    build_from_src || echo "构建失败！PATH: $PATH"

elif [ "$choice" = "1" ]; then
    echo "当前Go版本:"
    go version || echo "未找到Go！"
    echo "本地记录的go版本："
    echo "$(cat $MODDIR/VERSION || echo "未记录版本")"

    echo "检查最新GO版本..."
    latest_go_version=$(curl -s https://go.dev/VERSION?m=text)
    echo "最新GO版本: $latest_go_version"

    # 提取当前Go版本号（去掉go前缀，只保留数字版本）
    current_go_version=$(go version 2>/dev/null | awk '{print $3}' | sed 's/go//')
    latest_version_number=$(echo "$latest_go_version" | sed 's/go//')
    
    echo "当前版本号: $current_go_version"
    echo "最新版本号: $latest_version_number"

    # 检查是否需要更新
    if [ "$current_go_version" != "$latest_version_number" ]; then
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
    echo "退出 && 关闭开发者模式"
    echo "0" > $MODDIR/gogogo.dev
    exit 0
fi