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
crepus native add local-notifications --dir .
crepus native add secure-storage --dir .
crepus native add biometrics --dir .
crepus native add permissions --dir .
crepus native add microphone --dir .
crepus native add contacts --dir .
crepus native add calendar --dir .
crepus native add in-app-browser --dir .
crepus native add system-bars --dir .
crepus native add deep-links --dir .
```

`sensors` installs real gyro and accelerometer bridges. `bluetooth` installs Android and iOS BLE scan
bridges with `status`, `requestPermission`, `scan`, and `stopScan` operations.

`geolocation`, `battery`, `appearance`, `haptics`, `clipboard`, `image-picker`, and `documentPicker` (`documents`) add location permission/current-position,
battery status, color-scheme status, feedback generators, and clipboard access respectively. Geolocation is
cached: Android returns the most recent GPS/network fix and iOS returns the location manager's current location.
`battery.startWatch` and `battery.stopWatch` stream level or charging changes as `battery.change`.
`appearance.startWatch` and `appearance.stopWatch` stream system color-scheme changes as `appearance.change`.
`clipboard.startWatch` and `clipboard.stopWatch` stream pasteboard changes as `clipboard.change`.

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
`appState.get` returns whether the app is active or backgrounded. `appState.startWatch` and
`appState.stopWatch` stream lifecycle changes as `appState.change`.
`app.getInfo` returns the app name, identifier, version, and build.
`screenOrientation.get` returns portrait or landscape. `screenOrientation.lock` requires
`orientation: "portrait"` or `"landscape"`; `screenOrientation.unlock` restores rotation.
`accessibilityInfo.get` and `accessibilityInfo.status` return reduce-motion and screen-reader state.
`accessibilityInfo.startWatch` and `accessibilityInfo.stopWatch` stream status changes as `accessibilityInfo.change`.
`device.get` and `device.info` return the platform, device model, and operating-system version.
`preferences` stores string values with `get`, `set`, `remove`, and `clear`.
`network.status` returns whether the device has a validated connection and its transport; `network.startWatch` and `network.stopWatch` stream connection changes as `network.change`.
`keyboard.dismiss` closes the active system keyboard.
`settings.open` opens the operating system's page for this app.
`localNotifications.status` reports current notification authorization; `localNotifications` requests notification permission, posts immediate local notifications, and can `schedule`/`cancel`/`list` notifications. Scheduling requires an `id` and future Unix-millisecond `at` or `seconds` delay.
`secureStorage` persists values with Android Keystore encryption or the iOS Keychain.
`biometrics.authenticate` opens the system biometric/device-credential prompt.
`permissions.status`, `permissions.check`, and `permissions.request` operate on camera, microphone, location, photos, contacts, notifications, or Bluetooth; iOS notification checks emit the current OS authorization asynchronously.
`microphone.status` and `microphone.requestPermission` use Android `RECORD_AUDIO` or iOS `AVAudioSession` authorization.
`contacts.status`, `contacts.requestPermission`, and `contacts.list` use the Android contacts provider or iOS Contacts framework after explicit access is granted.
`calendar` requests access, lists calendars, and creates events using the platform calendar service.
`inAppBrowser.open` presents Android Custom Tabs or iOS Safari View Controller.
`systemBars.status` and `systemBars.set` inspect or update Android bar colors and icon contrast; iOS controls the supported window appearance style.
`deepLinks` receives `crepus://` URLs as `deepLinks.openUrl` events and opens URLs with `deepLinks.open`.

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
