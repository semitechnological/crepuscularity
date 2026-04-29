//! Terminal UI and headless drivers for cross-shell / V8 benchmarks.

use std::cmp::Ordering;
use std::fs;
use std::io::{self, stdout};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use clap::{Parser, Subcommand, ValueEnum};
use crepuscularity_lite::bench_eval::{
    bench_matrix_shared_for_configs, eval_guest_from_config, eval_guest_from_config_with_shared,
    BenchMatrixShared,
};
use crepuscularity_lite::config::CrepusLiteConfig;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table};
use ratatui::Terminal;
use serde_json::json;

const STARTERS: &[&str] = &[
    "vanilla",
    "typescript",
    "react",
    "solid",
    "vue",
    "svelte",
    "angular",
    "nextjs",
];

#[derive(Copy, Clone, Debug, Default, PartialEq, ValueEnum)]
enum SortBy {
    /// Lower wall time first (usually what you want).
    #[default]
    Ms,
    /// Lower benchmark score first.
    ScoreAsc,
    /// Higher benchmark score first (best throughput at top).
    ScoreDesc,
}

#[derive(Parser, Debug)]
#[command(
    name = "crepus-bench",
    about = "TUI + headless tools for crepuscularity-lite benchmarks"
)]
struct Cli {
    /// Repository root (guest paths in TOML are relative to this cwd).
    #[arg(long, global = true, default_value = ".")]
    repo: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Interactive terminal UI (default if no subcommand).
    Tui,
    /// Run the labeled V8 matrix headless; prints rows for `compare.sh`.
    Matrix {
        #[arg(long, value_enum)]
        sort: Option<SortBy>,
        /// Run one isolate per starter in parallel (default: on).
        #[arg(long, default_value_t = true)]
        parallel: bool,
        #[arg(long)]
        no_parallel: bool,
        /// Print machine-oriented lines only (V8_BENCH\\t…).
        #[arg(long)]
        plain: bool,
    },
}

#[derive(Debug, Clone)]
struct MatrixRow {
    label: String,
    sort_ms: f64,
    sort_score: f64,
    payload: Option<String>,
    err: Option<String>,
}

impl MatrixRow {
    fn from_loaded(
        repo: &Path,
        label: &str,
        cfg_res: &Result<CrepusLiteConfig, String>,
        shared: Option<&BenchMatrixShared>,
    ) -> Self {
        match cfg_res {
            Err(e) => Self {
                label: label.to_string(),
                sort_ms: f64::MAX,
                sort_score: f64::NEG_INFINITY,
                payload: None,
                err: Some(format!("config: {e}")),
            },
            Ok(config) => {
                let eval = match shared {
                    Some(s) => eval_guest_from_config_with_shared(repo, config, s),
                    None => eval_guest_from_config(repo, config),
                };
                match eval {
                    Ok(json) => {
                        let (ms, score) = parse_bench_numbers(&json);
                        Self {
                            label: label.to_string(),
                            sort_ms: ms,
                            sort_score: score,
                            payload: Some(json),
                            err: None,
                        }
                    }
                    Err(e) => Self {
                        label: label.to_string(),
                        sort_ms: f64::MAX,
                        sort_score: f64::NEG_INFINITY,
                        payload: None,
                        err: Some(e),
                    },
                }
            }
        }
    }
}

fn parse_bench_numbers(json: &str) -> (f64, f64) {
    let v: serde_json::Value = serde_json::from_str(json).unwrap_or(json!({}));
    let b = v.get("benchmark").cloned().unwrap_or(json!({}));
    let ms = b
        .get("ms")
        .and_then(|x| x.as_f64())
        .or_else(|| b.get("ms").and_then(|x| x.as_i64().map(|i| i as f64)))
        .unwrap_or(f64::MAX);
    let score = b
        .get("score")
        .and_then(|x| x.as_f64())
        .or_else(|| b.get("score").and_then(|x| x.as_i64().map(|i| i as f64)))
        .unwrap_or(0.0);
    (ms, score)
}

fn sort_rows(rows: &mut [MatrixRow], sort: SortBy) {
    rows.sort_by(|a, b| {
        let ord = match sort {
            SortBy::Ms => a.sort_ms.partial_cmp(&b.sort_ms).unwrap_or(Ordering::Equal),
            SortBy::ScoreAsc => a
                .sort_score
                .partial_cmp(&b.sort_score)
                .unwrap_or(Ordering::Equal),
            SortBy::ScoreDesc => b
                .sort_score
                .partial_cmp(&a.sort_score)
                .unwrap_or(Ordering::Equal),
        };
        ord.then_with(|| a.label.cmp(&b.label))
    });
}

