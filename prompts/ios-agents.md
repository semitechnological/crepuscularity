# crepus ios — XcodeGen + SwiftUI shells

Scaffold iOS apps with XcodeGen project generation and SwiftUI rendering via View IR.

## Quick start

```bash
crepus ios new my-app
cd my-app
crepus ios generate               # xcodegen
crepus ios build                  # xcodegen + xcodebuild
```

Requirements: `brew install xcodegen`

## Project structure

```
my-app/
  ios/
    project.yml         # XcodeGen spec
    NativeShell/        # SwiftPM package with SwiftUI host
      Sources/
        main.swift      # loads fixture.json, renders via SwiftUI
  fixture.json          # shared View IR (from crepus native)
  app.crepus            # template source
  crepus.toml           # [ios] section
```

## crepus.toml [ios]

```toml
[ios]
scheme = "MyApp"
xcodegen_spec = "project.yml"
destination = "platform=iOS Simulator,name=iPhone 16,OS=latest"
```

## Workflow

1. Edit `app.crepus` (`.crepus` template)
2. `crepus native sync` — updates `fixture.json` with rendered View IR
3. `crepus ios generate` — runs `xcodegen` to update Xcode project
4. `crepus ios build` — builds and runs

## Build options

```bash
crepus ios build --release
crepus ios build --scheme MyApp --destination "platform=iOS Simulator,name=iPhone 16,OS=latest"
crepus ios build --spec alternative.yml
```

## Key crates

- `crepuscularity-native` — View IR rendering
- `crepuscularity-core` — parser, AST