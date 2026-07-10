# Mobile App Example

A cross-platform task tracker built with `.crepus` templates, **codegen'd native views**, and a **Rust action dispatch backend**.

## Architecture

```
.crepus template → crepus native codegen → SwiftUI / Compose views
                     ↕
              Rust action dispatch (staticlib)
              state management + FFI eval
                     ↕
          iOS: @_silgen_name C ABI
          Android: JNI external functions
```

**Flow:** user taps button → `CrepusActions.perform("tasks.add")` → Rust dispatch → JSON result → `CrepusStateStore.applyResult()` → revision bump → UI re-renders via eval FFI.

## Structure

```
mobile-app/
  crepus.toml                    — target config (ios + android)
  views/
    main.crepus                  — app UI (tabs: Tasks, Notes, Settings)
  fixture.json                   — View IR for reference
  rust/
    Cargo.toml                   — staticlib + cdylib + rlib
    src/lib.rs                   — state, dispatch, C ABI + JNI exports
  ios/
    Package.swift                — SwiftPM package
    App/
      TaskTrackerApp.swift       — @main entry, calls CrepusRustActions.install()
    Sources/MobileApp/
      CrepusStateStore.swift     — ObservableObject wrapping Rust eval FFI
      CrepusRustActions.swift    — @_silgen_name FFI bridge
      TaskTrackerView.swift      — codegen'd from main.crepus (patched)
  android/
    build.gradle.kts             — root Gradle
    settings.gradle.kts
    app/
      build.gradle.kts           — Compose + kotlinx-serialization
      src/main/
        AndroidManifest.xml
        java/dev/crepuscularity/mobileapp/
          MainActivity.kt        — entry, calls CrepusRustActions.install()
          CrepusStateStore.kt    — Compose state wrapping Rust JNI evals
          CrepusRustActions.kt   — JNI external bridge
          TaskTrackerView.kt     — codegen'd from main.crepus (patched)
```

## Regenerate codegen views

```bash
# SwiftUI
crepus native codegen examples/mobile-app/views/main.crepus \
  --platform swiftui --out examples/mobile-app/ios/Sources/MobileApp \
  --view-name TaskTrackerView --pretty

# Compose
crepus native codegen examples/mobile-app/views/main.crepus \
  --platform compose --out examples/mobile-app/android/app/src/main/java/dev/crepuscularity/mobileapp \
  --view-name TaskTrackerView --pretty
```

After codegen, patch empty `Text()` / `VStack` blocks where `bind=` expressions should render text content (task.title, task.due, note.title, note.preview, app_version).

## Build

### Rust backend

```bash
cd rust

# iOS (staticlib for Xcode linking)
cargo build --target aarch64-apple-ios --release

# Android (staticlib for JNI)
cargo build --target aarch64-linux-android --release
```

### iOS

Link the Rust staticlib (`libmobile_app_actions.a`) in Xcode build settings, then:

```bash
cd ios && swift build
```

Or open `ios/Package.swift` in Xcode, add the staticlib to "Link Binary With Libraries", and run.

### Android

Place `libmobile_app_actions.so` in `app/src/main/jniLibs/arm64-v8a/`, then:

```bash
cd android && ./gradlew :app:assembleDebug
```

## Template syntax

```crepus
div flex flex-col w-full h-full bg-[#0a0a0a]
  tabs bind=current_tab
    tab value="tasks" label="Tasks" icon="checklist"
      for task in {tasks}
        toggle bind=task.done @change="tasks.toggle"
        div bind=task.title
```

Key features: `tabs`/`tab`, `if`/`for` conditionals, `toggle`/`slider` controls,
`bind=` data binding, `@click`/`@change` event handlers.
