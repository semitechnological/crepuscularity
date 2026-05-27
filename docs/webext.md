# Browser Extensions

**Also:** [Documentation home](README.md) · [DSL](dsl.md) · [Components](components.md) · [CLI](cli.md)

> **Repository-only detail:** Extended web + webext notes for compiler authors live in [`CREPUS_WEB_IMPLEMENTATION_SPEC.md` on GitHub](https://github.com/tschk/crepuscularity/blob/main/docs/CREPUS_WEB_IMPLEMENTATION_SPEC.md) (see **§9** for MV3). This file is not part of the published docs site.

The `crepuscularity-webext` crate provides support for building Chromium and Firefox extensions with Manifest V3.

## Quick Start

```bash
crepus webext new my-extension
cd my-extension
crepus build
# Load dist/unpacked/ in chrome://extensions

crepus webext build --browser firefox
# Load dist/firefox/manifest.json in about:debugging#/runtime/this-firefox
```

## Configuration

Extensions are configured via `crepus.toml`:

```toml
[[targets]]
type = "webext"
id = "extension"
app = "."

[targets.extension]
name = "My Extension"
version = "1.0.0"
description = "A browser extension built with crepuscularity"

[targets.capabilities]
storage = true
background-script = true
content-script = true
host-permissions = ["https://example.com/*"]
```

`host-permissions` is emitted exactly as configured. Leave it empty for popup-only extensions with no page access, or add the narrow URL patterns your content scripts need. Crepuscularity does not broaden an empty list to `<all_urls>`.

### Capabilities

| Capability | Description |
|------------|-------------|
| `storage` | Access to `chrome.storage` API |
| `background-script` | Service worker script |
| `content-script` | Scripts injected into pages |
| `host-permissions` | Array of URL patterns |
| `tabs` | Access to `chrome.tabs` API |
| `notifications` | Desktop notifications |
| `context-menus` | Right-click context menus |
| `alarms` | Scheduled tasks |

## Project Structure

```
my-extension/
├── crepus.toml        # Extension target and configuration
├── runtime/           # Rust WASM runtime
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── views/             # .crepus templates
│   └── ui.crepus
└── dist/              # Build output
    └── unpacked/
        ├── manifest.json
        ├── popup.html
        └── ...
```

## Manifest Generation

The CLI generates a Chrome Manifest V3 from `crepus.toml`:

```bash
crepus webext manifest
crepus webext manifest --browser firefox
```

Output:
```json
{
  "manifest_version": 3,
  "name": "My Extension",
  "version": "1.0.0",
  "description": "A browser extension built with crepuscularity",
  "permissions": ["storage"],
  "host_permissions": ["https://example.com/*"],
  "background": {
    "service_worker": "src/background.js",
    "type": "module"
  },
  "content_scripts": [{
    "matches": ["https://example.com/*"],
    "js": ["src/content.js"]
  }],
  "content_security_policy": {
    "extension_pages": "script-src 'self' 'wasm-unsafe-eval'; object-src 'self'"
  }
}
```

`crepus webext build --browser chromium` writes `dist/chromium/`; `--browser firefox` writes `dist/firefox/`. Without `--browser`, the default remains `dist/unpacked/` for the existing Chromium load flow.

## Positioning

Crepuscularity is not trying to be only an extension framework. Treat webext as the browser-extension target inside a broader `.crepus` systems UI pipeline: the same source language can also target GPUI, Ratatui, web output, View IR for native shells, embedded framebuffers, and LVGL Pro.

Oxichrome is stronger when the job is “write the whole extension as a Leptos-style Rust app with proc-macro entrypoints.” Crepuscularity should compete where the target is “React Native on steroids”: one compact UI language, Rust-owned build/runtime plumbing, and multiple non-browser surfaces from the same template model. The webext target should keep improving around typed browser APIs, generated entrypoint glue, and browser-specific bundles while preserving least-privilege manifests.

## WASM Runtime

Extensions can include a Rust WASM runtime for complex logic:

```rust
// runtime/src/lib.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn process_data(input: &str) -> String {
    // Complex processing in Rust
    format!("Processed: {}", input)
}
```

Build with:
```bash
cd runtime
wasm-pack build --target web
```

## Templates

Use `.crepus` templates for extension UI:

```text
# views/popup.crepus
div popup flex flex-col gap-4 p-4
  h1 text-xl font-bold
    "My Extension"
  button px-4 py-2 bg-blue-500 text-white rounded @click=handle_action
    "Do Something"
```

## Capability Auto-Detection

The `crepuscularity-webext` scanner detects API usage in templates and suggests capabilities:

```rust
use crepuscularity_webext::{scan_crepus_for_capabilities, CapabilityUsage};

let usage = scan_crepus_for_capabilities("views/popup.crepus")?;
for cap in usage.suggested {
    println!("Suggest adding: {}", cap);
}
```

## API Reference

### BrowserProgram

Fluent builder for generating extension JavaScript:

```rust
use crepuscularity_webext::{BrowserProgram, StorageArea, JsExpr};

let program = BrowserProgram::new()
    .storage_get(StorageArea::Local, "key", "value")
    .storage_set(StorageArea::Sync, "key", JsExpr::literal("value"))
    .send_message("action", serde_json::json!({"data": "test"}))
    .finish();

println!("{}", program.to_js());
```

### ExtensionManifest

Use `crepus.toml` for project builds. The lower-level `ExtensionManifest` type remains available when you already have extension metadata in Rust:

```rust
use crepuscularity_webext::ExtensionManifest;

let manifest: ExtensionManifest = toml::from_str(source)?;
println!("{}", manifest.to_manifest_v3_json());
```
