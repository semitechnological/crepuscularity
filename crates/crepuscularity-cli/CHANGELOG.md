# Changelog — `crepuscularity-cli`

## 0.11.0

- **`crepus components`:** `list` / `add <id> [--target flutter|svelte|moonshine|gpui]` / `themes` for the `plugins/crepuscularity-components` catalog (graceful when missing).
- **`crepus moonshine`:** `new <name>` scaffolds a minimal Moonshine + Crepus app; `dep` prints `package.json` snippets for `@tschk/moonshine`, `@tschk/crepus-moonshine`, and `@tschk/moonshine-components`.
- **`crepus web build --emit`:** accepts `html` (default), `moonshine`, `svelte`, `solid`, `react`. Non-html emits write View IR JSON plus a framework stub under `dist/`; the HTML/WASM path is unchanged.
- Workspace note: Moonshine runtime packages live under `packages/` (owned separately); this CLI only scaffolds and emits stubs.

## 0.9.15

- CI: Zig **0.16.0** in plugin-smoke; restore `zig build test` for plugins.
- No CLI behavior change vs 0.9.14 (release tag includes CI/audit fixes on main).

## 0.9.14

- **Mobile CLI:** `build`, `run`, `sync`, and `codegen` use typed `clap` flags (`--dir`, `--target`, `--configuration`, `--flavor`) instead of trailing `rest` argv slices.
- **Cleanup:** Removed native/mobile string-slice parsers (`parse_dir_arg`, `dir_from_cli_args`, etc.) and dead usage printers left after the clap migration.
- **TUI:** Gate `console::style` behind the `tui` feature to silence default-build warnings.

## 0.9.13

- **Cleanup:** Dropped `execute_from_argv`, manual `parse_*` loops, and `print_*_usage` helpers across CLI modules (~1.2k LOC).
- Mobile `build` / `run` / `sync` / `codegen` call `native::execute` with typed command enums.

## 0.9.12

- **CLI:** Top-level argument parsing migrated to **clap** (`cli.rs` + `dispatch.rs`).
- Preserved behavior for `benchmark` flat flags, `web dev` serve alias, and existing integration tests (help output on stdout/stderr).