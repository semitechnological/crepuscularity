# Crepuscularity — agent & contributor context

**Canonical copy:** this file is the single source for build, CI, and workspace conventions. **`CLAUDE.md`** in the repo root is a **symlink** to **`AGENTS.md`** so tooling that looks for either name stays in sync.

## What this project is

A UI framework that lets you write GPUI interfaces in a concise, indentation-based template DSL (`.crepus` files) instead of raw Rust. Templates can either be compiled at build time via the `view!` macro or rendered at runtime with full hot-reload support.

## Build

```bash
SDKROOT=$(xcrun --show-sdk-path) cargo build
```

`dispatch.h` lives inside the Xcode SDK, not `/usr/include`, so the explicit `SDKROOT` is always required on macOS without the extra command-line tools package. You can also add `SDKROOT=$(xcrun --show-sdk-path)` to your shell profile to avoid repeating it.

For GPUI builds that compile Metal shaders, Cargo inherits the same Xcode environment as the shell. Use the helper when Xcode reports a separate Metal Toolchain component:

```bash
eval "$(scripts/metal-env.sh)"
cargo build
scripts/metal-env.sh -- cargo check -p crepuscularity-gpui
```

The helper exports `SDKROOT`, `DEVELOPER_DIR`, and `TOOLCHAINS=Metal`; `TOOLCHAINS` is the `xcrun` variable that selects the downloaded Metal toolchain for GPUI's `xcrun -sdk macosx metal` build step. It also prepends the downloaded `Metal.xctoolchain/usr/bin` to `PATH` so direct `metal` / `metallib` checks use the same compiler. If `scripts/metal-env.sh --check` still prints `xcrun_metal=failed`, install or re-register the component with `xcodebuild -downloadComponent MetalToolchain` or Xcode Settings > Components before treating a Cargo failure as a code regression.

**Vendored GPUI (optional):** CI and `cargo publish` use **crates.io `gpui` 0.2.2** (the latest published release). The tree under `vendor/gpui` adds `letter_spacing` / `text_transform` on `Div`; to use it locally, copy `.cargo/config.toml.example` to `.cargo/config.toml` and enable `crepuscularity-gpui`’s `gpui-text-extras` feature (see `examples/text-features`’s `vendor-gpui-text`).

## Before pushing / CI requirements

All three checks must pass before pushing:

```bash
cargo fmt --all -- --check          # formatting
cargo clippy --workspace -- -D warnings              # lints
cargo test --workspace              # tests
```

To auto-fix formatting: `cargo fmt --all`

## Crate versioning (pre-1.0, publishable crates)

Use **semver `0.y.z`**. Until `1.0`:

- Bump **`z` (patch)** on **every change** you merge that affects a publishable crate’s behavior, API surface, or **published dependency graph** (for example aligning `version =` on path deps before/after a `cargo publish`).
- Bump **`y` (minor)** for a **larger feature batch** or substantive capability/API expansion in `0.x` (treat it like a minor release).
- Reserve **`1.0.x`** for the eventual stable API contract.

## Workspace layout

| Crate | Purpose |
|---|---|
| `crates/crepuscularity` | Main library re-exporting `view!` macro + prelude |
| `crates/crepuscularity_macros` | Proc-macro: compiles `.crepus` DSL strings at build time |
| `crates/crepuscularity-runtime` | Runtime parser, renderer, and hot-reload engine |
| `crates/crepuscularity-reactive` | Reactive signal/memo/effect graph for WASM client |
| `crates/crepuscularity-dev` | `crepus-dev` binary — hot-reload dev server |
| `crates/crepuscularity-cli` | `crepus` CLI for scaffolding and builds |
| `crates/crepuscularity-native` | View IR (JSON) for SwiftUI / Jetpack Compose shells — `render_template_to_ir`, schema export |
| `examples/text-features` | GPUI demo for letter-spacing and text-transform (vendored gpui) |
| `examples/weather` | Full weather-app example using the runtime |
| `examples/native-shells` | SwiftPM (**`ios/`**) + Gradle (**`android/`**) apps decoding View IR; shared [`fixture.json`](examples/native-shells/fixture.json) |

**Web / compiler / hot-reload implementation spec (single doc for agents):** [docs/CREPUS_WEB_IMPLEMENTATION_SPEC.md](docs/CREPUS_WEB_IMPLEMENTATION_SPEC.md)

**Documentation hub (Markdown):** [docs/README.md](docs/README.md). **`crepus web build --site docs-site`** also emits **`dist/docs/*.html`** (styled HTML from the same Markdown) for the GitHub Pages site.

## DSL quick reference

`.crepus` files support **two equivalent input syntaxes** that compile to the same AST and work with every backend (GPUI, web, webext). Auto-detected by whether the first content line starts with `<`.

### Indentation syntax (native)

Elements are `tag classes…` with indented children.

```
div w-full h-full bg-zinc-950 text-white flex flex-col gap-4
  div text-2xl font-bold
    "Hello {name}"
  if {score > 50}
    div text-green-400
      "High score!"
  else
    div text-red-400
      "Low score"
  for item in {list}
    div p-2 border rounded
      {item}
```

### JSX / HTML tag syntax

For developers familiar with React/TSX. Same semantics, angle-bracket style.

```jsx
<div class="w-full h-full bg-zinc-950 text-white flex flex-col gap-4">
  <div class="text-2xl font-bold">Hello {name}</div>
  <if condition={score > 50}>
    <div class="text-green-400">High score!</div>
    <else><div class="text-red-400">Low score</div></else>
  </if>
  <for let="item" in={list}>
    <div class="p-2 border rounded">{item}</div>
  </for>
</div>
```

