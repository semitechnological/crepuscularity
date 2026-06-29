//! Shared CLI output helpers — consistent styling across all commands.
//!
//! Theme mirrors ../wax: console::style for colours, indicatif for spinners.

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

// ── Symbols ─────────────────────────────────────────────────────────────────

pub fn ok() -> impl std::fmt::Display {
    style("✓").green().bold()
}
pub fn err() -> impl std::fmt::Display {
    style("✗").red().bold()
}
pub fn warn() -> impl std::fmt::Display {
    style("!").yellow().bold()
}
pub fn arrow() -> impl std::fmt::Display {
    style("→").dim()
}

// ── One-liner helpers ────────────────────────────────────────────────────────

pub fn success(msg: &str) {
    eprintln!("{} {}", ok(), msg);
}

pub fn error(msg: &str) -> ! {
    eprintln!("{} {}", err(), style(msg).red());
    std::process::exit(1);
}

pub fn warning(msg: &str) {
    eprintln!("  {} {}", warn(), style(msg).yellow());
}

pub fn step(label: &str) {
    eprintln!("  {} {}", arrow(), style(label).dim());
}

pub fn label(text: &str) -> impl std::fmt::Display + '_ {
    style(text).cyan().bold()
}

pub fn dim(text: &str) -> impl std::fmt::Display + '_ {
    style(text).dim()
}

// ── Timing ───────────────────────────────────────────────────────────────────

pub fn format_duration(elapsed: Duration) -> String {
    let ms = elapsed.as_millis();
    if ms < 1000 {
        format!("{ms}ms")
    } else {
        format!("{:.1}s", elapsed.as_secs_f64())
    }
}

/// Print a final "done in X" line.
pub fn done_in(elapsed: Duration) {
    eprintln!(
        "\n  {} done in {}",
        style("·").dim(),
        style(format_duration(elapsed)).cyan()
    );
}

// ── Spinner ──────────────────────────────────────────────────────────────────

pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        match ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}") {
            Ok(style) => style.tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
            Err(_) => ProgressStyle::default_spinner().tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
        },
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_message(msg.to_string());
    pb
}

/// Finish a spinner with a ✓ line and clear it.
pub fn spinner_ok(pb: &ProgressBar, msg: &str) {
    pb.finish_and_clear();
    eprintln!("  {} {}", ok(), msg);
}

/// Finish a spinner with a ✗ line, print error, and exit.
pub fn spinner_err(pb: &ProgressBar, msg: &str) -> ! {
    pb.finish_and_clear();
    error(msg);
}
