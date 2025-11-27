# Portable Recipe Syntax Reference

Portable recipes are JavaScript files executed in Fabrik's embedded QuickJS runtime. They can be run locally or fetched from Git repositories.

## Local Recipes

Run any `.js` file directly with Fabrik's embedded QuickJS runtime:

```bash
fabrik run <path/to/recipe.js>
```

### Examples

```bash
# Run a recipe in the current directory
fabrik run build.js

# Run a recipe in a subdirectory
fabrik run scripts/deploy.js

# Run with verbose output
fabrik run --verbose build.js

# Absolute path
fabrik run /path/to/my/recipe.js
```

### How It Works

When you run a `.js` file with `fabrik run`:

1. Fabrik detects the `.js` extension
2. Loads the file into the embedded QuickJS runtime
3. Provides access to Fabrik APIs (`fabrik:cache`, `fabrik:kv`, `fabrik:fs`)
4. Provides Node.js-compatible APIs (`fs`, `child_process`, `path`)
5. Executes the recipe

### Example Recipe

```javascript
// build.js
import { spawn } from 'child_process';
import { glob } from 'fabrik:fs';

console.log("Building project...");

const files = await glob("src/**/*.ts");
console.log(`Found ${files.length} TypeScript files`);

const result = await spawn("npm", ["run", "build"]);
if (result.exitCode !== 0) {
    throw new Error("Build failed!");
}

console.log("Build complete!");
```

---

## Remote Recipes

Fetch and run recipes directly from Git repositories using the `@` prefix syntax:

```bash
fabrik run @[host/]org/repo/path/script.js[@ref]
```

### Components

#### Required Components

**`@` Prefix**
All remote recipes must start with `@` to differentiate them from local file paths.

**Organization/User** (`org`)
The GitHub/GitLab organization or username.

**Repository** (`repo`)
The repository name.

**Path** (`path/script.js`)
The path to the recipe file within the repository. Can include subdirectories.

#### Optional Components

**Host** (`host`)
The Git server hostname. Defaults to `github.com` if not specified.

Examples:
- `github.com` (default)
- `gitlab.com`
- `git.company.com` (self-hosted)

**Git Reference** (`@ref`)
A Git branch, tag, or commit SHA. Defaults to `main` if not specified.

Examples:
- `@main` (branch)
- `@v1.0.0` (tag)
- `@abc123def` (commit SHA)

### Remote Syntax Examples

#### GitHub (Default Host)

```bash
# Simple (uses main branch)
fabrik run @tuist/recipes/build.js

# With version tag
fabrik run @tuist/recipes/build.js@v1.0.0

# Nested path
fabrik run @tuist/recipes/scripts/deploy/production.js

# With branch
fabrik run @tuist/recipes/build.js@develop
```

#### GitLab

```bash
# Simple
fabrik run @gitlab.com/myorg/myrepo/build.js

# With version
fabrik run @gitlab.com/myorg/myrepo/build.js@v2.0.0

# Nested path
fabrik run @gitlab.com/myorg/myrepo/ci/deploy.js@release
```

#### Self-Hosted Git

```bash
# Company Git server
fabrik run @git.company.com/team/project/build.js

# With specific commit
fabrik run @git.company.com/team/project/build.js@abc123def

# Nested path
fabrik run @git.company.com/team/project/scripts/test.js@main
```

### Cache Directory Structure

Remote recipes are cached following XDG Base Directory conventions:

```
~/.cache/fabrik/recipes/
├── github.com/
│   └── tuist/
│       └── recipes/
│           ├── main/              # Default branch
│           │   ├── build.js
│           │   └── deploy.js
│           ├── v1.0.0/            # Tagged version
│           │   └── build.js
│           └── develop/           # Branch
│               └── experimental.js
├── gitlab.com/
│   └── myorg/
│       └── myrepo/
│           └── v2.0.0/
│               └── build.js
└── git.company.com/
    └── team/
        └── project/
            └── main/
                └── build.js
```

### Git URL Generation

Fabrik converts the remote recipe syntax to HTTPS Git URLs:

| Syntax | Git URL |
|--------|---------|
| `@tuist/recipes/build.js` | `https://github.com/tuist/recipes.git` |
| `@gitlab.com/org/repo/script.js` | `https://gitlab.com/org/repo.git` |
| `@git.company.com/team/project/build.js` | `https://git.company.com/team/project.git` |

### Cloning Behavior

- **Shallow Clone** - Fabrik uses `git clone --depth 1` for efficient fetching - only the latest commit is downloaded.
- **Cached After First Fetch** - Once fetched, the recipe is cached locally. Subsequent runs reuse the cache without re-fetching.
- **Branch Tracking** - When using a branch reference (e.g., `@main`), the cache is specific to that branch. Switching branches fetches a new copy.

---

## Common Patterns

### Local Development, Remote Production

```bash
# During development - iterate on local recipe
fabrik run build.js

# In CI/production - use versioned remote recipe
fabrik run @myorg/recipes/build.js@v1.0.0
```

### Versioned Releases

```bash
# Production - use stable release
fabrik run @tuist/recipes/build.js@v1.0.0

# Development - use latest from main
fabrik run @tuist/recipes/build.js@main
```

### Monorepo with Multiple Recipes

```bash
# Different recipes in same repo
fabrik run @org/recipes/ci/build.js
fabrik run @org/recipes/ci/test.js
fabrik run @org/recipes/deploy/staging.js
fabrik run @org/recipes/deploy/production.js
```

### Multi-Environment Recipes

```bash
# Select environment via recipe path
fabrik run @company/infra/deploy/dev.js
fabrik run @company/infra/deploy/staging.js
fabrik run @company/infra/deploy/prod.js
```

---

## Quick Reference

| Type | Syntax | Example |
|------|--------|---------|
| Local recipe | `fabrik run <file.js>` | `fabrik run build.js` |
| Remote (GitHub) | `fabrik run @org/repo/file.js` | `fabrik run @tuist/recipes/build.js` |
| Remote with version | `fabrik run @org/repo/file.js@tag` | `fabrik run @tuist/recipes/build.js@v1.0.0` |
| Remote (GitLab) | `fabrik run @gitlab.com/org/repo/file.js` | `fabrik run @gitlab.com/myorg/recipes/build.js` |
| Remote (Self-hosted) | `fabrik run @host/org/repo/file.js` | `fabrik run @git.company.com/team/project/build.js` |

## Next Steps

- [Examples](/cache/recipes/portable/examples) - See real-world usage
- [JavaScript API Reference](/cache/recipes/api-reference) - Complete API documentation
- [Standard Recipes](/cache/recipes/standard/) - Learn about standard script recipes (bash, python, etc.)