Control-flow tags: `<if condition={...}>`, `<else>`, `<else-if condition={...}>`, `<for let="var" in={list}>`, `<match on={expr}><case pattern="...">`.
Include: `<include src="file.crepus#Card" title={t} />` or with slot children.
Declarations: `<let name="x" value={42} />`, `<let-default name="x" value={42} />`, or `$: let x = 42` lines still work.

## Writing .crepus templates

### Pragmas

File-level directives go at the very top, before any elements.

```
google-font "Inter"
google-fonts "Inter" "JetBrains Mono"
```

`google-font` loads a single family; `google-fonts` loads multiple in one pragma. Both inject the appropriate `<link>` tag on the web path.

### Element syntax

`tag classes… #id "inline-text"` on one line. `#id` emits `id="…"` on the element; an inline quoted literal becomes the first child text node.

```
section py-16 #hero "Hello"
  span text-sm
    "World"
```

### Text nodes

Quoted strings are text nodes. `{expr}` inside quotes interpolates. A bare `{expr}` on its own line renders the value directly.

```
div
  "Hello {name}, score: {score * 10}"
  {username}
```

### Control flow

```
if {score > 100}
  div text-green-400
    "High score!"
else if {score > 50}
  div text-yellow-400
    "Medium"
else
  div text-red-400
    "Low"

for item in {items}
  div p-2 border-b
    {item.name}

match {status}
  "active" =>
    div text-green-400
      "Active"
  _ =>
    div text-gray-400
      "Unknown"
```

### Variables

`$: let` computes a local variable from an expression; `$: default` sets a variable only when it is not already in context (canonical way to declare optional component props).

```
$: let total = {price * quantity}
$: default variant = "primary"
```

### Attributes

Static values use `key=value`, dynamic values use `key={expr}`. Event handlers use `@event="fn_name"` (on the web/WASM path this emits `data-onclick="fn_name"` and dispatches to an exported Rust function). Conditional classes use `class:name={expr}`.

```
input type="text" value={input_value} placeholder="Enter text"
button @click="handle_submit"
  "Submit"
div class:hidden={!visible} class:active={selected}
  "Content"
```

### Slot-rotate

`slot-rotate` is a built-in widget for cycling through text children at a fixed interval. Give it an `interval={ms}` attribute and optionally a class-alias name to style the active item. Each indented string child is one rotation slot.

```
slot-rotate interval={3200} slot-lede
  "a GPUI-first template pipeline"
  "a .crepus DSL with hot reload"
  "one syntax for GPUI, web, and extensions"
```

### Class aliases

Lines at the bottom of a `.crepus` file starting with `.name` define reusable class groups. Any element with that name in its class list expands to the aliased classes at render time.

```
.slot-lede text-zinc-100 font-medium
.footer-row flex flex-col sm:flex-row justify-between gap-4 text-sm text-zinc-500
```

Usage: `div footer-row` expands to `div flex flex-col sm:flex-row justify-between gap-4 text-sm text-zinc-500`.

### Include

`include` embeds another component. Props are passed as `key=value` or `key={expr}`. Indented children under the include call become the `slot` content inside the component.

```
include components/card.crepus title="Hello" subtitle={sub}
  div p-4
    "Slot content rendered inside the component"
```

### Animations

`animate:property={duration timing-function}` attaches a CSS transition or entry animation to the element.

```
div animate:opacity={300ms ease-in-out}
  "Fades in on mount"
```

## Component system

### Single-file components (classic)

One `.crepus` file = one component. Included with `include path/to/file.crepus prop=value`.

### Multi-component files (new)

Collect related components into one file using a `+++` TOML frontmatter block followed by `--- Name` section separators.

```
include ui.crepus#Card title="Hello" subtitle="World"
  div
    "Slot content"
```

See `examples/ui.crepus` for the format and `examples/ui-demo.crepus` for usage.

## Architecture notes

- **Two rendering paths**: compile-time (`view!` macro → `crepuscularity_macros`) and runtime (`parse_template` / `render_nodes` → `crepuscularity-runtime`). The `view!` macro does *not* support `include` or multi-component files; those are runtime-only.
- **Context**: props and variables live in `TemplateContext { vars: HashMap<String, TemplateValue>, base_dir, slot }`. Components get a fresh child context with parent props injected.
- **Slot system**: `slot` in a component template renders the caller's children (with the *caller's* context), or the component's fallback children if no slot was passed.
- **`$: default name = value`**: sets a variable only when it isn't already in the context — the canonical way to declare optional props with defaults inside a single-file component.
- **TOML defaults** in multi-component files: `[ComponentName.defaults]` section — evaluated before passed props so props always win.
- **Evaluator** (`eval.rs`): arithmetic, comparison, logical operators, property access (`obj.prop`). No function calls.

## Conventions

- Keep DSL changes mirrored between `crepuscularity_macros` and `crepuscularity-runtime` when both paths should support the feature.
- Error messages from the renderer use a red `div` with the message text — no panics.
- Multi-component files live alongside single-component ones; the `#Name` fragment in the include path is the only disambiguation.
- Rust API documentation is encouraged for public items, invariants, security boundaries, and non-obvious performance contracts. Prefer `//!` / `///` docs over ordinary comments; keep implementation comments rare and limited to places where the code would otherwise hide a correctness or safety requirement.
