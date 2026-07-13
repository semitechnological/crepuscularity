# Native Shells

**Also:** [Documentation home](README.md) · [DSL](dsl.md) · [Components](components.md) · [CLI](cli.md) · [Polyglot plugins](polyglot.md)

Crepuscularity can target native mobile platforms through **View IR** (Intermediate Representation), enabling code sharing between iOS (SwiftUI) and Android (Jetpack Compose) applications.

## Overview

Instead of compiling directly to native UI frameworks, Crepuscularity generates a **JSON-based intermediate representation** that native shells can interpret. This approach allows:

- **Code sharing** between platforms
- **Runtime flexibility** - update UI without app store releases
- **Consistent styling** across platforms
- **Platform-specific optimizations**

## Architecture

```
.crepus Template → Rust Parser → View IR (JSON) → Native Shell
                                      ↓
                               SwiftUI / Jetpack Compose
```

The `crepuscularity-native` crate converts templates to a structured JSON format that native applications can consume.

For non-Rust hosts, the stable boundary is the same View IR JSON emitted by `crepus native ir`. See [Polyglot plugins](polyglot.md).

## Quick Start

Prefer the unified mobile workflow for new iOS + Android apps:

```bash
crepus mobile new my-mobile-app
cd my-mobile-app
crepus mobile dev --platform all
crepus mobile build --platform ios
crepus mobile run --platform ios
crepus mobile build --platform android
```

### Cross-platform scaffold (iOS + Android in one command)

```bash
# Scaffold an iOS XcodeGen app + Android Gradle module sharing a fixture.json
crepus native new my-mobile-app
cd my-mobile-app

# iOS — build, install, and launch the simulator app
crepus mobile build --platform ios --dir .
crepus mobile run --platform ios --dir .
# or:    crepus native build ios --dir .

# Android — generate the Gradle wrapper (one-time), then build
cd android && gradle wrapper --gradle-version 8.10
./gradlew :app:assembleDebug
# or, from the project root:
#    crepus native build android --dir <root>
```

`crepus native run android --dir <root>` runs `gradle :app:installDebug` on a
connected device/emulator and prints the `adb shell am start` line you need to
launch the app.

The default shell contains View IR, host glue, and the app-local Rust bridge only: no
vendored Bluetooth Java or Bluetooth permission. Add platform capabilities per app:

```bash
crepus native add sensors --dir .
crepus native add bluetooth --dir .
```

Sensors installs real gyro/accelerometer bridges. Bluetooth installs the Android BLE central scan
bridge and its platform declarations.

The scaffold includes `views/main.crepus` as the Crepus-authored UI source.
When the template changes, sync the shared View IR fixture into the native
containers:

```bash
crepus native sync views/main.crepus --dir . --out desktop/share/dashboard.view-ir.json --var name=Ada --pretty
```

### iOS-only scaffold with XcodeGen + a real `.xcodeproj`

For iOS-only apps, use the iOS-specific scaffolder:

```bash
wax install xcodegen
crepus ios new my-ios-app
cd my-ios-app
crepus ios generate
open *.xcodeproj
```

### Android (Jetpack Compose)

`crepus native new` lays out a minimal Compose app under `<dir>/android/`.
For a deeper reference, see `examples/native-shells/android/`.

## Template to IR Conversion

Convert `.crepus` templates to View IR JSON:

```bash
crepus native ir views/main.crepus --ctx context.json --pretty
crepus native sync views/main.crepus --dir my-mobile-app --out desktop/share/dashboard.view-ir.json --var name=Ada --pretty
crepus native codegen views/main.crepus --platform swiftui --out Generated --view-name DashboardView --var name=Ada
crepus native codegen views/main.crepus --platform compose --out android/app/src/main/java/dev/example/generated --view-name DashboardView --var name=Ada
```

```rust
use crepuscularity_core::TemplateContext;
use crepuscularity_native::{render_template_to_ir, to_json_pretty};

let mut ctx = TemplateContext::new();
ctx.set("name", "Ada");

let ir = render_template_to_ir(
    r#"div flex flex-col gap-4
  span
    "Hello {name}""#,
    &ctx,
)?;

let json = to_json_pretty(&ir)?;
println!("{}", json);
```

This produces JSON that native shells can parse:

```json
{
  "version": 4,
  "root": [
    {
      "kind": "stack",
      "axis": "column",
      "spacing": 16.0,
      "style": {
        "flexDirection": "column"
      },
      "children": [
        {
          "kind": "text",
          "content": "Hello Ada"
        }
      ]
    }
  ]
}
```

## iOS Integration

### Using the Swift Package

Add the local Swift package to your Xcode project:

1. In Xcode: **File → Add Package Dependencies**
2. Select **Add Local...** and choose the `ios/` directory
3. Import `NativeShell` in your Swift code

