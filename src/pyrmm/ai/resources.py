SHELLSCRIPTBESTPRACTICES = """
# Shell 脚本编写最佳实践

## 基本规范

### 脚本头部
```bash
#!/system/bin/sh
# 脚本描述
# 作者: 你的名字
# 版本: 1.0.0
# 最后修改: 2025-06-11

set -e  # 遇到错误立即退出
set -u  # 使用未定义变量时报错
```

### 变量命名
```bash
# 使用有意义的变量名
MODULE_NAME="my_module"
VERSION_CODE=1
INSTALL_PATH="/data/adb/modules"

# 只读变量使用 readonly
readonly SYSTEM_ROOT="/system"
readonly DATA_ROOT="/data"
```

## 错误处理

### 检查命令执行结果
```bash
# 方法1: 使用 if 语句
if ! command -v busybox >/dev/null 2>&1; then
    echo "错误: busybox 未找到"
    exit 1
fi

# 方法2: 使用 || 操作符
cp source dest || {
    echo "错误: 复制文件失败"
    exit 1
}
```

### 文件和目录检查
```bash
# 检查文件存在
if [ ! -f "/system/build.prop" ]; then
    echo "错误: build.prop 不存在"
    exit 1
fi

# 检查目录存在
if [ ! -d "/data/adb" ]; then
    mkdir -p "/data/adb" || exit 1
fi

# 检查权限
if [ ! -w "/system" ]; then
    echo "错误: 无写入权限"
    exit 1
fi
```

## 函数编写

### 标准函数结构
```bash
# 函数定义
log_info() {
    local message="$1"
    echo "[INFO] $(date '+%Y-%m-%d %H:%M:%S'): $message"
}

log_error() {
    local message="$1"
    echo "[ERROR] $(date '+%Y-%m-%d %H:%M:%S'): $message" >&2
}

# 带返回值的函数
check_root() {
    if [ "$(id -u)" -eq 0 ]; then
        return 0  # 成功
    else
        return 1  # 失败
    fi
}
```

### 参数处理
```bash
# 解析命令行参数
while [ $# -gt 0 ]; do
    case $1 in
        --verbose)
            VERBOSE=true
            ;;
        --output)
            OUTPUT_DIR="$2"
            shift
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            echo "未知参数: $1"
            exit 1
            ;;
    esac
    shift
done
```

## 字符串操作

### 安全的字符串处理
```bash
# 使用双引号包围变量
echo "用户输入: $user_input"

# 处理包含空格的文件名
for file in "/path/to/files"/*; do
    if [ -f "$file" ]; then
        echo "处理文件: $file"
    fi
done

# 字符串比较
if [ "$status" = "success" ]; then
    echo "操作成功"
fi
```

### 路径处理
```bash
# 获取目录名和文件名
filepath="/data/adb/modules/module.zip"
dirname=$(dirname "$filepath")   # /data/adb/modules
basename=$(basename "$filepath") # module.zip

# 安全的路径拼接
config_file="$MODULE_DIR/config.conf"
```

## 系统交互

### 属性操作
```bash
# 读取系统属性
android_version=$(getprop ro.build.version.release)
device_model=$(getprop ro.product.model)

# 设置属性 (需要 root)
resetprop ro.debuggable 1
```

### 服务管理
```bash
# 检查服务状态
if pgrep -f "service_name" >/dev/null; then
    echo "服务正在运行"
else
    echo "服务已停止"
fi

# 启动服务
start_service() {
    local service_name="$1"
    if ! start "$service_name"; then
        log_error "启动服务失败: $service_name"
        return 1
    fi
    log_info "服务启动成功: $service_name"
}
```

## ShellCheck 常见规则

### SC2086: 变量需要引号
```bash
# 错误
cp $source $dest

# 正确
cp "$source" "$dest"
```

### SC2034: 未使用的变量
```bash
# 如果变量确实未使用，添加注释
# shellcheck disable=SC2034
unused_var="value"
```

### SC2155: 分离声明和赋值
```bash
# 错误
local result=$(some_command)

# 正确
local result
result=$(some_command)
```

## 调试技巧

### 启用调试模式
```bash
#!/system/bin/sh
# 调试模式
set -x  # 显示执行的命令

# 或者有条件启用
if [ "$DEBUG" = "true" ]; then
    set -x
fi
```

### 日志记录
```bash
# 重定向到日志文件
exec > "/data/local/tmp/script.log" 2>&1

# 或者使用 tee 同时输出到屏幕和文件
exec > >(tee "/data/local/tmp/script.log") 2>&1
```

## 性能优化

### 减少外部命令调用
```bash
# 使用内置命令替代外部命令
# 错误 (多次调用 echo)
for i in 1 2 3; do
    echo "$i"
done

# 正确 (一次性输出)
printf '%s\n' 1 2 3
```

### 避免重复操作
```bash
# 缓存命令结果
if [ -z "$cached_result" ]; then
    cached_result=$(expensive_command)
fi
```

这些实践将帮助你编写更加健壮、可维护的 Shell 脚本。
"""