fn collect_matrix(repo: &Path, parallel: bool, sort: SortBy) -> Vec<MatrixRow> {
    let job_results: Vec<(String, PathBuf, Result<CrepusLiteConfig, String>)> = STARTERS
        .iter()
        .filter_map(|st| {
            let p = repo.join(format!(
                "examples/benchmark/crepus-guest-starters/crepus-lite-bench-{st}.toml"
            ));
            if p.is_file() {
                Some(((*st).to_string(), p))
            } else {
                None
            }
        })
        .map(|(label, path)| {
            let cfg = CrepusLiteConfig::load_from_path(&path);
            (label, path, cfg)
        })
        .collect();

    let shared: Option<BenchMatrixShared> = {
        let ok: Vec<&CrepusLiteConfig> = job_results
            .iter()
            .filter_map(|(_, _, c)| c.as_ref().ok())
            .collect();
        if ok.len() == job_results.len() {
            bench_matrix_shared_for_configs(repo, &ok)
        } else {
            None
        }
    };

    let mut rows: Vec<MatrixRow> = if parallel {
        use rayon::prelude::*;
        job_results
            .par_iter()
            .map(|(label, _path, cfg_res)| {
                MatrixRow::from_loaded(repo, label, cfg_res, shared.as_ref())
            })
            .collect()
    } else {
        job_results
            .iter()
            .map(|(label, _path, cfg_res)| {
                MatrixRow::from_loaded(repo, label, cfg_res, shared.as_ref())
            })
            .collect()
    };

    sort_rows(&mut rows, sort);
    rows
}

fn print_matrix_stdout(rows: &[MatrixRow], plain: bool, sort: SortBy) {
    if plain {
        for r in rows {
            let payload = r
                .payload
                .clone()
                .unwrap_or_default()
                .chars()
                .filter(|&c| c != '\n' && c != '\r')
                .collect::<String>();
            println!("V8_BENCH\t{}\t{}", r.label, payload);
        }
        return;
    }

    println!("{:-<72}", format!(" V8 guest matrix (sorted {:?}) ", sort));
    println!("{:<14} {:>12} {:>12} status", "guestStarter", "ms", "score");
    for r in rows {
        let (ms_s, sc_s, st) = if let Some(ref e) = r.err {
            ("—".into(), "—".into(), format!("ERR {e}"))
        } else {
            (
                format!("{:.3}", r.sort_ms),
                format!("{}", r.sort_score as i64),
                "ok".into(),
            )
        };
        println!("{:<14} {:>12} {:>12} {}", r.label, ms_s, sc_s, st);
    }
}

fn run_compare_sh(repo: &Path, mode: &str) -> io::Result<String> {
    let script = repo.join("examples/benchmark/compare.sh");
    if !script.is_file() {
        return Ok(format!("missing {}", script.display()));
    }
    let out = Command::new("bash")
        .arg(script)
        .arg(mode)
        .current_dir(repo)
        .output()?;
    let mut s = String::new();
    s.push_str(&String::from_utf8_lossy(&out.stdout));
    if !out.stderr.is_empty() {
        s.push_str(&String::from_utf8_lossy(&out.stderr));
    }
    if !out.status.success() {
        s.push_str(&format!("\n(exit {})", out.status));
    }
    Ok(s)
}

enum Screen {
    Menu {
        idx: usize,
    },
    Matrix {
        rows: Vec<MatrixRow>,
        sort: SortBy,
        parallel: bool,
    },
    Log {
        title: String,
        body: String,
    },
}

