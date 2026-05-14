# Crepuscularity

> **Stability:** This project is **unstable** and in active development. APIs, CLI flags, and template semantics may change without a semver-major release until **1.0**. Pin exact dependency versions and expect occasional breakage.

The first GPUI component and runtime system with hot reloading, and the first non-web-language, plug-and-play browser extension framework. It also ships a Ratatui terminal backend and a GPUI desktop shell with an embedded V8 bridge for Capacitor-shaped native plugins.

Write UI in a concise, indentation-based template DSL (`.crepus` files). Templates compile at build time via the `view!` macro or render at runtime with full hot-reload support. The same `.crepus` syntax drives native desktop (GPUI), terminal UIs (Ratatui), browser extensions (MV3), HTML output, and React/JSX — and is the foundation for native mobile backends targeting SwiftUI and Jetpack Compose.

Use [Aurorality](https://github.com/semitechnological/aurorality) for united SwiftUI macOS + iOS apps.

## Why Crepuscularity

- **First GPUI component system with hot reload** — live template updates without recompiling; no other GPUI framework offers this
- **First plug-and-play browser extension framework in Rust** — write your popup/background/content scripts in `.crepus`, get a MV3-compliant extension bundle out; no JavaScript framework or bundler required
- **One syntax, multiple backends** — the same template works across GPUI (native desktop), HTML, React/JSX, and browser extensions today; **`crepuscularity-native`** lowers `.crepus` to JSON **View IR** for SwiftUI / Jetpack Compose shells (see [`examples/native-shells`](examples/native-shells/README.md))
- **Polyglot plugin contract** — host languages consume View IR JSON through `crepus native ir` or the optional `crepuscularity-abi` C session API, then create UI from the typed node tree; reference plugins live under [`plugins/`](plugins/README.md)
- **Terminal UIs without a second UI language** — **`crepuscularity-tui`** maps `.crepus` elements, includes, slots, control flow, and Tailwind-style terminal classes onto Ratatui frames
- **Desktop shell for embedded guest apps** — **`crepuscularity-lite`** embeds V8 in a GPUI host with a Capacitor-shaped Rust bridge, optional file watching, workers, plugin capabilities, and TypeScript/TSX guest transpilation
- **Compile-time and runtime paths** — `view!` macro for zero-overhead AOT compilation; `parse_template` / `render_nodes` for full runtime flexibility and hot reload

## Quick Start

```bash
# Install the CLI
cargo install --path crates/crepuscularity-cli

# If `crepus` is not found, Cargo still installed the binary under ~/.cargo/bin — add it to PATH,
# e.g. for zsh:  export PATH="$HOME/.cargo/bin:$PATH"
# (Rustup normally prepends this for login shells; some terminals omit it.)

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
- **Semantic native tags** — optional component tags for native shells (`navigationstack`, `sidebar`, `item`, `menu`, `label`, SF symbols, etc. in the SwiftUI `swiftgen` path)
- **Control flow** — `if/else`, `match`, `for`
- **String interpolation** — `"Hello {name}"`
- **Expressions** — arithmetic, comparison, logical operators, property access
- **Components** — single-file and multi-component files with slot support
- **Hot reload** — live template updates via the runtime renderer
- **Browser extensions** — `crepus webext` commands for MV3 extensions; popup pre-rendered at build time; no JS bundler needed
- **IDE integration** — structured JSON events with `--emit-events`

## Output Targets

The `.crepus` DSL is the primary language. Each output target is a renderer that consumes the same parsed template — not a different framework.


| Crate                   | Output                                                         |
| ----------------------- | -------------------------------------------------------------- |
| `crepuscularity-gpui`   | Native desktop (GPUI elements) — primary target                |
| `crepuscularity-tui`    | Ratatui terminal frames with `.crepus` templates and typed handles |
| `crepuscularity-lite`   | GPUI desktop shell with embedded V8, Rust plugins, and guest workers |
| `crepuscularity-native` | View IR JSON for SwiftUI / Compose host apps (not an on-screen renderer) |
| `crepuscularity-abi`    | Optional C ABI sessions for in-process IR rendering and event dispatch |
| `crepuscularity-web`    | HTML strings — server rendering, WASM, browser extensions      |
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

crepus ios new <name>                # XcodeGen + SwiftPM NativeShell (View IR) host app
crepus ios generate [--dir]          # Run xcodegen (finds crepus.toml [ios] upward)
crepus ios build [--dir] [...]       # Simulator build via xcodebuild

crepus native ir <file.crepus>       # Emit View IR JSON for plugins and native shells
```

## Documentation

- **Rendered site (GitHub Pages):** [here](https://crepuscularity.undivisible.dev) — WASM landing page plus HTML generated from the Markdown in [`docs/`](docs/). Built in CI with `crepus web build --site docs-site`.
- **Sources:** [docs/README.md](docs/README.md) indexes [DSL](docs/dsl.md) (including SwiftUI semantic tag mappings), [components](docs/components.md), [CLI](docs/cli.md), [runtime/reactivity](docs/runtime.md), [GPUI](docs/gpui.md), [TUI](docs/tui.md), [Lite](docs/lite.md), [native shells](docs/native.md), [polyglot plugins](docs/polyglot.md), and [extensions](docs/webext.md). Compiler-focused detail stays in-repo as [CREPUS_WEB_IMPLEMENTATION_SPEC.md](docs/CREPUS_WEB_IMPLEMENTATION_SPEC.md) but is not shipped on the public docs site.
- **Native shells:** [examples/native-shells](examples/native-shells/README.md) — SwiftPM (**`ios/`**), Gradle (**`android/`**), and shared **View IR** `fixture.json` next to **`crepuscularity-native`** (replaces the old separate **`crepuscularity-native-ui`** checkout).
- **Contributors & coding agents:** root [**`AGENTS.md`**](AGENTS.md) is the canonical instructions (macOS `SDKROOT`, `cargo fmt` / `clippy` / `test` before push, workspace layout, DSL notes). **`CLAUDE.md`** is a symlink to **`AGENTS.md`** so duplicate context files cannot drift.

## GPUI — Tailwind Class Support

The GPUI renderer maps Tailwind-style class strings to native GPUI style calls. This is a native desktop renderer, not a browser, so the mapping is intentionally selective.

### Supported


| Category             | Classes                                                                                                                                            |
| -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Layout**           | `flex`, `grid`, `block`, `hidden`, `flex-col/row`, `flex-wrap`, `flex-1/auto/none`, `grow`, `shrink`                                               |
| **Justify / Align**  | `justify-start/end/center/between/around`, `items-start/end/center/baseline`, `content-`*                                                          |
| **Align self**       | `self-start`, `self-end`, `self-center`, `self-stretch`, `self-baseline`, `self-auto`                                                              |
| **Sizing**           | `w-`*, `h-*`, `size-*`, `min-w/h-*`, `max-w/h-*` — numbers, fractions, `full`, `auto`, `[Npx]`                                                     |
| **Aspect ratio**     | `aspect-square`, `aspect-video`, `aspect-auto`, `aspect-[N/M]`                                                                                     |
| **Spacing**          | `p-`*, `px-*`, `py-*`, `pt/pb/pl/pr-*`, `m-*`, `mx/my-*`, `mt/mb/ml/mr-*`, `gap-*`, `gap-x/y-*`                                                    |
| **Position**         | `absolute`, `relative`, `top/bottom/left/right/inset-`*                                                                                            |
| **Overflow**         | `overflow-hidden`, `overflow-x/y-hidden`                                                                                                           |
| **Colors**           | Full Tailwind palette (`bg/text/border-{family}-{shade}`), hex (`bg-[#rrggbb]`), hsla, rgba with `/alpha`                                          |
| **Border**           | `border`, `border-0/2/4/8`, per-side `border-t/b/l/r-`*, `border-dashed`                                                                           |
| **Border radius**    | `rounded-`* — all sizes, all sides, all corners                                                                                                    |
| **Typography**       | Font weight (`font-thin` → `font-black`), style (`italic`), size (`text-xs` → `text-9xl`, `text-[Npx]`)                                            |
| **Typography**       | Alignment (`text-left/center/right`), decoration (`underline`, `line-through`, `decoration-`*)                                                     |
| **Typography**       | Line height (`leading-`*, `leading-[N]`), tracking (`tracking-*`, `tracking-[Npx]`), truncation (`truncate`, `text-ellipsis`, `whitespace-nowrap`) |
| **Typography**       | Text transform (`uppercase`, `lowercase`, `capitalize`, `normal-case`)                                                                             |
| **Font**             | `font-['Family']`, `line-clamp-N`, `font-features` via `FontFeatures` API                                                                          |
| **Shadow**           | `shadow-2xs` → `shadow-2xl`, `shadow-none`                                                                                                         |
| **Ring**             | `ring`, `ring-0/1/2/4/8`, `ring-[Npx]` — rendered as box-shadow spread                                                                             |
| **Opacity**          | `opacity-N` (0–100), `opacity-{expr}`                                                                                                              |
| **Cursor**           | `cursor-default/pointer/text/move/not-allowed` and all resize variants                                                                             |
| **Grid**             | `grid-cols-N`, `grid-rows-N`, `col-span-N`, `col-start/end-N`, `row-span/start/end-N`                                                              |
| **Arbitrary values** | `w-[Npx]`, `bg-[#hex]`, `text-[size]`, `rounded-[Npx]`, `border-[Npx]`, `aspect-[N/M]`, etc.                                                       |
| **Dynamic context**  | `bg-{expr}`, `text-{expr}`, `border-{expr}`, `opacity-{expr}` — evaluated against template context                                                 |
| **State prefixes**   | `hover:`, `focus:`, `active:` — accepted silently (state styling requires `.hover()`/`.on_click()` handlers in code)                               |


### Not Supported (GPUI hard limits)

These cannot be added without forking GPUI itself:


| Gap                                  | Reason                                                                                 |
| ------------------------------------ | -------------------------------------------------------------------------------------- |
| `ring-{color}`                       | Ring color customisation requires architectural change; default ring is blue-500/50    |
| `md:` / `lg:` / `xl:` breakpoints    | No CSS cascade or viewport queries — GPUI uses Taffy layout                            |
| `dark:` variant                      | No built-in dark mode detection — use `bg-{theme.surface}` context expressions instead |
| `group-hover:` / `peer:`             | No selector/relationship system                                                        |
| `before:` / `after:` pseudo-elements | No pseudo-elements — add child `div` nodes in the template instead                     |
| `z-`* (z-index)                      | GPUI uses painter's algorithm / GPU layers, not z-index                                |
| `float`                              | Not supported by Taffy layout engine                                                   |
| `ring-inset`                         | GPUI has no inset box-shadow — accepted silently                                       |


## Project Structure

```text
crates/
  crepuscularity/           Facade re-exporting prelude
  crepuscularity-core/      AST, parser, evaluator
  crepuscularity_macros/    Compile-time view! proc-macro
  crepuscularity-runtime/   Hot-reload renderer (Tailwind → GPUI styler)
  crepuscularity-web/       HTML backend
  crepuscularity-webext/    Browser extension support
  crepuscularity-gpui/      GPUI prelude + view! macro
  crepuscularity-tui/       Ratatui backend + template_refs! handles
  crepuscularity-lite/      GPUI + V8 shell and native plugin bridge
  crepuscularity-lite-macros/ Compile-time macros for lite plugin bindings
  crepuscularity-native/    View IR JSON for SwiftUI / Compose shells
  crepuscularity-reactive/  WASM signals, memos, effects, hydration lifecycle
  crepuscularity-ssr/       Server-rendering helpers
  crepuscularity-dev/       crepus-dev hot-reload server
  crepuscularity-cli/       crepus CLI
examples/
  weather/                  Weather app example
  quicknote/                Browser extension example
  native-shells/            SwiftPM and Gradle View IR hosts
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

When Xcode uses the downloadable Metal Toolchain component, let Cargo inherit the exact Xcode environment GPUI's build script needs:

```bash
eval "$(scripts/metal-env.sh)"
cargo build
scripts/metal-env.sh -- cargo check -p crepuscularity-gpui
scripts/metal-env.sh --check
```

The required variables are `SDKROOT` for SDK headers, `DEVELOPER_DIR` for the active Xcode, and `TOOLCHAINS=Metal` for `xcrun` toolchain selection. `scripts/metal-env.sh` reads the downloaded Metal toolchain search path from `xcodebuild -showComponent MetalToolchain -json`, exports the short selector that `xcrun -sdk macosx metal` accepts, and prepends `Metal.xctoolchain/usr/bin` to `PATH` for direct `metal` / `metallib` probes. If `--check` reports `xcrun_metal=failed`, run `xcodebuild -downloadComponent MetalToolchain` or install the component from Xcode Settings > Components before debugging Cargo code.

## License

Mozilla Public License 2.0 — see [LICENSE](LICENSE).
