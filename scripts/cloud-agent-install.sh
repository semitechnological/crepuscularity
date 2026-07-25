#!/usr/bin/env bash
# Idempotent Cloud Agent / local bootstrap for crepuscularity (+ optional moonshine).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> crepuscularity bootstrap ($ROOT)"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required" >&2
  exit 1
fi

cargo test -p crepuscularity-components --quiet
cargo test -p crepuscularity-cli --test components_moonshine_cli --quiet

# Install/refresh `crepus` from this checkout.
cargo install --path crates/crepuscularity-cli --force --locked 2>/dev/null \
  || cargo install --path crates/crepuscularity-cli --force

# Sibling moonshine checkout (Cloud Agent may clone via repositoryDependencies).
if [[ -d ../moonshine ]]; then
  export MOONSHINE_PATH="$(cd ../moonshine && pwd)"
  echo "MOONSHINE_PATH=$MOONSHINE_PATH"
  if command -v bun >/dev/null 2>&1; then
    (
      cd ../moonshine
      bun install --frozen-lockfile || bun install
      bun run check:catalog || true
    )
  fi
elif [[ -d "$HOME/projects/moonshine" ]]; then
  export MOONSHINE_PATH="$HOME/projects/moonshine"
  echo "MOONSHINE_PATH=$MOONSHINE_PATH"
fi

# Flutter plugin smoke when Flutter is available.
if command -v flutter >/dev/null 2>&1; then
  (
    cd plugins/crepuscularity-components/packages/flutter
    flutter pub get
    flutter test
  )
  (
    cd plugins/crepuscularity-flutter
    flutter pub get
    flutter test test/dither_sparkline_test.dart 2>/dev/null || flutter test --reporter compact
  )
else
  echo "warn: flutter not installed — skipping flutter tests"
fi

echo "==> crepuscularity bootstrap ok (crepus $(crepus --version 2>/dev/null || echo installed))"
