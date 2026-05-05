# Runtime And Reactivity

**Also:** [Documentation home](README.md) · [Production readiness](production.md) · [Web implementation spec](CREPUS_WEB_IMPLEMENTATION_SPEC.md)

Crepuscularity has two runtime layers:

- `crepuscularity-runtime` parses and renders `.crepus` templates for GPUI and hot-reload desktop apps.
- `crepuscularity-reactive` provides `Signal`, `Memo`, and `Effect` for WASM hydration and client-side updates.

## State model

Use `Signal<T>` for mutable state, `Memo<T>` for cached derived values, and `Effect` for DOM or app-side side effects.

```rust
use crepuscularity_reactive::{Effect, Memo, Signal};

let count = Signal::new(0);
let doubled = {
    let count = count.clone();
    Memo::new(move || count.get() * 2)
};

let effect = {
    let doubled = doubled.clone();
    Effect::new(move || {
        let _label = format!("doubled={}", doubled.get());
    })
};

count.set(2);
effect.dispose();
doubled.dispose();
```

Reads inside a `Memo` or `Effect` create subscriptions automatically. Setting a signal compares the old and new values with `PartialEq`; unchanged values do not flush subscribers.

## Update lifecycle

1. `Signal::set` marks dependent memos as `Check` and effects as `Dirty`.
2. Dirty memos recompute before their dependents are scheduled.
3. If a memo value is unchanged, dependent effects are skipped.
4. Effects flush immediately unless a batch is active.
5. `batch_begin` and `batch_end` coalesce multiple signal writes into one effect flush.

Nested observers are restored after each memo/effect run, so reading a memo inside an effect does not prevent the outer effect from subscribing to later signal reads.

## Disposal

Keep returned handles for long-lived `Effect` and `Memo` values. Call `dispose` when a component unmounts, a hydration island is replaced, or a test needs to tear down the graph.

DOM helpers return effect handles:

```rust
let text_effect = crepuscularity_reactive::bind_text("count", move || count.get().to_string());
text_effect.dispose();
```

## Hydration payloads

`crepuscularity-web` emits hydration context as base64 JSON inside a non-executable `application/json` script:

```html
<script id="__crepus_ctx__" type="application/json" data-encoding="base64">...</script>
```

`hydrate_root` decodes that payload and keeps raw JSON compatibility for older output.

## GPUI and Metal

GPUI-backed crates need the Xcode SDK and Metal compiler in the cargo environment. Use the helper when cargo cannot route `xcrun` to the installed Metal toolchain:

```bash
eval "$(scripts/metal-env.sh)"
cargo clippy -p crepuscularity-runtime --all-targets -- -D warnings
scripts/metal-env.sh -- cargo check -p crepuscularity-gpui
scripts/metal-env.sh --check
```

For a persistent shell setup:

```bash
export SDKROOT="$(xcrun --show-sdk-path)"
export DEVELOPER_DIR="$(xcode-select -p)"
export TOOLCHAINS="Metal"
```
