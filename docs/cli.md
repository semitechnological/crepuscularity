# CLI Guide

**Also:** [Documentation home](README.md) · [DSL](dsl.md) · [Components](components.md) · [Extensions](webext.md)

The `crepus` CLI provides commands for scaffolding, building, and developing crepuscularity applications.

## Installation

```bash
cargo install --path crates/crepuscularity-cli
```

## Commands

### `crepus new <name>`

Scaffold a new GPUI application:

```bash
crepus new my-app
cd my-app
SDKROOT=$(xcrun --show-sdk-path) cargo run
```

Creates:
- `Cargo.toml` with crepuscularity dependencies
- `src/main.rs` with GPUI boilerplate
- `views/` directory for `.crepus` templates

### `crepus dev`

Start the hot-reload development loop:

```bash
crepus dev
crepus dev --bin my-binary
crepus dev --release
crepus dev --emit-events  # IDE integration
```

Options:
- `--bin NAME` — specify which binary to run (for workspaces)
- `--release` — build in release mode
- `--emit-events` — emit structured JSON events to stdout

### `crepus build`

Build the project:

```bash
crepus build
crepus build --release
```

### `crepus preview <file.crepus>`

Live preview a single template file:

```bash
crepus preview views/dashboard.crepus
```

Opens a window showing the rendered template with hot-reload.

To provide context variables, create `context.toml` in the same directory:

```toml
# views/context.toml
username = "alice"
score = 1200
logged_in = true
```

## Static web sites (`crepus web`)

Author pages in `.crepus` (same virtual-file semantics as `crepus web serve`). Production **`crepus web build`** compiles the site’s `runtime/` crate to **`wasm32-unknown-unknown`**, runs **wasm-bindgen**, and writes a **`dist/`** folder: thin **`index.html`**, **`app.js`**, **`crepus-bundle.json`** (all `*.crepus` sources), **`vendor/unocss.js`**, and **`pkg/runtime.js`** + **`runtime_bg.wasm`**. Copy and dynamic data live in **`.crepus`** (quoted text nodes) and, when you need typed Rust context, in **`runtime/src/lib.rs`** via **`crepuscularity_web::render_from_files`** (same pattern as extension runtimes calling into `crepuscularity-web`).

### Prerequisites

- `rustup target add wasm32-unknown-unknown`
- `cargo install wasm-bindgen-cli`

If both **Homebrew** and **rustup** ship `cargo`/`rustc`, ensure rustup’s toolchain wins for WASM (the CLI prepends `~/.cargo/bin` to `PATH` for nested builds).

### `crepus web new <name>`

Scaffolds `index.crepus`, `web.toml`, and `runtime/` (thin `#[wasm_bindgen]` shim that calls **`crepuscularity_web::render_bundle`**).

### `crepus web build`

```bash
crepus web build --site ./my-site --out-dir ./dist
```

Optional **`site.json`**: SEO (`seo.title`, `seo.description`, `ogImage`) and **CSS variables** in the HTML shell only — not page structure.

### Interactivity (Svelte-style flexibility)

First paint is **`.crepus` → WASM** (same as `crepus web serve`). You can still add **fine-grained reactivity** the same way as on the web in general:

- **Client islands** — `embed ./islands/wave.ts title={title}` compiles the island entry with Bun, writes `dist/islands/*.js`, and mounts the module's `mount(el, props, ctx)` export after the `.crepus` shell renders. Shader imports work through Bun's browser bundler; use import attributes such as `import src from "./wave.glsl" with { type: "text" }` when the shader source should be bundled as a string.
- **Reactive graph** — the **`crepuscularity-reactive`** crate is the direction for signal-driven DOM updates; compose with WASM templates as “compiled shell + targeted patches”.
- **Hydration / islands** — described in **`docs/CREPUS_WEB_IMPLEMENTATION_SPEC.md`**; goal is optional client graphs without duplicating template semantics.

So: **flexible** — static SSR-like WASM output by default, **opt-in** client reactivity where you want it.

### DOM refs and events

Enable DOM helpers in the site runtime when you want typed `#id` access:

```toml
[dependencies]
crepuscularity-web = { path = "../../crates/crepuscularity-web", features = ["dom"] }
wasm-bindgen = "0.2"
```

Template:

```text
div #hero "Hi"
button @click="on_refresh_status" type="button"
  "Refresh"
```

Runtime:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn on_refresh_status() -> Result<(), JsValue> {
    let crepus = crepuscularity_web::crepus_refs!("../index.crepus");
    crepus.hero.text("Bye")
}
```

`crepus_refs!` scans the referenced `.crepus` file and reachable `include`s at compile time, then generates typed fields for discovered `#id`s. Missing DOM nodes return `Result::Err`; the web production path does not panic.

For Cargo rebuild tracking and early syntax validation, add a build dependency on the same crate and put this in `build.rs`:

```rust
fn main() {
    crepuscularity_web::build::compile_crepus("views").unwrap();
}
```

### Optional web features

- `dom`: wasm-side DOM lookup and mutation helpers such as `crepus.hero.text(...)`.
- `event-router`: reserved for event-router-related Rust glue. The default shell-side `data-on*` delegation lives in `app.js`, so it does not increase wasm size on its own.
- `full-web`: convenience feature enabling the optional web-facing feature set.

Keep minimal sites on default features and opt in only when the runtime needs DOM mutations.

### HTMX and Alpine in the shell

`crepus web build` copies `static/` into `dist/static/`. To vendor HTMX or Alpine without blocking WASM-first paint, add the script file under `static/vendor/` and inject it from `web.toml`:

```toml
[site]
head_html = """
  <script defer src="./static/vendor/htmx.min.js"></script>
  <script defer src="./static/vendor/alpine.min.js"></script>
"""
```

HTMX and Alpine should target stable subtrees that your Rust/WASM code is not replacing wholesale. Practical rule:

- Let WASM own `#hero`, `#status`, and other nodes mutated through `crepus_refs!`.
- Point HTMX swaps at sibling containers or leaf regions that Rust is not patching.
- Use Alpine for local state inside self-contained islands; avoid mixing Alpine `x-text` and Rust `text(...)` writes on the same node unless one side is clearly authoritative.

Canonical Alpine coexistence:

```text
div x-data="{ n: 0 }"
  span #counter_display x-text="n"
    "0"
  button x-on:click="n++" type="button"
    "++"
```

Use `x-on:*` for Alpine inside `.crepus`; Crepus reserves `@event=...` for Rust/WASM handler wiring.

### `crepus web serve`

Dev server with hot reload — see `crepus web --help`.

### `crepus web site-json`

Deprecated pretty-printer for `site.json`.

## iOS host apps (`crepus ios`)

Scaffold an **XcodeGen** + **SwiftPM** app with a local **`NativeShell`** package that renders **View IR** JSON (same contract as **`crepuscularity-native`**). Prerequisite: [XcodeGen](https://github.com/yonaskolb/XcodeGen) (`brew install xcodegen`).

```bash
crepus ios new my-native-demo
cd my-native-demo
crepus ios generate   # run xcodegen (walks up until it finds crepus.toml [ios])
open *.xcodeproj
```

Build for the iOS Simulator from the app tree (after generate):

```bash
crepus ios build
```

`crepus ios generate` and `crepus ios build` **walk up** from the current directory until they find **`crepus.toml`** with an **`[ios]`** section — you normally do not need `--dir` or `--scheme`. Optional overrides: `crepus ios build --dir . --scheme Foo --destination 'platform=iOS Simulator,name=iPhone 15'`.

To refresh bundled fixtures after template changes, regenerate IR with **`crepuscularity-native`** (`render_template_to_ir` / `to_json_pretty`) and replace `NativeShell/Sources/NativeShell/fixture.json`. See [`examples/native-shells/README.md`](../examples/native-shells/README.md).

## View IR JSON (`crepus native ir`)

Emit the same **View IR** JSON contract used by native shells and polyglot plugins:

```bash
crepus native ir views/main.crepus --ctx context.json --pretty
crepus native ir views/ui.crepus --component Card --var title=Hello
cat views/main.crepus | crepus native ir --stdin --base-dir views
```

For tool integrations, `crepus native ir --stdin-json` accepts an envelope with `entry`, `files`, `template`, `context`, and `pretty`. Successful output is JSON on stdout only; failures are JSON on stderr. See [Polyglot plugins](polyglot.md).

## Embedded framebuffer (`crepus embedded`) — UNSTABLE

> In active development and testing. Prefer the Rust [`crepuscularity-embedded`](../crates/crepuscularity-embedded) `Ui` API in firmware; CLI commands are for CI and debug snapshots.

```bash
crepus embedded check ui/dashboard.crepus
crepus embedded snapshot ui/dashboard.crepus --width 240 --height 320 --out /tmp/preview.ppm
crepus embedded snapshot ui.crepus --component Card --ctx context.json --var cpu=88 --width 128 --height 64 --out card.ppm
```

- **`check`** — parse validation (use in CI alongside `crepuscularity_core::build::compile_crepus` in `build.rs`).
- **`snapshot`** — writes RGB888 PPM (P6) for visual inspection only, not for shipping to devices.

See [Embedded / framebuffer](embedded.md).

## Browser Extension Commands

### `crepus webext new <name>`

Scaffold a new browser extension:

```bash
crepus webext new my-extension
cd my-extension
crepus webext build
```

Creates:
- `webext.toml` — extension configuration
- `runtime/` — Rust WASM runtime crate
- `views/` — `.crepus` templates

### `crepus webext build`

Build the extension to `dist/unpacked/`:

```bash
crepus webext build
crepus webext build --app ./path/to/extension
```

### `crepus webext manifest`

Print the generated `manifest.json`:

```bash
crepus webext manifest
crepus webext manifest --app ./path/to/extension
```

## IDE Integration

The `--emit-events` flag outputs structured JSON events for IDE integration:

```bash
crepus dev --emit-events
```

Events:
- `CompilationStarted` — build started
- `CompilationSuccess` — build succeeded with timing
- `CompilationError` — build failed with diagnostics

Example output:

```json
{"event":"CompilationStarted","timestamp":"2024-01-15T10:30:00Z"}
{"event":"CompilationSuccess","timestamp":"2024-01-15T10:30:05Z","duration_ms":5123}
{"event":"CompilationError","timestamp":"2024-01-15T10:31:00Z","errors":[{"message":"..."}]}
```

## Environment

On macOS, GPUI requires the Xcode SDK path and, on Xcode installs with a separate Metal Toolchain component, the matching toolchain selector:

```bash
export SDKROOT=$(xcrun --show-sdk-path)
export DEVELOPER_DIR=$(xcode-select -p)
export TOOLCHAINS=Metal
```

From the repository checkout, use `eval "$(scripts/metal-env.sh)"` to export those values and prepend the downloaded `Metal.xctoolchain/usr/bin` to `PATH` for direct `metal` / `metallib` checks. `TOOLCHAINS` is the environment variable `xcrun` reads when GPUI's build script runs `xcrun -sdk macosx metal`; on current Xcode installs the working selector is the short value `Metal`. Use `scripts/metal-env.sh --check` to verify local state without network access.
