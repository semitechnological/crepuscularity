# Plugin surface

**Also:** [View IR contract](view-ir-contract.md) · [Polyglot plugins](polyglot.md) · [Reference plugins](../plugins/README.md)

A **Crepuscularity language plugin** is a package in any host language that lets applications use `.crepus` without reimplementing the DSL. Plugins sit on the [View IR contract](view-ir-contract.md); they do not embed the Rust parser unless they opt into `crepuscularity-abi`.

## Required capabilities

| Capability | Description |
| --- | --- |
| **Types** | Host structs or classes matching `ViewIr` / `ViewNode` for the current `IR_VERSION`, generated from JSON Schema or hand-maintained against fixtures. |
| **Compile** | Call `crepus native ir` (subprocess) or `crepuscularity-abi` (in-process) to obtain IR JSON from templates, includes, and context. |
| **Render** | Map decoded nodes to a UI representation. Reference plugins implement portable **HTML** (`render_html` / `renderHtml`); platform plugins map to SwiftUI, Compose, GPUI, Qt, GTK, terminal widgets, etc. |

## Recommended capabilities

| Capability | Description |
| --- | --- |
| **Events** | Wire `onClick` / `data-onclick` handlers and binding events (`bind:var:value`) to host callbacks; re-render or patch after dispatch. |
| **Sessions** | Hold template path, virtual files, and context in a small session type (see Python `ViewSession`, Go `ViewSession`, TypeScript `CrepusViewSession`). |
| **Hot reload** | Subscribe to file changes; apply `HotReloadEnvelope` patch or full reload messages when integrating with `crepus dev` or custom watchers. |

## Out of scope for core plugins

- Shipping Equilibrium or another FFI framework inside Crepuscularity.
- Full GPUI/web parity in every host (each plugin chooses supported `ViewStyle` fields).
- Publishing all reference plugins to registries (optional; separate repos encouraged for production).

## Reference layout

In-tree references live under [`plugins/`](../plugins/). Manifest: [`plugins/crepuscularity-plugins.toml`](../plugins/crepuscularity-plugins.toml).

| Language | Path | Compile | ABI session |
| --- | --- | --- | --- |
| Python | `plugins/python` | CLI | Optional (`crepuscularity_abi.py`) |
| Go | `plugins/go` | CLI | — |
| TypeScript (Bun) | `plugins/typescript-bun` | CLI | — |
| Zig | `plugins/zig` | CLI | — |
| V | `plugins/v` | CLI | — |
| Rust | `plugins/rust` | CLI | — |
| Swift | `plugins/swift` | CLI | — |
| C# | `plugins/csharp` | CLI | Optional |
| C / C++ | `plugins/c`, `plugins/cpp` | CLI | C optional ABI |
| Ruby, PHP, Java, Kotlin | respective dirs | CLI | PHP optional ABI |

Run local smoke: `scripts/plugin-smoke.sh` (after `cargo build -p crepuscularity-cli` and `cargo build -p crepuscularity-abi`).

## Production plugins

For registry releases, prefer dedicated repositories (`crepuscularity-python`, etc.) with their own CI and semver. Depend on a pinned `crepus` binary or document the minimum `crepuscularity-cli` version. Link back to this contract when `IR_VERSION` changes.
