# crepus webext — browser extensions (MV3)

Build Chromium/Firefox/Safari extensions with `.crepus` templates + WASM runtime.

## Quick start

```bash
crepus webext new my-ext
cd my-ext
crepus webext build --browser chromium --release
crepus webext dev                    # watch + auto-reload
crepus webext manifest               # print manifest.json
```

## Project structure

```
my-ext/
  app/
    index.crepus          # popup / options page template
    manifest.crex         # extension manifest (TOML)
    runtime/
      Cargo.toml
      src/lib.rs          # WASM exports for background + content scripts
  crepus.toml             # optional build config
```

## manifest.crex

```toml
[extension]
name = "my-extension"
version = "1.0.0"

[capabilities]
content-script = true
storage = true
messaging = true
host-permissions = ["https://example.com/*"]
```

## Runtime exports

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn crepus_render(bundle_json: &str) -> Result<String, JsValue> {
    crepuscularity_web::render_bundle(bundle_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
```

Additional WASM exports for background scripts, content scripts, commands.

## Supported browsers

- `chromium` — Chrome, Edge, Brave, Opera
- `firefox` — Firefox Desktop + Android
- `safari` — Safari (macOS + iOS, via Xcode)

## Capabilities

Auto-detected from `.crepus` usage. Declare in manifest.crex:

- `storage` — chrome.storage API
- `messaging` — runtime messaging
- `content-script` — page-level JS injection
- `tabs` — tab manipulation
- `bookmarks` — bookmark access
- `history` — browsing history
- `commands` — keyboard shortcuts
- `context-menus` — right-click menus
- `notifications` — OS notifications
- `alarms` — scheduled tasks
- `scripting` — dynamic script injection

## Key crates

- `crepuscularity-webext` — manifest gen, capability scanning, WASM bridge
- `crepuscularity-web` — template rendering
- `crepuscularity-core` — parser, AST