---
title: 快速开始
---

# 快速开始

RMM 是一个使用 Rust 开发的命令行工具，用于初始化、构建、测试和发布 RMM 模块。


## 使用 Python Pip 安装

安装 `pyrmm`：

```bash
pip install pyrmm
```

然后可以使用：

```bash
python -m pyrmm init <project_path>
```

## uv 安装
如果你使用 `uv` 可以通过以下命令安装：

```bash
uv tool install pyrmm
```

任何命令都可以通过以下方式查看帮助：

```bash
rmm xxx --help
```

## 从源代码安装

git clone RMM 源代码：

```bash
git clone https://github.com/LIghtJUNction/RootManageModuleModel.git

cd RootManageModuleModel

然后你可以修改.python-version文件来指定 Python 版本。

uv sync # 使用该版本

maturin develop

uv build # 构建项目

uv tool install -e .
&& 
uv tool install -e . --force

# 安装完成，然后使用 rmm即可启动
rmm init <project_path>


```