MODULEDEVGUIDE = """
# Magisk 模块开发指南

## 模块结构

### 必需文件
- `module.prop` - 模块属性文件
- `META-INF/com/google/android/` - 安装脚本目录

### 可选文件
- `service.sh` - 开机启动脚本
- `post-fs-data.sh` - 文件系统挂载后执行
- `uninstall.sh` - 卸载脚本
- `customize.sh` - 自定义安装逻辑

## module.prop 配置

```properties
id=模块ID
name=模块名称
version=v1.0.0
versionCode=1
author=作者名称
description=模块描述
updateJson=更新链接
```

## 脚本编写规范

### service.sh
```bash
#!/system/bin/sh
# 服务脚本在系统启动完成后执行
# 可以使用所有系统 API

# 检查设备状态
if [ ! -f /data/data/com.topjohnwu.magisk/busybox ]; then
    exit 1
fi

# 执行模块逻辑
# ...
```

### post-fs-data.sh
```bash
#!/system/bin/sh
# 在 /data 挂载后立即执行
# 此时系统尚未完全启动

# 早期初始化逻辑
# ...
```

## 系统修改最佳实践

### 文件替换
1. 将原始文件复制到模块目录
2. 修改复制的文件
3. Magisk 会自动替换系统文件

### 属性修改
```bash
# 在 service.sh 中修改系统属性
resetprop ro.debuggable 1
resetprop ro.secure 0
```

### 服务管理
```bash
# 启动服务
start 服务名

# 停止服务
stop 服务名

# 重启服务
restart 服务名
```

## 调试技巧

### 日志记录
```bash
# 写入 Magisk 日志
echo "调试信息" >> /cache/magisk.log

# 写入自定义日志
echo "$(date): 模块日志" >> /data/local/tmp/module.log
```

### 模块状态检查
```bash
# 检查模块是否启用
[ -d "/data/adb/modules/模块ID" ] && echo "模块已安装"

# 检查模块文件
ls -la /data/adb/modules/模块ID/
```

## 兼容性考虑

### Android 版本
- Android 5.0+ (API 21+)
- 不同版本的系统路径可能不同

### 架构支持
- arm64-v8a (主流)
- armeabi-v7a (旧设备)
- x86_64 (模拟器)

### Root 方案
- Magisk (推荐)
- KernelSU (新兴)
- APatch (备选)

## 发布流程

1. **测试验证**: 在多个设备上测试
2. **版本标记**: 更新 versionCode
3. **生成包**: 使用 RMM 构建
4. **发布说明**: 编写 CHANGELOG
5. **社区分享**: 发布到相关社区

## 安全注意事项

1. **权限最小化**: 只请求必要权限
2. **数据保护**: 不收集用户隐私
3. **代码审查**: 确保代码安全性
4. **签名验证**: 使用可信的签名

## 社区资源

- Magisk 官方文档
- XDA 开发者论坛
- GitHub 模块仓库
- 中文社区支持
"""
RMMCLIHELP = """
# RMM (Root Module Manager) 使用指南

RMM 是一个高性能的 Magisk/APatch/KernelSU 模块开发工具，使用 Rust 编写以提供卓越的性能。

## 主要功能

### 项目管理
- `rmm init <name>` - 初始化新项目
- `rmm build` - 构建项目
- `rmm clean` - 清理构建产物
- `rmm sync` - 同步项目元数据

### 模块测试
- `rmm test` - 测试模块
- `rmm check` - 语法检查
- `rmm run <script>` - 运行自定义脚本

### 设备管理
- `rmm device list` - 列出连接的设备
- `rmm device install` - 安装模块到设备
- `rmm device info` - 获取设备信息

### 发布管理
- `rmm publish` - 发布到 GitHub Release
- `rmm config` - 配置管理

### 实用工具
- `rmm completion` - 生成shell补全脚本
- `rmm mcp` - 启动 MCP 服务器

## 项目模板

### basic
标准的 Magisk 模块模板，包含：
- module.prop - 模块属性
- service.sh - 服务脚本
- customize.sh - 安装脚本

### library
库模块模板，适用于提供API的模块

### ravd
Root AVD 模板，适用于模拟器环境

## 配置选项

### 全局配置
- `username` - Git 用户名
- `email` - Git 邮箱
- `proxy.enabled` - 是否启用代理
- `proxy.github` - GitHub 代理地址

### 项目配置
- `module.id` - 模块ID
- `module.name` - 模块名称
- `module.version` - 模块版本
- `module.description` - 模块描述
- `module.author` - 模块作者

## 最佳实践

1. **版本管理**: 使用语义化版本控制
2. **脚本编写**: 遵循 ShellCheck 建议
3. **测试策略**: 在真机和模拟器上都进行测试
4. **文档完善**: 维护 README 和 CHANGELOG

## 故障排除

### 常见问题
1. **构建失败**: 检查 ShellCheck 报告
2. **设备连接**: 确保 ADB 调试已开启
3. **权限问题**: 确保设备已获得 root 权限

### 调试技巧
- 使用 `--debug` 模式获取详细输出
- 查看构建日志定位问题
- 使用 `rmm check` 预先检查语法

## 社区资源

- GitHub: https://github.com/LIghtJUNction/RootManageModuleModel
- 文档: 查看项目 README.md
- 问题反馈: 提交 GitHub Issues
"""