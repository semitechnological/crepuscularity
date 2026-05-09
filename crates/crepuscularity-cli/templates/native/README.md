# Native shell scaffold

Scaffolded by `crepus native new`. This is a starter for iOS (SwiftPM) and
Android (Gradle) apps that decode **View IR** produced by the Rust crate
[`crepuscularity-native`](https://crates.io/crates/crepuscularity-native).

## Layout

| Path | What it is |
|---|---|
| `fixture.json` | Shared View IR fixture used by both shells. Regenerate with `crepuscularity-native::render_template_to_ir` after editing your template. |
| `ios/` | Swift Package (open `Package.swift` in Xcode, or `swift build` from this directory). |
| `android/` | Gradle module (open in Android Studio, or run `./gradlew :app:assembleDebug` after generating the Gradle wrapper). |

## iOS

```bash
cd ios
swift build               # CLI build
# or open ios/Package.swift in Xcode and pick a simulator
```

`SwiftUI` views in `ios/Sources/NativeShell/ViewIrTreeView.swift` decode the
shared fixture; `ViewIrModels.swift` is the matching data model. Both are
small enough to read top-to-bottom.

## Android

The Gradle wrapper is **not** committed (it requires a binary `.jar`).
Generate it once before the first build:

```bash
cd android
gradle wrapper --gradle-version 8.10
./gradlew :app:assembleDebug
```

Or open `android/` in Android Studio — it regenerates the wrapper on import.

`MainActivity.kt` loads `assets/fixture.json` and renders it through
`ViewIrTree.kt`'s Compose tree.

## Regenerating the fixture from a template

From a Rust crate that depends on `crepuscularity-native`:

```rust
use crepuscularity_core::TemplateContext;
use crepuscularity_native::{render_template_to_ir, to_json_pretty};

let mut ctx = TemplateContext::new();
ctx.set("name", "Ada");
let ir = render_template_to_ir(
    "div flex flex-col gap-4\n  span\n    \"Hello {name}\"",
    &ctx,
)?;
std::fs::write("fixture.json", to_json_pretty(&ir)?)?;
```

Then mirror the file into both `ios/Sources/NativeShell/fixture.json` and
`android/app/src/main/assets/fixture.json` so each shell sees the same IR.

## Build via the `crepus` CLI

```bash
crepus native build ios --dir .
crepus native build android --dir . --flavor Debug
crepus native run android --dir .          # gradle install + adb launch hint
```
