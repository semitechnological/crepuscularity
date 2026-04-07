# DSL Reference

The `.crepus` template language uses indentation-based syntax for defining UI structures.

## Elements

Elements are declared with a tag name followed by Tailwind-style classes:

```text
div w-full h-full bg-zinc-950 text-white
  span text-sm
    "Hello"
```

### `#id` shorthand

Any element line may include `#name`, which emits `id="name"`:

```text
section #hero
  "Hello"
```

## Text and Interpolation

Quoted strings are text nodes. Use `{expr}` for interpolation:

```text
div
  "Hello {name}"
  "Score: {score * 10}"
```

The indent DSL also accepts one same-line quoted literal after the tag/classes/bindings. It becomes the first child text node, then any indented children follow:

```text
div #hero "Hello"
  span
    "World"
```

This is equivalent to:

```text
div #hero
  "Hello"
  span
    "World"
```

Bare expressions render as text:

```text
div
  {username}
```

## Control Flow

### Conditionals

```text
if {score > 100}
  div text-green-400
    "High score!"
else if {score > 50}
  div text-yellow-400
    "Medium score"
else
  div text-red-400
    "Low score"
```

### Match

```text
match {status}
  "active" =>
    div text-green-400
      "Active"
  "pending" =>
    div text-yellow-400
      "Pending"
  _ =>
    div text-gray-400
      "Unknown"
```

### For Loops

```text
for item in {items}
  div p-2 border-b
    {item.name}
```

## Variables

### Local Variables

```text
$: let total = {price * quantity}
div
  "Total: ${total}"
```

### Prop Defaults

Used in components to declare optional props:

```text
$: default subtitle = ""
$: default variant = "primary"
```

## Attributes

### Event Handlers

```text
button @click="handle_click"
  "Click me"
```

On the web/WASM path, `@click="name"` renders `data-onclick="name"` and the generated shell dispatches it to an exported `#[wasm_bindgen] pub fn name()`.

### Conditional Classes

```text
div class:hidden={!visible} class:active={selected}
  "Content"
```

### Dynamic Attributes

```text
input type="text" value={input_value} placeholder="Enter text"
```

This same attribute path is how you emit HTMX and Alpine attributes, for example `hx-get="/status"` or `x-text="n"`.

## Animations

```text
div animate:opacity={300ms ease-in-out}
  "Fades in"
```

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
| Comparison | `{a > b}`, `{x == y}`, `{n != 0}`, `{a >= b}` |
| Logical | `{a && b}`, `{x \|\| y}`, `{!flag}` |
| Property access | `{user.name}`, `{item.price}` |
| Parentheses | `{(a + b) * c}` |
