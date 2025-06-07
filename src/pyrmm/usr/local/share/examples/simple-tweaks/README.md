# Simple System Tweaks Example Module

这是一个简单的系统调优示例模块，展示了KernelSU模块开发的最佳实践。

## 功能特性

- **性能优化**: 调整虚拟内存、网络和文件系统参数
- **电池优化**: 配置CPU调速器和禁用不必要的功能
- **稳定性改进**: 调整OOM killer和内核参数
- **完整的卸载支持**: 提供完整的卸载和恢复功能

## 文件结构

```
simple-tweaks/
├── module.prop          # 模块属性文件
├── service.sh           # 主服务脚本
├── uninstall.sh         # 卸载脚本
└── README.md           # 说明文档
```

## 安装要求

- Android 5.0+ (API 21)
- KernelSU v0.6.0+
- Root权限

## 使用方法

1. 下载模块文件
2. 通过KernelSU Manager安装
3. 重启设备以应用调优

## 调优内容

### 性能调优
- 虚拟内存交换频率优化
- 磁盘写入缓存优化
- 网络TCP参数调优
- 存储设备预读优化

### 电池优化
- CPU调速器策略调整
- 蓝牙ERTM禁用
- 保守的频率管理

### 稳定性改进
- 文件描述符限制提升
- OOM killer行为优化
- 内核崩溃处理改进

## 注意事项

- 所有调优都是保守和安全的
- 提供完整的卸载恢复功能
- 兼容大多数Android设备
- 不会修改系统分区文件

## 日志和调试

模块运行日志会写入 `/data/adb/modules_log`，可以通过以下命令查看：

```bash
adb shell cat /data/adb/modules_log | grep "Simple System Tweaks"
```

## 许可证

MIT License - 详见LICENSE文件
