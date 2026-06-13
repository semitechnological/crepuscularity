# Native shell scaffold

Scaffolded by `crepus mobile new` or `crepus native new`. This is a starter for iOS (XcodeGen + SwiftUI) and
Android (Gradle + Compose) apps that decode **View IR** produced by the Rust crate
[`crepuscularity-native`](https://crates.io/crates/crepuscularity-native).

## Layout

| Path | What it is |
|---|---|
| `views/main.crepus` | Starter Crepuscularity template that owns the UI source. |
| `fixture.json` | Shared View IR fixture used by both shells. Regenerate with `crepus native sync views/main.crepus --dir . --pretty` after editing your template. |
| `ios/` | XcodeGen app target plus local Swift Package runtime. |
| `android/` | Gradle module (open in Android Studio, or run `./gradlew :app:assembleDebug` after generating the Gradle wrapper). |

## Dev server

From the scaffold root:

```bash
crepus mobile dev --platform all --port 4001
```

The SwiftUI and Compose shells load `fixture.json` first, then connect to the local dev server in debug builds. The server exposes `/health`, `/ir`, and `/events`.

## iOS

```bash
crepus mobile build --platform ios --dir .
crepus mobile run --platform ios --dir .
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

From the scaffold root:

```bash
crepus native sync views/main.crepus --dir . --out desktop/share/dashboard.view-ir.json --var name=Ada --pretty
```

Or from a Rust crate that depends on `crepuscularity-native`:

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
`android/app/src/main/assets/fixture.json` so each shell sees the same IR. The
CLI sync command handles this mirroring for scaffolded projects. Use repeated
`--out FILE` flags for extra host resources such as desktop app bundles, and
add `--no-defaults` when only explicit host resources should be written.

## Build via the `crepus` CLI

```bash
crepus mobile build --platform ios --dir .
crepus mobile build --platform android --dir .
crepus mobile run --platform ios --dir .
crepus mobile run --platform android --dir .
crepus native build ios --dir .
crepus native build android --dir . --flavor Debug
crepus native run ios --dir .              # build + simctl install + simctl launch
crepus native run android --dir .          # gradle install + adb launch hint
```
