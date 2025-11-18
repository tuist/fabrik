# Remote Recipe Syntax Reference

## Format

```
@[host/]org/repo/path/script.js[@ref]
```

## Components

### Required Components

**`@` Prefix**
All remote recipes must start with `@` to differentiate them from local file paths.

**Organization/User** (`org`)
The GitHub/GitLab organization or username.

**Repository** (`repo`)
The repository name.

**Path** (`path/script.js`)
The path to the recipe file within the repository. Can include subdirectories.

### Optional Components

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

## Syntax Examples

### GitHub (Default Host)

```bash
# Simple (uses main branch)
@tuist/recipes/build.js

# With version tag
@tuist/recipes/build.js@v1.0.0

# Nested path
@tuist/recipes/scripts/deploy/production.js

# With branch
@tuist/recipes/build.js@develop
```

### GitLab

```bash
# Simple
@gitlab.com/myorg/myrepo/build.js

# With version
@gitlab.com/myorg/myrepo/build.js@v2.0.0

# Nested path
@gitlab.com/myorg/myrepo/ci/deploy.js@release
```

### Self-Hosted Git

```bash
# Company Git server
@git.company.com/team/project/build.js

# With specific commit
@git.company.com/team/project/build.js@abc123def

# Nested path
@git.company.com/team/project/scripts/test.js@main
```

## Cache Directory Structure

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

## Git URL Generation

Fabrik converts the remote recipe syntax to HTTPS Git URLs:

| Syntax | Git URL |
|--------|---------|
| `@tuist/recipes/build.js` | `https://github.com/tuist/recipes.git` |
| `@gitlab.com/org/repo/script.js` | `https://gitlab.com/org/repo.git` |
| `@git.company.com/team/project/build.js` | `https://git.company.com/team/project.git` |

## Cloning Behavior

- **Shallow Clone** - Fabrik uses `git clone --depth 1` for efficient fetching - only the latest commit is downloaded.
- **Cached After First Fetch** - Once fetched, the recipe is cached locally. Subsequent runs reuse the cache without re-fetching.
- **Branch Tracking** - When using a branch reference (e.g., `@main`), the cache is specific to that branch. Switching branches fetches a new copy.

## Common Patterns

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
@org/recipes/ci/build.js
@org/recipes/ci/test.js
@org/recipes/deploy/staging.js
@org/recipes/deploy/production.js
```

### Multi-Environment Recipes

```bash
# Select environment via recipe path
@company/infra/deploy/dev.js
@company/infra/deploy/staging.js
@company/infra/deploy/prod.js
```

## Error Messages

### Recipe Not Found
```
Error: Script not found at build.js in repository
```

**Solution:** Verify the path exists in the repository at the specified ref.

### Repository Not Found
```
Error: Failed to clone repository https://github.com/org/repo.git
```

**Solutions:**
- Check the repository exists
- Verify it's public or you have SSH keys configured
- Check network connectivity

### Invalid Syntax
```
Error: Remote recipe must start with @
```

**Solution:** Add `@` prefix: `@org/repo/script.js`

## Tips

1. **Pin to versions in production**
   ```bash
   # Good - stable
   @tuist/recipes/build.js@v1.0.0

   # Risky - may break
   @tuist/recipes/build.js@main
   ```

2. **Use semantic versioning**
   ```bash
   # Major version (breaking changes expected)
   @tuist/recipes/build.js@v2.0.0

   # Minor version (new features, no breaking changes)
   @tuist/recipes/build.js@v1.1.0

   # Patch version (bug fixes only)
   @tuist/recipes/build.js@v1.0.1
   ```

3. **Clear cache when needed**
   ```bash
   rm -rf ~/.cache/fabrik/recipes/github.com/org/repo/
   ```

4. **Use verbose mode for debugging**
   ```bash
   fabrik run --verbose @org/repo/script.js
   ```

## Next Steps

- [Examples](/cache/recipes/remote/examples) - See real-world usage
- [Local Recipes](/cache/recipes/local/) - Learn about local script recipes
