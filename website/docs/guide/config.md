---
title: 配置命令
---

# rmm config

`rmm config [OPTIONS]`

查看或修改 RMM 全局配置。

## 用法示例

```bash
# 打印当前配置
rmm config

# 设置用户名
rmm config --user.name "Your Name"

# 设置邮箱
rmm config --user.email you@example.com
```

## 选项

- `--user.name <NAME>`：设置全局用户名。
- `--user.email <EMAIL>`：设置全局用户邮箱。
