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
                text: "Local",
                collapsed: true,
                items: [
                  { text: "Introduction", link: "/cache/recipes/local/" },
                  {
                    text: "Configuration Reference",
                    link: "/cache/recipes/local/reference",
                  },
                  { text: "Examples", link: "/cache/recipes/local/examples" },
                ],
              },
              {
                text: "Remote",
                collapsed: true,
                items: [
                  { text: "Introduction", link: "/cache/recipes/remote/" },
                  {
                    text: "JavaScript API Reference",
                    link: "/cache/recipes/api-reference",
                  },
                  {
                    text: "Syntax Reference",
                    link: "/cache/recipes/remote/syntax",
                  },
                  { text: "Examples", link: "/cache/recipes/remote/examples" },
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
