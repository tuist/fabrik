import { defineConfig } from "vitepress";

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Fabrik",
  description: "Multi-Layer Build Cache Technology",
  ignoreDeadLinks: true,
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: "Home", link: "/" },
      { text: "Guide", link: "/guide/introduction" },
      { text: "Reference", link: "/reference/cli" },
    ],

    sidebar: [
      {
        text: "Guide",
        items: [
          { text: "Introduction", link: "/guide/introduction" },
          { text: "Getting Started", link: "/getting-started" },
          { text: "Authentication", link: "/guide/authentication" },
          { text: "Architecture", link: "/guide/architecture" },
        ],
      },
      {
        text: "Cache",
        items: [
          { text: "Peer to Peer", link: "/cache/p2p" },
          {
            text: "Build Systems",
            items: [
              {
                text: '<span class="sidebar-icon-text"><img src="/images/bazel.svg" class="sidebar-icon" alt="Bazel">Bazel</span>',
                link: "/cache/build-systems/bazel",
              },
              {
                text: '<span class="sidebar-icon-text"><img src="/images/gradle.svg" class="sidebar-icon" alt="Gradle">Gradle</span>',
                link: "/cache/build-systems/gradle",
              },
              {
                text: '<span class="sidebar-icon-text"><img src="/images/metro.svg" class="sidebar-icon" alt="Metro">Metro</span>',
                link: "/cache/build-systems/metro",
              },
              {
                text: '<span class="sidebar-icon-text"><img src="/images/nx.svg" class="sidebar-icon" alt="Nx">Nx</span>',
                link: "/cache/build-systems/nx",
              },
              {
                text: '<span class="sidebar-icon-text"><img src="/images/turborepo-icon.svg" class="sidebar-icon" alt="TurboRepo">TurboRepo</span>',
                link: "/cache/build-systems/turborepo",
              },
              {
                text: '<span class="sidebar-icon-text"><img src="/images/xcode.png" class="sidebar-icon" alt="Xcode">Xcode</span>',
                link: "/cache/build-systems/xcode",
              },
            ],
          },
          {
            text: "Scripts",
            items: [
              { text: "Introduction", link: "/cache/scripts/" },
              { text: "Configuration Reference", link: "/cache/scripts/reference" },
              { text: "Examples", link: "/cache/scripts/examples" },
            ],
          },
        ],
      },
      {
        text: "Reference",
        items: [
          { text: "CLI Commands", link: "/reference/cli" },
          { text: "Configuration File", link: "/reference/config-file" },
          { text: "API Reference", link: "/reference/api" },
        ],
      },
    ],

    socialLinks: [{ icon: "github", link: "https://github.com/tuist/fabrik" }],
  },
});