```swift
import NativeShell
import SwiftUI

struct ContentView: View {
    var body: some View {
        // Use the fixture
        FixtureRootView()

        // Or load custom IR
        if let ir = loadViewIR() {
            ViewIrRootView(ir: ir)
        }
    }
}
```

### CLI-Scaffolded Apps

The `crepus ios` commands create a complete Xcode project:

```bash
crepus ios new MyApp
cd MyApp
crepus ios generate  # Creates .xcodeproj
crepus ios build    # Builds for simulator
```

## Android Integration

### Jetpack Compose Setup

`crepus native new <name>` lays out a Compose app under `<name>/android/` —
the same source as `examples/native-shells/android/`:

```kotlin
// MainActivity.kt
class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            // Load and render View IR
            val ir = loadViewIR(assets)
            ViewIrRootView(ir)
        }
    }
}
```

## Supported Features

### Layout

- **Flexbox**: `flex`, `flex-row`, `flex-col`, `justify-center`, `items-center`
- **Spacing**: `gap-4`, `p-4`, `m-4`
- **Scroll containers**: `scroll`, `scroll-x`, `scroll-y`

### Styling

- **Colors**: `bg-blue-500`, `text-white`, `border-gray-300`
- **Typography**: `text-lg`, `font-bold`, `leading-relaxed`
- **Borders**: `rounded`, `border`, `border-2`
- **Shadows**: `shadow`, `shadow-lg`

### Components

- **Basic elements**: `div`, `span`, `button`, `img`
- **Interactive**: `button @click` emits an action string for the native shell to handle

### Advanced Widgets

- **Slot rotate**: Emits phrase data; the example shells currently display the first phrase
- **Custom components**: Via `include`
- **Control flow**: `if`, `for`, and `match`
- **Slots**: Component slots with caller context

The example SwiftUI and Jetpack Compose shells are intentionally small interpreters for the IR. They decode and render the core nodes, while app-specific image loading, navigation, action routing, accessibility, and animation behavior belong in the host shell.

## Generated Source

For production mobile shells that should compile native source instead of interpreting JSON at runtime, generate SwiftUI or Jetpack Compose from the same View IR:

```bash
crepus native codegen views/main.crepus --platform swiftui --out Generated --view-name DashboardView --var name=Ada
crepus native codegen views/main.crepus --platform compose --out android/app/src/main/java/dev/example/generated --view-name DashboardView --var name=Ada
```

The generator keeps View IR as the stable compiler boundary, then writes `DashboardView.swift` or `DashboardView.kt`. The generated source covers current IR nodes with native SwiftUI/Compose controls and conservative textual fallbacks for host-specific surfaces like image loading and drop handling.

## Runtime Updates

Since View IR is JSON, you can update UI at runtime:

```swift
// iOS
func updateUI(with newIR: ViewIR) {
    // Parse new IR and update view hierarchy
    viewIrRootView.updateIR(newIR)
}
```

```kotlin
// Android
fun updateUI(newIR: ViewIR) {
    // Recompose with new IR
    setContent {
        ViewIrRootView(newIR)
    }
}
```

## Development Workflow

1. **Design templates** in `.crepus` files
2. **Generate IR** using `crepuscularity-native`
3. **Test in native shells** using the examples
4. **Integrate** into your app's native code

### Hot Reload

For development, you can set up file watching:

```bash
# Watch templates and regenerate IR
cargo watch -x 'run --bin generate-ir'
```

## Platform-Specific Customization

While the core layout is shared, each platform can customize:

### iOS (SwiftUI)

- **Custom fonts** and **SF Symbols**
- **iOS-specific animations** and transitions
- **Accessibility** features
- **Dark mode** integration

### Android (Jetpack Compose)

- **Material Design 3** components
- **Platform animations**
- **System UI integration**
- **Dynamic color** themes

## Performance Considerations

- **IR parsing** happens at runtime - keep templates reasonably sized
- **Includes** are expanded before lowering to IR; file-backed includes reject absolute paths and `..` traversal
- **Image loading** should be optimized per platform
- **Animations** should be implemented in the host shell until animation nodes are added to the IR
- **Memory usage** scales with template complexity

## Examples

See `examples/native-shells/` for complete working examples (the same source
that `crepus native new` scaffolds):

- **iOS**: SwiftUI app with Swift Package integration
- **Android**: Jetpack Compose app with Gradle setup
- **Shared fixture**: Common View IR for testing

`crepus native new <name>` writes that tree (minus the binary Gradle wrapper
jar — generate it with `gradle wrapper --gradle-version 8.10` or open the
project in Android Studio).

## Extending the IR

To add new features:

1. **Update the Rust parser** in `crepuscularity-native`
2. **Extend the JSON schema** (run `export-view-ir-schema`)
3. **Update native shells** to handle new IR nodes
4. **Test across platforms**

The IR format is designed to be extensible while maintaining compatibility.
