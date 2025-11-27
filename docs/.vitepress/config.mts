import { defineConfig } from "vitepress";

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Fabrik",
  description: "Multi-Layer Build Cache Technology",
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
            collapsed: false,
            items: [
              {
                text: "Bazel",
                link: "/cache/build-systems/bazel",
              },
              {
                text: "Gradle",
                link: "/cache/build-systems/gradle",
              },
              {
                text: "Metro",
                link: "/cache/build-systems/metro",
              },
              {
                text: "Nx",
                link: "/cache/build-systems/nx",
              },
              {
                text: "TurboRepo",
                link: "/cache/build-systems/turborepo",
              },
              {
                text: "Xcode",
                link: "/cache/build-systems/xcode",
              },
            ],
          },
          {
            text: "Recipes",
            collapsed: true,
            items: [
              {
                text: "Standard",
                collapsed: true,
                items: [
                  { text: "Introduction", link: "/cache/recipes/standard/" },
                  {
                    text: "Annotations Reference",
                    link: "/cache/recipes/standard/annotations",
                  },
                  {
                    text: "Examples",
                    link: "/cache/recipes/standard/examples",
                  },
                ],
              },
              {
                text: "Portable",
                collapsed: true,
                items: [
                  { text: "Introduction", link: "/cache/recipes/portable/" },
                  {
                    text: "JavaScript API Reference",
                    link: "/cache/recipes/api-reference",
                  },
                  {
                    text: "Syntax Reference",
                    link: "/cache/recipes/portable/syntax",
                  },
                  {
                    text: "Examples",
                    link: "/cache/recipes/portable/examples",
                  },
                ],
              },
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
