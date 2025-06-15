# TEST Module

这是一个 rmm 模块项目。

## 说明

RMMP ID: TEST

## 安装

1. 使用 ROOT 管理器安装此模块
2. 重启设备

## 开发

```bash
# 构建模块
rmm build

# 安装到设备
rmm device install

# 运行测试
rmm test
```

## 文件结构

```
TEST
├── .rmmp/              # RMM 项目文件
│   ├── Rmake.toml     # 构建配置
│   ├── build/         # 构建输出
│   └── dist/          # 发布文件
├── system/            # 系统文件覆盖
├── module.prop        # 模块属性
├── customize.sh       # 安装脚本
├── rmmproject.toml    # 项目配置
└── README.md          # 说明文档
```

## 许可证

见 LICENSE 文件。
