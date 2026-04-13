# TUI Mode

**Also:** [Documentation home](README.md) · [DSL](dsl.md) · [Components](components.md) · [CLI](cli.md)

> **Note:** TUI (Terminal User Interface) rendering is currently in development. The `tui` branch contains experimental work for rendering `.crepus` templates in the terminal using [Ratatui](https://ratatui.rs).

## Overview

Terminal-based rendering allows you to preview and develop Crepuscularity templates directly in the terminal without needing a graphical environment. This is useful for:

- **Server environments** without display
- **SSH sessions** and remote development
- **Quick prototyping** and testing
- **CI/CD pipelines** with visual output

## Current Status

The TUI implementation is experimental and focuses on:

- **Benchmarking dashboards** - Terminal UI for performance monitoring
- **Template preview** - Basic rendering of templates in terminal
- **Hot reload** - Live updates as templates change

## Quick Start

```bash
# Switch to TUI branch
git checkout tui

# Build with TUI features
cargo build --features tui

# Preview a template in terminal
crepus preview views/ui.crepus --tui
```

## Architecture

TUI rendering uses [Ratatui](https://ratatui.rs) for terminal drawing and [Crossterm](https://docs.rs/crossterm) for input handling.

```
.crepus Template → Parser → Layout Engine → Ratatui Widgets → Terminal
```

## Supported Features

### Layout

- **Flexbox**: Basic flex layouts rendered as terminal grids
- **Grid**: Table-based grid layouts
- **Text flow**: Word wrapping and text alignment

### Styling

- **Colors**: Terminal color support (16/256/truecolor)
- **Typography**: Bold, italic, underline text styling
- **Borders**: ASCII box drawing characters

### Components

- **Text elements**: Paragraphs, spans with styling
- **Containers**: Bordered boxes, padding/margins
- **Interactive elements**: Basic button representations

## Development

The TUI implementation is located in the `tui` branch. Key files:

- `crates/crepuscularity-cli/src/benchmark_tui.rs` - Benchmarking dashboard
- `crates/crepuscularity-tui/` - (planned) Main TUI rendering crate

### Building

```bash
# From tui branch
SDKROOT=$(xcrun --show-sdk-path) cargo build --features tui
```

### Testing

```bash
# Run TUI-based benchmarks
cargo run --bin crepus -- benchmark --tui

# Preview templates
cargo run --bin crepus -- preview templates/ui.crepus
```

## Limitations

- **Color support**: Limited to terminal capabilities
- **Layout precision**: Terminal character grid constraints
- **Interactivity**: Basic navigation, no mouse support
- **Animations**: Limited to simple terminal animations

## Future Plans

- **Full template rendering** in terminal
- **Interactive development** with keyboard navigation
- **Layout debugging** tools
- **Export capabilities** (SVG, images)
- **Integration** with development servers

## Examples

See the `examples/` directory for templates that work well in TUI mode. Simple layouts with clear typography render best in terminals.

## Contributing

The TUI implementation is in active development. Contributions are welcome for:

- **Layout algorithms** for terminal constraints
- **Widget implementations** for common UI patterns
- **Input handling** and navigation
- **Performance optimization** for terminal rendering