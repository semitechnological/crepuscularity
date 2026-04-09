# Migration: `crepus web build` (site.json → `.crepus` + WASM)

**Also:** [Documentation home](README.md) · [DSL](dsl.md) · [CLI](cli.md)

## Current model (closed)

The **legacy `site.json` → static HTML** pipeline is **removed** as of **`crepuscularity-cli` 0.6.0**. There is no `--legacy-site-json` escape hatch.

Use a **site directory** with:

- **`*.crepus`** templates,
- a **`runtime/`** Cargo crate (**wasm32** + **wasm-bindgen**),
- optional **`site.json`** for **head metadata only** (title, description, OpenGraph image, theme CSS variables),
- optional **`web.toml`** for site name / `head_html` / `google_fonts`.

**Output:** **`index.html`**, **`crepus-bundle.json`**, **`app.js`**, **`pkg/`**, **`vendor/unocss.js`**, and (when the repo has a parent **`docs/`** folder) **`docs/*.html`** from Markdown.

## Monorepo: `crepus.toml`

At the repository root (or any directory), a **`crepus.toml`** can list **multiple WASM web sites** and still hold **`[ios]`** for XcodeGen apps:

```toml
[ios]
scheme = "MyApp"
xcodegen_spec = "project.yml"
destination = "platform=iOS Simulator,name=iPhone 16,OS=latest"

[[targets]]
type = "web"
id = "docs"
site = "docs-site"
out = "docs-site/dist"
entry = "index.crepus"

[[targets]]
type = "web"
id = "marketing"
site = "sites/marketing"
out = "sites/marketing/dist"
```

Shorthand for a **single** site (no `[[targets]]`):

```toml
[web]
site = "my-site"
out = "my-site/dist"
entry = "index.crepus"
```

- **`crepus web build`** — with **one** web target, that target is used automatically when **`--site`** is omitted (CLI walks up from cwd for **`crepus.toml`**). With **multiple** targets, pass **`--target <id>`** (or **`--site`** to ignore the manifest).
- **`crepus web serve`** — same **`--target`** / **`--manifest`** rules when **`--site`** is omitted.
- **`--manifest path/to/crepus.toml`** — skip walk-up and pin the manifest file.

Paths **`site`**, **`out`** are relative to the directory containing **`crepus.toml`**.

## Migrating from structured `site.json` pages

1. Run **`crepus web new <slug>`** (or copy **`examples/web-site`** / **`docs-site`**).
2. Move layout and copy from **`site.json`** `elements` into **`index.crepus`** (and partials).
3. Keep a **small `site.json`** only if you want shared SEO/theme for the HTML shell.
4. Replace **`crepus web build --site . -o dist/index.html`** with **`crepus web build --site . --out-dir dist`** (or **`--target <id>`** from **`crepus.toml`**).
5. Serve **`dist/`** over HTTP so **`fetch`** and WASM modules work.

## Release note summary

**`crepus web build`** is **`.crepus` + WASM** only; **`site.json`** is metadata-only inside each site directory; **`crepus.toml`** can list **`[ios]`** and multiple **`type = "web"`** targets.
