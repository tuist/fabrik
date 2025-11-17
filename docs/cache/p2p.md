# P2P Cache Sharing

## Philosophy

Traditional build caches follow a client-server model: your local machine fetches artifacts from a remote cache server. While this works well for distributed teams, it introduces unnecessary latency when team members are on the same local network.

**The P2P Vision:**

What if your coworker sitting next to you just built the same target you need? Their machine already has the artifact cached locally. Instead of both of you fetching from a cloud server 50ms away, you could fetch directly from each other in just 1-5ms.

This is the philosophy behind Fabrik's P2P cache sharing: **leverage the natural clustering of developers on the same network** to create a faster, intermediate cache layer.

## The Problem

Consider a typical development scenario:

```
Developer A (MacBook):                  Developer B (Linux Desktop):
  ├─ Local Cache (0-1ms) ✓               ├─ Local Cache (0-1ms) ✗
  ├─ Regional Cache (20-50ms) ✗           ├─ Regional Cache (20-50ms) → 50ms wait
  └─ S3 Backup (100-200ms) ✗              └─ S3 Backup (100-200ms) ✗
```

Developer A just built a large artifact. Developer B needs the same artifact seconds later. Without P2P, Developer B must:
1. Check local cache → MISS
2. Query regional cache → 50ms round trip
3. Download artifact → additional time

**The waste:** Developer A has the exact artifact B needs, sitting idle on a machine 2 meters away.

## The Solution: Layer 0.5

Fabrik introduces **Layer 0.5** - a peer-to-peer layer that sits between your local cache and the regional cache:

```
Layer 0: Local Cache       (0-1ms)   ✓ Fastest, but limited to your machine
Layer 0.5: P2P Peers       (1-5ms)   ← NEW! Same office/home network
Layer 1: Regional Cache    (20-50ms) ↓ Geographically distributed
Layer 2: S3 Backup         (100-200ms) ↓ Permanent storage
```

When you need an artifact:
1. Check local cache → MISS
2. **Check P2P peers on LAN** → HIT! (1-5ms) ✨
3. Download directly from peer's machine over local network
4. *(Skip regional/S3 entirely)*

## When P2P Shines

### 1. **Office/Team Environments**

Teams working in the same office naturally benefit:
- First developer builds → caches locally
- Second developer builds → fetches from first (LAN speed)
- No cloud bandwidth consumed
- 10-50x faster than cloud cache

### 2. **Multi-Machine Developers**

Developers with multiple machines (MacBook + Linux desktop):
- Build on one machine → P2P cache available on other
- Seamless switching between machines
- Consistent cache across home network

### 3. **CI/CD Runners on Same Network**

Multiple CI runners on the same infrastructure:
- First runner builds base dependencies
- Subsequent runners fetch via P2P
- Reduced cloud cache costs
- Faster parallel builds

## Design Principles

### Zero Configuration

P2P "just works" with minimal setup:
- **mDNS discovery** - Automatically finds peers on your network
- **No IP addresses** - No manual network configuration
- **Cross-platform** - Works on Linux, macOS, Windows

### Security First

P2P is secure by default:
- **HMAC authentication** - Shared team secret prevents unauthorized access
- **User consent** - System notifications before granting cache access
- **Replay protection** - Timestamps prevent request replay attacks
- **Network isolation** - Only discovers peers on same local network

### Graceful Degradation

P2P is always optional:
- If no peers found → falls back to regional cache
- If P2P fails → falls back to regional cache
- If P2P disabled → no impact on existing workflow
- **Never breaks your build**, only makes it faster

## Privacy & Consent

Fabrik respects your privacy:

**You control access** via three consent modes:

1. `notify-once` *(recommended)* - One notification per peer, remembered
2. `notify-always` - Notification every time (maximum control)
3. `always-allow` - No notifications (trusted networks only)

**What peers can access:**
- ✅ Build artifacts you've already built
- ✅ Content-addressed by hash (no metadata exposure)
- ❌ Cannot access files you haven't built
- ❌ Cannot browse your filesystem
- ❌ Cannot execute code on your machine

## Performance Impact

Real-world latency comparison:

| Cache Layer | Typical Latency | Example Use Case |
|-------------|----------------|------------------|
| Local Cache | 0-1ms | Already built on this machine |
| P2P Peers | **1-5ms** | Coworker built it 5 minutes ago |
| Regional Cache | 20-50ms | Team mate in different office |
| S3 Backup | 100-200ms | First time anyone builds this |

**Speedup:** For artifacts your peers have, you get **4-10x faster fetches** compared to regional cache.

## Trade-offs

### When P2P Helps

✅ **Office/Team environments** - Multiple developers on same network
✅ **Multi-machine setups** - Developer with MacBook + desktop at home
✅ **Local CI runners** - Jenkins/GitLab runners on same LAN
✅ **Large artifacts** - P2P latency advantage matters more
✅ **Frequent rebuilds** - Team iterating on same codebase

### When P2P Doesn't Help

❌ **Solo developers** - No peers to share with
❌ **Remote-first teams** - Everyone on different networks
❌ **Cloud-only CI** - Runners provisioned dynamically, no persistent network
❌ **Tiny artifacts** - Latency savings negligible (<1KB)

## Getting Started

> [!NOTE]
> For configuration details, see the [Configuration Reference](/reference/config-file#p2p).
> For CLI usage, see the [CLI Reference](/reference/cli#fabrik-p2p).

**Quick setup:**

1. Enable P2P in your `.fabrik.toml`:
   ```toml
   [p2p]
   enabled = true
   secret = "my-team-secret-2024"
   ```

2. Share the secret with your team (via 1Password, team config, etc.)

3. Start the daemon - P2P discovery happens automatically:
   ```bash
   fabrik daemon start
   ```

4. Check discovered peers:
   ```bash
   fabrik p2p list
   ```

That's it! Your builds will now automatically check P2P peers before hitting the cloud cache.

## Philosophy in Practice

> "The best cache is the one you didn't know was there."

P2P cache sharing embodies this principle:
- No explicit "push to team cache" steps
- No manual cache synchronization
- No complex network setup
- Just faster builds, transparently

When it works, you don't notice it. When it helps, you see the latency savings. When it's not available, your build continues normally.

This is infrastructure that gets out of your way while making your workflow faster.

## Next Steps

- **Configuration:** [P2P Configuration Reference](/reference/config-file#p2p)
- **CLI Commands:** [P2P CLI Reference](/reference/cli#fabrik-p2p)
- **Architecture:** [Architecture Guide](/guide/architecture)
