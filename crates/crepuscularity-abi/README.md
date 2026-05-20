# crepuscularity-abi

Optional **C ABI** (`cdylib`) for in-process View IR compilation and event dispatch. Polyglot plugins should prefer **`crepus native ir`** (subprocess JSON); use this crate when a host language needs sessions without spawning the CLI.

- Header: [`include/crepuscularity_abi.h`](include/crepuscularity_abi.h)
- Does **not** depend on [Equilibrium](https://github.com/tschk/equilibrium)
- Replaces any experimental `crepuscularity-ffi` approach: plain `extern "C"`, no proc-macro FFI framework in-tree

```bash
cargo build -p crepuscularity-abi
cargo test -p crepuscularity-abi
```

See [docs/view-ir-contract.md](../../docs/view-ir-contract.md) and [docs/polyglot.md](../../docs/polyglot.md).
