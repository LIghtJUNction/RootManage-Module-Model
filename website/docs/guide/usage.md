---
title: 使用教程
---

# 使用教程

本章节介绍 RMM 命令行工具的常用命令和使用示例。

## 1. 初始化项目

使用 `rmm init` 快速创建一个新的 RMM 模块项目：

```bash
# 在当前目录初始化
rmm init .

# 指定路径并自动确认
rmm init ./my_module -y

# 创建库项目
rmm init ./my_lib --lib

# 创建 RAVD 项目
rmm init ./my_ravd --ravd
```

初始化完成后，会生成以下目录结构：

```
my_module/
├── .rmmp/              # RMM 配置目录
│   └── Rmake.toml      # 构建配置
├── CHANGELOG.MD        # 更新日志
├── LICENSE             # 许可证
├── README.MD           # 项目说明
├── module.prop         # Magisk 兼容模块属性
└── system/             # 模块文件目录
```

## 2. 构建模块

使用 `rmm build` 构建并打包模块：

```bash
# 默认构建
rmm build

# 指定输出目录
rmm build -o ./dist

# 清理构建目录并构建
rmm build --clean

# 启用调试模式
rmm build --debug

# 跳过 shellcheck 检查
rmm build --skip-shellcheck
```

构建成功后，输出目录（默认 `.rmmp/dist`）会生成：

- `<id>-<versionCode>.zip` （模块包）
- `*-source.tar.gz` （源码包）
- `update.json`（版本更新信息）

## 3. 运行测试

使用 `rmm test` 对项目进行脚本或单元测试：

```bash
# 运行所有测试
rmm test

# 包含 shellcheck 检查
rmm test --shellcheck
```

## 4. 发布模块

将构建好的模块发布到 GitHub Releases：

```bash
# 发布正式版本
rmm publish

# 发布为草稿
rmm publish --draft

# 发布预发布版本
rmm publish --prerelease
```

## 5. 同步项目列表

同步全局配置中的项目（自动发现或移除无效项目）：

```bash
# 发现并同步新项目
rmm sync

# 仅同步项目列表，不更新元数据
rmm sync --projects-only

# 指定最大搜索深度
rmm sync --max-depth 3
```

## 6. 管理配置

查看或修改 RMM 全局配置：

```bash
# 查看当前配置
rmm config

# 设置用户名
rmm config --user.name "Your Name"

# 设置邮箱
rmm config --user.email you@example.com
```

## 7. 执行自定义脚本

在 `Rmake.toml` 中定义脚本后，可以使用 `rmm run` 调用：

```bash
# 列出可用脚本
cat .rmmp/Rmake.toml | grep '\[scripts\]' -A 10

# 运行脚本
rmm run <script_name>
```

## 8. 其他命令

- `rmm clean`：清理项目目录
- `rmm check`：检查项目语法和配置
- `rmm completion`：生成 shell 补全脚本
- `rmm help <command>`：查看命令帮助

更多详细说明，请参考各命令的 `--help` 输出。
