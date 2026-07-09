#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cargo build -p crepuscularity-cli --no-default-features
cargo build -p crepuscularity-abi

case "$(uname -s)" in
  Darwin) ABI_LIB="$ROOT/target/debug/libcrepuscularity_abi.dylib" ;;
  Linux) ABI_LIB="$ROOT/target/debug/libcrepuscularity_abi.so" ;;
  MINGW*|MSYS*|CYGWIN*) ABI_LIB="$ROOT/target/debug/crepuscularity_abi.dll" ;;
  *) ABI_LIB="$ROOT/target/debug/libcrepuscularity_abi.so" ;;
esac

export PATH="$ROOT/target/debug:$PATH"
export CREPUS_BIN=crepus
export CREPUS_ABI_LIB="$ABI_LIB"

python3 -m unittest discover plugins/python
go test ./plugins/go/...
bun --cwd plugins test
command -v v >/dev/null 2>&1 && v -gc none test plugins/v || echo "skipping V plugin (not installed)"
zig build test --build-file plugins/build.zig
cargo test --manifest-path plugins/rust/Cargo.toml
swift build --package-path plugins
gcc plugins/c/crepuscularity_plugin.c -o /tmp/crepus-c-smoke
/tmp/crepus-c-smoke plugins/fixtures/hello.crepus
gcc plugins/c/abi_smoke.c "$ABI_LIB" -o /tmp/crepus-c-abi-smoke
/tmp/crepus-c-abi-smoke
g++ plugins/cpp/crepuscularity_plugin.cpp -std=c++17 -o /tmp/crepus-cpp-smoke
/tmp/crepus-cpp-smoke plugins/fixtures/hello.crepus
ruby -Iplugins/ruby -e 'require "crepuscularity_plugin"; raise unless CrepuscularityPlugin.render_ir("plugins/fixtures/hello.crepus").version == 5; raise unless CrepuscularityPlugin.render_html("plugins/fixtures/hello.crepus").include?("data-crepus-kind")'
php -r 'require "plugins/php/CrepuscularityPlugin.php"; if (CrepuscularityPlugin::renderIr("plugins/fixtures/hello.crepus")["version"] !== 5) exit(1);'
php -r 'require "plugins/php/CrepuscularityAbi.php"; $s = new CrepuscularityAbiSession(); $s->setTemplate("input bind=count\nspan\n  \"Count {count}\""); $s->setContext(["count" => "1"]); $r = $s->dispatchEvent(["handler" => "bind:count:2"]); if (strpos(json_encode($r), "Count 2") === false) exit(1);'
javac -d /tmp/crepus-java plugins/java/CrepuscularityPlugin.java
kotlinc plugins/kotlin/CrepuscularityPlugin.kt -d /tmp/crepus-kotlin
dotnet build plugins/CrepuscularityPlugins.sln --nologo --verbosity:minimal
