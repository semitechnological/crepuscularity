# Crepuscularity — Zed extension (dev)

## No syntax highlighting

Tree-sitter **query files must only reference tokens that exist in the grammar**. For example, `brackets.scm` patterns like `\"{\"` only work if `{` is a symbol in `grammar.js`; otherwise query compilation fails and highlighting does not run. After editing queries, validate with:

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

`crepus-lsp` must be on `PATH` or built at `<worktree>/target/debug/crepus-lsp` (see `src/lib.rs`).
