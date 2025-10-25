import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Fabrik",
  description: "Multi-Layer Build Cache Technology",
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Guide', link: '/guide/introduction' },
      { text: 'Reference', link: '/reference/cli' }
    ],

    sidebar: [
      {
        text: 'Guide',
        items: [
          { text: 'Introduction', link: '/guide/introduction' }
        ]
      },
      {
        text: 'Build Systems',
        items: [
          { text: 'Xcode', link: '/build-systems/xcode' }
        ]
      },
      {
        text: 'Reference',
        items: [
          { text: 'CLI Commands', link: '/reference/cli' },
          { text: 'Configuration File', link: '/reference/config-file' },
          { text: 'API Reference', link: '/reference/api' }
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/tuist/fabrik' }
    ]
  }
})
