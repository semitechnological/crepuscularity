# TODO

## Housekeeping

### Licensing & metadata
- [ ] Add `LICENSE` file (MPL-2.0)
- [ ] Add `license` field to all `Cargo.toml` files
- [ ] Add `description` and `repository` fields to all `Cargo.toml` files
- [ ] Add `authors` field to workspace root `Cargo.toml`
- [ ] Add SPDX headers to source files

### CLI rename (`crepu` → `crepus`)
- [x] Rename binary in `crepuscularity-cli/Cargo.toml`
- [x] Rename `crepu-dev` → `crepus-dev` in `crepuscularity-dev/Cargo.toml`
- [x] Update all docs and source references
- [ ] Update `CLAUDE.md` binary name reference
- [ ] Update CI install steps and scripts that reference `crepu`
- [ ] Add a deprecation shim / shell alias note in migration guide

### Documentation gaps
- [ ] Write `docs/mobile.md` — mobile backend overview and roadmap
- [ ] Write `docs/architecture.md` — two rendering paths, context system, IR plan
- [ ] Add a contributing guide (`CONTRIBUTING.md`)
- [ ] Add a changelog (`CHANGELOG.md`)
- [ ] `docs/cli.md`: document `crepus-dev` as a separate binary from `crepus dev`
- [ ] Add `docs/webext.md` section on cross-browser support (Firefox, Edge)

### Testing
- [ ] Add integration tests for the runtime renderer against `.crepus` fixtures
- [ ] Add snapshot tests for GPUI rendering output (if GPUI supports headless)
- [ ] Add property-based tests for the evaluator (`eval.rs`)
- [ ] Add browser extension manifest generation tests
- [ ] Add CLI smoke tests (scaffold, build, manifest print)

### CI / tooling
- [ ] Pin Rust toolchain version in `rust-toolchain.toml`
- [ ] Add `cargo deny` or `cargo audit` step for supply chain checks
- [ ] Add a lint step (`clippy --deny warnings`)
- [ ] Add a formatting check (`rustfmt --check`)
- [ ] Publish crates to crates.io (decide crate publication order)
- [ ] Add `cargo-nextest` to CI for faster test runs
- [ ] Set up `cargo-release` for versioning

### Code quality
- [ ] Audit all `unwrap()` / `expect()` calls in non-test code
- [ ] Audit all `todo!()` / `unimplemented!()` macros
- [ ] Replace ad-hoc error strings with structured error types (`thiserror`)
- [ ] Remove any dead code and unused `#[allow(dead_code)]` annotations

### Examples
- [ ] Ensure `examples/weather` builds against the latest crate versions
- [ ] Ensure `examples/quicknote` builds against the latest crate versions
- [ ] Add a minimal GPUI hot-reload example to `examples/`
- [ ] Add an example that demonstrates multi-component files

---

## Render IR

Goal: let `.crepus` stay the shared syntax while each backend becomes a standalone IR consumer, not a fork of the language.

- [ ] Define `RenderNode` IR enum in `crepuscularity-core`
- [ ] Port HTML backend to consume the IR
- [ ] Port React backend to consume the IR
- [ ] Port GPUI backend to consume the IR
- [ ] Make backend selection explicit in the CLI (`--backend gpui|html|react|mobile`)
- [ ] Document the IR design in `docs/architecture.md`

---

## Mobile — Architecture: React Native–Style Bridge

The right model for Crepuscularity mobile is structurally identical to React Native's New Architecture (JSI + Fabric) — just with Rust doing what JS does in React Native.

```
.crepus templates
    ↓  parse + eval (Rust — crepuscularity-core)
Render IR  (platform-neutral node tree)
    ↓  bridge layer (C FFI / JNI)
Platform widget tree
    iOS: SwiftUI @ViewBuilder closures
    Android: @Composable functions
```

How this maps to React Native concepts:

| React Native | Crepuscularity Mobile |
|---|---|
| JSX components | `.crepus` templates |
| JavaScript runtime | Rust parser + evaluator |
| JSI (synchronous bridge) | C FFI (iOS) / JNI (Android) |
| Fabric Renderer | IR → SwiftUI / Compose mapper |
| Host Components (View, Text…) | IR node types (View, Text, Stack…) |
| Native Modules (TurboModules) | Platform capabilities in `mobile-core` |
| Metro bundler | `crepus mobile build` |

