# Observability

**Also:** [Documentation home](README.md) · [Production readiness](production.md) · [Runtime and reactivity](runtime.md)

Use this when a build, render, or dev-server request feels slow. Measure first, then change code.

## Cargo timings

Capture a build timing report from the repository root:

```bash
scripts/metal-env.sh -- cargo build --workspace --timings
```

Cargo writes an HTML report under `target/cargo-timings/`. Use it to separate crate graph cost from template parsing, HTML rendering, and WASM packaging.

For a web/docs reference build:

```bash
rustup target add wasm32-unknown-unknown
scripts/metal-env.sh -- cargo run -p crepuscularity-cli -- web build --manifest docs-site/crepus.toml --out-dir /tmp/crepus-docs-dist
```

## Tracing spans

Useful spans already emitted by the core/web/dev paths:

| Span | Area |
| --- | --- |
| `parse_template` | DSL parser |
| `eval_expr` | expression evaluator |
| `render_html` | HTML renderer |
| `dev_request` | web dev server request handling |

Run with tracing visible:

```bash
RUST_LOG=crepuscularity=trace scripts/metal-env.sh -- cargo run -p crepuscularity-cli -- web dev --manifest docs-site/crepus.toml
```

## Flamegraph

Profile release-mode work only:

```bash
scripts/metal-env.sh -- cargo build -p crepuscularity-cli --release
cargo flamegraph --bin crepus -- web build --manifest docs-site/crepus.toml --out-dir /tmp/crepus-docs-dist
```

On macOS, `samply` is often easier for interactive runs:

```bash
scripts/metal-env.sh -- cargo build -p crepuscularity-cli --release
samply record target/release/crepus web build --manifest docs-site/crepus.toml --out-dir /tmp/crepus-docs-dist
```

## Report shape

Record:

- command
- machine and OS
- build profile
- cold or warm target directory
- top timing crate or flamegraph stack
- whether time is parser, evaluator, render, Cargo, WASM bindgen, or file I/O
