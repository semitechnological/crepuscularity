#!/usr/bin/env bash
set -euo pipefail

crate_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
workspace_root="$(cd "$crate_dir/../.." && pwd)"

cargo build -p crepuscularity-wasm --target wasm32-unknown-unknown --release

rm -rf "$crate_dir/npm/wasm"
wasm-bindgen \
    "$workspace_root/target/wasm32-unknown-unknown/release/crepuscularity_wasm.wasm" \
    --out-dir "$crate_dir/npm/wasm" \
    --target nodejs

rm -f "$crate_dir/npm/wasm/.gitignore" "$crate_dir/npm/wasm/package.json"
echo "built $crate_dir/npm/wasm"
