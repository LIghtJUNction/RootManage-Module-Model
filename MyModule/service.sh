#!/system/bin/sh
# GoGogo模块启动脚本

# 设置Go环境变量
export GOENV=/data/adb/modules/gogogo/gogogo.env 

# export GOROOT=/data/adb/modules/gogogo/GOROOT
# export GOPATH=/data/adb/modules/gogogo/go
# export GOCACHE=/data/adb/modules/gogogo/GOCACHE
# export GOTELEMETRYDIR=/data/adb/modules/gogogo/GOTELEMETRYDIR
# export GO111MODULE=on
# export GOMODCACHE=/data/adb/modules/gogogo/go/pkg/mod

# 记录日志
echo "GoGogo模块已启动 $(date)" >> /data/adb/modules/gogogo/gogogo.log