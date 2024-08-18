#!/bin/sh

version=$(awk -F= '/version=/ {print $2}' my module/module.prop)
#从module.prop获取版本

zip -r "my module-${version}.zip" ./my\ module
#打包模块
