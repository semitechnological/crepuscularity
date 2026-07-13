# Native shell scaffold

Scaffolded by `crepus mobile new` or `crepus native new`. This is a starter for iOS (XcodeGen + SwiftUI) and
Android (Gradle + Compose) apps whose thin host shells call into the Rust crate
[`crepuscularity-native`](https://crates.io/crates/crepuscularity-native) and render generated native source.

## Layout

| Path | What it is |
|---|---|
| `views/main.crepus` | Starter Crepuscularity template that owns the UI source. |
| `fixture.json` | Shared IR artifact kept in sync with the template. |
| `ios/` | XcodeGen app target plus local Swift Package runtime. |
| `android/` | Gradle module (open in Android Studio, or run `./gradlew :app:assembleDebug` after generating the Gradle wrapper). |
| `rust/` | App-local cdylib/staticlib action bridge. |

## Dev server

From the scaffold root:

```bash
crepus mobile dev --platform all --port 4001
```

## iOS

```bash
crepus mobile build --platform ios --dir .
crepus mobile run --platform ios --dir .
```

## Android

The Gradle wrapper is **not** committed (it requires a binary `.jar`).
Generate it once before the first build:

```bash
cd android
gradle wrapper --gradle-version 8.10
./gradlew :app:assembleDebug
```

Or open `android/` in Android Studio — it regenerates the wrapper on import.

## Capabilities

The default scaffold ships no Bluetooth Java and requests no Bluetooth permissions.
Install on-demand capabilities into the app instead:

```bash
crepus native add sensors --dir .
crepus native add bluetooth --dir .
crepus native add photo-library --dir .
crepus native add camera --dir .
crepus native add dimensions --dir .
crepus native add dialog --dir .
crepus native add action-sheet --dir .
crepus native add app-state --dir .
crepus native add screen-orientation --dir .
crepus native add accessibility-info --dir .
crepus native add device --dir .
crepus native add preferences --dir .
crepus native add network --dir .
crepus native add keyboard --dir .
crepus native add settings --dir .
```

`sensors` installs real gyro and accelerometer bridges. `bluetooth` installs Android and iOS BLE scan
bridges with `status`, `requestPermission`, `scan`, and `stopScan` operations.

`geolocation`, `battery`, `appearance`, `haptics`, `clipboard`, `image-picker`, and `documentPicker` (`documents`) add location permission/current-position,
battery status, color-scheme status, feedback generators, and clipboard access respectively. Geolocation is
cached: Android returns the most recent GPS/network fix and iOS returns the location manager's current location.

`documentPicker.pick` opens the platform multiple-document picker and emits imported cached-file metadata. The
named `import_files` action remains available for built-in UI actions and uses the same picker lifecycle.
`imagePicker.pick` opens the system image/video picker and emits imported cached-file metadata; it also enables
the named `pick_media` action.
`photo-library` requests platform media permission only for `scan` and `getRecentMedia`, then emits cached-file
metadata for each asset it can access.
`camera.takePhoto` opens the system camera and emits one cached JPEG result.
`dimensions.get` returns the current display width, height, and scale.
`dialog.show` presents a native alert and streams whether it was accepted or cancelled.
`actionSheet.show` presents native choices and streams the selected label and index.
`appState.get` returns whether the app is active or backgrounded.
`screenOrientation.get` returns portrait or landscape.
`accessibilityInfo.get` and `accessibilityInfo.status` return reduce-motion and screen-reader state.
`device.get` and `device.info` return the platform, device model, and operating-system version.
`preferences` stores string values with `get`, `set`, `remove`, and `clear`.
`network.status` returns whether the device has a validated connection and its transport.
`keyboard.dismiss` closes the active system keyboard.
`settings.open` opens the operating system's page for this app.

## Regenerating the fixture from a template

From the scaffold root:

```bash
crepus native sync views/main.crepus --dir . --out desktop/share/dashboard.view-ir.json --var name=Ada --pretty
crepus native codegen views/main.crepus --platform swiftui --out ios/Sources/NativeShell/Generated --view-name CrepusGeneratedView
crepus native codegen views/main.crepus --platform compose --out android/app/src/main/java/dev/crepuscularity/nativeshell/generated --view-name CrepusGeneratedView
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
