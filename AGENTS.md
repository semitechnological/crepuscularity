# Crepuscularity

Write once. Target desktop, terminal, web, browser extensions, mobile, and embedded.

## Project Structure

```
crepuscularity/
  crates/
    crepuscularity/          — manifest/target build (crepus.toml)
    crepuscularity-cli/      — `crepus` CLI (main entrypoint)
    crepuscularity-core/     — parser, eval, AST, context, error types
    crepuscularity-web/      — HTML/WASM rendering (web + SSR)
    crepuscularity-webext/   — browser extension builds (MV3)
    crepuscularity-tui/      — Ratatui terminal rendering
    crepuscularity-native/   — View IR for iOS/Android native
    crepuscularity-embedded/ — LVGL embedded rendering
    crepuscularity-lvgl/     — LVGL XML generation
    crepuscularity-gpui/     — GPUI desktop rendering
    crepuscularity-runtime/  — hot-reload + shared runtime
        crepuscularity-lite/     — V8-based JS runtime
    crepuscularity-macros/   — proc macros
      examples/
    web-site/                — reference site: index.crepus + runtime/
    counter/                 — SSR counter
    todo-web/                — SSR todo
    weather-web/             — SSR weather
    embedded-*/              — LVGL/STM32 examples
```

## Targets & Conventions

### `crepus web` — Static sites + WASM runtime

- Scaffold: `crepus web new <name>` → `index.crepus` + `runtime/` + `crepus.toml`
- Runtime entry: `runtime/src/lib.rs` exports `crepus_render(bundle_json: &str)`
- WASM bridge: `crepuscularity_web::render_bundle(bundle_json).map_err(|e| JsValue::from_str(&e.to_string()))`
  - **ALWAYS use `.to_string()`** on the error — `CrepusError` is not `&str`
- Template syntax: indent-based, UnoCSS classes, `slot-rotate`, `embed` islands
- Build output: `dist/` with `index.html` + `app.js` + `pkg/` (WASM) + `crepus-bundle.json`
- Dev server: `crepus web dev --site . --port 4000`
- WASM target: `wasm32-unknown-unknown`
- Required deps: `crepuscularity-web`, `wasm-bindgen`

### `crepus webext` — Browser extensions (MV3)

- Scaffold: `crepus webext new <name>` → `app/` + `runtime/`
- Supports: `chromium`, `firefox`, `safari`
- WASM-powered background/service workers + content scripts
- Uses `crepuscularity-webext` for manifest generation and WASM bridge
- Build: `crepus webext build --browser chromium --release`
- Dev: `crepus webext dev` (watch + auto-reload)

### `crepus tui` — Terminal apps (Ratatui)

- Scaffold: `crepus tui new <name>` → `app.crepus` + `Cargo.toml` + `src/main.rs`
- Rendering: `crepuscularity-tui` with `HotTemplate` for hot-reload
- Template vars: `set("key", "value")` method
- Preview: `crepus tui preview app.crepus` (q/Esc to quit)
- No WASM — native binary

### `crepus native` — iOS + Android (View IR)

- Scaffold: `crepus native new <name>` → iOS (SwiftPM) + Android (Gradle)
- Rendering: `crepuscularity-native` emits View IR JSON
- Synced view tree: `crepus native sync` updates shared `fixture.json`
- Build: `crepus native build ios|android`
- Preview IR: `crepus native ir file.crepus`

### `crepus ios` — XcodeGen + SwiftUI shells

- Scaffold: `crepus ios new <name>` → XcodeGen project + NativeShell SwiftPM
- View IR rendered via SwiftUI adapter
- Requires: `xcodegen` (`brew install xcodegen`)
- Build: `crepus ios build` (runs xcodegen + xcodebuild)

### `crepus embedded` — LVGL firmware

- Commands: `check` (validate template), `snapshot` (render PPM for debug)
- Rendering: `crepuscularity-embedded` with `crepuscularity-lvgl`
- Target: embedded MCUs (STM32, ESP32) via LVGL
- XML output for Screen/Component root types

### `crepus aurora` — SwiftUI (via Aurorality CLI)

- Delegates to `aurorality` CLI (`cargo install aurorality-cli`)
- Commands: `dev`, `build`, `new`, `swiftgen`
- Generates SwiftUI views from `.crepus` templates

### `crepus new` / `crepus init` — GPUI desktop apps

- `crepus new <name>` — scaffolds a GPUI desktop app
- `crepus init <kind> <name>` — same as `crepus <kind> new <name>`
- GPUI apps use `view! {}` macro with `.crepus` string templates

## Core CLI

| Command | Purpose |
|---------|---------|
| `crepus new <name>` | Scaffold GPUI app |
| `crepus init <kind> <name>` | Scaffold any target |
| `crepus dev [--bin]` | Hot-reload dev loop |
| `crepus build [--target]` | Build crepus.toml targets |
| `crepus preview <file>` | Live-preview (GPUI) |
| `crepus render <file>` | Render to HTML stdout |

## Common Error Patterns

- `CrepusError` → String: always use `.to_string()` when converting to `JsValue::from_str`
- WASM builds need `--target wasm32-unknown-unknown` and `wasm-bindgen-cli`
- Template syntax: indentation-based, 2-space indents, shared root element class on same line as tag