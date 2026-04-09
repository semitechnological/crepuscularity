# Native shells (SwiftUI + Jetpack Compose)

Minimal apps that decode the same **View IR** JSON produced by the Rust crate `crepuscularity-native` (`render_template_to_ir` / `to_json`). Use this to prototype how `.crepus` templates map to real platform widgets before wiring a full embedding or FFI pipeline.

**Rust on Android:** the [`android-activity`](https://docs.rs/android-activity) crate *does* let you ship **native Rust** as the process entrypoint (GameActivity / NativeActivity + `winit`/`wgpu`, etc.). It is not a UI framework—it replaces the Kotlin `Activity` shell while you draw or bridge UI yourself. A future path is: Rust (`crepuscularity-native` + parser) → IR → small Kotlin Compose **or** IR → GPU/text in Rust. The [AOSP Rust modules](https://source.android.com/docs/setup/build/rust/building-rust-modules/overview) doc is for **platform / Soong** builds, not typical Play Store Gradle apps—keep that separate from the sample here.

**Coverage:** IR and shells track a **growing** subset of Tailwind (`style.rs`) and widgets (`button`, `img`, `scroll`, `slotRotate`, …). This is **not** 100% parity with `crepuscularity-gpui` `styler.rs`—extend `crates/crepuscularity-native/src/style.rs` for missing classes.

## Shared fixture

[`fixture.json`](fixture.json) matches the golden output for:

```text
div flex flex-col gap-4
  span
    "Hello {name}"
```

with `name = Ada`. The same file is copied into the iOS package resources and Android `assets/`; keep them in sync when you change the sample.

## iOS (Swift Package)

1. Open [`ios/Package.swift`](ios/Package.swift) in Xcode (or add the `ios` folder as a local Swift package dependency to your app target).
2. Import `NativeShell` and use `FixtureRootView()`, `ViewIrRootView(ir:)`, or `ViewNodeView(node:)` from a hosting controller.
3. For a quick look, select `ViewIrTreeView.swift` and run an Xcode Preview on `FixtureRootView`.

Command-line build (macOS):

```bash
cd ios && swift build
```

## Android

1. Open the [`android`](android) directory in Android Studio (JDK 17+).
2. Sync Gradle and run the `app` configuration on an emulator or device.

From a terminal (with `JAVA_HOME` set):

```bash
cd android && ./gradlew :app:assembleDebug
```

## Rust ↔ JSON

Generate IR from a template in tests or your own binary:

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

CI does not build these shells yet; verify them locally when you touch the IR schema.
