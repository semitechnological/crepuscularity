#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
APP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUNTIME_DIR="$APP_DIR/runtime"
DIST_DIR="$APP_DIR/dist/unpacked"
WEBEXT_ASSETS="$REPO_ROOT/crates/crepuscularity-webext/assets"

mkdir -p "$DIST_DIR"

cargo build --manifest-path "$RUNTIME_DIR/Cargo.toml" --target wasm32-unknown-unknown --release

VENDOR_TMP="$(mktemp -d)"
trap 'rm -rf "$VENDOR_TMP"' EXIT

wasm-bindgen --target web --out-dir "$VENDOR_TMP" "$RUNTIME_DIR/target/wasm32-unknown-unknown/release/vimium_crepus_runtime.wasm"

rm -rf "$DIST_DIR/src" "$DIST_DIR/vendor"
mkdir -p "$DIST_DIR/src" "$DIST_DIR/vendor"

cp "$VENDOR_TMP/vimium_crepus_runtime.js" "$DIST_DIR/vendor/runtime.js"
cp "$VENDOR_TMP/vimium_crepus_runtime_bg.wasm" "$DIST_DIR/vendor/runtime_bg.wasm"
cp "$WEBEXT_ASSETS/browser-shim.js" "$DIST_DIR/src/browser-shim.js"
cp "$WEBEXT_ASSETS/popup.html" "$DIST_DIR/src/popup.html"
cp "$APP_DIR/src/popup.js" "$DIST_DIR/src/popup.js"
cp "$APP_DIR/src/background.js" "$DIST_DIR/src/background.js"
cp "$APP_DIR/src/content.js" "$DIST_DIR/src/content.js"
cp "$APP_DIR/src/content.css" "$DIST_DIR/src/content.css"

crepus webext manifest --app "$APP_DIR" > "$DIST_DIR/manifest.json"

printf 'Built vimium-crepus at %s\n' "$DIST_DIR"
