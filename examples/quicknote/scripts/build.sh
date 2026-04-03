#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
APP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUNTIME_DIR="$APP_DIR/runtime"
DIST_DIR="$APP_DIR/dist/unpacked"

# Assets come from crepuscularity-webext in the main repo.
WEBEXT_ASSETS="$REPO_ROOT/crates/crepuscularity-webext/assets"

# --- Build WASM ---------------------------------------------------------
mkdir -p "$DIST_DIR"

cargo build --manifest-path "$RUNTIME_DIR/Cargo.toml" \
  --target wasm32-unknown-unknown --release

VENDOR_TMP="$(mktemp -d)"
trap 'rm -rf "$VENDOR_TMP"' EXIT

wasm-bindgen \
  --target web \
  --out-dir "$VENDOR_TMP" \
  "$REPO_ROOT/target/wasm32-unknown-unknown/release/quicknote_runtime.wasm"

DIST_VENDOR="$DIST_DIR/vendor"
mkdir -p "$DIST_VENDOR"
cp "$VENDOR_TMP/quicknote_runtime.js"      "$DIST_VENDOR/runtime.js"
cp "$VENDOR_TMP/quicknote_runtime_bg.wasm" "$DIST_VENDOR/runtime_bg.wasm"

# --- Generate manifest.json from webext.toml ----------------------------
crepus webext manifest --app "$APP_DIR" > "$DIST_DIR/manifest.json"

# --- Assemble dist ------------------------------------------------------
rm -rf "$DIST_DIR/src"
mkdir -p "$DIST_DIR/src"

# Framework assets from crepuscularity-webext (no app-owned markup needed).
cp "$WEBEXT_ASSETS/browser-shim.js" "$DIST_DIR/src/browser-shim.js"
cp "$WEBEXT_ASSETS/background.js"   "$DIST_DIR/src/background.js"
cp "$WEBEXT_ASSETS/popup.js"        "$DIST_DIR/src/popup.js"
cp "$WEBEXT_ASSETS/popup.html"      "$DIST_DIR/src/popup.html"
cp "$WEBEXT_ASSETS/popup.css"       "$DIST_DIR/src/popup.css"

printf 'Built quicknote at %s\n' "$DIST_DIR"
