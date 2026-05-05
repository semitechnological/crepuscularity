# Production Readiness

**Also:** [Documentation home](README.md) · [CLI](cli.md) · [DSL](dsl.md) · [Components](components.md)

Use this checklist before shipping an application, docs site, or extension built with Crepuscularity.

## Required gates

Run the workspace gates from the repository root:

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

On macOS, set `SDKROOT` when building GPUI-backed crates:

```bash
SDKROOT=$(xcrun --show-sdk-path) cargo build
```

If the build reaches GPUI's Metal shader step, use the repository helper so Cargo inherits the same Xcode toolchain environment as the shell:

```bash
eval "$(scripts/metal-env.sh)"
cargo build
scripts/metal-env.sh -- cargo check -p crepuscularity-gpui
scripts/metal-env.sh --check
```

The exact variables are `SDKROOT`, `DEVELOPER_DIR`, and `TOOLCHAINS=Metal`. `TOOLCHAINS` is the environment variable `xcrun` reads when selecting a toolchain for GPUI's `xcrun -sdk macosx metal` invocation; current Xcode installs route the downloaded Metal component through the short selector `Metal`. The helper also prepends the downloaded `Metal.xctoolchain/usr/bin` to `PATH` for direct `metal` and `metallib` probes. If the helper reports `xcrun_metal=failed`, install or re-register the Metal component with `xcodebuild -downloadComponent MetalToolchain` or Xcode Settings > Components before treating the Cargo failure as a code or docs-site regression.

For static web and docs-site output, also verify the WASM target and generated files:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
cargo run -p crepuscularity-cli -- web build --site docs-site --out-dir /tmp/crepus-docs-dist
```

The generated docs should include `docs/index.html`, one HTML page for each published Markdown guide, `docs/docs-search-index.json`, `pkg/*.wasm`, and `.nojekyll` at the site root.

## Security boundaries

Crepuscularity templates are source code, not an untrusted markup sandbox. Do not render attacker-controlled `.crepus` source unless you wrap it in a separate policy layer.

Runtime template values are escaped before they become HTML text nodes or attribute values. Use raw HTML only from trusted sources, and keep `site.json` `head_html` limited to assets you intentionally ship.

For browser extensions, keep the generated Manifest V3 CSP strict. The extension runtime needs `wasm-unsafe-eval` for WebAssembly; do not add remote script origins or broad inline-script allowances.

Keep extension page access narrow. Empty `host-permissions` stays empty in generated manifests, and default content scripts are only generated for declared host patterns. Use explicit `[[content_scripts]]` entries when a content script needs a different match set than the top-level host permissions.

For `include` directives, keep component paths relative to the project’s template root. The runtime and TUI paths reject parent-directory and absolute includes; new backends should preserve that boundary.

Do not put secrets in templates, `site.json`, `web.toml`, generated bundles, or extension assets. Everything copied to `dist/` is public.

## Performance checks

Prefer release builds for any meaningful timing:

```bash
cargo build --workspace --release
cargo test --workspace --release
```

Use `crepus web build-full` when you need to measure the parallel render/build pipeline rather than only the final WASM bundle. For renderer-level work, profile the parser, evaluator, and renderer separately before changing data structures.

High-value checks:

- Keep templates small enough that the generated `crepus-bundle.json` is cacheable and easy to diff.
- Use includes to share stable component trees instead of duplicating large subtrees.
- Keep client islands under `static/` targeted at leaf nodes that Rust/WASM is not also mutating.
- Enable optional features only where needed; minimal web builds should avoid DOM helpers unless the runtime calls them.

## Release hygiene

Use semver `0.y.z` while the crates remain pre-1.0. Bump patch versions for behavior, API, or published dependency graph changes; use minor versions for larger capability batches.

Before publishing or cutting release artifacts, verify that publishable crates do not depend on unpublished path-only workspace crates. Use `cargo package -p <crate> --allow-dirty --no-verify` as an early structural check, then run the full package step without `--no-verify` once the dependency graph is ready.
