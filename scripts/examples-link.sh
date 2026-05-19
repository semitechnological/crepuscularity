#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXT="$ROOT/examples/extensions"

mkdir -p "$EXT"

link() {
  local name="$1"
  local target="$ROOT/../../$name"
  local link="$EXT/$name"
  if [[ ! -d "$target" ]]; then
    echo "skip $name: clone https://github.com/undivisible/$name next to crepuscularity ($target)" >&2
    return 0
  fi
  ln -sfn "../../../$name" "$link"
  echo "linked $link → ../../../$name"
}

link anywhere
link rs_vimium
