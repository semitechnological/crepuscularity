# Input Frontends

**Also:** [Documentation home](README.md) · [DSL reference](dsl.md) · [View IR contract](view-ir-contract.md) · [Polyglot plugins](polyglot.md)

The core parser has **four frontends**. They accept four different syntaxes and
produce **one shared AST** (`crepuscularity_core::ast::Node`, `crates/crepuscularity-core/src/ast.rs`).
Lowering that AST is a separate step: `render_nodes_to_ir`
(`crates/crepuscularity-native/src/render.rs`) turns it into the single
[View IR](view-ir-contract.md), which is what SwiftUI, Jetpack Compose, the
Moonshine TSX emit, the C ABI, Tauri, embedded, and the polyglot plugins consume.
The GPUI, TUI, LVGL, and web renderers walk the shared AST directly instead.

None of this depends on the frameworks whose syntax it accepts. There is no
`svelte`, `vue`, or JSX toolchain in any `Cargo.toml` — all four frontends are
hand-written Rust.

## Selection

`parse_template_with_path(source, path)` picks the frontend from the file
extension (`crates/crepuscularity-core/src/parser/mod.rs`), in this order:

| Extension                     | Frontend | Module               |
| ----------------------------- | -------- | -------------------- |
| `.vue`                        | Vue SFC  | `parser/vue/`        |
| `.svelte`                     | Svelte   | `parser/svelte/`     |
| `.csx`, `.jsx`, `.tsx`        | JSX      | `parser/jsx/`        |
| anything else (`.crepus`, …)  | indent   | `parser/indent/`     |

With no path, the JSX frontend still activates when the first non-blank,
non-`#`, non-`$:` line starts with `<`; otherwise the indent frontend runs. The
`.svelte` and `.vue` frontends are **path-driven only** — there is no content
heuristic for them.

From JavaScript, `parseTemplate(source, filename)` in
`@tschk/crepuscularity-wasm` is the one entry point for all four; the filename
is what selects the frontend.

```ts
import { parseTemplate } from "@tschk/crepuscularity-wasm";

parseTemplate(indentSource, "App.crepus");
parseTemplate(jsxSource, "App.tsx");
parseTemplate(svelteSource, "Counter.svelte");
parseTemplate(vueSource, "Counter.vue");
```

All four return the same `ViewIr` shape.

## What the Svelte and Vue frontends actually compile

**The template only.** `<script>` and `<style>` blocks are extracted verbatim
and never parsed or executed. Everything that gives Svelte and Vue their
component semantics lives in `<script>`, so none of it runs:

- **Svelte:** runes (`$state`, `$derived`, `$effect`, `$props`), `$:` reactive
  statements, stores and `$store` access, imports, lifecycle (`onMount`,
  `onDestroy`, `tick`), context API.
- **Vue:** the Composition API (`ref`, `reactive`, `computed`, `watch`,
  `defineProps`, `defineEmits`), the Options API, lifecycle hooks, provide /
  inject.

Expressions in the markup are evaluated by the crepuscularity template
evaluator against the crepuscularity template context, not by a JavaScript
engine. `<style scoped>` is recorded but scoping is not implemented.

Unsupported markup constructs are **hard parse errors**, not silent drops. A
template that uses them fails to compile rather than rendering something
subtly wrong.

### Svelte

Supported: HTML elements and nesting, comments, `{expr}` interpolation,
`{#if}` / `{:else if}` / `{:else}`, `{#each list as item}`, `{@html expr}`,
static `class`, `class:foo={cond}`, `on:click={h}` with modifiers, Svelte 5
`onclick={h}` for the known DOM event names, `bind:value|checked|group|files`,
and `{value}` attribute shorthand.

Rejected with an error:

- Every block except `{#if}` and `{#each}` — `{#await}`, `{#key}`, `{#snippet}`.
- Every `{@…}` tag except `{@html}` — `{@render}`, `{@const}`, `{@debug}`.
- `{#each}` without `as`, with an index (`as item, i`), with destructuring
  (`as {a, b}`), or with an `{:else}` branch. The shared `ForBlock` binds one
  item variable and has no empty-list branch.
- The directives `use:`, `transition:`, `in:`, `out:`, `animate:`, `let:`,
  `style:` — so no actions, transitions, animations, slot props, or inline
  style directives.
- Spread attributes `{...props}`.
- `<slot>` — use crepuscularity `include` slots instead.
- Any capitalised tag. This frontend compiles markup and performs no module
  resolution, so there is **no component composition at all**.
- `<svelte:window>` and the other `<svelte:*>` special elements.

Silently dropped: the key expression in `{#each … as item (key)}`.

### Vue

Supported: SFC splitting, `{{ expr }}` mustaches, `v-if` / `v-else-if` /
`v-else`, `v-for` (`item in items`, `(item, i) in items`, `of` form), `v-show`
(lowered to a conditional `hidden` class), `v-html`, `v-text`, plain `v-model`,
`v-bind:x` / `:x` including `:class` with string, object, and array syntax,
`v-on:evt` / `@evt` with the modifiers `prevent`, `stop`, `self`, `once`,
`capture`, `<template>` as a structural wrapper, void elements, comments.

Rejected with an error:

- Object spread `v-bind="obj"` — bind properties individually.
- `:style` — use utility classes.
- `v-bind` modifiers (`:x.prop`), dynamic binding arguments (`:[key]`),
  dynamic event names (`@[evt]`).
- `v-model` modifiers (`v-model.number`, `.trim`, `.lazy`).
- Any event modifier outside the five above — including **all key modifiers**,
  so `@keyup.enter` is an error.
- Every other directive: `v-once`, `v-memo`, `v-pre`, `v-cloak`, `v-slot`, the
  `#name` slot shorthand, and any custom directive.
- More than one top-level `<template>`, unclosed `<template>`, missing
  `</script>`, trailing markup after the template.

Silently dropped: the `v-for` index variable. Unlike Svelte, a capitalised tag
is **not** an error — it becomes an element with that tag name, and since no
module resolution happens, custom components render as unknown elements.

## Limits

All three tag-based frontends cap nesting depth at 256 and error beyond it.
Errors carry a message and a byte offset (`RawParseError`).
