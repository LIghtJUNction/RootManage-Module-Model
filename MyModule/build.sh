#!/bin/sh
id=$(awk -F= '/id=/ {print $2}' ./module.prop)
name=$(awk -F= '/name=/ {print $2}' ./module.prop)
version=$(awk -F= '/version=/ {print $2}' ./module.prop)
versionCode=$(awk -F= '/versionCode=/ {print $2}' ./module.prop)
author=$(awk -F= '/author=/ {print $2}' ./module.prop)
description=$(awk -F= '/description=/ {print $2}' ./module.prop)
#从module.prop获取信息

zip -r "${name}-${version}(${versionCode})-by${author}.zip" ./
#打包模块
