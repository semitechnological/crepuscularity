# Components

Crepuscularity supports reusable components via `include` statements.

## Single-File Components

One `.crepus` file = one component. Use `$: default` to declare optional props:

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
      "No content provided"
```

Include from another template:

```text
include components/card.crepus title="Dashboard" subtitle="Overview"
  div p-4
    "Card body content"
```

## Multi-Component Files

Group related components in a single file with TOML frontmatter:

```text
+++
[Card]
description = "A bordered card"

[Card.defaults]
title = "Untitled"

[Button]
description = "A clickable button"

[Button.defaults]
label = "Click"
variant = "primary"
+++

--- Card
div rounded-lg border border-zinc-700 p-4
  div font-bold
    {title}
  slot

--- Button
$: default variant = "primary"
button px-4 py-2 rounded cursor-pointer
  {label}
```

Include with `#Name` fragment:

```text
include ui.crepus#Card title="Settings"
  div
    "Settings content"

include ui.crepus#Button label="Save" variant="primary"
```

## Slots

The `slot` keyword renders content passed by the parent:

```text
# Layout component
div flex flex-col min-h-screen
  header bg-zinc-900 p-4
    "Header"
  main flex-1
    slot
  footer bg-zinc-900 p-4
    "Footer"
```

Usage:

```text
include layout.crepus
  div p-8
    "Main content goes here"
```

### Fallback Content

Slot can have fallback children if nothing is passed:

```text
slot
  div text-gray-500
    "Default content"
```

## Component Context

Each component gets its own context with:

- Props passed via `prop=value` syntax
- Defaults from `$: default` or TOML `[Component.defaults]`
- The slot content (accessible via `slot` keyword)

Props always override defaults:

```text
# In component:
$: default color = "blue"

# In parent:
include component.crepus color="red"  # color is "red"
include component.crepus              # color is "blue"
```

## Runtime vs Compile-Time

The `include` statement and multi-component files are **runtime-only** features. The compile-time `view!` macro does not support includes.

For compile-time composition, use Rust functions or the GPUI component model directly.
