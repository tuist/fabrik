# Docker Build Quick Reference

## TL;DR - Fast Builds

```bash
# Recommended: Use BuildKit for fastest builds
DOCKER_BUILDKIT=1 docker build -t fabrik:latest .
```

**Expected times:**
- First build: 8-12 minutes
- Code changes only: 1-3 minutes
- Dependency changes: 3-5 minutes

## Requirements

- Docker 18.09+ (for BuildKit support)
- 4+ CPU cores recommended
- 8GB+ RAM recommended

## Optimizations Applied

1. **cargo-chef**: Separates dependency building from application building
2. **BuildKit cache mounts**: Shares cargo registry across builds
3. **Disabled LTO**: Prevents build hangs (LTO can cause 30+ min builds)
4. **Parallel codegen**: 16 codegen units for faster compilation
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
