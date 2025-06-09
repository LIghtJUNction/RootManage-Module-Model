"""模块基本脚本
This file is part of PyRMM.

CUSTOMIZE_SH: customize.sh
SERVERS_SH: servers.sh
...

"""

CUSTOMIZE_SH = """
# This file is part of PyRMM.
ui_print "开始安装模块..."

"""

SERVERS_SH = """
# This file is part of PyRMM.
"""

README = """
# {project_name}

一个基于 RMM (Root Module Manager) 的模块项目。

## 功能特性

- 支持 Magisk、APatch、KernelSU
- 自动版本管理
- 构建输出优化
- GitHub 集成

## 安装方法

1. 下载最新的 release 文件
2. 通过 Magisk/APatch/KernelSU 安装模块
3. 重启设备

## 构建

```bash
# 构建模块
rmm build

# 发布到 GitHub
rmm publish
```

## 开发

```bash
# 安装开发依赖
uv tool install pyrmm

# 初始化项目
rmm init .

# 构建并测试
rmm build && rmm test
```

## 许可证

MIT License - 查看 [LICENSE](LICENSE) 文件了解详情。

## 作者

- {author_name}

---

使用 [RMM](https://github.com/LIghtJUNction/RootManage-Module-Model) 构建

"""

from datetime import datetime
CHANGELOG = f"""
# 更新日志

所有对该项目的重要更改都会记录在此文件中。

## [未发布]

### 新增
- 初始项目设置
- 基本模块结构

### 变更
- 无

### 修复
- 无

## [1.0.0] - {datetime.now().strftime('%Y-%m-%d')}

### 新增
- 项目初始版本
- 基本功能实现

---

## 版本格式说明

- **[未发布]** - 即将发布的更改
- **[版本号]** - 已发布的版本及发布日期

### 更改类型

- **新增** - 新功能
- **变更** - 现有功能的更改
- **弃用** - 即将移除的功能
- **移除** - 已移除的功能
- **修复** - Bug 修复
- **安全** - 安全相关的修复
"""

LICENSE = """
# LICENSES        
# ADD YOUR LICENSES HERE

# RMM Project License
MIT License

Copyright (c) 2025 LIghtJUNction

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"""

