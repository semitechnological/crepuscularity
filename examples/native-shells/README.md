# Native shells (SwiftUI + Jetpack Compose)

iOS workflow is **one command** from the Crepuscularity CLI:

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

Replace `NativeShell/Sources/NativeShell/fixture.json` with IR from `crepuscularity-native` (`render_template_to_ir` / `to_json_pretty`) when you change templates.

Android: open `android/` in Android Studio and run the `app` configuration (Gradle project lives next to this README when the full example tree is present).
