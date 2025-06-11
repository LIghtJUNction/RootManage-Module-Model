---
title: 初始化命令
---

# rmm init

`rmm init [PATH] [OPTIONS]`

用于创建新的 RMM 模块项目，生成项目结构和初始文件。

## 用法示例

- 在当前目录初始化项目：
  ```bash
  rmm init .
  ```
- 指定路径并自动确认：
  ```bash
  rmm init ./my_module -y
  ```
- 创建库（Library）项目：
  ```bash
  rmm init ./my_lib --lib
  ```
- 创建 RAVD 项目：
  ```bash
  rmm init ./my_ravd --ravd
  ```

## 参数

- `PATH`：项目路径，默认为当前目录 `.`。

## 选项

- `-y, --yes`：自动确认所有提示。
- `--lib`：创建库项目结构。
- `--ravd`：创建 RAVD 项目结构。

## 生成内容

- `.rmmp/Rmake.toml`：构建配置文件。
- `rmmproject.toml`：项目元数据。
- `README.MD`、`CHANGELOG.MD`、`LICENSE`、`module.prop` 等基础文件。
