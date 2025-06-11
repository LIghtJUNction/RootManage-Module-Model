import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'RMM 文档',
  description: 'RMM CLI 文档和教程',
  base: '/',
  lastUpdated: true,  // 显示页面最后更新时间
  themeConfig: {
    logo: 'assets/logo.png',
    outline: [2,3],        // 在右侧显示大纲（h2/h3）
    nav: [
      { text: '快速开始', link: '/guide/installation' },
      { text: '使用教程', link: '/guide/usage' },
      {
        text: '命令参考',
        items: [
          { text: 'init', link: '/guide/init' },
          { text: 'build', link: '/guide/build' },
          { text: 'test', link: '/guide/test' },
          { text: 'publish', link: '/guide/publish' },
          { text: 'sync', link: '/guide/sync' },
          { text: 'config', link: '/guide/config' },
          { text: 'deploy', link: '/guide/deploy' }
        ]
      },
      { text: 'GitHub', link: 'https://github.com/LIghtJUNction/RootManage-Module-Model' }
    ],
    sidebar: {
      '/guide/': [
        { text: '快速开始', link: '/guide/installation' },
        { text: '使用教程', link: '/guide/usage' },
        { text: 'rmm init', link: '/guide/init' },
        { text: 'rmm build', link: '/guide/build' },
        { text: 'rmm test', link: '/guide/test' },
        { text: 'rmm publish', link: '/guide/publish' },
        { text: 'rmm sync', link: '/guide/sync' },
        { text: 'rmm config', link: '/guide/config' },
        { text: '部署指南', link: '/guide/deploy' }
      ]
    },
    socialLinks: [
      { icon: 'github', link: 'https://github.com/LIghtJUNction/RootManage-Module-Model' }
    ],
    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2025 LIghtJUNction'
    },
    editLink: {
      pattern: 'https://github.com/LIghtJUNction/RootManage-Module-Model/edit/main/website/docs/:path',
      text: '在 GitHub 上编辑此页'
    }
  }
})
