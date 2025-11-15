# Authentication

Learn how to authenticate Fabrik with remote cache servers using token-based or OAuth2 authentication.

## Overview

Fabrik supports two authentication methods for connecting to remote cache servers:

- **Token-based**: Simple token authentication for CI/CD and automated workflows
- **OAuth2 with PKCE**: Secure, user-friendly authentication for interactive use

> [!TIP]
> Fabrik automatically detects which method to use, so the same configuration works seamlessly in both local development (OAuth2) and CI/CD (token-based).

## Auto-Detection (Zero Config)

Fabrik automatically detects which authentication method to use based on what's available:

```toml
# fabrik.toml (works everywhere!)
url = "https://tuist.dev"

[auth]
# No provider needed - auto-detects!

[auth.oauth2]
client_id = "fabrik-cli"
scopes = "cache:read cache:write"
storage = "file"
```

**How it works:**

1. **FABRIK_AUTH_PROVIDER** env var set → Use specified provider (`token` or `oauth2`)
2. **FABRIK_TOKEN** env var present → Use token authentication
3. **OAuth2 token in storage** → Use OAuth2 (from previous `fabrik auth login`)
4. **Config file `provider`** → Use explicit config setting
5. **Nothing available** → Error with helpful message

**Examples:**

```bash
# Local development: Login once, auto-uses OAuth2 thereafter
fabrik auth login
fabrik daemon  # ✅ Uses OAuth2 automatically

# CI/CD: Just set token, works automatically
export FABRIK_TOKEN=${{ secrets.FABRIK_TOKEN }}
fabrik daemon  # ✅ Uses token automatically

# Explicit override (if needed)
export FABRIK_AUTH_PROVIDER=token
fabrik daemon  # ✅ Forces token auth
```

> [!IMPORTANT]
> Auto-detection allows the same config file to work in both local development (OAuth2) and CI/CD (token-based):
> - ✅ Same config file for local dev and CI
> - ✅ No hardcoded auth methods
> - ✅ Works naturally with existing workflows
> - ✅ Explicit override when needed

## Authentication Methods

### Token-Based Authentication

Simple authentication using a static token. Best for CI/CD pipelines or when OAuth2 is not available.

#### Zero-Configuration (Convention-Based)

Fabrik automatically checks for tokens in the standard environment variable:

```bash
# Use FABRIK_TOKEN (no config needed!)
export FABRIK_TOKEN="your-token-here"

# Verify authentication
fabrik auth status
```

**Minimal config** (not even required with auto-detection):
```toml
[auth]
# That's it! FABRIK_TOKEN auto-detected

# Optional: Explicit provider (useful for debugging)
# provider = "token"
```

#### Custom Configuration

Override the default behavior if needed:

```toml
[auth]
provider = "token"

[auth.token]
# Option 1: Custom environment variable
env_var = "MY_CUSTOM_TOKEN_VAR"

# Option 2: File path (recommended for local development)
file = "~/.fabrik/token"
```

#### Usage Examples

```bash
# Zero-config: Just set the env var
export FABRIK_TOKEN="your-token-here"
fabrik daemon  # Works automatically!

# Custom env var
export MY_CUSTOM_TOKEN_VAR="your-token-here"
fabrik daemon --config fabrik.toml  # Uses custom env var from config

# File-based token
echo "your-token-here" > ~/.fabrik/token
chmod 600 ~/.fabrik/token
fabrik daemon --config fabrik.toml  # Reads from file

# Verify authentication
fabrik auth status
```

### OAuth2 with PKCE Authentication

Secure authentication with automatic token refresh. Best for interactive use and development workflows.

#### Configuration

```toml
# Service URL (used for OAuth2, service discovery, etc.)
url = "https://tuist.dev"

[auth]
# No provider needed - auto-detects OAuth2 after login!

# Optional: Explicit provider (useful for debugging)
# provider = "oauth2"

[auth.oauth2]
client_id = "fabrik-cli"
scopes = "cache:read cache:write"
storage = "file"  # or "keychain" or "memory"

# Optional: Override service URL for OAuth2 specifically
# url = "https://custom-auth.example.com"

# Optional: Custom endpoints (defaults use url)
# authorization_endpoint = "https://tuist.dev/oauth/authorize"
# token_endpoint = "https://tuist.dev/oauth/token"
# device_authorization_endpoint = "https://tuist.dev/oauth/device/code"
```

