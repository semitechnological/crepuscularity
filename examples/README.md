# Examples

Examples are grouped by **output target** in [`examples.toml`](examples.toml).

## Browser extensions (`webext`)

| Example | Location | Repository |
| --- | --- | --- |
| Quicknote | [`quicknote/`](quicknote/) | in-repo |
| AI Anywhere | [`extensions/anywhere`](extensions/anywhere) | [undivisible/anywhere](https://github.com/undivisible/anywhere) |
| Vimium (Rust) | [`extensions/rs_vimium`](extensions/rs_vimium) | [undivisible/rs_vimium](https://github.com/undivisible/rs_vimium) |

External repos are **relative git symlinks** under `extensions/`. Clone them as siblings of this repo:

```text
projects/
  crepuscularity/
  anywhere/
  rs_vimium/
```

Recreate symlinks after clone:

```bash
./scripts/examples-link.sh
```

Build an extension from its directory:

```bash
cd examples/extensions/anywhere && crepus webext build
cd examples/extensions/rs_vimium && crepus webext build
```

## Other targets

| Target | Examples (see `examples.toml`) |
| --- | --- |
| `web` | `web-site`, `weather-web`, `todo-web`, `counter`, `ssr-demo` |
| `gpui` | `weather`, `todo`, `text-features` |
| `native` | `native-shells` |
| `embedded` | `embedded-dashboard`, `embedded-stm32`, `embedded-esp32` |
| `lvgl` | `lvgl-pro-mode`, `lvgl-stm32` |
| `render` | `render/`, `ui-library/`, `*.crepus` demos at this level |
| `benchmark` | [`benchmarks/`](benchmarks/) — `crepus benchmark` |
