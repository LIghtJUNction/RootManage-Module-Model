---
title: 构建命令
---

# rmm build

`rmm build [OPTIONS]`

用于构建当前 RMM 模块项目，执行预构建、构建、后构建和打包流程。

## 用法示例

```bash
# 默认构建
rmm build

# 指定输出目录
rmm build -o ./dist

# 清理构建目录再构建
rmm build --clean

# 启用调试模式
rmm build --debug

# 跳过 shellcheck 检查
rmm build --skip-shellcheck
```

## 选项

- `-o, --output <PATH>`：构建输出目录。
- `-c, --clean`：清理输出目录后再构建。
- `-d, --debug`：启用调试模式。
- `--skip-shellcheck`：跳过 Shell 脚本语法检查。
