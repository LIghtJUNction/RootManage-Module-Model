# KernelSU模块开发快速入门指南
# KernelSU Module Development Quick Start Guide

## 简介 | Introduction

本指南将帮助您快速开始KernelSU模块开发。KernelSU是一个内核级的root解决方案，支持Android设备的系统级修改。

This guide will help you quickly get started with KernelSU module development. KernelSU is a kernel-level root solution that supports system-level modifications on Android devices.

## 环境准备 | Environment Setup

### 1. 初始化开发环境

```bash
# 设置开发环境
setup-dev-env

# 或者手动初始化
source /usr/local/lib/dev-environment.sh
init_dev_environment
```

### 2. 验证环境

```bash
# 检查开发环境状态
get_dev_status

# 验证工具可用性
module-builder --version
module-validator --version
```

## 创建第一个模块 | Create Your First Module

### 1. 创建基础模块

```bash
# 创建基础模块项目
mkmodule my-first-module basic

# 或者使用完整命令
create_project "my-first-module" "basic" "my-first-module"
```

### 2. 项目结构

```
my-first-module/
├── .kernelsu-project       # 项目配置文件
├── build.conf             # 构建配置文件
├── module.prop            # 模块属性文件
├── service.sh             # 服务脚本
├── uninstall.sh           # 卸载脚本
├── META-INF/
│   └── com/google/android/
│       ├── updater-script # 安装脚本
│       └── update-binary  # 安装程序
└── .vscode/               # VS Code配置
    ├── tasks.json
    ├── launch.json
    └── settings.json
```

### 3. 编辑模块属性

编辑 `module.prop` 文件：

```properties
id=my_first_module
name=My First Module
version=v1.0.0
versionCode=1
author=YourName
description=My first KernelSU module
```

### 4. 编写服务脚本

编辑 `service.sh` 文件：

```bash
#!/system/bin/sh
# 服务脚本在系统启动后运行

# 示例：修改系统属性
resetprop ro.debuggable 1

# 示例：创建文件
touch /data/my_module_running

# 输出日志
echo "My First Module: Service started" >> /data/adb/modules_log
```

## 构建和测试 | Build and Test

### 1. 验证模块

```bash
# 验证模块结构和配置
validate
# 或者
module-validator .
```

### 2. 构建模块

```bash
# 构建模块
build
# 或者
module-builder .
```

### 3. 打包模块

```bash
# 打包模块为ZIP文件
pack
# 或者
module-packager .
```

## 部署模块 | Deploy Module

### 1. 部署到设备

```bash
# 通过ADB部署模块
deploy-module --module my-first-module.zip

# 或者手动部署
adb push my-first-module.zip /data/local/tmp/
adb shell su -c "cp /data/local/tmp/my-first-module.zip /data/adb/modules/"
```

### 2. 安装模块

在设备上通过KernelSU Manager或命令行安装：

```bash
# 命令行安装（需要root权限）
su -c "cd /data/adb/modules && unzip -o my-first-module.zip"
```

## 高级功能 | Advanced Features

### 1. WebUI模块

创建带有Web界面的模块：

```bash
mkmodule my-webui-module webui
```

### 2. Magisk兼容模块

创建与Magisk兼容的模块：

```bash
mkmodule my-compat-module magisk-compat
```

### 3. 系统修改模块

创建修改系统文件的模块：

```bash
mkmodule my-system-module system-modifier
```

## 调试技巧 | Debugging Tips

### 1. 启用调试模式

```bash
# 启用详细输出
debug-on

# 构建时显示详细信息
MODULE_DEBUG=true module-builder .
```

### 2. 查看日志

```bash
# 查看系统日志
adb logcat | grep KernelSU

# 查看模块日志
adb shell cat /data/adb/modules_log
```

### 3. 验证安装

```bash
# 检查模块状态
adb shell su -c "ls -la /data/adb/modules/"

# 验证文件权限
adb shell su -c "ls -la /data/adb/modules/my_first_module/"
```

## 最佳实践 | Best Practices

### 1. 代码质量

- 使用ShellCheck验证脚本语法
- 添加错误处理和日志输出
- 遵循Shell脚本编程规范

### 2. 兼容性

- 测试不同Android版本
- 支持不同架构（arm64, arm, x86）
- 检查KernelSU版本兼容性

### 3. 安全性

- 最小权限原则
- 验证用户输入
- 安全地处理敏感数据

### 4. 文档

- 编写清晰的README文件
- 添加代码注释
- 提供使用示例

## 故障排除 | Troubleshooting

### 常见问题

1. **模块无法安装**
   - 检查module.prop格式
   - 验证ZIP文件结构
   - 确认KernelSU版本兼容性

2. **模块无法启动**
   - 检查service.sh脚本权限
   - 验证脚本语法
   - 查看系统日志

3. **功能不正常**
   - 检查SELinux策略
   - 验证文件路径
   - 确认权限设置

### 获取帮助

- 查看官方文档：https://kernelsu.org/
- 参考示例模块：/usr/local/share/examples/
- 使用内置帮助：`module-builder --help`

## 下一步 | Next Steps

1. 阅读完整的API文档
2. 查看更多示例模块
3. 加入KernelSU社区
4. 贡献代码和文档

---

## 相关资源 | Related Resources

- [KernelSU官方网站](https://kernelsu.org/)
- [KernelSU GitHub仓库](https://github.com/tiann/KernelSU)
- [模块开发API参考](/usr/include/kernelsu-module.h)
- [Shell编程最佳实践](/usr/include/shell-utils.h)
