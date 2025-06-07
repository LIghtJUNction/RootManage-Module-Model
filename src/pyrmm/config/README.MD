# 为什么将RMMROOT默认设置为 $HOME/data/adb/.rmm/ ?

## 如果不设计为$HOME/.rmm

> 当用户在mt管理器或者在termux运行：不会报错 $HOME 均为可读写的目录
> 当用户在adb shell / magisk运行脚本的环境 运行：会报错 因为$HOME : /
> 因此将RMMROOT默认设置为 $HOME/data/adb/.rmm/ 以兼容windows / linux /android
> 当然欢迎设置 RMMROOT 环境变量 自定义路径 

## 为什么使用python3.12?
> 因为在termux中，python3.12是能直接通过pkg install python 安装到的最新版本。（包括 uv python install）


