#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
APP_DIR="$REPO_ROOT/apps/quicknote"
RUNTIME_DIR="$APP_DIR/runtime"
EXT_DIR="$APP_DIR/extension"
DIST_DIR="$APP_DIR/dist/unpacked"

# Locate anywhere-webext source (works for both local path and git deps).
WEBEXT_SRC="$(cargo metadata \
    --manifest-path "$RUNTIME_DIR/Cargo.toml" \
    --format-version 1 \
  | python3 -c "
import json, sys
meta = json.load(sys.stdin)
pkg = next(p for p in meta['packages'] if p['name'] == 'anywhere-webext')
print(pkg['manifest_path'].removesuffix('/Cargo.toml'))
")"
ASSETS_DIR="$WEBEXT_SRC/assets"

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

# --- Assemble dist ------------------------------------------------------
rm -rf "$DIST_DIR/src"
mkdir -p "$DIST_DIR/src"

cp "$EXT_DIR/manifest.json" "$DIST_DIR/manifest.json"

# All JS, HTML, and CSS from the framework — no app-owned markup.
# quicknote has no content script so content.js / content.css are omitted.
cp "$ASSETS_DIR/browser-shim.js" "$DIST_DIR/src/browser-shim.js"
cp "$ASSETS_DIR/background.js"   "$DIST_DIR/src/background.js"
cp "$ASSETS_DIR/popup.js"        "$DIST_DIR/src/popup.js"
cp "$ASSETS_DIR/popup.html"      "$DIST_DIR/src/popup.html"
cp "$ASSETS_DIR/popup.css"       "$DIST_DIR/src/popup.css"

printf 'Built quicknote at %s\n' "$DIST_DIR"
