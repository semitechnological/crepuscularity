# Mobile App Example

A cross-platform mobile app built with `.crepus` templates and **View IR**.
Demonstrates tabs, toggles, sliders, conditionals, and list rendering.

## Structure

```
mobile-app/
  crepus.toml            — target config (ios + android)
  views/
    main.crepus          — app UI (tabs: Tasks, Notes, Settings)
  fixture.json           — generated View IR (version 4)
  ios/
    Package.swift        — SwiftPM package
    Sources/MobileApp/
      ViewIrModels.swift — IR decoder (ViewNode, ViewStyle, etc.)
      ViewIrTreeView.swift — SwiftUI recursive renderer
      fixture.json       — bundled IR for iOS
  android/
    build.gradle.kts     — root Gradle
    settings.gradle.kts
    app/
      build.gradle.kts   — app module (Compose)
      src/main/
        AndroidManifest.xml
        assets/fixture.json
        java/dev/crepuscularity/mobileapp/
          MainActivity.kt
          ViewIr.kt      — IR decoder
          ViewIrTree.kt  — Compose recursive renderer
```

## Regenerate fixture

```bash
crepus native sync examples/mobile-app/views/main.crepus \
  --out examples/mobile-app/fixture.json --pretty \
  --var current_tab=tasks \
  --var tasks_count=2 --var notes_count=1 \
  --var dark_mode=true --var notifications=true \
  --var sync_enabled=false --var font_size=16 \
  --var app_version="v1.0.0"
```

## iOS

```bash
cd ios && swift build
```

Open in Xcode: add `ios/` as a local SwiftPM dependency, import `MobileApp`,
and use `FixtureRootView()` or `ViewIrRootView(ir:)`.

## Android

```bash
cd android && ./gradlew :app:assembleDebug
```

Open in Android Studio (JDK 17+). The app reads `fixture.json` from assets
and renders via Compose.

## Template syntax

The `.crepus` template uses indent-based markup with Tailwind-like classes:

```
div flex flex-col w-full h-full bg-[#0a0a0a]
  tabs bind=current_tab
    tab value="tasks" label="Tasks" icon="checklist"
      for task in {tasks}
        toggle bind=task.done @change="tasks.toggle"
```

Key features used: `tabs`/`tab`, `if`/`for` conditionals, `toggle`/`slider` controls,
`bind=` data binding, `@click`/`@change` event handlers.
