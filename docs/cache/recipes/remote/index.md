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

> [!NOTE]
> **Git as Distribution Mechanism**
> Git repositories are used purely as a distribution mechanism for sharing recipes. Recipes are self-contained JavaScript files that cannot depend on other recipes or import external modules. Each recipe runs independently with access to Fabrik's built-in APIs only.

Fabrik automatically:
- Fetches the repository using `git clone --depth 1`
- Caches it locally following XDG conventions
- Executes the recipe using the embedded QuickJS runtime

## Why Remote Recipes?

**Easy Sharing**
Share recipes across teams by publishing them in Git repositories. No need to copy files manually.

**Version Control**
Pin recipes to specific versions using Git tags:
```bash
fabrik run @tuist/recipes/build.js@v1.0.0
```

**Always Up-to-Date**
Reference `@main` or `@latest` to always use the newest version.

**Cross-Platform**
Remote recipes run on any platform with the embedded QuickJS runtime - no external dependencies needed.

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
