[![Alt](https://repobeats.axiom.co/api/embed/4dbcdf8b2d24156dcf08cef7cc801d9adb317cae.svg "Repobeats analytics image")](https://github.com/LIghtJUNction/RootManage-Module-Model/)
# RMM 传统模式
> 运行Action Workflow
> 完成完整的构建流程

不包含以下功能：
- prebuild script ❌ 不支持 编译前脚本
- postbuild script ❌ 不支持 编译后脚本
- 分模板初始化功能 ❌ 不支持
- 多项目合并构建 ❌ 不支持
- 依赖管理 ❌ 不支持
- 多模块合并 ❌ 不支持
- 模块仓库 ❌ 不支持
- AI测试&审计&优化&修复  ❌ 不支持
- Telegram / Discord 通知 / QQ / 酷安 模块推送功能 ❌ 不支持
- 代理加速 ❌ 不支持
- 虚拟机仿真模块测试 ❌ 不支持
- 模块构建日志 ❌ 不支持
- 快捷安装至物理机 ❌ 不支持
- GPG 签名 ❌ 不支持

# RMM 新模式


> 运行Action Workflow
> apt install pyrmm
> pyrmm build & test $ release 一条龙服务
- 在任意地方运行而无需新建github仓库 ☑️ 
- 支持 prebuild script / postbuild script ☑️
- 支持分模板初始化功能 ☑️ 
- 支持多项目合并构建 ☑️
- 支持依赖管理 ☑️
- 支持多模块合并 ☑️
- 支持模块仓库 ☑️
- 支持 AI 测试&审计&优化&修复 ☑️
- 支持 Telegram / Discord 通知 / QQ / 酷安 模块推送功能 ☑️
- 支持代理加速 ☑️
- 支持虚拟机仿真模块测试 ☑️
- 支持模块构建日志 ☑️
- 支持快捷安装至物理机 ☑️
- 支持 GPG 签名 ☑️



## 使用方法
### 安装 uv (推荐)
uv tool install pyrmm 
或者要使用AI功能，请使用以下命令：
uv tool install pyrmm[ai]

(如果你是开发者，请使用 在项目根目录uv tool install -e .)

### 使用pip
pip install pyrmm

### 使用 pyrmm

pyrmm init my-first-magisk-module
(= pyrmm init my-first-magisk-module)
cd my-first-magisk-module
pyrmm build
pyrmm test

推荐使用：
pyrmm init .

这样，项目根目录作为模块根目录
单仓库单模块（推荐）

# DEV

### rootAVD:
致谢：[rootAVD](https://gitlab.com/newbit/rootAVD)
示例命令：
.\rootAVD.bat "system-images\android-36\google_apis\x86_64\ramdisk.img"


WIN + R 输入以下命令

%LOCALAPPDATA%\Android\Sdk\system-images

system-images\android-36\google_apis\x86_64\ramdisk.img 需要替换为实际路径







### RMM正式启动时间 2025-06-07 高考首日
预祝各位考生金榜题名，前程似锦！

让我们携手构建一个更庞大的模块生态系统！
Let's build a bigger module ecosystem together!
# License
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

# 声明
本开源项目旨在促进模块生态系统的发展和创新。
拥抱AI技术，提升模块开发效率和质量。

> 前提
- 具备一定的Python编程基础
- 熟悉基本的命令行操作
- 了解模块化开发的基本概念
- 开启静态类型检查，等级为strict 

# 贡献

我们欢迎任何形式的贡献，包括但不限于：

- 提交问题和建议
- 提交代码和文档的改进
- 参与讨论和社区活动
请遵循以下步骤进行贡献：

1. Fork 本仓库
2. 创建一个新的分支

> vscode 启动
> 注意开启类型检查工具
> 不接受的PR:
 - 类型检查爆红的PR
 - 大量使用过时的Python语法或库的PR
 - 破坏包独立性的PR
> 作者我会认真审查每个PR
 - 如果你的PR被拒绝，我会给出详细的理由。如果你提供邮件，我会通过邮件通知你。
 - 如果你的PR被接受，我会在合并时注明你的贡献。

> 如果你有任何问题或建议，请随时联系我。
 - LIghtJUNction.me@gmail.com
 - 本仓库已加入PROJECT: RMM 
 - 请多多提交功能建议，BUG反馈
 - 团队会在项目中进行跟踪ISSUE

3. 提交你的改动
4. 提交 Pull Request

感谢你的支持与贡献！

# 致谢名单
> Credits
    Kernel-Assisted Superuser: The KernelSU idea.
    Magisk: The powerful root tool.
    genuine: APK v2 signature validation.
    Diamorphine: Some rootkit skills.
    KernelSU: The kernel based root solution.
    APATCH : The kernel based root solution.