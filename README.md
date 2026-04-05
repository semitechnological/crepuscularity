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
- **Browser extensions** — `crepus webext` commands for MV3 extensions; popup pre-rendered at build time; no JS bundler needed
- **IDE integration** — structured JSON events with `--emit-events`

## Output Targets

The `.crepus` DSL is the primary language. Each output target is a renderer that consumes the same parsed template — not a different framework.

| Crate | Output |
|---|---|
| `crepuscularity-gpui` | Native desktop (GPUI elements) — primary target |
| `crepuscularity-web` | HTML strings — server rendering, WASM, browser extensions |
| `crepuscularity-webext` | MV3 browser extensions — manifest, assets, capability scanning |

JSX/HTML tag syntax is supported as an **input format** in the core parser — the same `.crepus` templates can be written in either indentation style or `<tag>` style, and both compile to the same AST.

## CLI Commands

```bash
crepus new <name>                    # Scaffold GPUI app
crepus dev [--emit-events]           # Hot-reload dev loop
crepus build [--release]             # Build wrapper
crepus preview <file.crepus>         # Live preview

crepus webext new <name>             # Scaffold browser extension
crepus webext build [--app PATH]     # Build to dist/unpacked/ (WASM + manifest + popup pre-render)
crepus webext manifest               # Print manifest.json
```

## Documentation

See [docs/](docs/) for detailed documentation:

- [DSL Reference](docs/dsl.md)
- [Components](docs/components.md)
- [CLI Guide](docs/cli.md)
- [Browser Extensions](docs/webext.md)

## GPUI — Tailwind Class Support

The GPUI renderer maps Tailwind-style class strings to native GPUI style calls. This is a native desktop renderer, not a browser, so the mapping is intentionally selective.

### Supported

| Category | Classes |
|---|---|
| **Layout** | `flex`, `grid`, `block`, `hidden`, `flex-col/row`, `flex-wrap`, `flex-1/auto/none`, `grow`, `shrink` |
| **Justify / Align** | `justify-start/end/center/between/around`, `items-start/end/center/baseline`, `content-*` |
| **Align self** | `self-start`, `self-end`, `self-center`, `self-stretch`, `self-baseline`, `self-auto` |
| **Sizing** | `w-*`, `h-*`, `size-*`, `min-w/h-*`, `max-w/h-*` — numbers, fractions, `full`, `auto`, `[Npx]` |
| **Aspect ratio** | `aspect-square`, `aspect-video`, `aspect-auto`, `aspect-[N/M]` |
| **Spacing** | `p-*`, `px-*`, `py-*`, `pt/pb/pl/pr-*`, `m-*`, `mx/my-*`, `mt/mb/ml/mr-*`, `gap-*`, `gap-x/y-*` |
| **Position** | `absolute`, `relative`, `top/bottom/left/right/inset-*` |
| **Overflow** | `overflow-hidden`, `overflow-x/y-hidden` |
| **Colors** | Full Tailwind palette (`bg/text/border-{family}-{shade}`), hex (`bg-[#rrggbb]`), hsla, rgba with `/alpha` |
| **Border** | `border`, `border-0/2/4/8`, per-side `border-t/b/l/r-*`, `border-dashed` |
| **Border radius** | `rounded-*` — all sizes, all sides, all corners |
| **Typography** | Font weight (`font-thin` → `font-black`), style (`italic`), size (`text-xs` → `text-9xl`, `text-[Npx]`) |
| **Typography** | Alignment (`text-left/center/right`), decoration (`underline`, `line-through`, `decoration-*`) |
| **Typography** | Line height (`leading-*`, `leading-[N]`), tracking (`tracking-*`, `tracking-[Npx]`), truncation (`truncate`, `text-ellipsis`, `whitespace-nowrap`) |
| **Typography** | Text transform (`uppercase`, `lowercase`, `capitalize`, `normal-case`) |
| **Font** | `font-['Family']`, `line-clamp-N`, `font-features` via `FontFeatures` API |
| **Shadow** | `shadow-2xs` → `shadow-2xl`, `shadow-none` |
| **Ring** | `ring`, `ring-0/1/2/4/8`, `ring-[Npx]` — rendered as box-shadow spread |
| **Opacity** | `opacity-N` (0–100), `opacity-{expr}` |
| **Cursor** | `cursor-default/pointer/text/move/not-allowed` and all resize variants |
| **Grid** | `grid-cols-N`, `grid-rows-N`, `col-span-N`, `col-start/end-N`, `row-span/start/end-N` |
| **Arbitrary values** | `w-[Npx]`, `bg-[#hex]`, `text-[size]`, `rounded-[Npx]`, `border-[Npx]`, `aspect-[N/M]`, etc. |
| **Dynamic context** | `bg-{expr}`, `text-{expr}`, `border-{expr}`, `opacity-{expr}` — evaluated against template context |
| **State prefixes** | `hover:`, `focus:`, `active:` — accepted silently (state styling requires `.hover()`/`.on_click()` handlers in code) |

### Not Supported (GPUI hard limits)

These cannot be added without forking GPUI itself:

| Gap | Reason |
|---|---|
| `ring-{color}` | Ring color customisation requires architectural change; default ring is blue-500/50 |
| `md:` / `lg:` / `xl:` breakpoints | No CSS cascade or viewport queries — GPUI uses Taffy layout |
| `dark:` variant | No built-in dark mode detection — use `bg-{theme.surface}` context expressions instead |
| `group-hover:` / `peer:` | No selector/relationship system |
| `before:` / `after:` pseudo-elements | No pseudo-elements — add child `div` nodes in the template instead |
| `z-*` (z-index) | GPUI uses painter's algorithm / GPU layers, not z-index |
| `float` | Not supported by Taffy layout engine |
| `ring-inset` | GPUI has no inset box-shadow — accepted silently |

## Project Structure

```text
crates/
  crepuscularity/           Facade re-exporting prelude
  crepuscularity-core/      AST, parser, evaluator
  crepuscularity-web/       HTML backend
  crepuscularity-gpui/      GPUI prelude + view! macro
  crepuscularity_macros/    Compile-time view! proc-macro
  crepuscularity-runtime/   Hot-reload renderer (Tailwind → GPUI styler)
  crepuscularity-cli/       crepus CLI
  crepuscularity-webext/    Browser extension support
examples/
  weather/                  Weather app example
  quicknote/                Browser extension example
```

## Building

Install the CLI:

```bash
cargo install --path crates/crepuscularity-cli
```

Pre-built binaries for Linux and Windows are in the repo root (`crepus-linux-x86_64`, `crepus-windows-x86_64.exe`).

On macOS, GPUI requires the Xcode SDK path:

```bash
SDKROOT=$(xcrun --show-sdk-path) cargo build
```

Add `export SDKROOT=$(xcrun --show-sdk-path)` to your shell profile to avoid repeating it.

## License

Mozilla Public License 2.0 — see [LICENSE](LICENSE).
