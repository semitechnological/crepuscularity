# CLI Guide

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

Legacy pipeline (HTML only from structured `site.json`):

```bash
crepus web build --legacy-site-json --site ./old-site -o ./out.html
```

### Interactivity (Svelte-style flexibility)

First paint is **`.crepus` → WASM** (same as `crepus web serve`). You can still add **fine-grained reactivity** the same way as on the web in general:

- **Client islands** — small scripts under **`static/`** (copied to `dist/static/`) or hooks in the shipped **`app.js`** pattern; the template can emit `data-*` hooks (see **`docs-site/index.crepus`** + **`initSlotRotate`** in `assets/web/app.js` for a slot-machine phrase example).
- **Reactive graph** — the **`crepuscularity-reactive`** crate is the direction for signal-driven DOM updates; compose with WASM templates as “compiled shell + targeted patches”.
- **Hydration / islands** — described in **`docs/CREPUS_WEB_IMPLEMENTATION_SPEC.md`**; goal is optional client graphs without duplicating template semantics.

So: **flexible** — static SSR-like WASM output by default, **opt-in** client reactivity where you want it.

### `crepus web serve`

Dev server with hot reload — see `crepus web --help`.

### `crepus web site-json`

Deprecated pretty-printer for `site.json`.

**Migration from `site.json`-only builds:** see [WEB_BUILD_MIGRATION.md](./WEB_BUILD_MIGRATION.md).

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

On macOS, GPUI requires the Xcode SDK path:

```bash
export SDKROOT=$(xcrun --show-sdk-path)
```

Add this to your shell profile to avoid repeating it.
