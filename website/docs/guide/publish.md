---
title: 发布命令
---

# rmm publish

`rmm publish [OPTIONS]`

将构建好的 RMM 模块发布到 GitHub Releases。

## 用法示例

```bash
# 发布正式版本
rmm publish

# 发布为草稿版本
rmm publish --draft

# 发布为预发布版本
rmm publish --prerelease
```

## 选项

- `--draft`：将发布标记为草稿，不公开。
- `--prerelease`：将发布标记为预发布。
