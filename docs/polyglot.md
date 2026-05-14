# Polyglot Plugins

**Also:** [Documentation home](README.md) · [Native shells](native.md) · [CLI](cli.md)

Crepuscularity's cross-language contract is **View IR JSON**, not an in-process FFI framework. Rust remains the compiler: it parses `.crepus`, evaluates context, expands includes, and lowers to `ViewIr`. Language plugins call the `crepus` CLI, decode JSON, and render or adapt the IR in their own ecosystem.

Equilibrium stays separate. Plugin authors can use Equilibrium, PyO3, JNI, `cgo`, `ctypes`, or another bridge in their own package when they want native loading, but core Crepuscularity does not require those tools.

## Contract

- `ViewIr` JSON is the stable data boundary.
- `version` must equal the Rust `IR_VERSION`; current version is `2`.
- JSON Schema is available from `crepuscularity-native` with the `schema` feature:

```bash
cargo run -p crepuscularity-native --features schema --bin export-view-ir-schema
```

- A plugin must provide host-language types generated from or compatible with the schema.
- A plugin must provide a render or compile entrypoint that calls `crepus native ir`.
- Hot reload, if supported, should transport the native hot-reload envelope from `crepuscularity-native`.

## CLI Boundary

```bash
crepus native ir views/main.crepus --ctx context.json --pretty
```

`crepus native ir` writes only IR JSON to stdout. Errors are JSON on stderr and exit nonzero.

Supported forms:

```bash
crepus native ir <file.crepus> [--component Name] [--ctx FILE] [--var k=v] [--pretty]
crepus native ir --stdin --base-dir DIR
crepus native ir --stdin-json
```

`--stdin-json` accepts:

```json
{
  "entry": "main.crepus",
  "files": {
    "main.crepus": "div\n  \"Hello\""
  },
  "template": "div\n  \"Hello\"",
  "context": {
    "name": "Ada"
  },
  "pretty": true
}
```

When `files` and `entry` are present, they win over `template`. Context supports scalar values and arrays of objects for loop contexts.

## Reference Plugins

Reference plugins live under `plugins/` and all use subprocess JSON:

| Plugin | Runtime |
| --- | --- |
| `plugins/python` | Python |
| `plugins/go` | Go |
| `plugins/typescript-bun` | TypeScript on Bun |
| `plugins/v` | V |
| `plugins/zig` | Zig |
| `plugins/csharp` | C# / .NET |
| `plugins/swift` | Swift |
| `plugins/java` | Java |
| `plugins/ruby` | Ruby |
| `plugins/php` | PHP |
| `plugins/c` | C |
| `plugins/cpp` | C++ |

These are reference implementations, not registry packages. Production plugins can live in dedicated package repositories with their own CI, versioning, and release cadence.
