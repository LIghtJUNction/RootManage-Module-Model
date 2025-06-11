# RMM 命令补全

RMM 支持为多种 shell 生成命令补全脚本，让您在输入命令时享受 Tab 自动补全的便利。

## 支持的 Shell

- **Bash** - Linux/macOS 常用 shell
- **Zsh** - macOS 默认 shell
- **Fish** - 现代交互式 shell
- **PowerShell** - Windows 默认 shell
- **Elvish** - 新一代 shell

## 快速开始

### 生成补全脚本

```bash
# 生成 bash 补全脚本
rmm completion bash

# 生成 zsh 补全脚本
rmm completion zsh

# 生成 fish 补全脚本
rmm completion fish

# 生成 PowerShell 补全脚本
rmm completion powershell

# 生成 elvish 补全脚本
rmm completion elvish
```

### 保存到文件

```bash
# 保存补全脚本到指定文件
rmm completion bash -o ~/.rmm_completion.bash
rmm completion zsh -o ~/.zsh/completions/_rmm
rmm completion fish -o ~/.config/fish/completions/rmm.fish
```

## 安装方法

### Bash

1. **临时启用（当前会话）**：
   ```bash
   eval "$(rmm completion bash)"
   ```

2. **永久启用**：
   ```bash
   # 方法1: 添加到 .bashrc
   echo 'eval "$(rmm completion bash)"' >> ~/.bashrc
   
   # 方法2: 保存到文件并 source
   rmm completion bash > ~/.rmm_completion.bash
   echo 'source ~/.rmm_completion.bash' >> ~/.bashrc
   ```

### Zsh

1. **添加到 .zshrc**：
   ```zsh
   echo 'eval "$(rmm completion zsh)"' >> ~/.zshrc
   ```

2. **使用 zsh 补全目录（推荐）**：
   ```zsh
   # 创建补全目录（如果不存在）
   mkdir -p ~/.zsh/completions
   
   # 生成补全脚本
   rmm completion zsh > ~/.zsh/completions/_rmm
   
   # 确保 fpath 包含补全目录（添加到 .zshrc）
   echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc
   echo 'autoload -U compinit && compinit' >> ~/.zshrc
   ```

### Fish

Fish shell 会自动加载 `~/.config/fish/completions/` 目录中的补全脚本：

```fish
# 确保目录存在
mkdir -p ~/.config/fish/completions

# 生成补全脚本
rmm completion fish > ~/.config/fish/completions/rmm.fish
```

### PowerShell

1. **找到 PowerShell 配置文件位置**：
   ```powershell
   $PROFILE
   ```

2. **添加补全脚本**：
   ```powershell
   # 方法1: 直接添加到配置文件
   rmm completion powershell >> $PROFILE
   
   # 方法2: 动态加载
   Add-Content $PROFILE 'rmm completion powershell | Out-String | Invoke-Expression'
   ```

### Elvish

1. **创建补全文件**：
   ```bash
   mkdir -p ~/.elvish/completions
   rmm completion elvish > ~/.elvish/completions/rmm.elv
   ```

2. **在 `~/.elvish/rc.elv` 中添加**：
   ```elvish
   use ~/.elvish/completions/rmm
   ```

## 功能特性

### 命令补全

补全脚本支持以下功能：

- **主命令补全**：输入 `rmm ` 然后按 Tab 显示所有可用命令
- **子命令补全**：如 `rmm device ` 会显示设备管理相关的子命令
- **选项补全**：补全 `-v`, `--verbose` 等选项
- **参数提示**：显示命令参数的说明信息

### 示例

```bash
# 主命令补全
rmm <Tab>
# 显示: init, build, sync, check, publish, config, run, device, clean, test, completion, help

# 设备子命令补全
rmm device <Tab>
# 显示: list, info, shell, install, uninstall, push, pull, reboot, logs, check, test, help

# 选项补全
rmm build --<Tab>
# 显示: --output, --clean, --debug, --skip-shellcheck, --verbose, --quiet, --help

# completion 命令的 shell 类型补全
rmm completion <Tab>
# 显示: bash, zsh, fish, powershell, elvish
```

## 故障排除

### 补全不工作

1. **确认 RMM 在 PATH 中**：
   ```bash
   which rmm
   ```

2. **重新加载 shell 配置**：
   ```bash
   # Bash
   source ~/.bashrc
   
   # Zsh
   source ~/.zshrc
   
   # Fish
   source ~/.config/fish/config.fish
   ```

3. **重启终端** 或启动新的 shell 会话

### PowerShell 权限问题

如果遇到执行策略问题：

```powershell
# 查看当前执行策略
Get-ExecutionPolicy

# 设置允许本地脚本执行
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

## 验证安装

安装补全脚本后，可以通过以下方式验证：

1. 输入 `rmm ` 然后按 Tab 键，应该看到命令列表
2. 输入 `rmm completion ` 然后按 Tab 键，应该看到支持的 shell 列表
3. 输入 `rmm build --` 然后按 Tab 键，应该看到选项列表

## 更新补全脚本

当 RMM 更新并添加新命令时，需要重新生成补全脚本：

```bash
# 重新生成并替换现有的补全脚本
rmm completion bash > ~/.rmm_completion.bash
rmm completion zsh > ~/.zsh/completions/_rmm
rmm completion fish > ~/.config/fish/completions/rmm.fish
```

## 技术细节

RMM 使用 `clap_complete` crate 自动生成补全脚本，确保：

- 所有命令和选项都被正确识别
- 帮助文本和描述信息准确显示
- 支持嵌套子命令的补全
- 参数类型和约束被正确处理

补全脚本是基于 RMM 的 CLI 定义动态生成的，因此总是与当前版本的命令结构保持同步。
