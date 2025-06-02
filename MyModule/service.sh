#!/system/bin/sh
# GoGogo模块启动脚本

# 设置Go环境变量
export GOENV=/data/adb/modules/gogogo/gogogo.env
export GOROOT=/data/adb/modules/gogogo/GOROOT
export GOPATH=/data/adb/modules/gogogo/go
export GOCACHE=/data/adb/modules/gogogo/GOCACHE
export GOTELEMETRYDIR=/data/adb/modules/gogogo/GOTELEMETRYDIR
export GO111MODULE=on
export GOMODCACHE=/data/adb/modules/gogogo/go/pkg/mod

# 添加Go和gogogo命令到PATH
export PATH=$PATH:/data/adb/modules/gogogo/GOROOT/bin:/data/adb/modules/gogogo/system/bin

# 设置系统属性以便其他应用程序可以检测到Go环境
resetprop -p go.env.path /data/adb/modules/gogogo/gogogo.env
resetprop -p go.installed true
resetprop -p go.version "1.24.3"

# 记录日志
echo "GoGogo模块已启动 $(date)" >> /data/adb/modules/gogogo/gogogo.log