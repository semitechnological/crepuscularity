//! Ratatui backend for the Crepuscularity `.crepus` DSL.
//!
//! Write full-screen terminal UIs using the same indentation-based template
//! syntax as the web/GPUI backends — nested `div`s, flexbox-like sizing,
//! Tailwind class names for colours and modifiers — and render them straight
//! into a [`ratatui::Frame`].
//!
//! # Quick start
//!
//! ```rust,no_run
//! use crepuscularity_tui::{render_template, TemplateContext};
//! use ratatui::{backend::CrosstermBackend, Terminal};
//! use std::io;
//!
//! let template = r#"
//! div w-full h-full flex-col bg-zinc-950
//!   div h-[3] border-b flex-row px-2
//!     span text-white font-bold
//!       "My App"
//!     span flex-1
//!     span text-gray-400
//!       "press q to quit"
//!   div flex-1 flex-row
//!     div w-[20] border-r flex-col p-1
//!       "Sidebar"
//!     div flex-1 flex-col p-2
//!       "Main content — {user}"
//!   div h-[1] bg-zinc-800 text-gray-500 px-2
//!     "Ready"
//! "#;
//!
//! let mut ctx = TemplateContext::default();
//! ctx.set("user", "alice");
//!
//! let backend = CrosstermBackend::new(io::stdout());
//! let mut terminal = Terminal::new(backend).unwrap();
//! terminal.draw(|frame| {
//!     let area = frame.area();
//!     render_template(template, &ctx, frame, area).unwrap();
//! }).unwrap();
//! ```
//!
//! For file-backed templates, use [`template()`] when you want to keep ownership
//! of the Ratatui draw loop:
//!
//! ```rust,no_run
//! # use ratatui::{backend::TestBackend, Terminal};
//! # let backend = TestBackend::new(80, 24);
//! # let mut terminal = Terminal::new(backend).unwrap();
//! let mut ui = crepuscularity_tui::template("ui/ui.crepus").unwrap();
//! ui.set("title", "My App");
//! ui.set("input", "hello");
//! terminal.draw(|frame| {
//!     ui.draw_full(frame).unwrap();
//! }).unwrap();
//! ```
//!
//! Or let Crepus run one draw pass:
//!
//! ```rust,no_run
//! # use ratatui::{backend::TestBackend, Terminal};
//! # let backend = TestBackend::new(80, 24);
//! # let mut terminal = Terminal::new(backend).unwrap();
//! crepuscularity_tui::draw(&mut terminal, "ui/ui.crepus", |ui| {
//!     ui.set("title", "My App");
//!     ui.set("input", "hello");
//! }).unwrap();
//! ```
//!
//! # Layout model
//!
//! Templates map directly onto ratatui's split-`Rect` layout model:
//!
//! | `.crepus` class | Terminal behaviour |
//! |---|---|
//! | `flex-col` (default) | Children stacked vertically (`Direction::Vertical`) |
//! | `flex` / `flex-row` | Children stacked horizontally (`Direction::Horizontal`) |
//! | `flex-1` / no explicit size | `Constraint::Fill(1)` — equal share of remaining space |
//! | `w-[20]` / `h-[3]` | `Constraint::Length(n)` — exact terminal columns / rows |
//! | `w-1/2` | `Constraint::Percentage(50)` |
//! | `w-full` / `h-full` | `Constraint::Percentage(100)` |
//! | `gap-1` | `Layout::spacing(1)` — cells between children |
//!
//! # Styling
//!
//! Most Tailwind colour utilities work: `text-white`, `bg-zinc-950`,
//! `text-green-400`, `bg-[#1e1e2e]`, etc.  Text modifiers: `font-bold`,
//! `italic`, `underline`, `line-through`, `dim`.
//!
//! Borders: `border` (all), `border-t/b/l/r`, `rounded`, `border-double`,
//! `border-thick`.  A `title={expr}` binding on any bordered element sets the
//! block title.
//!
//! # Hot reload
//!
//! For live-coding `.crepus` files in a running terminal app, use
//! [`HotTemplate`]: it wraps a [`Template`] with a background `notify` watcher
//! and a polled "changed" flag. Drive your own crossterm event loop and call
//! [`HotTemplate::poll_and_draw_full`] each tick — when the file on disk
//! changes, the next draw renders the new source. The
//! [`TemplateContext`] (variables, `base_dir`) is preserved across reloads.
//!
//! ```rust,no_run
//! use crepuscularity_tui::HotTemplate;
//! # use ratatui::{backend::TestBackend, Terminal};
//! # let backend = TestBackend::new(80, 24);
//! # let mut terminal = Terminal::new(backend).unwrap();
//! let mut hot = HotTemplate::watch("ui/ui.crepus").unwrap();
//! hot.template_mut().set("title", "My App");
//! terminal.draw(|frame| {
//!     let _ = hot.poll_and_draw_full(frame);
//! }).unwrap();
//! ```
//!
//! # Current scope
//!
//! This crate is a rendering backend kernel. It supports core template
//! evaluation (`if`/`else`, `for`, `match`, `include`, `slot`, `$: let`,
//! `$: default`), maps static layout/styling into Ratatui widgets, and offers
//! file-watch hot reload via [`HotTemplate`]. `slot-rotate` cycles phrases on
//! a wall-clock timer (matching the web/SSR `interval={ms}` semantics) so a
//! poll-and-draw loop that ticks every ~100 ms — see
//! [`HotTemplate::poll_and_draw_full`] — animates phrases automatically. It
//! does not yet provide a terminal app runtime or generic event dispatch.

pub mod hot_reload;
pub mod render;
pub mod style;
pub mod template;
#[cfg(test)]
mod tests;

pub use crepuscularity_core::{
    build, parse_component_file, parse_template, CrepusError, TemplateContext, TemplateValue,
};
pub use crepuscularity_macros::template_refs;
pub use hot_reload::{HotTemplate, ReloadOutcome};
pub use ratatui;
pub use render::{paint_node, render_component, render_nodes, render_template};
pub use style::parse_classes;
pub use template::{draw, template, ElementRef, Template};
