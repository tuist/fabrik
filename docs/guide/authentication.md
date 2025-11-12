# Authentication

Learn how to authenticate Fabrik with remote cache servers using token-based or OAuth2 authentication.

## Overview

Fabrik supports two authentication methods for connecting to remote cache servers:

- **Token-based**: Simple token authentication for CI/CD and automated workflows
- **OAuth2 with PKCE**: Secure, user-friendly authentication for interactive use

## Authentication Methods

### Token-Based Authentication

Simple authentication using a static token. Best for CI/CD pipelines or when OAuth2 is not available.

#### Configuration

```toml
[auth]
provider = "token"

[auth.token]
# Option 1: Hardcoded (not recommended for production)
value = "your-token-here"

# Option 2: Environment variable (recommended)
env_var = "FABRIK_AUTH_TOKEN"

# Option 3: File path (recommended for local development)
file = "~/.fabrik/token"
```

#### Usage

```bash
# Set token via environment variable
export FABRIK_AUTH_TOKEN="your-token-here"

# Or store in file
echo "your-token-here" > ~/.fabrik/token
chmod 600 ~/.fabrik/token

# Verify authentication
fabrik auth status
```

#### CI/CD Example

```yaml
# .github/workflows/build.yml
name: Build with Cache

on: [push]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build with Fabrik cache
        env:
          FABRIK_AUTH_TOKEN: ${{ secrets.FABRIK_TOKEN }}
        run: fabrik exec gradle build
```

### OAuth2 with PKCE Authentication

Secure authentication with automatic token refresh. Best for interactive use and development workflows.

#### Configuration

```toml
[auth]
provider = "oauth2"

[auth.oauth2]
server_url = "https://tuist.dev"
client_id = "fabrik-cli"
scopes = "cache:read cache:write"
storage = "keychain"  # or "file" or "memory"

# Optional: Custom endpoints (defaults use server_url)
authorization_endpoint = "https://tuist.dev/oauth/authorize"
token_endpoint = "https://tuist.dev/oauth/token"
device_authorization_endpoint = "https://tuist.dev/oauth/device/code"
```

#### Storage Backends

Choose where to store OAuth2 tokens:

| Backend | Description | Use Case |
|---------|-------------|----------|
| `keychain` | OS credential manager (Keychain, Credential Manager, Secret Service) | **Recommended** for local development |
| `file` | File-based storage in `~/.fabrik/oauth-tokens/` | Cross-process safe with file locking |
| `memory` | In-memory only | Temporary sessions, tokens lost on restart |

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

## Common Workflows

### Local Development with OAuth2

```bash
# One-time setup
fabrik auth login
fabrik activate bash

# Daily use (token automatically refreshed)
cd ~/project
gradle build  # Uses cached credentials automatically
```

### CI/CD with Token Authentication

**GitHub Actions:**
```yaml
- name: Build with cache
  env:
    FABRIK_AUTH_TOKEN: ${{ secrets.FABRIK_TOKEN }}
  run: fabrik exec gradle build
```

**GitLab CI:**
```yaml
build:
  script:
    - fabrik exec gradle build
  variables:
    FABRIK_AUTH_TOKEN: $CI_FABRIK_TOKEN
```

### Switching Between Environments

Use different configs for development and CI:

**`.fabrik.toml` (development):**
```toml
[cache]
dir = ".fabrik/cache"

[[upstream]]
url = "grpc://cache.tuist.dev:7070"

[auth]
provider = "oauth2"

[auth.oauth2]
server_url = "https://tuist.dev"
client_id = "fabrik-cli"
storage = "keychain"
```

**`.fabrik-ci.toml` (CI):**
```toml
[cache]
dir = "/tmp/fabrik-cache"

[[upstream]]
url = "grpc://cache.tuist.dev:7070"

[auth]
provider = "token"

[auth.token]
env_var = "FABRIK_AUTH_TOKEN"
```

## CLI Commands

### Check Authentication Status

```bash
fabrik auth status
```

**Output for OAuth2:**
```
Authentication Status: ✓ Authenticated
Provider: oauth2
Token: abc12345...xyz9
Expires: 2025-11-13 12:00:00 UTC
Time remaining: 23h 45m
```

**Output for Token:**
```
Authentication Status: ✓ Authenticated
Provider: token
Token: sk_test_...abc9
```

### Show Current Token

Get the raw access token for debugging or manual API calls:

```bash
fabrik auth token
# eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

### Logout

Delete stored tokens:

```bash
fabrik auth logout
# ✓ Successfully logged out
```

For token-based authentication:
```
[fabrik] Token-based authentication doesn't require logout
```

## Troubleshooting

### "Token expired" Error

```bash
# Check token status
fabrik auth status

# Re-authenticate
fabrik auth logout
fabrik auth login
```

### "No authentication provider configured" Error

```bash
# Verify configuration
fabrik config show

# Ensure [auth] section exists
fabrik config validate .fabrik.toml
```

### Token Not Found in Environment/File

```bash
# Verify environment variable
echo $FABRIK_AUTH_TOKEN

# Verify file exists and is readable
cat ~/.fabrik/token
ls -la ~/.fabrik/token

# Check file permissions (should be 600)
chmod 600 ~/.fabrik/token
```

### OAuth2 Device Flow Not Working

```bash
# Verify server URL is correct
fabrik config show

# Check network connectivity
curl -I https://tuist.dev

# Try with verbose logging
RUST_LOG=debug fabrik auth login
```

## Security Best Practices

### Token Storage

✅ **Do:**
- Store tokens in environment variables or secure files
- Use OAuth2 keychain storage for local development
- Set file permissions to `600` (owner read/write only)
- Use separate tokens for different environments

❌ **Don't:**
- Hardcode tokens in configuration files committed to git
- Share tokens between team members
- Use production tokens in development
- Log tokens to stdout/stderr

### CI/CD

✅ **Do:**
- Use CI/CD platform secrets management (GitHub Secrets, GitLab CI Variables)
- Rotate tokens regularly
- Use scoped tokens with minimal permissions
- Set token expiry for CI tokens

❌ **Don't:**
- Print tokens in build logs
- Store tokens in source control
- Reuse personal tokens in CI

### OAuth2

✅ **Do:**
- Use keychain storage on trusted machines
- Review authorized applications periodically
- Logout when done on shared machines

❌ **Don't:**
- Skip device authorization step
- Ignore token expiry warnings
- Use memory storage for long-term sessions

## Environment Variables

### Authentication

| Variable | Description | Example |
|----------|-------------|---------|
| `FABRIK_AUTH_TOKEN` | Token for authentication | `sk_test_abc123...` |
| `TUIST_TOKEN` | Shorthand for auth token | `sk_test_abc123...` |
| `TUIST_CONFIG_AUTH_TOKEN` | Full form for auth token | `sk_test_abc123...` |

### AWS Credentials (for S3 upstream)

| Variable | Description |
|----------|-------------|
| `AWS_ACCESS_KEY_ID` | AWS access key |
| `AWS_SECRET_ACCESS_KEY` | AWS secret key |
| `AWS_REGION` | AWS region |

## Next Steps

- [CLI Reference](/reference/cli#fabrik-auth) - Complete auth command reference
- [Configuration File](/reference/config-file) - Full auth configuration options
- [Getting Started](/getting-started) - Set up your first project with auth
