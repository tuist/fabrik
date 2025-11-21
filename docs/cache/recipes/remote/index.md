# Remote Recipes

Remote recipes allow you to execute portable JavaScript recipes directly from Git repositories without manually downloading or managing them. This enables easy sharing and reuse of build automation across teams and projects.

> [!IMPORTANT]
> **Why JavaScript?** Remote recipes use JavaScript because Fabrik embeds the **QuickJS runtime** (with AWS LLRT modules) directly into the binary. This means:
> - **Zero external dependencies** - No need to install Node.js, Deno, or any other runtime
> - **Guaranteed cross-platform** - Works identically on macOS, Linux, and Windows
> - **Fast startup** - QuickJS starts in milliseconds, making recipe execution instant
> - **Small binary size** - Embedded runtime adds minimal overhead to Fabrik's binary
>
> Unlike local script recipes (which require bash, python, etc. to be installed), remote recipes work out-of-the-box on any system with Fabrik installed.

## Overview

Remote recipes use the `@` prefix syntax to reference recipes stored in Git repositories:

```bash
fabrik run @org/repo/path/script.js
```

> [!NOTE] Git as Distribution Mechanism
> Git repositories are used purely as a distribution mechanism for sharing recipes. Recipes are self-contained JavaScript files that cannot depend on other recipes or import external modules. Each recipe runs independently with access to Fabrik's built-in APIs only.

Fabrik automatically:
- Fetches the repository using `git clone --depth 1`
- Caches it locally following XDG conventions
- Executes the recipe using the embedded QuickJS runtime

## Comparison with CI Reusable Steps

Remote recipes share similarities with CI reusable steps (like GitHub Actions, GitLab CI Components, and Forgejo Actions) but are designed for a different purpose:

**CI Reusable Steps** (GitHub Actions, GitLab CI Components, Forgejo Actions) are CI/CD workflows that:
- Run in cloud infrastructure (provider-specific runners)
- Tightly coupled to specific CI/CD platforms
- Require platform-specific YAML configuration
- Execute in response to repository events (push, pull request, etc.)
- Ideal for automated testing, deployment, and release workflows

**Remote Recipes** are portable automation scripts that:
- Run locally on developer machines or in any CI environment
- Not coupled to any specific CI provider
- Use simple JavaScript with Fabrik's embedded runtime
- Execute on-demand via `fabrik run` command
- Ideal for cached build steps, code generation, and reproducible automation

Think of remote recipes as **lightweight, portable actions** that work anywhere Fabrik is installed, with the added benefit of content-addressed caching for fast, incremental builds.

## Why Remote Recipes?

- **Easy Sharing** - Share recipes across teams by publishing them in Git repositories. No need to copy files manually.
- **Version Control** - Pin recipes to specific versions using Git tags:
  ```bash
  fabrik run @tuist/recipes/build.js@v1.0.0
  ```
- **Always Up-to-Date** - Reference `@main` or `@latest` to always use the newest version.
- **Cross-Platform** - Remote recipes run on any platform with the embedded QuickJS runtime - no external dependencies needed.

## Quick Example

```bash
# Run a remote recipe from GitHub (default host)
fabrik run @tuist/recipes/typescript-build.js

# With a specific version
fabrik run @tuist/recipes/typescript-build.js@v1.0.0

# From GitLab
fabrik run @gitlab.com/myorg/recipes/deploy.js

# Verbose mode to see what's happening
fabrik run --verbose @tuist/recipes/build.js
```

## How It Works

1. **Parse** - Fabrik parses the `@org/repo/path/script.js[@ref]` syntax
2. **Fetch** - Clones the repository to `~/.cache/fabrik/recipes/{host}/{org}/{repo}/{ref}/`
3. **Cache** - Subsequent runs reuse the cached version (no re-fetch)
4. **Execute** - Runs the recipe with QuickJS runtime and Fabrik APIs

## Supported Git Hosts

- **GitHub** (default) - `@org/repo/script.js`
- **GitLab** - `@gitlab.com/org/repo/script.js`
- **Self-hosted** - `@git.company.com/team/project/script.js`

## Next Steps

- [Syntax Reference](/cache/recipes/remote/syntax) - Learn the full `@` prefix syntax
- [Examples](/cache/recipes/remote/examples) - See real-world remote recipe examples
