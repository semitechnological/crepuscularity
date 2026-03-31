# Crepuscularity

A UI framework for writing [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) interfaces in a concise, indentation-based template language.

```
div w-full h-full bg-zinc-950 text-white flex flex-col p-8
  div text-2xl font-bold mb-4
    "Hello {name}"
  if {score > 50}
    div text-green-400
      "High score!"
  else
    div text-red-400
      "Keep going"
```

## Features

- **Tailwind-style classes** mapped to GPUI styled methods
- **Control flow** — `if / else if / else`, `match`, `for`
- **String interpolation** — `"Hello {name}"`
- **Expressions** — arithmetic, comparison, logical operators, property access
- **Animations** — `animate:opacity={300ms ease-in-out}`
- **Components** — single-file and multi-component files with slot support
- **Hot reload** — live template updates without recompiling Rust
- **Compile-time macro** — `view!` for zero-runtime-cost layouts

## Getting started

Add to your `Cargo.toml`:

```toml
[dependencies]
crepuscularity = { path = "crates/crepuscularity" }
crepuscularity-runtime = { path = "crates/crepuscularity-runtime" }
```

> **macOS note**: GPUI requires the Xcode SDK. Set `SDKROOT=$(xcrun --show-sdk-path)` before building.

## Compile-time templates

Use the `view!` macro to embed templates directly in Rust. The template is parsed and compiled to GPUI builder calls at build time.

```rust
use crepuscularity::prelude::*;

impl Render for MyView {
    fn render(&mut self, _cx: &mut Context<Self>) -> impl IntoElement {
        view! {
            div w-full h-full bg-zinc-950 text-white p-8
              div text-2xl font-bold
                "Hello world"
        }
    }
}
```

## Runtime templates

Use the runtime crate for hot-reloadable templates read from `.crepus` files at runtime.

```rust
use crepuscularity_runtime::{TemplateContext, parse_template, render_nodes};

let content = std::fs::read_to_string("views/dashboard.crepus")?;
let nodes = parse_template(&content)?;

let mut ctx = TemplateContext::new();
ctx.set_str("username", "alice");
ctx.set_int("score", 1200);

let element = render_nodes(&nodes, &ctx);
```

## Components

### Single-file component

One `.crepus` file = one component. Declare optional props with `$: default`:

```
# components/card.crepus
$: default subtitle = ""
div rounded-lg border border-zinc-700 p-4
  div font-bold
    {title}
  if {subtitle}
    div text-sm text-zinc-400
      {subtitle}
  slot
    div text-zinc-500 italic
      "No content"
```

Include it from another template:

```
include components/card.crepus title="Hello" subtitle="World"
  div p-2
    "Slot content"
```

### Multi-component files

Collect related components into one file with a `+++` TOML frontmatter header and `--- Name` section separators. Good for small apps where one file per component is overkill.

```
+++
[Card]
description = "A bordered card"

[Card.defaults]
title = "Untitled"

[Button]
description = "A clickable button"

[Button.defaults]
label = "Click me"
variant = "primary"
+++

--- Card
div rounded-lg border border-zinc-700 p-4
  div font-bold
    {title}
  slot

--- Button
$: default variant = "primary"
button cursor-pointer px-4 py-2 rounded
  {label}
```

Include a named component with the `#Name` fragment:

```
include ui.crepus#Card title="Dashboard"
  div text-sm
    "Card body here"

include ui.crepus#Button label="Save" variant="primary"
```

The TOML `[ComponentName.defaults]` values are injected before passed props, so passed props always win. Components can also use `$: default` inside the template body for the same effect.

## DSL reference

| Syntax | Meaning |
|---|---|
| `div class-a class-b` | Element with Tailwind classes |
| `"text {expr}"` | Text with interpolation |
| `{expr}` | Inline expression (rendered as text) |
| `if {cond}` / `else if` / `else` | Conditional |
| `match {expr}` / `"value" =>` / `_ =>` | Pattern match |
| `for item in {list}` | Loop |
| `$: let x = {expr}` | Local variable |
| `$: default x = value` | Prop default (skipped if already set) |
| `class:hidden={cond}` | Conditional class |
| `@click=handler` | Event handler (compile-time only) |
| `animate:opacity={300ms ease-in-out}` | Animation |
| `include path/file.crepus prop=val` | Single-file component |
| `include file.crepus#Name prop=val` | Named component from multi-component file |
| `slot` | Render parent slot, or fallback children |

## Examples

```bash
# Run the full demo
SDKROOT=$(xcrun --show-sdk-path) cargo run -p crepuscularity-dev -- examples/demo.crepus

# Run the multi-component demo
SDKROOT=$(xcrun --show-sdk-path) cargo run -p crepuscularity-dev -- examples/ui-demo.crepus

# Hot reload: edit examples/ui.crepus or examples/ui-demo.crepus while it runs
```

## Project structure

```
crates/
  crepuscularity/           Main library (re-exports view! macro + prelude)
  crepuscularity_macros/    Proc-macro for compile-time view! templates
  crepuscularity-runtime/   Runtime parser, renderer, hot-reload
  crepuscularity-dev/       crepu-dev binary
  crepuscularity-cli/       crepu CLI
examples/
  demo.crepus               Full DSL feature tour
  ui.crepus                 Multi-component file (Card, Badge, Button, Alert)
  ui-demo.crepus            Demo using ui.crepus
  components/card.crepus    Classic single-file component
```
