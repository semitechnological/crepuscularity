//! `crepus tui` — Terminal User Interface apps with Ratatui.
//!
//! Scaffold and build TUI applications that render `.crepus` templates in the terminal
//! using Ratatui for layout and Crossterm for input handling.

use std::fs;
use std::path::Path;
use std::process::Command;

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
        _ => print_tui_usage(),
    }
}

fn scaffold_tui_app(name: &str) {
    let dir = Path::new(name);

    // Create directory structure
    fs::create_dir_all(&dir).unwrap_or_else(|e| {
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
    eprintln!();
    eprintln!("{}", style("EXAMPLES").dim());
    eprintln!("  crepus tui new my-tui-app");
    eprintln!("  cd my-tui-app && crepus tui build");
    eprintln!("  crepus tui run");
}
