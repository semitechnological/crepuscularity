#!/usr/bin/env bash
# Run every target in benchmark.toml (same as: crepus benchmark all).
# Extra args are forwarded, e.g.
#   ./run-all.sh --json > bench.json        # machine-readable timings
#   ./run-all.sh --quiet                    # no streaming output (CI / logs)
# Default (no --quiet): live cargo/npm output + step preambles.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CONFIG="$SCRIPT_DIR/benchmark.toml"

cd "$ROOT"

args=(benchmark all --config "$CONFIG")
if [[ $# -gt 0 ]]; then
  args+=("$@")
fi

if command -v crepus >/dev/null 2>&1; then
  exec crepus "${args[@]}"
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "crepus: not in PATH and cargo not found — install the CLI or use a Rust checkout." >&2
  exit 1
fi

if [[ "$(uname -s)" == "Darwin" ]]; then
  if [[ -x "$ROOT/scripts/metal-env.sh" ]]; then
    eval "$("$ROOT/scripts/metal-env.sh" 2>/dev/null || true)"
  elif [[ -z "${SDKROOT:-}" ]]; then
    SDKROOT="$(xcrun --show-sdk-path 2>/dev/null || true)"
    export SDKROOT
  fi
fi

exec cargo run -p crepuscularity-cli -- "${args[@]}"
