# SSR Timing Profile

> Auto-generated from the observability infrastructure. Run a reference build and
> record where milliseconds go. Update this doc after major pipeline changes.

**Reference app:** `docs-site` (this project's own documentation site)
**Machine:** record your OS + CPU here
**Profile:** `--release`

## One-shot SSR request breakdown

Run the reference build with tracing enabled:

```bash
RUST_LOG=crepuscularity=info cargo run -p crepuscularity-cli --release -- \
  web build --manifest docs-site/crepus.toml --out-dir /tmp/crepus-docs-dist
```

### Timing table (fill after run)

| Span | Typical ms | Notes |
|---|---|---|
| `parse_template` | _ | Single .crepus file parse |
| `eval_expr` | _ | Sampling at TRACE level |
| `render_html` | _ | HTML string generation |
| `dev_request` | _ | Full dev-server request |
| `hot_reload` | _ | File change → SSE event |

### Flamegraph

```bash
cargo flamegraph --bin crepus -- web build --manifest docs-site/crepus.toml --out-dir /tmp/crepus-docs-dist
```

Or with `samply` (macOS):

```bash
samply record target/release/crepus web build --manifest docs-site/crepus.toml --out-dir /tmp/crepus-docs-dist
```

### Cargo build timings

```bash
cargo build --workspace --release --timings
# Open target/cargo-timings/cargo-timing.html
```

## Where time typically goes

1. **Cargo + rustc** — compiling Rust crates (dominates first build)
2. **WASM bindgen** — `wasm-bindgen` on the compiled `.wasm`
3. **Template parsing** — `parse_template` spans, visible at INFO
4. **HTML rendering** — `render_html` spans, visible at DEBUG
5. **File I/O** — writing output files (usually negligible)

## Acceptance criteria

- [ ] One SSR request profiled with flamegraph
- [ ] Timing table filled for reference app
- [ ] `cargo build --timings` HTML report saved
- [ ] Answer documented: "where did the ms go?"
