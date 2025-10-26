# Docker Build Quick Reference

## TL;DR - Fast Builds

```bash
# Fastest method (uses BuildKit cache mounts)
./scripts/docker-build.sh

# Or manually:
DOCKER_BUILDKIT=1 docker build -f Dockerfile.fast -t fabrik:latest .
```

**Expected times:**
- First build: 8-12 minutes
- Code changes only: 1-3 minutes
- Dependency changes: 3-5 minutes

## Available Dockerfiles

| File | Use Case | Speed | Notes |
|------|----------|-------|-------|
| `Dockerfile.fast` | **Recommended** | ⚡⚡⚡ | Requires BuildKit, best caching |
| `Dockerfile` | Standard | ⚡⚡ | Works everywhere, good caching |

## Requirements

- Docker 18.09+ (for BuildKit support)
- 4+ CPU cores recommended
- 8GB+ RAM recommended

## Optimizations Applied

1. **cargo-chef**: Separates dependency building from application building
2. **BuildKit cache mounts**: Shares cargo registry across builds
3. **lld linker**: 2-3x faster linking than GNU ld
4. **Thin LTO**: Balanced optimization vs build time
5. **Symbol stripping**: Smaller binaries

## Common Issues

### "unknown flag: syntax"
**Problem:** BuildKit not enabled
**Solution:** `export DOCKER_BUILDKIT=1`

### Still timing out?
**Problem:** Insufficient resources
**Solution:** Increase Docker CPU/RAM in Docker Desktop settings

### Cache not working?
**Problem:** Old Docker version
**Solution:** Update to Docker 20.10+ for best results

## CI/CD

See [docs/docker-build-optimization.md](./docs/docker-build-optimization.md) for GitHub Actions and GitLab CI examples.

## Further Reading

- [Complete optimization guide](./docs/docker-build-optimization.md)
- [Build scripts documentation](./scripts/README.md)
