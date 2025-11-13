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

### OAuth2 with PKCE Authentication

Secure authentication with automatic token refresh. Best for interactive use and development workflows.

#### Configuration

```toml
# Service URL (used for OAuth2, service discovery, etc.)
url = "https://tuist.dev"

[auth]
provider = "oauth2"

[auth.oauth2]
client_id = "fabrik-cli"
scopes = "cache:read cache:write"
storage = "keychain"  # or "file" or "memory"

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
