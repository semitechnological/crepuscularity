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

## Where each piece comes from

| Source | Kind | Becomes |
| --- | --- | --- |
| `templates/page.crepus` | indentation syntax | `src/App.tsx` |
| `templates/badge.csx` | crepusx (JSX-flavoured) | `src/components/Badge.tsx` |
| inline in `src/main.rs` | raw string literal | `src/components/Footer.tsx` |
| `ts/Counter.tsx` | hand-written React | `src/ts/Counter.tsx` |

`src/main.tsx` composes all four. `crepus moon build` writes that file once and
then leaves it alone, so edits there survive a rebuild; `src/App.tsx` and
`src/components/` are regenerated from the templates every run.

## Interactivity

The generated component takes two props:

```tsx
<App scope={{ count, tags: TAGS }} handlers={{ increment: () => setCount(n => n + 1) }} />
```

`scope` supplies the values template expressions read (`if {count > 0}`,
`for tag in {tags}`), and `handlers` supplies the functions its events call
(`button @click=increment`). `if`/`else` become real conditionals and `for`
becomes `.map()`, so the template decides what renders. State itself lives in
ordinary React in `src/main.tsx` — the template does not own it.

## What this example does not do yet

`on_long_press` has no DOM equivalent, so it is emitted as
`data-crepus-on-long-press` for a host to pick up rather than invented as an
event.

`bun` is required to produce `dist/`, because Moonshine is a JavaScript
runtime. Nothing in this project is authored in JavaScript.

## Add more templates

Add more `const` raw strings to `src/main.rs`, render them with
`render_template_to_ir`, and emit them with `emit_moonshine_app` (a page) or
`emit_moonshine_component(&ir, "Name")` (a reusable component) — write the
result under `generated/` and `crepus moon build` will pick it up.