fn run_tui(repo: PathBuf) -> io::Result<()> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(out))?;

    let menu_items = [
        "V8 matrix (parallel, headless)",
        "V8 matrix (serial)",
        "Sort table: by ms ↑ (time)",
        "Sort table: by score ↑",
        "Sort table: by score ↓ (best first)",
        "Run compare.sh disk",
        "Run compare.sh memory",
        "Quit",
    ];

    let mut screen = Screen::Menu { idx: 0 };
    let mut sort_mode = SortBy::Ms;

    loop {
        terminal.draw(|f| {
            let area = f.area();
            match &screen {
                Screen::Menu { idx } => {
                    let list_items: Vec<ListItem> = menu_items
                        .iter()
                        .enumerate()
                        .map(|(i, s)| {
                            let style = if i == *idx {
                                Style::default().black().on_cyan()
                            } else {
                                Style::default()
                            };
                            ListItem::new(Line::from((*s).to_string()).style(style))
                        })
                        .collect();
                    let list = List::new(list_items).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" crepus-bench — ↑/↓ enter — q quit "),
                    );
                    let hint = Paragraph::new(Line::from(format!(
                        "repo: {} │ sort preset: {:?}",
                        repo.display(),
                        sort_mode,
                    )))
                    .block(Block::default().borders(Borders::ALL).title(" status "));
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Min(8), Constraint::Length(3)])
                        .split(area);
                    f.render_widget(list, chunks[0]);
                    f.render_widget(hint, chunks[1]);
                }
                Screen::Matrix {
                    rows,
                    sort,
                    parallel,
                } => {
                    let header = Row::new(vec!["guestStarter", "ms", "score", "note"]);
                    let table_rows: Vec<Row> = rows
                        .iter()
                        .map(|r| {
                            if let Some(e) = &r.err {
                                Row::new(vec![r.label.clone(), "—".into(), "—".into(), e.clone()])
                            } else {
                                Row::new(vec![
                                    r.label.clone(),
                                    format!("{:.3}", r.sort_ms),
                                    format!("{}", r.sort_score as i64),
                                    "ok".into(),
                                ])
                            }
                        })
                        .collect();
                    let t = Table::new(
                        table_rows,
                        [
                            Constraint::Length(14),
                            Constraint::Length(12),
                            Constraint::Length(12),
                            Constraint::Min(20),
                        ],
                    )
                    .header(header)
                    .block(Block::default().borders(Borders::ALL).title(format!(
                        " V8 matrix parallel={parallel} sort={sort:?} — b menu — q quit "
                    )));
                    f.render_widget(t, area);
                }
                Screen::Log { title, body } => {
                    let p = Paragraph::new(body.as_str())
                        .block(Block::default().borders(Borders::ALL).title(title.as_str()));
                    f.render_widget(p, area);
                }
            }
        })?;

        if !event::poll(Duration::from_millis(200))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match &mut screen {
            Screen::Menu { idx } => match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Up => *idx = idx.saturating_sub(1),
                KeyCode::Down => *idx = (*idx + 1).min(menu_items.len() - 1),
                KeyCode::Enter => match *idx {
                    0 => {
                        let rows = collect_matrix(&repo, true, sort_mode);
                        screen = Screen::Matrix {
                            rows,
                            sort: sort_mode,
                            parallel: true,
                        };
                    }
                    1 => {
                        let rows = collect_matrix(&repo, false, sort_mode);
                        screen = Screen::Matrix {
                            rows,
                            sort: sort_mode,
                            parallel: false,
                        };
                    }
                    2 => {
                        sort_mode = SortBy::Ms;
                    }
                    3 => {
                        sort_mode = SortBy::ScoreAsc;
                    }
                    4 => {
                        sort_mode = SortBy::ScoreDesc;
                    }
                    5 => {
                        let body = run_compare_sh(&repo, "disk").unwrap_or_else(|e| e.to_string());
                        screen = Screen::Log {
                            title: "compare.sh disk".into(),
                            body,
                        };
                    }
                    6 => {
                        let body =
                            run_compare_sh(&repo, "memory").unwrap_or_else(|e| e.to_string());
                        screen = Screen::Log {
                            title: "compare.sh memory".into(),
                            body,
                        };
                    }
                    7 => break,
                    _ => {}
                },
                KeyCode::Esc => break,
                _ => {}
            },
            Screen::Matrix {
                rows,
                sort,
                parallel,
            } => match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('b') => {
                    screen = Screen::Menu { idx: 0 };
                }
                KeyCode::Char('r') => {
                    // re-run with current sort / parallel
                    let new_rows = collect_matrix(&repo, *parallel, *sort);
                    *rows = new_rows;
                }
                _ => {}
            },
            Screen::Log { .. } => match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('b') | KeyCode::Esc => {
                    screen = Screen::Menu { idx: 0 };
                }
                _ => {}
            },
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let repo = fs::canonicalize(&cli.repo).unwrap_or(cli.repo);

    match cli.command {
        None | Some(Commands::Tui) => run_tui(repo),
        Some(Commands::Matrix {
            sort,
            parallel,
            no_parallel,
            plain,
        }) => {
            let sort = sort.unwrap_or_default();
            let parallel = parallel && !no_parallel;
            let rows = collect_matrix(&repo, parallel, sort);
            print_matrix_stdout(&rows, plain, sort);
            Ok(())
        }
    }
}
