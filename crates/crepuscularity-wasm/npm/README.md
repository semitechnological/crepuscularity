# @tschk/crepus-wasm

The crepuscularity `.crepus` parser, compiled to WebAssembly. This is the same
Rust code that backs the `crepus` CLI — not a reimplementation — so JavaScript
consumers and native targets lower templates identically.

```ts
import { parseCrepus, IR_VERSION } from "@tschk/crepus-wasm";

const ir = parseCrepus(`div flex flex-col gap-4\n  span text-lg\n    "hello"`);
// { version: 6, root: [ { kind: "stack", ... } ] }
```

Template variables are bound by passing a context object:

```ts
parseCrepus('span\n  "hello {name}"', { name: "Ada" });
```

## View IR

`parseCrepus` returns a `ViewIr`, a versioned tree of `ViewNode`s. Every node
carries the original template class tokens on `style.classes`, so web renderers
can pass them straight through as `className` instead of re-deriving CSS from
the lowered style hints.

Check `IR_VERSION` if you persist or transport IR — the schema is versioned and
bumps on incompatible changes.

## Building

Requires the `wasm32-unknown-unknown` target and a `wasm-bindgen` CLI matching
the `wasm-bindgen` crate version.

```sh
./build.sh
```
