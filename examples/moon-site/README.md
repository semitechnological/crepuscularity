# moon-site

Rust-only Moonshine app scaffolded by `crepus moon new` (alias for
`crepus moonshine new`).

crepus templates are inlined as raw string literals in `src/main.rs`. Running
the project (`cargo run`) parses them with
`crepuscularity_native::render_template_to_ir` and emits a real Moonshine
React/TSX app under `generated/` via `emit_moonshine_app` /
`emit_moonshine_component`.

## Build

```bash
crepus moon build
```

This runs `cargo run` to (re)generate `generated/`, refreshes
`package.json`, `index.html`, `vite.config.ts`, and `tsconfig.json`, copies
`ts/` into the app so your TypeScript modules stay importable, and then runs
`bun install && bun run build` if `bun` is on PATH. If `bun` is missing, the
project is still fully generated — install bun and re-run `crepus moon
build`, or run the printed commands yourself.

## Add TypeScript modules or npm libraries

- Drop `.ts`/`.tsx` files in `ts/` — they're copied in and importable from
  the generated app.
- This is a real bun project after `crepus moon build`: run `bun add
  <package>` in the project directory like any other bun/Vite app.

## What this example does not do yet

The emitted JSX is structural. Event handlers (`@click`) are not wired to
Moonshine signals, and `if` / `for` are emitted as `data-crepus-if` /
`data-crepus-for-each` attributes rather than live control flow, so this path
suits content-driven pages rather than interactive state today.

`bun` is required to produce `dist/`, because Moonshine is a JavaScript
runtime. Nothing in this project is authored in JavaScript.

## Add more templates

Add more `const` raw strings to `src/main.rs`, render them with
`render_template_to_ir`, and emit them with `emit_moonshine_app` (a page) or
`emit_moonshine_component(&ir, "Name")` (a reusable component) — write the
result under `generated/` and `crepus moon build` will pick it up.
