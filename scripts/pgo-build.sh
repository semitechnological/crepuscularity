#!/usr/bin/env bash
set -euo pipefail
# PGO build for crepus CLI.
#
# Usage:
#   scripts/pgo-build.sh pgo-gen     # instrumented build → profile run
#   scripts/pgo-build.sh pgo-use     # final PGO-optimised binary
#
# Prerequisites:
#   1. A representative benchmark target that exercises crepus (e.g. `crepus web build`
#      on a multi-page site).  Edit BENCHMARK_CMD below to match your workload.
#   2. `cargo` with the wasm32 target if the benchmark builds WASM.
#
# How it works:
#   pgo-gen — builds crepus with `-Cprofile-generate`, runs it on the benchmark
#             workload, merges the profiles under target/pgo-profiles.
#   pgo-use — rebuilds crepus with `-Cprofile-use` pointing at the merged data.

CARGO_MANIFEST="$(cd "$(dirname "$0")/.." && pwd)"
PGO_DIR="$CARGO_MANIFEST/target/pgo-profiles"
CREPUS_BIN="$CARGO_MANIFEST/target/pgo/crepus"
BENCHMARK_SCRIPT="${BENCHMARK_SCRIPT:-}"
# Default benchmark: render the example ui.crepus template repeatedly to exercise
# parsing, evaluation, and HTML rendering — the hot code paths.
BENCHMARK_TEMPLATE="${BENCHMARK_TEMPLATE:-$CARGO_MANIFEST/examples/ui.crepus}"

pgo_gen() {
  echo "==> Step 1: instrumented build (profile=pgo, PGO_DIR=$PGO_DIR)"
  rm -rf "$PGO_DIR"
  mkdir -p "$PGO_DIR"

  SDKROOT="$(xcrun --show-sdk-path 2>/dev/null || echo '')"
  export SDKROOT
  RUSTFLAGS="-Cprofile-generate=$PGO_DIR" \
    cargo build --profile pgo -p crepuscularity-cli
  echo "  → built: $CREPUS_BIN"

  echo "==> Step 2: workload run (collect profiles)"
  if [ -n "$BENCHMARK_SCRIPT" ]; then
    echo "  running custom benchmark script: $BENCHMARK_SCRIPT"
    "$BENCHMARK_SCRIPT"
  else
    echo "  rendering $BENCHMARK_TEMPLATE repeatedly to exercise hot code paths"
    echo "  (10 iterations of render + render with context)"
    if [ -f "$BENCHMARK_TEMPLATE" ]; then
      for i in $(seq 1 10); do
        "$CREPUS_BIN" render "$BENCHMARK_TEMPLATE" >/dev/null 2>&1 || true
      done
      # Also exercise --component path for multi-component files
      "$CREPUS_BIN" render "$BENCHMARK_TEMPLATE" --component Card >/dev/null 2>&1 || true
    else
      echo "  (template not found at $BENCHMARK_TEMPLATE — running --version as lightweight fallback)"
      "$CREPUS_BIN" --version >/dev/null 2>&1 || true
    fi
  fi

  echo "==> Merging raw profiles..."
  "${LLVM_PROFDATA:-llvm-profdata}" merge -output="$PGO_DIR/merged.profdata" "$PGO_DIR"/*.profraw
  echo "  → merged profile: $PGO_DIR/merged.profdata"
  echo ""
  echo "Next:  scripts/pgo-build.sh pgo-use"
}

pgo_use() {
  if [ ! -f "$PGO_DIR/merged.profdata" ]; then
    echo "ERROR: merged profile not found at $PGO_DIR/merged.profdata" >&2
    echo "Run  scripts/pgo-build.sh pgo-gen  first." >&2
    exit 1
  fi

  echo "==> Step 3: PGO-optimised build (profile=pgo)"
  SDKROOT="$(xcrun --show-sdk-path 2>/dev/null || echo '')"
  export SDKROOT
  RUSTFLAGS="-Cprofile-use=$PGO_DIR/merged.profdata -Cprofile-use=$PGO_DIR" \
    cargo build --profile pgo -p crepuscularity-cli
  echo "  → PGO binary: $CREPUS_BIN"
  echo ""
  "$CREPUS_BIN" --version
}

case "${1:-help}" in
  pgo-gen)  pgo_gen  ;;
  pgo-use)  pgo_use  ;;
  *)
    sed -n '2,20p' "$0"
    exit 1
    ;;
esac