#### Storage Backends

Choose where to store OAuth2 tokens:

| Backend | Description | Use Case |
|---------|-------------|----------|
| `keychain` | OS credential manager (Keychain, Credential Manager, Secret Service) | **Recommended** for local development |
| `file` | File-based storage (XDG compliant: `~/.local/share/fabrik/`) | Cross-process safe with file locking |
| `memory` | In-memory only | Temporary sessions, tokens lost on restart |

> [!TIP]
> Use `file` storage for maximum compatibility across platforms and processes. It follows XDG Base Directory Specification on Linux/Unix systems.

#### Login Flow

```bash
# Login with OAuth2
fabrik auth login --config .fabrik.toml
```

**Output:**
```
[fabrik] Starting OAuth2 device code flow
[fabrik] Please visit: https://tuist.dev/activate
[fabrik] Enter code: ABCD-EFGH
[fabrik] Waiting for authorization...
✓ Successfully authenticated!
```

The device code flow:
1. Fabrik generates a user code
2. You visit the activation URL in your browser
3. Enter the code and authorize
4. Token is securely stored

#### Token Refresh

OAuth2 tokens are automatically refreshed when:
- Token has 20% or less of its lifetime remaining (80% threshold)
- A request is made with an expired token

Token refresh is:
- **Cross-process safe**: Uses file locking to prevent concurrent refreshes
- **Transparent**: Happens automatically without user intervention
- **Efficient**: Proactive refresh prevents request delays

## CI/CD Integration Patterns

The auto-detection feature makes CI/CD integration seamless - use the same config file everywhere.

### GitHub Actions

```yaml
name: Build with Fabrik

on: [push]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build with Fabrik cache
        run: fabrik daemon &
        env:
          FABRIK_TOKEN: ${{ secrets.FABRIK_TOKEN }}

      - name: Run build
        run: ./build.sh
```

> [!NOTE]
> **What happens:**
> 1. GitHub Actions sets `FABRIK_TOKEN` from secrets
> 2. Fabrik auto-detects token authentication
> 3. Same `fabrik.toml` used locally (OAuth2) and CI (token)

### GitLab CI

```yaml
build:
  script:
    - fabrik daemon &
    - ./build.sh
  variables:
    FABRIK_TOKEN: $FABRIK_TOKEN_SECRET
```

### Explicit Override

Force a specific provider when auto-detection isn't desired:

```bash
# Force token auth (even if OAuth2 token exists)
export FABRIK_AUTH_PROVIDER=token
export FABRIK_TOKEN="ci-token"
fabrik daemon

# Force OAuth2 (even if FABRIK_TOKEN is set)
export FABRIK_AUTH_PROVIDER=oauth2
fabrik daemon
```

## Environment Variables Reference

| Variable | Purpose | Example |
|----------|---------|---------|
| `FABRIK_AUTH_PROVIDER` | Override auto-detection | `token` or `oauth2` |
| `FABRIK_TOKEN` | Provide authentication token | `eyJ0eXAi...` |
| `SCHLUSSEL_NO_BROWSER` | Disable browser opening (OAuth2) | `1` |

## Troubleshooting

### Check Authentication Status

```bash
fabrik auth status
```

**Example output (auto-detected token):**
```
Provider: token
Authenticated: ✓
Token: eyJ0eXAi...xyz
```

**Example output (auto-detected OAuth2):**
```
Provider: oauth2
Authenticated: ✓
Token: eyJ0eXAi...xyz (expires in 3h 42m)
```

### Debug Auto-Detection

```bash
# Enable debug logging
export RUST_LOG=fabrik=debug
fabrik auth status

# You'll see:
# DEBUG fabrik::auth - Auto-detected OAuth2 (token found in storage)
```

### Authentication Not Working

1. **Check what's configured:**
   ```bash
   fabrik auth status
   ```

2. **Verify token is set (CI/CD):**
   ```bash
   echo $FABRIK_TOKEN | head -c 20
   # Should show: eyJ0eXAiOiJKV1QiLC...
   ```

3. **Check OAuth2 storage (local dev):**
   ```bash
   ls -la ~/.local/share/fabrik/
   # Should show token files
   ```

4. **Force specific provider:**
   ```bash
   export FABRIK_AUTH_PROVIDER=token
   fabrik auth status
   ```
