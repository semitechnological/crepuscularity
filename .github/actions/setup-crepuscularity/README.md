# Example Usage: Crepuscularity Action

To use the crepuscularity setup action in your own GPUI project:

```yaml
name: Build

on: [push, pull_request]

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Crepuscularity
        uses: semitechnological/crepuscularity/.github/actions/setup-crepuscularity@main
        with:
          rust-version: stable
          cache: true
      
      - name: Build your project
        run: cargo build --release
      
      - name: Run tests
        run: cargo test
```

## Inputs

| Input | Description | Default |
|-------|-------------|---------|
| `rust-version` | Rust toolchain version | `stable` |
| `cache` | Enable cargo caching | `true` |

## What it does

1. Installs Rust toolchain
2. Sets up SDKROOT on macOS for GPUI
3. Configures cargo caching
4. Optionally installs crepu CLI
5. Verifies installation
