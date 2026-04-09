# Browser Extensions

**Also:** [Documentation home](README.md) · [DSL](dsl.md) · [Components](components.md) · [CLI](cli.md)

> **Implementation spec (web + webext + cross-language perf patterns):** [CREPUS_WEB_IMPLEMENTATION_SPEC.md](./CREPUS_WEB_IMPLEMENTATION_SPEC.md) — canonical doc for AI/agents; **§9** covers MV3 and this crate.

The `crepuscularity-webext` crate provides support for building Chrome/Firefox extensions with Manifest V3.

## Quick Start

```bash
crepus webext new my-extension
cd my-extension
crepus webext build
# Load dist/unpacked/ in chrome://extensions
```

## Configuration

Extensions are configured via `webext.toml`:

```toml
[extension]
name = "My Extension"
version = "1.0.0"
description = "A browser extension built with crepuscularity"

[capabilities]
storage = true
background-script = true
content-script = true
host-permissions = ["https://example.com/*"]
```

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
├── webext.toml        # Extension configuration
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

The CLI generates a Chrome Manifest V3 from `webext.toml`:

```bash
crepus webext manifest
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
    "matches": ["<all_urls>"],
    "js": ["src/content.js"]
  }],
  "content_security_policy": {
    "extension_pages": "script-src 'self' 'wasm-unsafe-eval'; object-src 'self'"
  }
}
```

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

Load and generate manifests:

```rust
use crepuscularity_webext::ExtensionManifest;

let manifest = ExtensionManifest::load("webext.toml")?;
println!("{}", manifest.to_manifest_v3_json());
```
