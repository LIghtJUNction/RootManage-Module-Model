#!/system/bin/sh
# lience MIT
# Magisk GoGogo 模块环境变量配置文件
# 该文件由 Magisk 模块安卓脚本生成
# 作者 : $LIghtJUNction

# 检查环境变量是否已设置，避免重复执行
if [ "$GOGOGO_ENV_LOADED" = "1" ]; then
    # 环境已加载，直接返回
    return 0 2>/dev/null || exit 0
fi

echo "正在加载 必需 环境变量..."

# Go 环境变量设置 - 一次性设置和导出
export GOROOT=/data/adb/modules/gogogo/GOROOT \
       GOPATH=/data/adb/modules/gogogo/go \
       GOCACHE=/data/adb/modules/gogogo/GOCACHE \
       GOENV=/data/adb/modules/gogogo/gogogo.env \
       GOTELEMETRYDIR=/data/adb/modules/gogogo/GOTELEMETRYDIR \
       GOTMPDIR=/data/adb/modules/gogogo/tmp \
       GOMODCACHE=/data/adb/modules/gogogo/go/pkg/mod \
       GO111MODULE=on

# 高效的 PATH 设置函数 - 减少进程调用和字符串操作
setup_path() {
    local old_path="$1"
    local add_paths="$2"
    local new_path=""
    local p
    
    # 创建带分隔符的路径字符串用于快速查询
    local paths_with_sep=":${old_path}:"
    
    # 1. 优先添加 /system/bin (如存在)
    if [ "${paths_with_sep#*:/system/bin:}" != "$paths_with_sep" ]; then
        new_path="/system/bin"
        # 在已处理列表中标记
        paths_with_sep="${paths_with_sep//:\/system\/bin:/:DONE:}"
    fi
    
    # 2. 添加非/0/路径
    IFS=":"
    for p in $old_path; do
        # 跳过空路径和已处理路径
        [ -n "$p" ] || continue
        [ "${paths_with_sep#*:$p:}" = "$paths_with_sep" ] && continue
        
        # 跳过/0/路径(稍后处理)
        case "$p" in
            */0/*) continue ;;
        esac
        
        # 添加到新路径
        [ -z "$new_path" ] && new_path="$p" || new_path="${new_path}:${p}"
        # 标记为已处理
        paths_with_sep="${paths_with_sep//:$p:/:DONE:}"
    done
    
    # 3. 添加新Go路径(如果不存在)
    for p in ${add_paths//:/ }; do
        [ -n "$p" ] || continue
        if [ "${paths_with_sep#*:$p:}" = "$paths_with_sep" ]; then
            [ -z "$new_path" ] && new_path="$p" || new_path="${new_path}:${p}"
            paths_with_sep="${paths_with_sep}${p}:DONE:"
        fi
    done
    
    # 4. 最后添加/0/目录路径
    for p in $old_path; do
        [ -n "$p" ] || continue
        case "$p" in
            */0/*)
                # 检查是否未处理
                if [ "${paths_with_sep#*:$p:}" != "$paths_with_sep" ] && [ "${paths_with_sep#*:DONE:}" != "${paths_with_sep}" ]; then
                    [ -z "$new_path" ] && new_path="$p" || new_path="${new_path}:${p}"
                fi
                ;;
        esac
    done
    
    echo "$new_path"
}

# 添加Go相关路径 - 直接设置
export PATH=$(setup_path "$PATH" "/data/adb/modules/gogogo/GOROOT/bin:/data/adb/modules/gogogo/go/bin")

# 设置标志表明环境已加载
export GOGOGO_ENV_LOADED=1