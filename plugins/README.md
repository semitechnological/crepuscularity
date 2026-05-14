# Crepuscularity Reference Plugins

These plugins demonstrate the polyglot contract: spawn `crepus native ir`, read View IR JSON from stdout, and decode it into host-language types. They are not FFI bindings and do not depend on Equilibrium.

Build the CLI first:

```bash
cargo build -p crepuscularity-cli --no-default-features
export CREPUS_BIN="$PWD/target/debug/crepus"
```

Smoke commands for installed local toolchains:

```bash
python3 -m unittest discover plugins/python
go test ./plugins/go/...
bun test plugins/typescript-bun
v -gc none test plugins/v
zig build test --build-file plugins/zig/build.zig
swiftc plugins/swift/CrepuscularityPlugin.swift -o /tmp/crepus-swift-smoke
gcc plugins/c/crepuscularity_plugin.c -o /tmp/crepus-c-smoke && /tmp/crepus-c-smoke plugins/fixtures/hello.crepus
g++ plugins/cpp/crepuscularity_plugin.cpp -std=c++17 -o /tmp/crepus-cpp-smoke && /tmp/crepus-cpp-smoke plugins/fixtures/hello.crepus
ruby -Iplugins/ruby -e 'require "crepuscularity_plugin"; p CrepuscularityPlugin.render_ir("plugins/fixtures/hello.crepus").version'
```

Optional when the toolchains are installed:

```bash
dotnet build plugins/csharp
javac -d /tmp/crepus-java plugins/java/CrepuscularityPlugin.java
php -r 'require "plugins/php/CrepuscularityPlugin.php"; var_dump(CrepuscularityPlugin::renderIr("plugins/fixtures/hello.crepus")["version"]);'
```
