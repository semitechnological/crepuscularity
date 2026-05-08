//! `crepus tui` — Terminal User Interface apps with Ratatui.
//!
//! Scaffold and build TUI applications that render `.crepus` templates in the terminal
//! using Ratatui for layout and Crossterm for input handling.

use std::fs;
use std::path::{Path, PathBuf};

use console::style;

use crate::ui;

pub fn run(args: &[String]) {
    match args.first().map(|s| s.as_str()) {
        Some("new") => {
            let name = args.get(1).map(|s| s.as_str()).unwrap_or_else(|| {
                ui::error("Usage: crepus tui new <name>");
            });
            scaffold_tui_app(name);
        }
        Some("build") => {
            let release = args.iter().any(|a| a == "--release");
            build_tui_app(release);
        }
        Some("run") => {
            run_tui_app();
        }
        Some("preview") => {
            let path = args.get(1).map(PathBuf::from).unwrap_or_else(|| {
                ui::error("Usage: crepus tui preview <file.crepus>");
            });
            preview_tui_template(&path);
        }
        _ => print_tui_usage(),
    }
}

fn scaffold_tui_app(name: &str) {
    let dir = Path::new(name);

    // Create directory structure
    fs::create_dir_all(dir).unwrap_or_else(|e| {
        ui::error(&format!("failed to create directory: {}", e));
    });

    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
crepuscularity-runtime = {{ path = "../../crates/crepuscularity-runtime" }}
ratatui = {{ version = "0.29", default-features = false, features = ["crossterm"] }}
crossterm = "0.28"
anyhow = "1.0"
tokio = {{ version = "1.0", features = ["full"] }}
"#,
        name
    );

    let main_rs = r###"use crepuscularity_runtime::TemplateContext;
use ratatui::prelude::*;
use crossterm::event::KeyCode;
use std::io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();
    let app_result = App::new().run(&mut terminal).await;
    ratatui::restore();
    app_result
}

struct App {
    should_quit: bool,
    template_ctx: TemplateContext,
}

impl App {
    fn new() -> Self {
        let mut ctx = TemplateContext::new();
        ctx.set("title", "My TUI App");
        ctx.set("message", "Hello from Crepuscularity!");
        ctx.set("quit_hint", "Press 'q' to quit");

        Self {
            should_quit: false,
            template_ctx: ctx,
        }
    }

    async fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Ok(true) = crossterm::event::poll(std::time::Duration::from_millis(100)) {
                if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                        _ => {}
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(frame.area());

        // Render header using template
        let header_template = r#"div bg-blue-900 text-white p-2
    div text-xl font-bold
        {title}
    div text-sm
        {quit_hint}"#;

        // For now, render as simple text - full template rendering would need more work
        frame.render_widget(
            Paragraph::new(format!("{}\n{}", self.template_ctx.get("title").unwrap_or("App"), self.template_ctx.get("quit_hint").unwrap_or("Press q to quit")))
                .style(Style::default().bg(Color::Blue).fg(Color::White)),
            layout[0]
        );

        // Render main content
        let content_template = r#"div flex flex-col items-center justify-center h-full text-center
    div text-2xl font-bold text-green-400
        {message}
    div text-lg text-gray-300 mt-4
        "This is rendered in the terminal"
    div text-sm text-gray-500 mt-8
        "Built with Crepuscularity + Ratatui"#;

        frame.render_widget(
            Paragraph::new(format!("{}", self.template_ctx.get("message").unwrap_or("Hello!")))
                .style(Style::default().fg(Color::Green))
                .alignment(Alignment::Center),
            layout[1]
        );
    }
}
"###;
    fs::write(dir.join("Cargo.toml"), cargo_toml).unwrap_or_else(|e| {
        ui::error(&format!("failed to write Cargo.toml: {}", e));
    });

    // Create src directory
    fs::create_dir_all(dir.join("src")).unwrap_or_else(|e| {
        ui::error(&format!("failed to create src directory: {}", e));
    });

    // Write src/main.rs
    fs::write(dir.join("src/main.rs"), main_rs).unwrap_or_else(|e| {
        ui::error(&format!("failed to write src/main.rs: {}", e));
    });

    // Create a basic .crepus template file
    let template_content = r#"div bg-black text-white flex flex-col gap-4
  div text-2xl font-bold text-green-400
    "Welcome to {title}"
  div text-lg
    "{message}"
  div text-sm text-gray-500
    "{quit_hint}"
