# CLI Guide

The `crepu` CLI provides commands for scaffolding, building, and developing crepuscularity applications.

## Installation

```bash
cargo install --path crates/crepuscularity-cli
```

## Commands

### `crepu new <name>`

Scaffold a new GPUI application:

```bash
crepu new my-app
cd my-app
SDKROOT=$(xcrun --show-sdk-path) cargo run
```

Creates:
- `Cargo.toml` with crepuscularity dependencies
- `src/main.rs` with GPUI boilerplate
- `views/` directory for `.crepus` templates

### `crepu dev`

Start the hot-reload development loop:

```bash
crepu dev
crepu dev --bin my-binary
crepu dev --release
crepu dev --emit-events  # IDE integration
```

Options:
- `--bin NAME` — specify which binary to run (for workspaces)
- `--release` — build in release mode
- `--emit-events` — emit structured JSON events to stdout

### `crepu build`

Build the project:

```bash
crepu build
crepu build --release
```

### `crepu preview <file.crepus>`

Live preview a single template file:

```bash
crepu preview views/dashboard.crepus
```

Opens a window showing the rendered template with hot-reload.

To provide context variables, create `context.toml` in the same directory:

```toml
# views/context.toml
username = "alice"
score = 1200
logged_in = true
```

## Browser Extension Commands

### `crepu webext new <name>`

Scaffold a new browser extension:

```bash
crepu webext new my-extension
cd my-extension
crepu webext build
```

Creates:
- `webext.toml` — extension configuration
- `runtime/` — Rust WASM runtime crate
- `views/` — `.crepus` templates

### `crepu webext build`

Build the extension to `dist/unpacked/`:

```bash
crepu webext build
crepu webext build --app ./path/to/extension
```

### `crepu webext manifest`

Print the generated `manifest.json`:

```bash
crepu webext manifest
crepu webext manifest --app ./path/to/extension
```

## IDE Integration

The `--emit-events` flag outputs structured JSON events for IDE integration:

```bash
crepu dev --emit-events
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
