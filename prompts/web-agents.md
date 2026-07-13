# crepus web — static sites + WASM runtime

Write `.crepus` templates, ship as static HTML with a WASM renderer.

## Quick start

```bash
crepus web new my-site
cd my-site
crepus web dev --site . --port 4000
crepus web build --site .          # outputs dist/
```

## Project structure

```
my-site/
  index.crepus          # entry template (UnoCSS classes, indent-based)
  runtime/
    Cargo.toml          # dep: crepuscularity-web, wasm-bindgen
    src/lib.rs          # wasm_bindgen export: crepus_render(bundle_json)
  crepus.toml           # target config
  dist/                 # build output (index.html + app.js + pkg/ + crepus-bundle.json)
```

## WASM bridge (runtime/src/lib.rs)

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn crepus_render(bundle_json: &str) -> Result<String, JsValue> {
    crepuscularity_web::render_bundle(bundle_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))  // ALWAYS .to_string()
}
```

## Template syntax

```
div w-full min-h-screen bg-zinc-950 text-zinc-50 p-8 flex flex-col gap-4
 div text-3xl font-bold
   "Hello"
 div text-zinc-400 max-w-xl
   "Body text"
 a text-blue-400 hover:text-blue-300 href="https://example.com"
   "Link"
```

- Indentation = nesting (2 spaces)
- First word = HTML tag
- Remaining words on element line = UnoCSS classes
- `key="value"` = attribute bindings
- `"quoted string"` = text content
- `{expr}` = interpolation
- `#id` or `.class` on the element line
- `$: let x = expr` = variable declarations
- `if` / `for` / `match` control flow
- `include "file.crepus"` or `include "file.crepus#ComponentName"`
- `slot-rotate` for cycling phrases
- `embed src="..."` for islands (JS modules or WASM)

## Head block

```
head
  title "My Page"
  meta name="description" content="..."
  link rel="canonical" href="https://..."
  style
    .custom { color: red; }
  google-fonts: Inter
```

## Features

- UnoCSS runtime in browser (vendor/unocss.js)
- slot-rotate for animated rotating text
- Web Islands: embed dynamic JS modules or WASM components
- SEO config in crepus.toml (OG, Twitter, sitemap, robots.txt)
- Google Fonts auto-injection
- Hot-reload dev server with WASM recompilation

## crepus.toml

```toml
[[targets]]
type = "web"
id = "site"
site = "."
out = "dist"
entry = "index.crepus"
name = "My Site"

[targets.seo]
title = "My Site"
description = "..."
canonical = "https://example.com"
twitter_card = "summary_large_image"
twitter_site = "@handle"
```

## Key crates

- `crepuscularity-web` — HTML rendering, bundle parser, SSR
- `crepuscularity-core` — parser, AST, context, eval
- `crepuscularity-web` (`axum` feature) — Axum SSR handlers