"#;

    fs::write(dir.join("app.crepus"), template_content).unwrap_or_else(|e| {
        ui::error(&format!("failed to write app.crepus: {}", e));
    });

    ui::success(&format!(
        "Created new TUI app '{}' in directory '{}'",
        name,
        dir.display()
    ));
    eprintln!("Run 'cd {} && cargo run' to start the app", name);
}

fn build_tui_app(_release: bool) {
    ui::error("TUI build not implemented yet - use cargo build directly");
}

fn run_tui_app() {
    ui::error("TUI run not implemented yet - use cargo run directly");
}

fn print_tui_usage() {
    eprintln!(
        "{}",
        style("crepus tui — Terminal User Interface applications")
            .cyan()
            .bold()
    );
    eprintln!();
    eprintln!("{}", style("COMMANDS").dim());
    eprintln!(
        "  {}  {}",
        style("new <name>      ").green(),
        style("scaffold a new TUI app").dim()
    );
    eprintln!(
        "  {}  {}",
        style("build [--release]").green(),
        style("build the TUI app").dim()
    );
    eprintln!(
        "  {}  {}",
        style("run              ").green(),
        style("run the TUI app").dim()
    );
    eprintln!(
        "  {}  {}",
        style("preview <file>  ").green(),
        style("live-preview a .crepus template in the terminal").dim()
    );
    eprintln!();
    eprintln!("{}", style("EXAMPLES").dim());
    eprintln!("  crepus tui new my-tui-app");
    eprintln!("  cd my-tui-app && crepus tui build");
    eprintln!("  crepus tui run");
    eprintln!("  crepus tui preview app.crepus     # hot-reload preview, q/Esc to quit");
}

/// Live-preview a `.crepus` template in the current terminal.
///
/// Sets up crossterm raw mode via `ratatui::init`, watches the file for
/// changes via [`crepuscularity_tui::HotTemplate`], and re-renders on each
/// save. `q` or `Esc` exits cleanly; `r` forces a reload. A `context.toml`
/// next to the template is loaded as initial variables.
fn preview_tui_template(path: &Path) {
    use crepuscularity_tui::{HotTemplate, ReloadOutcome};
    use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
    use std::time::{Duration, Instant};

    if !path.exists() {
        ui::error(&format!("file not found: {}", path.display()));
    }

    let mut hot = match HotTemplate::watch(path) {
        Ok(h) => h,
        Err(e) => ui::error(&format!("could not load template: {e}")),
    };

    if let Some(dir) = path.parent() {
        let ctx_path = dir.join("context.toml");
        if ctx_path.exists() {
            load_tui_context_toml(&ctx_path, hot.template_mut().context_mut());
        }
    }

    eprintln!(
        "{} previewing {} (q/Esc to quit, r to force reload)",
        style("crepus tui").dim(),
        style(path.display().to_string()).cyan().bold()
    );

    let mut terminal = ratatui::init();
    let mut last_reload: Option<(Instant, ReloadOutcome)> = None;

    let result: Result<(), String> = loop {
        let outcome_holder = std::cell::Cell::new(ReloadOutcome::Unchanged);
        if let Err(e) = terminal.draw(|frame| match hot.poll_and_draw_full(frame) {
            Ok(o) => outcome_holder.set(o),
            Err(_) => outcome_holder.set(ReloadOutcome::Unchanged),
        }) {
            break Err(format!("terminal draw: {e}"));
        }
        match outcome_holder.into_inner() {
            ReloadOutcome::Unchanged => {}
            other => last_reload = Some((Instant::now(), other)),
        }

        if matches!(event::poll(Duration::from_millis(100)), Ok(true)) {
            match event::read() {
                Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                    KeyCode::Char('r') => {
                        if let Err(e) = hot.template_mut().reload() {
                            last_reload = Some((Instant::now(), ReloadOutcome::Error(e)));
                        } else {
                            last_reload = Some((Instant::now(), ReloadOutcome::Reloaded));
                        }
                    }
                    _ => {}
                },
                Ok(_) => {}
                Err(e) => break Err(format!("event read: {e}")),
            }
        }

        if let Some((t, _)) = &last_reload {
            if t.elapsed() > Duration::from_secs(2) {
                last_reload = None;
            }
        }
    };
    ratatui::restore();

    if let Some((_, ReloadOutcome::Error(msg))) = last_reload {
        eprintln!("{} {}", style("⚠").yellow(), msg);
    }
    if let Err(e) = result {
        ui::error(&e);
    }
}

