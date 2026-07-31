#!/usr/bin/env bash
set -euo pipefail

crate_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
workspace_root="$(cd "$crate_dir/../.." && pwd)"

cargo build -p crepuscularity-wasm --target wasm32-unknown-unknown --release

# The generator refuses to run when its schema differs from the wasm-bindgen
# crate's, and an unrelated `cargo install` can swap the binary underneath us.
crate_version="$(cargo metadata --format-version 1 2>/dev/null |
    grep -o '"name":"wasm-bindgen","version":"[^"]*"' | head -1 |
    sed 's/.*"version":"\([^"]*\)".*/\1/')"
cli_version="$(wasm-bindgen --version 2>/dev/null | awk '{print $2}')"
if [ -n "$crate_version" ] && [ "$crate_version" != "$cli_version" ]; then
    echo "wasm-bindgen CLI $cli_version != crate $crate_version; installing match" >&2
    cargo install -f wasm-bindgen-cli --version "$crate_version"
fi

rm -rf "$crate_dir/npm/wasm"
wasm-bindgen \
    "$workspace_root/target/wasm32-unknown-unknown/release/crepuscularity_wasm.wasm" \
    --out-dir "$crate_dir/npm/wasm" \
    --target nodejs

rm -f "$crate_dir/npm/wasm/.gitignore" "$crate_dir/npm/wasm/package.json"
echo "built $crate_dir/npm/wasm"
