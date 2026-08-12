# Examples

The examples are split into a small showcase and an `advanced/` shelf. Start with the showcase; the advanced shelf keeps hardware, integrations, benchmarks, and alternate app variants available without crowding the first view.

## Showcase

| Goal | Example | First command |
| --- | --- | --- |
| See a polished data-rich screen | [`showcase/product-dashboard.crepus`](showcase/product-dashboard.crepus) | `crepus render examples/showcase/product-dashboard.crepus --ctx examples/showcase/product-dashboard.json` |
| Learn the full indentation DSL | [`showcase/full-range.crepus`](showcase/full-range.crepus) | `crepus render examples/showcase/full-range.crepus --ctx examples/showcase/demo-context.json` |
| See components, props, and slots | [`showcase/ui-demo.crepus`](showcase/ui-demo.crepus) | `crepus render examples/showcase/ui-demo.crepus` |
| See the JSX frontend | [`showcase/jsx-demo.crepus`](showcase/jsx-demo.crepus) | `crepus render examples/showcase/jsx-demo.crepus --ctx examples/showcase/jsx-demo.toml` |
| Build a static website | [`web-site/`](web-site/) | `cd examples/web-site && crepus web build` |
| Build a browser extension | [`quicknote/`](quicknote/) | `cd examples/quicknote && crepus webext build` |
| Run a desktop app | [`weather/`](weather/) | `cargo run --manifest-path examples/weather/Cargo.toml` |
| Generate native mobile views | [`native-shells/`](native-shells/) | `cd examples/native-shells/ios && swift build` |
| Render an embedded panel on the host | [`embedded-dashboard/`](embedded-dashboard/) | `cargo run --manifest-path examples/embedded-dashboard/Cargo.toml` |
| Use the reusable component catalog | [`ui-library/`](ui-library/) | `crepus render examples/ui-library/examples/dashboard.crepus` |

Install the CLI first if `crepus` is not already on `PATH`:

```bash
cargo install --path crates/crepuscularity-cli
```

The [`showcase/`](showcase/) directory also contains focused layout, typography, controls, and component examples. They are intentionally small enough to read in one sitting and broad enough to demonstrate context, interpolation, computed values, conditionals, match arms, loops, events, includes, slots, JSX input, and common controls.

## Advanced

The advanced shelf is organized by project rather than by beginner learning path:

- [`advanced/benchmarks/`](advanced/benchmarks/) — build-time comparisons with other UI stacks.
- [`advanced/counter/`](advanced/counter/) and [`advanced/ssr-demo/`](advanced/ssr-demo/) — server rendering and hydration.
- [`advanced/embedded-stm32/`](advanced/embedded-stm32/), [`advanced/embedded-esp32/`](advanced/embedded-esp32/), [`advanced/lvgl-pro-mode/`](advanced/lvgl-pro-mode/), and [`advanced/lvgl-stm32/`](advanced/lvgl-stm32/) — board and LVGL workflows.
- [`advanced/mobile-app/`](advanced/mobile-app/) and [`advanced/moon-site/`](advanced/moon-site/) — larger native and Moonshine applications.
- [`advanced/tauri-v1-crepus/`](advanced/tauri-v1-crepus/), [`advanced/tauri-v2-crepus/`](advanced/tauri-v2-crepus/), [`advanced/tauri-v2-multiwindow/`](advanced/tauri-v2-multiwindow/), and [`advanced/tauri-v2-webview/`](advanced/tauri-v2-webview/) — Tauri host integrations.
- [`advanced/extensions/`](advanced/extensions/) — external browser extensions linked from sibling checkouts.
- [`advanced/render/`](advanced/render/), [`advanced/text-features/`](advanced/text-features/), [`advanced/todo/`](advanced/todo/), [`advanced/todo-web/`](advanced/todo-web/), [`advanced/weather-web/`](advanced/weather-web/), and [`advanced/local-scrobbler/`](advanced/local-scrobbler/) — focused backend and integration references.

External extension links are relative symlinks. Clone their repositories next to this checkout:

```text
projects/
  crepuscularity/
  anywhere/
  rs_vimium/
```

Recreate the links with [`scripts/examples-link.sh`](../scripts/examples-link.sh), then build from the linked directory:

```bash
cd examples/advanced/extensions/anywhere && crepus webext build
```

Build products and dependency installs are local-only. Rust output is written to `target/`, web output to `dist/`, JavaScript dependencies to `node_modules/`, native code generation to `generated/`, and benchmark scratch data to `advanced/benchmarks/.work/`.
