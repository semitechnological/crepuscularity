# Crepuscularity

A general syntax and runtime system for writing UI in a concise, indentation-based template language.

`crepuscularity` now has separate backend crates for:

- HTML rendering for websites and WASM-oriented flows
- React/JSX rendering for TSX-style integration
- GPUI rendering for native desktop apps

```text
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

- **General `.crepus` syntax** shared across backends
- **Tailwind-style classes** and utility-oriented element declarations
- **Control flow** — `if / else if / else`, `match`, `for`
- **String interpolation** — `"Hello {name}"`
- **Expressions** — arithmetic, comparison, logical operators, property access
- **Animations** — `animate:opacity={300ms ease-in-out}`
- **Components** — single-file and multi-component files with slot support
- **HTML backend** — render templates to normal HTML strings
- **React backend** — render templates to JSX/TSX-style output
- **GPUI backend** — separate crate with `view!` macro and GPUI prelude
- **Hot reload** — live template updates in the GPUI runtime path

## Getting started

Add the backend crates you want to your `Cargo.toml`:

```toml
[dependencies]
crepuscularity = { path = "crates/crepuscularity" }
crepuscularity-web = { path = "crates/crepuscularity-web" }
crepuscularity-react = { path = "crates/crepuscularity-react" }
crepuscularity-gpui = { path = "crates/crepuscularity-gpui" }
```

## HTML backend

```rust
use crepuscularity::prelude::*;

let content = std::fs::read_to_string("views/dashboard.crepus")?;
let mut ctx = TemplateContext::new();
ctx.set("username", "alice");
ctx.set("score", 1200);

let html = render_template_to_html(&content, &ctx)?;
```

## React backend

```rust
use crepuscularity::prelude::*;

let content = std::fs::read_to_string("views/dashboard.crepus")?;
let mut ctx = TemplateContext::new();
ctx.set("username", "alice");
ctx.set("score", 1200);

let jsx = render_template_to_jsx(&content, &ctx)?;
```

## GPUI backend

The GPUI-specific API lives in `crepuscularity-gpui`.

```rust
use crepuscularity_gpui::prelude::*;
```

## Components

### Single-file component

One `.crepus` file = one component. Declare optional props with `$: default`:

```text
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

```text
include components/card.crepus title="Hello" subtitle="World"
  div p-2
    "Slot content"
```

### Multi-component files

Collect related components into one file with a `+++` TOML frontmatter header and `--- Name` section separators.

```text
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

```text
include ui.crepus#Card title="Dashboard"
  div text-sm
    "Card body here"

include ui.crepus#Button label="Save" variant="primary"
```

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
| `$: default x = value` | Prop default |
| `class:hidden={cond}` | Conditional class |
| `@click=handler` | Event handler |
| `animate:opacity={300ms ease-in-out}` | Animation |
| `include path/file.crepus prop=val` | Single-file component |
| `include file.crepus#Name prop=val` | Named component |
| `slot` | Render parent slot or fallback children |

## Project structure

```text
crates/
  crepuscularity/           General syntax/runtime facade
  crepuscularity-core/      Shared AST, parser, context, evaluator
  crepuscularity-web/       HTML backend
  crepuscularity-react/     React/JSX backend
  crepuscularity-gpui/      GPUI backend crate
  crepuscularity_macros/    Proc-macro for compile-time GPUI `view!`
  crepuscularity-runtime/   GPUI runtime renderer and hot reload
  crepuscularity-dev/       crepu-dev binary
  crepuscularity-cli/       crepu CLI
```
