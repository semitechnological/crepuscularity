# Migration: `crepus web build` (site.json → `.crepus` + WASM)

## What changed

- **Default `crepus web build`** no longer turns **`site.json`** into a single static HTML document.
- It now expects a **site directory** with:
  - **`*.crepus`** templates (same include/virtual-file rules as **`crepus web serve`**),
  - a **`runtime/`** Cargo crate (same **wasm32 + wasm-bindgen** model as **`crepus webext build`**),
  - optional **`site.json`** for **head metadata only** (title, description, OpenGraph image, theme CSS variables).
- **Output** is a directory (default **`dist/`** under the site): **`index.html`**, **`crepus-bundle.json`**, **`app.js`**, **`pkg/`** (wasm-bindgen), **`vendor/unocss.js`**.

## Content and data

- **Static copy** belongs in **`.crepus`** using normal text nodes, e.g. indented **`"Hello"`** lines (same as examples and extension views).
- **Rust-side context** (lists, fetched values, etc.): implement in **`runtime/src/lib.rs`** by parsing the bundle’s **`files`** map and calling **`crepuscularity_web::render_from_files`** with a **`TemplateContext`** you build in Rust — parallel to how **`crepus webext`** apps own their WASM API.

## Migrating a consumer (e.g. undivisible.dev)

1. Run **`crepus web new <slug>`** in a scratch folder and copy the **`runtime/`** layout, or start from **`examples/web-site`** in this repo.
2. Move landing content from **`site.json`** `elements` into **`index.crepus`** (and partials), using quoted strings for text.
3. Keep a **small `site.json`** only if you still want shared SEO/theme for the HTML shell.
4. Replace **`crepus web build --site . -o dist/index.html`** with **`crepus web build --site . --out-dir dist`** (or omit **`--out-dir`** to use **`./dist`**).
5. Serve **`dist/`** with any static host (GitHub Pages, etc.). Open **`index.html` via HTTP** so **`fetch`** and WASM modules work.

## Temporary escape hatch

```bash
crepus web build --legacy-site-json …
```

behaves like the pre-v3 **`site.json` → HTML** pipeline (including **`-`** stdin and **`-o file.html`**).

## Release note summary

**`crepus web build`** is now **`.crepus`-first** with a **WASM runtime** aligned with **`crepus webext`**; **`site.json`** is optional shell metadata only unless **`--legacy-site-json`** is used.
