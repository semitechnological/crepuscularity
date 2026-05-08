# DSL Reference

**Also:** [Documentation home](README.md) · [Components](components.md) · [CLI](cli.md) · [Extensions](webext.md)

The `.crepus` template language supports **two equivalent input syntaxes** that compile to the same AST and work with every backend (GPUI, web, webext). Auto-detected by whether the first content line starts with `<`.

## Indentation Syntax (Native)

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

## JSX / HTML Tag Syntax

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

## Pragmas

File-level directives go at the very top, before any elements.

```
google-font "Inter"
google-fonts "Inter" "JetBrains Mono"
```

`google-font` loads a single family; `google-fonts` loads multiple in one pragma. Both inject the appropriate `<link>` tag on the web path.

## Element Syntax

`tag classes… #id "inline-text"` on one line. `#id` emits `id="…"` on the element; an inline quoted literal becomes the first child text node.

```
section py-16 #hero "Hello"
  span text-sm
    "World"
```

## Native SwiftUI Component Tags

When `.crepus` is compiled through Aurorality `swiftgen`, templates can declare native SwiftUI-style components directly. This semantic layer is additive: utility-first layout (`div flex`, `div flex-col`, spacing classes, etc.) still works and can be mixed in the same tree.

### Supported semantic tags (swiftgen path)

| Category | Tags |
| --- | --- |
| Navigation | `navigationstack`, `navigationsplitview`, `sidebar`, `detail`, `content` |
| Layout containers | `vstack`, `hstack`, `zstack`, `group`, `form`, `section`, `list`, `item`, `scrollview`, `scroll`, `spacer`, `divider` |
| Controls | `button`, `toggle`, `picker`, `textfield`, `input`, `securefield`, `menu` |
| Content | `label`, `image`, `sf`, `sf-*`, `span`, `p` |

### Example: hybrid semantic + utility layout

```text
navigationsplitview
  sidebar
    vstack gap-2 p-3 w-72
      hstack
        label title="Inbox" symbol="tray"
        spacer
        button button-bordered @click="newConversation"
          sf-plus
      sidebar flex-grow
        for row in {rows}
          item @click="select:{row.id}"
            span "{row.title}"
  detail
    vstack gap-2 p-3
      textfield bind=draft placeholder="Message"
      hstack
        spacer
        button button-prominent @click="send"
          "Send"
```

### Notes

- `div flex` and `div flex-col` remain first-class and continue to map to `HStack` / `VStack`.
- Semantic tags are primarily for readability and explicit intent when building native shells.
- Backends may support different subsets; the table above applies to the Aurorality SwiftUI `swiftgen` path.

## Text Nodes

Quoted strings are text nodes. `{expr}` inside quotes interpolates. A bare `{expr}` on its own line renders the value directly.

```
div
  "Hello {name}, score: {score * 10}"
  {username}
```

## Control Flow

```
if {score > 100}
  div text-green-400
    "High score!"
else if {score > 50}
  div text-yellow-400
    "Medium"
else
  div text-red-400
    "Low"

for item in {items}
  div p-2 border-b
    {item.name}

match {status}
  "active" =>
    div text-green-400
      "Active"
  _ =>
    div text-gray-400
      "Unknown"
```

## Variables

`$: let` computes a local variable from an expression; `$: default` sets a variable only when it is not already in context (canonical way to declare optional component props).

```
$: let total = {price * quantity}
$: default variant = "primary"
```

## Attributes

Static values use `key=value`, dynamic values use `key={expr}`. Event handlers use `@event="fn_name"` (on the web/WASM path this emits `data-onclick="fn_name"` and dispatches to an exported Rust function). Conditional classes use `class:name={expr}`.

```
input type="text" value={input_value} placeholder="Enter text"
button @click="handle_submit"
  "Submit"
div class:hidden={!visible} class:active={selected}
  "Content"
```

## Slot-Rotate

`slot-rotate` is a built-in widget for cycling through text children at a fixed interval. Give it an `interval={ms}` attribute and optionally a class-alias name to style the active item. Each indented string child is one rotation slot.

```
slot-rotate interval={3200} slot-lede
  "a GPUI-first template pipeline"
  "a .crepus DSL with hot reload"
  "one syntax for GPUI, web, and extensions"
```

## Include

`include` embeds another component. Props are passed as `key=value` or `key={expr}`. Indented children under the include call become the `slot` content inside the component.

```
include components/card.crepus title={t} subtitle={sub}
  div p-4
    "Slot content rendered inside the component"
```

## Animations

`animate:property={duration timing-function}` attaches a CSS transition or entry animation to the element.

```
div animate:opacity={300ms ease-in-out}
  "Fades in on mount"
```

For web builds, you can also add trailing raw CSS (including `@keyframes`, class rules, and media queries) directly in the `.crepus` file without wrapping it in `<style>`:

```css
@keyframes sunset {
  0% { opacity: 0.6; }
  100% { opacity: 1; }
}

.animate-sunset {
  animation: sunset 10s ease-in-out infinite alternate;
}
```

That CSS is collected and injected into the generated document head automatically by `crepus web build` and `crepus web serve`.

Note: class-alias lines (`.alias ...`) still belong at the very bottom of the file. Put trailing CSS above alias lines (or at the end if you have no aliases).

## Class Aliases

Lines at the bottom of a `.crepus` file starting with `.name` define reusable class groups. Any element with that name in its class list expands to the aliased classes at render time.

```
.slot-lede text-zinc-100 font-medium
.footer-row flex flex-col sm:flex-row justify-between gap-4 text-sm text-zinc-500
```

Usage: `div footer-row` expands to `div flex flex-col sm:flex-row justify-between gap-4 text-sm text-zinc-500`.

## Comments

Lines starting with `#` are comments:

```text
# This is a comment
div
  # Another comment
  "Hello"
```

## Expression Operators

| Operator | Example |
|----------|---------|
| Arithmetic | `{a + b}`, `{x * y}`, `{n / 2}`, `{n % 2}` |
| Comparison | `{a > b}`, `{x == y}`, `{n != 0}`, `{a >= b}`, `{a <= b}` |
| Logical | `{a && b}`, `{x \|\| y}`, `{!flag}` |
| Property access | `{user.name}`, `{item.price}` |
| Parentheses | `{(a + b) * c}` |
