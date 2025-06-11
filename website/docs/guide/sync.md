---
title: 同步命令
---

# rmm sync

`rmm sync [OPTIONS]`

同步全局配置中的 RMM 项目列表，发现新项目或移除无效项目。

## 用法示例

```bash
# 自动发现和同步项目
rmm sync

# 仅同步项目列表，不更新项目元数据
rmm sync --projects-only

# 指定最大搜索深度为 2
rmm sync --max-depth 2
```

## 选项

- `--projects-only`：仅同步项目列表，跳过元数据更新。
- `--max-depth <N>`：搜索项目的最大目录深度。
