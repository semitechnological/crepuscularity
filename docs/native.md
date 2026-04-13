# Native Shells

**Also:** [Documentation home](README.md) · [DSL](dsl.md) · [Components](components.md) · [CLI](cli.md)

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

## Quick Start

### iOS (SwiftUI)

```bash
# Scaffold a new iOS app
crepus ios new my-ios-app
cd my-ios-app
crepus ios generate
open *.xcodeproj
```

### Android (Jetpack Compose)

Use the example in `examples/native-shells/android/` as a starting point.

## Template to IR Conversion

Convert `.crepus` templates to View IR JSON:

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
  "type": "div",
  "style": {
    "display": "flex",
    "flexDirection": "column",
    "gap": 16
  },
  "children": [
    {
      "type": "span",
      "children": ["Hello Ada"]
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

Use the example in `examples/native-shells/android/`:

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
- **Grid**: `grid`, `grid-cols-3`, `grid-rows-2`
- **Spacing**: `gap-4`, `p-4`, `m-4`

### Styling

- **Colors**: `bg-blue-500`, `text-white`, `border-gray-300`
- **Typography**: `text-lg`, `font-bold`, `leading-relaxed`
- **Borders**: `rounded`, `border`, `border-2`
- **Shadows**: `shadow`, `shadow-lg`

### Components

- **Basic elements**: `div`, `span`, `button`, `img`
- **Form elements**: `input`, `textarea`
- **Interactive**: `@click`, `@submit` handlers
- **Animations**: `animate:opacity`, `animate:scale`

### Advanced Widgets

- **Scroll views**: `scroll`, `scroll-x`, `scroll-y`
- **Slot rotate**: Cycling text display
- **Custom components**: Via `include`

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
- **Image loading** should be optimized per platform
- **Animations** use platform-native implementations
- **Memory usage** scales with template complexity

## Examples

See `examples/native-shells/` for complete working examples:

- **iOS**: SwiftUI app with Swift Package integration
- **Android**: Jetpack Compose app with Gradle setup
- **Shared fixture**: Common View IR for testing

## Extending the IR

To add new features:

1. **Update the Rust parser** in `crepuscularity-native`
2. **Extend the JSON schema** (run `export-view-ir-schema`)
3. **Update native shells** to handle new IR nodes
4. **Test across platforms**

The IR format is designed to be extensible while maintaining compatibility.