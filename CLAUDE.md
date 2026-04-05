# Crepuscularity — Claude Context

## What this project is

A UI framework that lets you write GPUI interfaces in a concise, indentation-based template DSL (`.crepus` files) instead of raw Rust. Templates can either be compiled at build time via the `view!` macro or rendered at runtime with full hot-reload support.

## Build

```bash
SDKROOT=$(xcrun --show-sdk-path) cargo build
```

`dispatch.h` lives inside the Xcode SDK, not `/usr/include`, so the explicit `SDKROOT` is always required on macOS without the extra command-line tools package. You can also add `SDKROOT=$(xcrun --show-sdk-path)` to your shell profile to avoid repeating it.

## Before pushing / CI requirements

All three checks must pass before pushing:

```bash
cargo fmt --all -- --check          # formatting
cargo clippy --all-features --workspace -- -D warnings  # lints
cargo test --workspace              # tests
```

To auto-fix formatting: `cargo fmt --all`

## Workspace layout

| Crate | Purpose |
|---|---|
| `crates/crepuscularity` | Main library re-exporting `view!` macro + prelude |
| `crates/crepuscularity_macros` | Proc-macro: compiles `.crepus` DSL strings at build time |
| `crates/crepuscularity-runtime` | Runtime parser, renderer, and hot-reload engine |
| `crates/crepuscularity-dev` | `crepus-dev` binary — hot-reload dev server |
| `crates/crepuscularity-cli` | `crepus` CLI for scaffolding and builds |
| `examples/text-features` | GPUI demo for letter-spacing and text-transform (vendored gpui) |
| `examples/weather` | Full weather-app example using the runtime |

**Web / compiler / hot-reload implementation spec (single doc for agents):** [docs/CREPUS_WEB_IMPLEMENTATION_SPEC.md](docs/CREPUS_WEB_IMPLEMENTATION_SPEC.md)

## DSL quick reference

`.crepus` files support **two equivalent input syntaxes** that compile to the same AST and work with every backend (GPUI, web, webext). Auto-detected by whether the first content line starts with `<`.

### Indentation syntax (native)

Elements are `tag classes…` with indented children.

```
div w-full h-full bg-zinc-950 text-white flex flex-col gap-4
  div text-2xl font-bold
    "Hello {name}"
  if {score > 50}
    div text-green-400
      "High score!"
  else
    div text-red-400
      "Low score"
  for item in {list}
    div p-2 border rounded
      {item}
```

### JSX / HTML tag syntax

For developers familiar with React/TSX. Same semantics, angle-bracket style.

```jsx
<div class="w-full h-full bg-zinc-950 text-white flex flex-col gap-4">
  <div class="text-2xl font-bold">Hello {name}</div>
  <if condition={score > 50}>
    <div class="text-green-400">High score!</div>
    <else><div class="text-red-400">Low score</div></else>
  </if>
  <for let="item" in={list}>
    <div class="p-2 border rounded">{item}</div>
  </for>
</div>
```

Control-flow tags: `<if condition={...}>`, `<else>`, `<else-if condition={...}>`, `<for let="var" in={list}>`, `<match on={expr}><case pattern="...">`.
Include: `<include src="file.crepus#Card" title={t} />` or with slot children.
Declarations: `<let name="x" value={42} />`, `<let-default name="x" value={42} />`, or `$: let x = 42` lines still work.

## Component system

### Single-file components (classic)

One `.crepus` file = one component. Included with `include path/to/file.crepus prop=value`.

### Multi-component files (new)

Collect related components into one file using a `+++` TOML frontmatter block followed by `--- Name` section separators.

```
include ui.crepus#Card title="Hello" subtitle="World"
  div
    "Slot content"
```

See `examples/ui.crepus` for the format and `examples/ui-demo.crepus` for usage.

## Architecture notes

- **Two rendering paths**: compile-time (`view!` macro → `crepuscularity_macros`) and runtime (`parse_template` / `render_nodes` → `crepuscularity-runtime`). The `view!` macro does *not* support `include` or multi-component files; those are runtime-only.
- **Context**: props and variables live in `TemplateContext { vars: HashMap<String, TemplateValue>, base_dir, slot }`. Components get a fresh child context with parent props injected.
- **Slot system**: `slot` in a component template renders the caller's children (with the *caller's* context), or the component's fallback children if no slot was passed.
- **`$: default name = value`**: sets a variable only when it isn't already in the context — the canonical way to declare optional props with defaults inside a single-file component.
- **TOML defaults** in multi-component files: `[ComponentName.defaults]` section — evaluated before passed props so props always win.
- **Evaluator** (`eval.rs`): arithmetic, comparison, logical operators, property access (`obj.prop`). No function calls.

## Conventions

- Keep DSL changes mirrored between `crepuscularity_macros` and `crepuscularity-runtime` when both paths should support the feature.
- Error messages from the renderer use a red `div` with the message text — no panics.
- Multi-component files live alongside single-component ones; the `#Name` fragment in the include path is the only disambiguation.
