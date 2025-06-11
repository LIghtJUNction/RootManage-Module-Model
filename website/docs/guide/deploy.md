---
title: 部署指南
---

# 部署 RMM 文档网站

本指南介绍如何将 RMM 文档网站（基于 VitePress）部署到 GitHub Pages、Netlify 或其他静态网站托管服务。

## 前提条件

- 已安装 Node.js (>=14.x) 和 npm/yarn。
- 已在项目根目录下（包含 `website` 文件夹）。

## 本地预览

1. 进入 `website` 目录：
   ```bash
   cd website
   ```
2. 安装依赖：
   ```bash
   npm install
   # 或者使用 yarn
   yarn
   ```
3. 启动开发服务器：
   ```bash
   npm run dev
   ```
4. 注意：文中不应出现本地链接。请将 `http://localhost:5173` 替换为文档内有效链接或移除。

## 生成静态文件

1. 仍在 `website` 目录下，运行：
   ```bash
   npm run build
   ```
2. 构建输出目录：
   - 默认输出在 `website/docs/.vitepress/dist`。可以在项目根目录下通过 `.vitepress/config.js` 修改 `outDir`。

## GitHub Pages 部署

1. 在仓库根目录创建 GitHub Actions Workflow，例如：`.github/workflows/gh-pages.yml`

   ```yaml
   name: Deploy Docs
   on:
     push:
       branches: [ main ]
   jobs:
     deploy:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3
         - name: Use Node.js
           uses: actions/setup-node@v3
           with:
             node-version: '16'
         - name: Install dependencies
           working-directory: ./website
           run: npm install
         - name: Build site
           working-directory: ./website
           run: npm run build
         - name: Deploy to GitHub Pages
           uses: peaceiris/actions-gh-pages@v3
           with:
             github_token: ${{ secrets.GITHUB_TOKEN }}
             publish_dir: website/docs/.vitepress/dist
   ```
2. 提交并推送到 `main` 分支，GitHub Actions 会自动构建并部署到 GitHub Pages。
3. 在仓库设置 (Settings → Pages) 中，将发布源设置为 `gh-pages` 分支的根目录。

## Netlify 部署

1. 登录 Netlify，选择 "New site from Git"。
2. 连接到 GitHub 仓库，选择对应项目。
3. 在 Build settings 中：
   - Build command: `npm run build`
   - Publish directory: `website/docs/.vitepress/dist`
4. 点击 Deploy，即可完成部署。

## 其他托管方案

- 使用 Surge、Cloudflare Pages、Vercel 等静态托管服务。
- 将 `dist` 目录上传到任何支持静态文件的服务器。

更多内容，请参考 VitePress 官方文档：https://vitepress.vuejs.org/zh/guide/deploy.html
