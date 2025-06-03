#!system/bin/sh
MODID=${0%/*}

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
echo "音量- : 检查更新"
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
    cd $MODPATH/GOROOT/src 
    . make.bash || exit 1
    echo "Go源代码构建完成"
    exit 0
}



if [ "$choice" = "0" ]; then
    build_from_src || echo "构建失败！PATH: $PATH"


elif [ "$choice" = "1" ]; then
    echo "当前Go版本:"
    which go
    go version || echo "未找到Go！"
    echo "本地记录的go版本："
    echo "$(cat $MODPATH/VERSION || echo "未记录版本")"

    echo "检查最新GO版本..."
    latest_go_version=$(curl -s https://go.dev/VERSION?m=text)
    echo "最新GO版本: $latest_go_version"

    # 检查是否需要更新
    if [ "$(go version | awk '{print $3}')" != "$latest_go_version" ]; then
        echo "有新版本可用，正在更新..."
        if update_go; then
            echo "$latest_go_version" > "$MODPATH/VERSION"
            echo "更新成功，已记录版本：$latest_go_version"
        else
            build_choice=$(Key_monitoring)

        fi
    else
        echo "已是最新版本"
    fi

else
    echo "退出"
    exit 0
fi