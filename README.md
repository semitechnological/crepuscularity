# Crepuscularity

A framework for writing UI in a concise, indentation-based template DSL (`.crepus` files). Templates compile at build time via the `view!` macro or render at runtime with full hot-reload support.

## Quick Start

```bash
# Install the CLI
cargo install --path crates/crepuscularity-cli

# Create a new GPUI app
crepu new my-app
cd my-app
SDKROOT=$(xcrun --show-sdk-path) cargo run

# Or create a browser extension
crepu webext new my-extension
cd my-extension
crepu webext build
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
- **Hot reload** — live template updates in runtime mode
- **Browser extensions** — `crepu webext` commands for MV3 extensions
- **IDE integration** — structured JSON events with `--emit-events`

## Backends

| Crate | Target |
|---|---|
| `crepuscularity-web` | HTML strings |
| `crepuscularity-react` | JSX/TSX output |
| `crepuscularity-gpui` | Native desktop (GPUI) |
| `crepuscularity-webext` | Browser extensions (MV3) |

## CLI Commands

```bash
crepu new <name>                    # Scaffold GPUI app
crepu dev [--emit-events]           # Hot-reload dev loop
crepu build [--release]             # Build wrapper
crepu preview <file.crepus>         # Live preview

crepu webext new <name>             # Scaffold browser extension
crepu webext build [--app PATH]     # Build to dist/unpacked/
crepu webext manifest               # Print manifest.json
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
  crepuscularity-cli/       crepu CLI
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

## License

MIT
