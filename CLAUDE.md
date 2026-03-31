# Crepuscularity — Claude Context

## What this project is

A UI framework that lets you write GPUI interfaces in a concise, indentation-based template DSL (`.crepus` files) instead of raw Rust. Templates can either be compiled at build time via the `view!` macro or rendered at runtime with full hot-reload support.

## Build

```bash
SDKROOT=$(xcrun --show-sdk-path) cargo build
```

`dispatch.h` lives inside the Xcode SDK, not `/usr/include`, so the explicit `SDKROOT` is always required on macOS without the extra command-line tools package. You can also add `SDKROOT=$(xcrun --show-sdk-path)` to your shell profile to avoid repeating it.

## Workspace layout

| Crate | Purpose |
|---|---|
| `crates/crepuscularity` | Main library re-exporting `view!` macro + prelude |
| `crates/crepuscularity_macros` | Proc-macro: compiles `.crepus` DSL strings at build time |
| `crates/crepuscularity-runtime` | Runtime parser, renderer, and hot-reload engine |
| `crates/crepuscularity-dev` | `crepu-dev` binary — hot-reload dev server |
| `crates/crepuscularity-cli` | `crepu` CLI for scaffolding and builds |
| `examples/weather` | Full weather-app example using the runtime |

## DSL quick reference

Elements are `tag classes…` with indented children. See `examples/demo.crepus` for an exhaustive feature tour.

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