Key differences from React Native:
- **No JS runtime**: the parser and evaluator are Rust, compiled directly to a `.dylib` (iOS) or `.so` (Android). Zero JS overhead.
- **No async bridge**: the IR is built synchronously in Rust memory and passed over a thin FFI boundary once per render, not serialized through a message queue.
- **No two codebases**: the same `.crepus` template drives GPUI on desktop, HTML in browser extensions, and SwiftUI/Compose on mobile.

The `uniffi` crate (Mozilla) generates type-safe Swift and Kotlin bindings from a Rust interface definition, eliminating hand-written FFI for the happy path.

---

## Mobile — Native Widget Backends

Goal: map `.crepus` + the render IR to platform-native widget trees, not a custom renderer.

### `crepuscularity-mobile-core`
- [ ] Define shared mobile view primitives: `View`, `Text`, `Image`, `Input`, `Scroll`, `Stack`, `Overlay`, `NavigationStack`
- [ ] Define a layout model independent of iOS/Android details
- [ ] Define an event and state bridge trait
- [ ] Wire into the backend-neutral render IR

### `crepuscularity-ios`
- [ ] Set up a `cdylib` Rust crate with `swift-bridge` or `uniffi` code-gen
- [ ] Generate a Swift package that exposes the Crepuscularity parser and evaluator
- [ ] Map IR `View`, `Text`, `Stack`, `If`, `For` nodes to SwiftUI `@ViewBuilder` closures
- [ ] Map IR styling tokens (Tailwind-style classes) to SwiftUI modifiers
- [ ] Implement slot support via SwiftUI `@ViewBuilder` parameter patterns
- [ ] Implement state bridge using `ObservableObject` / `@Published`
- [ ] Add a sample iOS Xcode project in `examples/ios-counter/`

### `crepuscularity-android`
- [ ] Set up a `cdylib` Rust crate with JNI exports (`jni` crate)
- [ ] Generate Kotlin bindings via `uniffi` or manual JNI wrappers
- [ ] Map IR nodes to Jetpack Compose `@Composable` functions
- [ ] Map IR styling tokens to Compose `Modifier` chains
- [ ] Implement state bridge using `ViewModel` / `StateFlow`
- [ ] Add a sample Android Gradle project in `examples/android-counter/`

### CLI mobile commands
- [ ] `crepus mobile new <name> --platform ios|android|both`
- [ ] `crepus mobile build --platform ios|android`
- [ ] `crepus mobile preview` (live reload over a local socket, similar to `crepus dev`)

---

## Mobile — Custom Renderer (future / optional)

Not the first priority. Only pursue after the render IR and native widget backends stabilise.

Architecture if we go this route:
- **Graphics**: `wgpu` (Metal on iOS, Vulkan/GL on Android) — already the foundation of GPUI desktop
- **2D vector rendering**: `vello` (GPU compute-based, excellent for UI shapes and text paths)
- **Text shaping/layout**: `cosmic-text` + `swash` (cross-platform, no system font dependency required)
- **Layout engine**: `taffy` (flexbox, already used by several Rust UI frameworks)
- **Platform integration**:
  - iOS: `UIWindow` + `CAMetalLayer`, safe area APIs, `CoreText` as fallback
  - Android: `ANativeWindow`, Vulkan surface, system UI insets
- **Gesture/input**: thin platform layer converting touch events to a shared abstraction

Decision gate: do not start this until (a) the render IR is stable, (b) at least one native widget backend ships, and (c) there is a clear use case that native widgets cannot cover.

---

## Browser Extension Polish

- [ ] Firefox MV3 manifest differences (action vs. browser_action, service workers vs. background pages)
- [ ] Cross-browser capability matrix documentation
- [ ] Hot-reload support for extension popups via `crepus dev` (WebSocket injection)
- [ ] `crepus webext watch` command for iterative development
- [ ] Auto-reload extension in Chrome on build (via `chrome.runtime.reload` injection)
- [ ] Publish `crepuscularity-webext` as a standalone crate on crates.io

---

## Open Questions

- How much of the DSL should map 1:1 across web, GPUI, and mobile? (styling tokens vs. layout semantics)
- Shared styling model or per-backend styling adapters?
- Native navigation lifecycles in the core IR or in backend adapters?
- Should slot content cross the Swift/Kotlin FFI boundary, or be fully resolved on the Rust side before FFI?
- Versioning strategy: semver per crate, or a unified workspace version?
