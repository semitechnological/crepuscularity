//! Full-screen results dashboard after `crepus benchmark` (human mode, TTY stderr).
//!
//! The interactive dashboard (ratatui) is only compiled when the `tui` feature
//! is enabled.  Without it, only [`TargetOutcomeSummary`] is available so the
//! plain-text fallback in `benchmark.rs` always works.

#[cfg(feature = "tui")]
use std::io::{self, stderr, IsTerminal};

// ── Always-available types ──────────────────────────────────────────────────

pub(crate) struct TargetOutcomeSummary {
    pub suite: String,
    pub id: String,
    pub label: String,
    pub wall_ms: u128,
    pub status: &'static str,
    pub detail: String,
}

// ── Ratatui interactive dashboard (feature = "tui") ────────────────────────

#[cfg(feature = "tui")]
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
#[cfg(feature = "tui")]
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
#[cfg(feature = "tui")]
use ratatui::crossterm::ExecutableCommand;
#[cfg(feature = "tui")]
use ratatui::layout::{Constraint, Direction, Layout};
#[cfg(feature = "tui")]
use ratatui::prelude::*;
#[cfg(feature = "tui")]
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

#[cfg(feature = "tui")]
fn fmt_ms(ms: u128) -> String {
    if ms < 1000 {
        format!("{ms}ms")
    } else {
        format!("{:.1}s", ms as f64 / 1000.0)
    }
}

#[cfg(feature = "tui")]
fn elide(s: &str, max_chars: usize) -> String {
    let t = s.trim().replace('\n', " ");
    if t.chars().count() <= max_chars {
        t
    } else {
        format!(
            "{}…",
            t.chars()
                .take(max_chars.saturating_sub(1))
                .collect::<String>()
        )
    }
}

#[cfg(feature = "tui")]
fn draw_title(area: Rect, buf: &mut Buffer, dry_run: bool) {
    let title = if dry_run {
        Line::from(" crepus benchmark — dry-run plan ")
    } else {
        Line::from(" crepus benchmark — results ")
    };
    Paragraph::new(title)
        .block(Block::default().borders(Borders::ALL))
        .render(area, buf);
}

#[cfg(feature = "tui")]
fn draw_table(area: Rect, buf: &mut Buffer, outcomes: &[TargetOutcomeSummary], scroll: usize) {
    let header = Row::new(vec![
        Cell::from("Suite"),
        Cell::from("Target"),
        Cell::from("St"),
        Cell::from("Wall"),
        Cell::from("Label / notes"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));

    let inner_h = area.height.saturating_sub(2).max(3);
    let visible = inner_h as usize;
    let max_scroll = outcomes.len().saturating_sub(1);
    let scroll = scroll.min(max_scroll);

    let slice: Vec<&TargetOutcomeSummary> = outcomes.iter().skip(scroll).take(visible).collect();

    let rows: Vec<Row> = slice
        .iter()
        .map(|o| {
            let st_style = match o.status {
                "ok" => Style::default().fg(Color::Green),
                "skip" => Style::default().fg(Color::Yellow),
                "fail" => Style::default().fg(Color::Red),
                _ => Style::default(),
            };
            let note = if o.detail.is_empty() {
                o.label.as_str()
            } else {
                o.detail.as_str()
            };
            Row::new(vec![
                Cell::from(o.suite.as_str()),
                Cell::from(o.id.as_str()),
                Cell::from(o.status).style(st_style),
                Cell::from(fmt_ms(o.wall_ms)),
                Cell::from(elide(note, 64)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(14),
            Constraint::Percentage(18),
            Constraint::Percentage(8),
            Constraint::Percentage(12),
            Constraint::Percentage(48),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Targets (↑↓ scroll) "),
    );

    Widget::render(table, area, buf);
}

#[cfg(feature = "tui")]
fn draw_insights(area: Rect, buf: &mut Buffer, top_completed: &[(String, f64, u128)], dry_run: bool) {
    let mut insight = String::new();
    if dry_run {
        insight.push_str("Dry-run — no wall times recorded.\n");
    } else if top_completed.is_empty() {
        insight.push_str("No completed targets.\n");
    } else {
        for (label, pct, ms) in top_completed.iter().take(8) {
            insight.push_str(&format!(
                "{:>5.1}%  {:>8}  {}\n",
                pct,
                fmt_ms(*ms),
                elide(label, 72)
            ));
        }
    }

    Paragraph::new(insight)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Share of completed wall time "),
        )
        .render(area, buf);
}

#[cfg(feature = "tui")]
fn draw_footer(area: Rect, buf: &mut Buffer) {
    Paragraph::new(Line::from(
        " q / Esc — quit   ↑↓ j/k — scroll   PgUp/PgDn — page ",
    ))
    .render(area, buf);
}

#[cfg(feature = "tui")]
struct BenchmarkDashboard<'a> {
    outcomes: &'a [TargetOutcomeSummary],
    top_completed: &'a [(String, f64, u128)],
    dry_run: bool,
    scroll: usize,
}

#[cfg(feature = "tui")]
impl<'a> Widget for BenchmarkDashboard<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(6),
                Constraint::Length(1),
            ])
            .split(area);

        draw_title(chunks[0], buf, self.dry_run);
        draw_table(chunks[1], buf, self.outcomes, self.scroll);
        draw_insights(chunks[2], buf, self.top_completed, self.dry_run);
        draw_footer(chunks[3], buf);
    }
}

/// `Ok(true)` if an interactive screen was shown; `Ok(false)` if skipped (no TTY).
#[cfg(feature = "tui")]
pub(crate) fn try_show_dashboard(
    outcomes: &[TargetOutcomeSummary],
    top_completed: &[(String, f64, u128)],
    dry_run: bool,
) -> io::Result<bool> {
    if outcomes.is_empty() || !stderr().is_terminal() {
        return Ok(false);
    }

    let mut stderr = stderr();
    stderr.execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let mut scroll: usize = 0;

    let run: io::Result<()> = (|| {
        fn visible_rows(term: &Terminal<CrosstermBackend<std::io::Stderr>>) -> usize {
            term.size()
                .map(|r| r.height.saturating_sub(12) as usize)
                .unwrap_or(10)
                .max(1)
        }

        terminal.draw(|f| {
            let dashboard = BenchmarkDashboard {
                outcomes,
                top_completed,
                dry_run,
                scroll,
            };
            f.render_widget(dashboard, f.area());
        })?;

        loop {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Down | KeyCode::Char('j') => {
                        scroll = (scroll + 1).min(outcomes.len().saturating_sub(1));
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        scroll = scroll.saturating_sub(1);
                    }
                    KeyCode::PageDown => {
                        let v = visible_rows(&terminal);
                        scroll = (scroll + v).min(outcomes.len().saturating_sub(1));
                    }
                    KeyCode::PageUp => {
                        let v = visible_rows(&terminal);
                        scroll = scroll.saturating_sub(v);
                    }
                    _ => continue,
                },
                _ => continue,
            }
            terminal.draw(|f| {
                let dashboard = BenchmarkDashboard {
                    outcomes,
                    top_completed,
                    dry_run,
                    scroll,
                };
                f.render_widget(dashboard, f.area());
            })?;
        }
        Ok(())
    })();

    disable_raw_mode()?;
    drop(terminal);
    io::stderr().execute(LeaveAlternateScreen)?;

    run?;
    Ok(true)
}
