#!/usr/bin/env bash
set -euo pipefail

mode="${1:---exports}"

if [[ "$(uname -s)" != "Darwin" ]]; then
  exit 0
fi

quote() {
  printf "%q" "$1"
}

sdkroot="${SDKROOT:-}"
if [[ -z "$sdkroot" ]]; then
  sdkroot="$(xcrun --show-sdk-path 2>/dev/null || true)"
fi

developer_dir="${DEVELOPER_DIR:-}"
if [[ -z "$developer_dir" ]]; then
  developer_dir="$(xcode-select -p 2>/dev/null || true)"
fi

component_json="$(xcodebuild -showComponent MetalToolchain -json 2>/dev/null || true)"
toolchains="${TOOLCHAINS:-}"
component_toolchain_identifier=""
if [[ -n "$component_json" ]]; then
  component_toolchain_identifier="$(printf '%s' "$component_json" | plutil -extract toolchainIdentifier raw -o - - 2>/dev/null || true)"
fi

toolchain_search_path=""
if [[ -n "$component_json" ]]; then
  toolchain_search_path="$(printf '%s' "$component_json" | plutil -extract toolchainSearchPath raw -o - - 2>/dev/null || true)"
fi

if [[ -z "$toolchains" && -n "$toolchain_search_path" ]]; then
  toolchains="Metal"
elif [[ -z "$toolchains" && -n "$component_toolchain_identifier" ]]; then
  toolchains="$component_toolchain_identifier"
fi

metal_bin=""
if [[ -n "$toolchain_search_path" && -x "$toolchain_search_path/Metal.xctoolchain/usr/bin/metal" ]]; then
  metal_bin="$toolchain_search_path/Metal.xctoolchain/usr/bin"
fi

if [[ "$mode" == "--check" ]]; then
  printf 'SDKROOT=%s\n' "${sdkroot:-missing}"
  printf 'DEVELOPER_DIR=%s\n' "${developer_dir:-missing}"
  printf 'TOOLCHAINS=%s\n' "${toolchains:-missing}"
  printf 'METAL_TOOLCHAIN_IDENTIFIER=%s\n' "${component_toolchain_identifier:-missing}"
  printf 'METAL_BIN=%s\n' "${metal_bin:-missing}"

  status=0
  if [[ -z "$sdkroot" || ! -d "$sdkroot" ]]; then
    status=1
  fi
  if [[ -z "$developer_dir" || ! -d "$developer_dir" ]]; then
    status=1
  fi
  if [[ -n "$toolchains" ]]; then
    if ! TOOLCHAINS="$toolchains" xcrun -sdk macosx metal -v >/dev/null 2>&1; then
      printf 'xcrun_metal=failed\n'
      status=1
    else
      printf 'xcrun_metal=ok\n'
    fi
  else
    if ! xcrun -sdk macosx metal -v >/dev/null 2>&1; then
      printf 'xcrun_metal=failed\n'
      status=1
    else
      printf 'xcrun_metal=ok\n'
    fi
  fi
  if [[ -n "$metal_bin" && -x "$metal_bin/metal" ]]; then
    if PATH="$metal_bin:$PATH" metal -v >/dev/null 2>&1; then
      printf 'path_metal=ok\n'
    else
      printf 'path_metal=failed\n'
      status=1
    fi
  fi
  exit "$status"
fi

if [[ "$mode" == "--" ]]; then
  shift
  if [[ "$#" -eq 0 ]]; then
    printf 'usage: %s -- <command> [args...]\n' "$0" >&2
    exit 2
  fi
  if [[ -n "$sdkroot" ]]; then
    export SDKROOT="$sdkroot"
  fi
  if [[ -n "$developer_dir" ]]; then
    export DEVELOPER_DIR="$developer_dir"
  fi
  if [[ -n "$toolchains" ]]; then
    export TOOLCHAINS="$toolchains"
  fi
  if [[ -n "$metal_bin" ]]; then
    export PATH="$metal_bin:$PATH"
  fi
  exec "$@"
fi

if [[ -n "$sdkroot" ]]; then
  printf 'export SDKROOT=%s\n' "$(quote "$sdkroot")"
fi
if [[ -n "$developer_dir" ]]; then
  printf 'export DEVELOPER_DIR=%s\n' "$(quote "$developer_dir")"
fi
if [[ -n "$toolchains" ]]; then
  printf 'export TOOLCHAINS=%s\n' "$(quote "$toolchains")"
fi
if [[ -n "$metal_bin" ]]; then
  printf 'export PATH=%s:%s\n' "$(quote "$metal_bin")" "$(quote "$PATH")"
fi
