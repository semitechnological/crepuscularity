# crepus tui — terminal apps (Ratatui)

Full-screen terminal UIs using `.crepus` templates with Ratatui + Crossterm.

## Quick start

```bash
crepus tui new my-tui
cd my-tui
crepus tui run                  # cargo run
crepus tui build --release      # cargo build --release
crepus tui preview app.crepus   # hot-reload preview (q/Esc to quit)
```

## Project structure

```
my-tui/
  app.crepus           # template (UnoCSS-like classes for terminal)
  Cargo.toml           # dep: crepuscularity-tui, ratatui, crossterm
  src/main.rs          # HotTemplate-based app loop
```

## Template syntax

Templates use the same indentation-based syntax as web/GPUI, but with terminal-specific classes:

```
div w-full h-full bg-black text-white flex flex-col gap-4
 div text-2xl font-bold text-green-400
   "Welcome to {title}"
 div text-lg
   "{message}"
 div text-sm text-gray-500
   "{quit_hint}"
```

Terminal classes: `w-full`, `h-full`, `w-[N]`, `h-[N]`, `flex-col`, `flex-row`, `flex-1`, `border`, `border-r`, `border-b`, `px-N`, `py-N`, `p-N`, `gap-N`, text colors, bg colors.

## Rust integration

```rust
use crepuscularity_tui::{HotTemplate, ReloadOutcome};
use crossterm::event::{Event, KeyCode};
use ratatui::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut hot = HotTemplate::watch("app.crepus")?;
    hot.template_mut()
        .set("title", "My TUI App")
        .set("message", "Hello!")
        .set("quit_hint", "Press 'q' to quit");

    let mut terminal = ratatui::init();
    // draw loop...
    ratatui::restore();
}
```

## Hot-reload

- `HotTemplate::watch("file.crepus")` watches the file for changes
- `hot.poll_and_draw_full(frame)` re-renders on save
- `context.toml` next to the template is loaded as initial variables
- Preview mode: `q`/`Esc` to quit, `r` to force reload

## Key crates

- `crepuscularity-tui` — Ratatui rendering, HotTemplate
- `crepuscularity-core` — parser, AST, context