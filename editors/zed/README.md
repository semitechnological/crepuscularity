# Crepuscularity — Zed extension (dev)

## No syntax highlighting

### Pin mismatch (`extension.toml` `rev`)

Zed builds the WASM grammar from **`extension.toml` → `[grammars.crepus]`** (`repository`, `path`, **`rev`**), not from your unchecked-out working tree. The extension’s **`languages/crepus/*.scm` queries ship with the extension** and must match that **`rev`**.

If `rev` still points at an **old** grammar (for example one that only had `element_class`) but `highlights.scm` expects **`class_segment`, `hash_id`, `attr_binding_braced`, …**, query compilation fails and **highlighting is off entirely**.

**Fix:** Bump `rev` to a **full Git SHA** on `repository` that includes the matching `tree-sitter-crepus/grammar.js`, **push** that commit (Zed fetches from GitHub), then **Install dev extension** again. To clear a bad WASM build:

```sh
cd editors/zed && rm -rf grammars crepus.wasm
```

### Query vs grammar

Tree-sitter **query files must only reference nodes that exist in the grammar**. For example, `brackets.scm` patterns like `\"{\"` only work if `{` is a symbol in `grammar.js`; otherwise query compilation fails and highlighting does not run. After editing queries, validate with:

`tree-sitter query path/to/query.scm sample.crepus`

(from the `tree-sitter-crepus/` directory).

## Grammar compile errors

Zed clones `tree-sitter-crepus` into **`grammars/crepus/`** (nested git repo) and writes **`grammars/crepus.wasm`**. If install fails with **failed to compile grammar 'crepus'**:

1. Remove stale checkouts and wasm next to this folder:

   ```sh
   rm -rf grammars crepus.wasm
   ```

2. Ensure `[grammars.crepus].repository` in `extension.toml` **exactly matches** `git remote get-url origin` for that URL class (Zed reuses the directory only when the origin URL string matches). If you use `url.insteadOf` in git config, align the manifest URL or adjust git config ([zed#48163](https://github.com/zed-industries/zed/issues/48163)).

3. Retry **Install Dev Extension** so Zed runs a fresh `git fetch` / WASI `clang` build.

The `[grammars.crepus].rev` field in `extension.toml` pins a **full Git SHA** so Zed’s shallow fetch/checkout is deterministic. After changing the grammar, bump that SHA to a commit that contains the updated `tree-sitter-crepus/` tree.

## Language server

Zed runs **`crepus-lsp --stdio`**. Resolution (see `src/lib.rs`):

1. **`crepus-lsp` on your `PATH`** (e.g. `cargo install --path crates/crepuscularity-lsp`), or  
2. **`{worktree}/target/debug/crepus-lsp`** (worktree = Zed project folder) — build with  
   `SDKROOT="$(xcrun --show-sdk-path)" cargo build -p crepuscularity-lsp`  
   from the **Cargo workspace root** (the folder you added as the Zed project).

Open the **repo / workspace root** in Zed (`crepuscularity/`), not only a nested crate, so `target/debug/` matches the path the extension uses.
