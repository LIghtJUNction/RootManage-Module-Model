#!/bin/sh
$MODDIR=${0%/*}
GOBIN="$MODDIR/GOBIN"

# API URL
API_URL="https://api.akams.cn/github"
export PATH="$GOBIN:$PATH"

sort_proxies(){
    # 发送 GET 请求并获取 JSON 响应
    response=$(curl -s "$API_URL")

    # 检查 curl 是否成功
    if [ $? -ne 0 ]; then
        echo "错误: 无法访问 API"
        exit 1
    fi

    # 使用 jq 解析 JSON 并按 speed 降序排序
    sorted_proxies=$(echo "$response" | jq -r '.data | sort_by(-.speed)[] | "\(.url) \(.speed)"')

    # 检查 jq 是否成功
    if [ $? -ne 0 ]; then
        echo "错误: 解析 JSON 失败"
        exit 1
    fi

    # 输出排序后的代理节点
    echo "代理节点 URL 和 speed（按 speed 降序排序）："
    echo "$sorted_proxies"
}

# 替换 Go 代理