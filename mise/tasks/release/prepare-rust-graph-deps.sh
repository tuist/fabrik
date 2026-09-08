#!/usr/bin/env bash
#MISE description="Materialize Cargo-resolved Rust dependencies for Once graph builds"
#USAGE flag "--target <target>" help="Optional Rust target triple for metadata resolution"
set -euo pipefail

target=""
while (($# > 0)); do
  case "$1" in
    --target)
      target="${2}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ -n "${target}" && ! "${target}" =~ ^[A-Za-z0-9_.-]+$ ]]; then
  echo "--target must be a Rust target triple" >&2
  exit 1
fi

for tool in jq mise; do
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "${tool} is required" >&2
    exit 1
  fi
done

export CARGO_NET_RETRY="${CARGO_NET_RETRY:-10}"

retry_command() {
  local description="$1"
  shift

  local attempt=1
  local max_attempts=3
  while true; do
    if "$@"; then
      return 0
    fi
    if ((attempt >= max_attempts)); then
      return 1
    fi
    echo "${description} failed; retrying (${attempt}/${max_attempts})" >&2
    sleep $((attempt * 5))
    attempt=$((attempt + 1))
  done
}

cargo_vendor() {
  mise exec -- cargo vendor --locked --versioned-dirs third_party/rust/vendor >/tmp/once-cargo-vendor-config
}

rm -rf third_party/rust/vendor
mkdir -p third_party/rust
retry_command "cargo vendor" cargo_vendor

metadata_args=(metadata --locked --format-version 1)
if [[ -n "${target}" ]]; then
  metadata_args+=(--filter-platform "${target}")
fi

cargo_metadata() {
  mise exec -- cargo "${metadata_args[@]}"
}

metadata="$(retry_command "cargo metadata" cargo_metadata)"
schlussel_manifest="$(
  jq -r '
    .packages[]
    | select(.name == "schlussel")
    | select(.source | startswith("git+"))
    | .manifest_path
  ' <<<"${metadata}" | head -n 1
)"

rm -rf third_party/rust/src/formulas
if [[ -n "${schlussel_manifest}" && "${schlussel_manifest}" != "null" ]]; then
  schlussel_root="$(cd "$(dirname "${schlussel_manifest}")/../.." && pwd)"
  if [[ -d "${schlussel_root}/src/formulas" ]]; then
    mkdir -p third_party/rust/src
    cp -R "${schlussel_root}/src/formulas" third_party/rust/src/formulas
  fi
fi

# microsandbox-filesystem's build script downloads the arch-specific
# `agentd` binary from GitHub at compile time. Under Once's hermetic
# execution the child process cannot reach the network, so pre-stage the
# binary where the crate's documented CI escape hatch looks for it:
# `<workspace_root>/build/agentd`, where `workspace_root` is the vendored
# crate's `CARGO_MANIFEST_DIR/../..`. The crate then copies the file into
# its `OUT_DIR` instead of downloading. The `CI` env var is forwarded
# through the Rust build-script action so the escape hatch fires.
microsandbox_version="$(
  jq -r '
    .packages[]
    | select(.name == "microsandbox-filesystem")
    | .version
  ' <<<"${metadata}" | head -n 1
)"
echo "prepare-rust-graph-deps: microsandbox-filesystem version = '${microsandbox_version}', target = '${target}'"
if [[ -n "${microsandbox_version}" && "${microsandbox_version}" != "null" ]]; then
  case "${target}" in
    aarch64-apple-darwin|arm64-apple-darwin) agentd_arch="aarch64" ;;
    x86_64-*) agentd_arch="x86_64" ;;
    aarch64-*) agentd_arch="aarch64" ;;
    "")
      host_arch="$(uname -m)"
      case "${host_arch}" in
        arm64|aarch64) agentd_arch="aarch64" ;;
        x86_64|amd64) agentd_arch="x86_64" ;;
        *) agentd_arch="" ;;
      esac
      ;;
    *) agentd_arch="" ;;
  esac
  echo "prepare-rust-graph-deps: agentd_arch = '${agentd_arch}'"
  if [[ -n "${agentd_arch}" ]]; then
    agentd_url="https://github.com/superradcompany/microsandbox/releases/download/v${microsandbox_version}/agentd-${agentd_arch}"
    mkdir -p third_party/rust/vendor/build
    echo "prepare-rust-graph-deps: fetching ${agentd_url}"
    fetch_agentd() {
      curl --fail --location --silent --show-error \
        --output third_party/rust/vendor/build/agentd \
        "${agentd_url}"
    }
    retry_command "download agentd" fetch_agentd
    chmod +x third_party/rust/vendor/build/agentd
    ls -la third_party/rust/vendor/build/agentd
  fi
fi
