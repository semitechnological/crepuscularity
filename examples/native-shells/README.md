# Native shells (SwiftUI + Jetpack Compose)

This directory is the **in-repo** counterpart to the old **`crepuscularity-native-ui`** worktree: Swift and Kotlin sample shells plus a shared IR fixture, alongside the Rust crate **`crepuscularity-native`** ([`../../crates/crepuscularity-native`](../../crates/crepuscularity-native)) that emits **View IR** (`render_template_to_ir` / `to_json_pretty`).

**Rust on Android:** the [`android-activity`](https://docs.rs/android-activity) crate *does* let you ship **native Rust** as the process entrypoint (GameActivity / NativeActivity + `winit`/`wgpu`, etc.). It is not a UI framework—it replaces the Kotlin `Activity` shell while you draw or bridge UI yourself. A future path is: Rust (`crepuscularity-native` + parser) → IR → small Kotlin Compose **or** IR → GPU/text in Rust. The [AOSP Rust modules](https://source.android.com/docs/setup/build/rust/building-rust-modules/overview) doc is for **platform / Soong** builds, not typical Play Store Gradle apps—keep that separate from any sample here.

**Coverage:** IR and shells track a **growing** subset of Tailwind (`style.rs`) and widgets (`button`, `img`, `scroll`, `slotRotate`, …). This is **not** 100% parity with `crepuscularity-gpui` `styler.rs`—extend [`crates/crepuscularity-native/src/style.rs`](../../crates/crepuscularity-native/src/style.rs) for missing classes.

## Shared fixture

[`fixture.json`](fixture.json) matches the golden output for:

```text
div flex flex-col gap-4
  span
    "Hello {name}"
```

with `name = Ada`. Keep these copies in sync when you change the sample:

- [`fixture.json`](fixture.json) (repo root of this example)
- [`ios/Sources/NativeShell/fixture.json`](ios/Sources/NativeShell/fixture.json) (SwiftPM resource)
- [`android/app/src/main/assets/fixture.json`](android/app/src/main/assets/fixture.json)

## iOS: Swift Package (this repo)

Use the checked-in **`ios/`** tree as a local package (no XcodeGen required):

1. Open [`ios/Package.swift`](ios/Package.swift) in Xcode or add **`ios/`** as a local Swift package dependency.
2. Import `NativeShell` and use `FixtureRootView()`, `ViewIrRootView(ir:)`, or `ViewNodeView(node:)` from a hosting controller.
3. For a quick look, open `ViewIrTreeView.swift` and run an Xcode Preview on `FixtureRootView`.

Command-line build (macOS):

```bash
cd ios && swift build
```

SwiftPM writes **`ios/.build/`** locally; it is listed in [`.gitignore`](.gitignore) and must not be committed.

## iOS: CLI scaffold (`crepus ios`)

Alternatively, scaffold a fresh app (XcodeGen + app target) from anywhere you have the CLI installed:

```bash
brew install xcodegen
crepus ios new my-native-demo
cd my-native-demo
crepus ios generate
open *.xcodeproj
```

That lays out **`crepus.toml`** (`[ios]` holds `scheme`, `xcodegen_spec`, `destination`), **XcodeGen** `project.yml`, a local **SwiftPM** `NativeShell` package, and a minimal **App** target. `crepus ios generate` and `crepus ios build` **walk up from the current directory** (or any subfolder) until they find `crepus.toml`, so you do not need `--dir` / `--scheme` in normal use.

Optional overrides: `crepus ios build --dir . --scheme Foo --destination 'platform=iOS Simulator,name=iPhone 15'`.

CLI build (simulator), from inside the app tree:

```bash
crepus ios build
```

After changing templates, refresh **`NativeShell/Sources/NativeShell/fixture.json`** in the scaffolded app using **`crepuscularity-native`** (see **Rust ↔ JSON** below).

## Android

Open [`android`](android) in Android Studio (JDK 17+). Sync Gradle and run the **`app`** configuration.

From a terminal (with `JAVA_HOME` set):

```bash
cd android && ./gradlew :app:assembleDebug
```

## Rust ↔ JSON

Generate IR from a template (from any crate that depends on **`crepuscularity-native`**):

```rust
use crepuscularity_core::TemplateContext;
use crepuscularity_native::{render_template_to_ir, to_json_pretty};

let mut ctx = TemplateContext::new();
ctx.set("name", "Ada");
let ir = render_template_to_ir(
    "div flex flex-col gap-4\n  span\n    \"Hello {name}\"",
    &ctx,
)?;
println!("{}", to_json_pretty(&ir)?);
```

**JSON Schema** for quicktype / other generators:

```bash
cargo run -p crepuscularity-native --features schema --bin export-view-ir-schema
```

CI does not build these shells yet; verify them locally when you touch the IR schema.
