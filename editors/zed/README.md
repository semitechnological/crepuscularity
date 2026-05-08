# Crepuscularity — Zed extension (dev)

## Grammar compile errors

Zed clones `tree-sitter-crepus` into **`grammars/crepus/`** (nested git repo) and writes **`grammars/crepus.wasm`**. If install fails with **failed to compile grammar 'crepus'**:

1. Remove stale checkouts and wasm next to this folder:

   ```sh
   rm -rf grammars crepus.wasm
   ```

2. Ensure `[grammars.crepus].repository` in `extension.toml` **exactly matches** `git remote get-url origin` for that URL class (Zed reuses the directory only when the origin URL string matches). If you use `url.insteadOf` in git config, align the manifest URL or adjust git config ([zed#48163](https://github.com/zed-industries/zed/issues/48163)).

3. Retry **Install Dev Extension** so Zed runs a fresh `git fetch` / WASI `clang` build.

The committed grammar on `main` must include `tree-sitter-crepus/src/parser.c` (path `tree-sitter-crepus` in the repo root).

## Language server

`crepus-lsp` must be on `PATH` or built at `<worktree>/target/debug/crepus-lsp` (see `src/lib.rs`).