/// Minimal TOML loader (key=value, strings/bools/ints/floats) shared with
/// `crepus preview`.  Lives here to avoid the `desktop` cfg-gate so non-GPUI
/// builds still get TUI preview support.
fn load_tui_context_toml(path: &Path, ctx: &mut crepuscularity_tui::TemplateContext) {
    use crepuscularity_tui::TemplateValue;
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(eq) = line.find('=') {
            let key = line[..eq].trim();
            let val = line[eq + 1..].trim();
            if val == "true" {
                ctx.set(key, TemplateValue::Bool(true));
            } else if val == "false" {
                ctx.set(key, TemplateValue::Bool(false));
            } else if let Ok(n) = val.parse::<i64>() {
                ctx.set(key, TemplateValue::Int(n));
            } else if let Ok(f) = val.parse::<f64>() {
                ctx.set(key, TemplateValue::Float(f));
            } else {
                ctx.set(key, strip_one_outer_quote_pair(val).to_string());
            }
        }
    }
}

/// Strip exactly one matching pair of `"`/`'` quotes from the outside of `s`.
///
/// Unlike `str::trim_matches('"').trim_matches('\'')` this preserves repeated
/// quotes inside the value, e.g. `"\"\"hi\"\""` → `\"hi\"` rather than `hi`.
fn strip_one_outer_quote_pair(s: &str) -> &str {
    let bytes = s.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            // Slice on byte indices that we just verified are `"` / `'` — both
            // ASCII, so we are always on a UTF-8 char boundary.
            return &s[1..s.len() - 1];
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::strip_one_outer_quote_pair;

    #[test]
    fn strips_single_outer_double_quotes() {
        assert_eq!(strip_one_outer_quote_pair("\"hello\""), "hello");
    }

    #[test]
    fn strips_single_outer_single_quotes() {
        assert_eq!(strip_one_outer_quote_pair("'hi'"), "hi");
    }

    #[test]
    fn preserves_inner_quotes_when_repeated() {
        assert_eq!(
            strip_one_outer_quote_pair("\"\"keep\"\""),
            "\"keep\"",
            "only the outermost pair should be stripped"
        );
    }

    #[test]
    fn preserves_mismatched_quotes() {
        assert_eq!(strip_one_outer_quote_pair("\"hi'"), "\"hi'");
        assert_eq!(strip_one_outer_quote_pair("'hi\""), "'hi\"");
    }

    #[test]
    fn preserves_unquoted_value() {
        assert_eq!(strip_one_outer_quote_pair("plain"), "plain");
    }

    #[test]
    fn handles_short_inputs() {
        assert_eq!(strip_one_outer_quote_pair(""), "");
        assert_eq!(strip_one_outer_quote_pair("\""), "\"");
        assert_eq!(strip_one_outer_quote_pair("\"\""), "");
    }
}
