# Crepuscularity

The first GPUI component and runtime system with hot reloading, and the first non-web-language, plug-and-play browser extension framework.

Write UI in a concise, indentation-based template DSL (`.crepus` files). Templates compile at build time via the `view!` macro or render at runtime with full hot-reload support. The same `.crepus` syntax drives native desktop (GPUI), browser extensions (MV3), HTML output, and React/JSX — and is the foundation for native mobile backends targeting SwiftUI and Jetpack Compose.

## Why Crepuscularity

- **First GPUI component system with hot reload** — live template updates without recompiling; no other GPUI framework offers this
- **First plug-and-play browser extension framework in Rust** — write your popup/background/content scripts in `.crepus`, get a MV3-compliant extension bundle out; no JavaScript framework or bundler required
- **One syntax, multiple backends** — the same template works across GPUI (native desktop), HTML, React/JSX, and browser extensions today, with native mobile (SwiftUI/Jetpack Compose) on the roadmap
- **Compile-time and runtime paths** — `view!` macro for zero-overhead AOT compilation; `parse_template` / `render_nodes` for full runtime flexibility and hot reload

## Quick Start

```bash
# Install the CLI
cargo install --path crates/crepuscularity-cli

# Create a new GPUI app
crepus new my-app
cd my-app
SDKROOT=$(xcrun --show-sdk-path) cargo run

# Or create a browser extension
crepus webext new my-extension
cd my-extension
crepus webext build
# Load dist/unpacked/ in chrome://extensions
```

## Template Syntax

```text
div w-full h-full bg-zinc-950 text-white flex flex-col p-8
  div text-2xl font-bold mb-4
    "Hello {name}"
  if {score > 50}
    div text-green-400
      "High score!"
  else
    div text-red-400
      "Keep going"
```

## Features

- **`.crepus` syntax** — indentation-based, Tailwind-style classes
- **Control flow** — `if/else`, `match`, `for`
- **String interpolation** — `"Hello {name}"`
- **Expressions** — arithmetic, comparison, logical operators, property access
- **Components** — single-file and multi-component files with slot support
- **Hot reload** — live template updates via the runtime renderer
- **Browser extensions** — `crepus webext` commands for MV3 extensions; no JS bundler needed
- **IDE integration** — structured JSON events with `--emit-events`

## Output Targets

The `.crepus` DSL is the primary language. Each output target is a renderer that consumes the same parsed template — not a different framework.

| Crate | Output |
|---|---|
| `crepuscularity-gpui` | Native desktop (GPUI elements) — primary target |
| `crepuscularity-web` | HTML strings — server rendering, WASM, browser extensions |
| `crepuscularity-react` | JSX/TSX — familiar syntax output for teams already on React |
| `crepuscularity-webext` | MV3 browser extensions — manifest, assets, capability scanning |

`crepuscularity-react` is not a React framework — it outputs `.crepus` templates as JSX so teams comfortable with React syntax can read the output. The DSL and component model are the same regardless of which renderer you use.

## CLI Commands

```bash
crepus new <name>                    # Scaffold GPUI app
crepus dev [--emit-events]           # Hot-reload dev loop
crepus build [--release]             # Build wrapper
crepus preview <file.crepus>         # Live preview

crepus webext new <name>             # Scaffold browser extension
crepus webext build [--app PATH]     # Build to dist/unpacked/
crepus webext manifest               # Print manifest.json
```

## Documentation

See [docs/](docs/) for detailed documentation:

- [DSL Reference](docs/dsl.md)
- [Components](docs/components.md)
- [CLI Guide](docs/cli.md)
- [Browser Extensions](docs/webext.md)

## Project Structure

```text
crates/
  crepuscularity/           Facade re-exporting prelude
  crepuscularity-core/      AST, parser, evaluator
  crepuscularity-web/       HTML backend
  crepuscularity-react/     React/JSX backend
  crepuscularity-gpui/      GPUI prelude + view! macro
  crepuscularity_macros/    Compile-time view! proc-macro
  crepuscularity-runtime/   Hot-reload renderer
  crepuscularity-cli/       crepus CLI
  crepuscularity-webext/    Browser extension support
examples/
  weather/                  Weather app example
  quicknote/                Browser extension example
```

## Building

On macOS, GPUI requires the Xcode SDK path:

```bash
SDKROOT=$(xcrun --show-sdk-path) cargo build
```

Add `export SDKROOT=$(xcrun --show-sdk-path)` to your shell profile to avoid repeating it.

## License

Mozilla Public License 2.0 — see [LICENSE](LICENSE).
