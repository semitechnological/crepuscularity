# Examples

Start with the smallest example that matches the output you want. The complete catalog is kept in [`examples.toml`](examples.toml); this page is the human-friendly guide.

## Start here

| Goal | Example | First command |
| --- | --- | --- |
| Render one template to HTML | [`demo.crepus`](demo.crepus) | `crepus render examples/demo.crepus` |
| Build a static web site | [`web-site/`](web-site/) | `cd examples/web-site && crepus web build` |
| Build a browser extension | [`quicknote/`](quicknote/) | `cd examples/quicknote && crepus webext build` |
| Run a desktop app | [`weather/`](weather/) | `cargo run --manifest-path examples/weather/Cargo.toml` |
| Generate native mobile views | [`native-shells/`](native-shells/) | `cd examples/native-shells/ios && swift build` |
| Render an embedded panel on the host | [`embedded-dashboard/`](embedded-dashboard/) | `cargo run --manifest-path examples/embedded-dashboard/Cargo.toml` |
| Compare other UI stacks | [`benchmarks/`](benchmarks/) | `crepus benchmark check --config examples/benchmarks/benchmark.toml` |

Install the CLI first if `crepus` is not already on `PATH`:

```bash
cargo install --path crates/crepuscularity-cli
```

## Examples by output target

### Render and web

- [`demo.crepus`](demo.crepus) — language and context showcase; pair it with [`demo-context.json`](demo-context.json).
- [`layout.crepus`](layout.crepus) — layout patterns.
- [`typography.crepus`](typography.crepus) — typography styles.
- [`components-demo.crepus`](components-demo.crepus) — shared component usage.
- [`jsx-demo.crepus`](jsx-demo.crepus) — JSX input syntax.
- [`render/`](render/) — context-driven render examples.
- [`ui-library/`](ui-library/) — reusable components and transfer-progress demo.
- [`web-site/`](web-site/) — minimal static site scaffold.
- [`weather-web/`](weather-web/) — web weather app.
- [`todo-web/`](todo-web/) — web todo app.
- [`counter/`](counter/) — web app with an SSR server.
- [`ssr-demo/`](ssr-demo/) — SSR router demo.
- [`moon-site/`](moon-site/) — Moonshine TSX output.

For a web build, run the command from the example directory. Build output goes to that example's `dist/` directory.

### GPUI desktop

- [`weather/`](weather/) — weather app.
- [`todo/`](todo/) — todo app.
- [`text-features/`](text-features/) — letter spacing and text-transform examples.

Run these with Cargo using their manifest path, for example:

```bash
cargo run --manifest-path examples/weather/Cargo.toml
```

### Browser extensions

- [`quicknote/`](quicknote/) — in-repository MV3 scratchpad.
- [`extensions/anywhere`](extensions/anywhere) — AI Anywhere, linked from its own repository.
- [`extensions/rs_vimium`](extensions/rs_vimium) — Rust Vimium, linked from its own repository.

The external examples are relative symlinks. Their sibling checkouts should look like this:

```text
projects/
  crepuscularity/
  anywhere/
  rs_vimium/
```

Recreate the links with [`scripts/examples-link.sh`](../scripts/examples-link.sh), then build from the linked example directory:

```bash
cd examples/extensions/anywhere && crepus webext build
```

### Native mobile

- [`native-shells/`](native-shells/) — shared View IR with SwiftUI and Jetpack Compose shells.
- [`mobile-app/`](mobile-app/) — code-generated task tracker with Rust action dispatch.

Read each example's README before building. iOS requires Xcode/Swift; Android requires Android Studio or a JDK 17 Gradle environment.

### Embedded and LVGL

- [`embedded-dashboard/`](embedded-dashboard/) — host-side RGB565 panel demo; no board required.
- [`embedded-stm32/`](embedded-stm32/) — STM32F411 + ILI9341 host-to-panel workflow.
- [`embedded-esp32/`](embedded-esp32/) — ESP32-S3 + ST7789 workflow.
- [`lvgl-pro-mode/`](lvgl-pro-mode/) — host LVGL XML generator.
- [`lvgl-stm32/`](lvgl-stm32/) — build-time STM32 LVGL XML asset.

The embedded examples are experimental. Start with [`embedded-dashboard/README.md`](embedded-dashboard/README.md), then use the board-specific README for hardware steps.

### Benchmarks and integrations

- [`benchmarks/`](benchmarks/) — benchmark configuration, internal Crepus targets, and checked-in comparison apps under `apps/` and `starters/`.
- [`tauri-v1-crepus/`](tauri-v1-crepus/) — Tauri v1 host example.
- [`tauri-v2-crepus/`](tauri-v2-crepus/) — Tauri v2 host example.
- [`tauri-v2-multiwindow/`](tauri-v2-multiwindow/) — Tauri v2 multi-window example.
- [`tauri-v2-webview/`](tauri-v2-webview/) — Tauri v2 webview example.
- [`local-scrobbler/`](local-scrobbler/) — local native integration example.

## What belongs in an example

Keep source templates, manifests, and small fixtures in the example directory. Most build products and dependency installs are local-only; a few examples intentionally check in generated fixtures under `dist/` or `generated/`.

- Rust output: `target/`
- Web and extension output: `dist/`
- JavaScript dependencies: `node_modules/`
- Native code generation: `generated/`
- Benchmark scratch trees: `benchmarks/.work/`

These directories are ignored or regenerated by the relevant example command. If an example has a README, use it as the source of truth for its platform-specific setup.
