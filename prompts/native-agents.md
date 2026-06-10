# crepus native — iOS + Android (View IR)

Lower `.crepus` templates to a JSON view tree (View IR) for SwiftUI and Jetpack Compose.

## Quick start

```bash
crepus native new my-app
cd my-app
crepus native ir ui.crepus       # preview View IR JSON
crepus native build ios          # xcodegen + xcodebuild
crepus native build android      # gradle assemble
crepus native run ios
crepus native run android
```

## Project structure

```
my-app/
  ios/                  # SwiftPM package (XcodeGen project.yml)
  android/              # Gradle module
  fixture.json          # shared View IR fixture (sync target)
  app.crepus            # template source
```

## View IR

Templates are lowered to a JSON intermediate representation:

```json
{
  "version": 3,
  "root": {
    "tag": "vstack",
    "children": [
      { "tag": "text", "value": "Hello" }
    ],
    "style": {
      "background": "#09090b",
      "textColor": "#fafafa",
      "spacing": 16
    }
  }
}
```

## Rust API

```rust
use crepuscularity_native::{
    render_template_to_ir, render_from_files,
    to_json, to_json_pretty
};

let ir = render_template_to_ir(template, &ctx)?;
let json = to_json_pretty(&ir)?;
```

## Syncing

```bash
crepus native sync                # update fixture.json from templates
```

The IR includes `diff_ir`/`apply_mutations` for hot-reload-style updates.

## View IR features

- `vstack` / `hstack` / `zstack` layout
- `text`, `image`, `button`, `textfield`, `list`, `scrollview`
- Tailwind-like color classes
- `if` / `for` / `match` control flow
- `include` for component reuse
- `bind` for two-way data binding
- Picker, toggle, slider form controls

## Key crates

- `crepuscularity-native` — View IR lowering, serialization, diffing
- `crepuscularity-core` — parser, AST, context