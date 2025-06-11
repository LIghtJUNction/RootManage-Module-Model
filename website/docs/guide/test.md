---
title: 测试命令
---

# rmm test

`rmm test [OPTIONS]`

对 RMM 项目执行脚本测试或 Shell 脚本语法检查。

## 用法示例

```bash
# 运行所有测试
rmm test

# 同时执行 shellcheck 检查
rmm test --shellcheck
```

## 选项

- `--shellcheck`：启用 Shell 脚本语法检查。
