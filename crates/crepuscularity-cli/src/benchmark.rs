//! `crepus benchmark` — declarative build-time comparisons (see `examples/benchmarks/benchmark.toml`).

use console::style;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use crate::ui;

#[derive(Debug, Deserialize)]
struct BenchmarkFile {
    version: u32,
    #[serde(default)]
    settings: Settings,
    #[serde(default)]
    targets: Vec<BenchTarget>,
}

#[derive(Debug, Default, Deserialize)]
struct Settings {
    work_root: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BenchTarget {
    suite: String,
    id: String,
    label: String,
    #[serde(default)]
    optional: bool,
    builtin: Option<String>,
    site: Option<String>,
    #[serde(default)]
    requires: Vec<String>,
    #[serde(default)]
    steps: Vec<BenchStep>,
}

#[derive(Debug, Deserialize)]
struct BenchStep {
    name: String,
    #[serde(default)]
    cwd: Option<String>,
    shell: String,
}

#[derive(Serialize)]
struct JsonReport {
    summary: JsonRunSummary,
    suites: Vec<JsonSuite>,
}

#[derive(Serialize)]
struct JsonRunSummary {
    /// Sum of `total_ms` for targets that finished successfully (`skipped: false`).
    total_wall_ms_completed: u128,
    completed_target_count: usize,
    skipped_target_count: usize,
    /// Completed targets sorted slowest first (compare-across stacks).
    by_wall_time: Vec<JsonInsightRow>,
}

#[derive(Serialize)]
struct JsonInsightRow {
    suite: String,
    id: String,
    label: String,
    wall_ms: u128,
    step_sum_ms: u128,
    /// `wall_ms − step_sum_ms` (setup, shell, logging, or clock skew).
    overhead_ms: u128,
    /// `100 * wall_ms / total_wall_ms_completed` (0 if alone or no completed runs).
    share_of_completed_wall_pct: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    peak_rss_kb: Option<u64>,
}

#[derive(Serialize)]
struct JsonSuite {
    id: String,
    targets: Vec<JsonTargetResult>,
}

#[derive(Serialize)]
struct JsonTargetResult {
    id: String,
    label: String,
    optional: bool,
    skipped: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    steps: Vec<JsonStepResult>,
    /// Wall time for this target (harness around all steps).
    total_ms: u128,
    /// Sum of per-step `duration_ms` (serial work as measured).
    step_sum_ms: u128,
    /// `total_ms.saturating_sub(step_sum_ms)`.
    overhead_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    peak_rss_kb: Option<u64>,
}

struct StepRunCtx<'a> {
    target_dir: &'a Path,
    vars: &'a HashMap<String, String>,
    dry_run: bool,
    json_out: bool,
    verbose: bool,
    measure_memory: bool,
    step_total: usize,
}

fn step_aggregates(steps: &[JsonStepResult], wall_ms: u128) -> (u128, u128, Option<u64>) {
    let sum: u128 = steps.iter().map(|s| s.duration_ms).sum();
    let overhead = wall_ms.saturating_sub(sum);
    let peak = steps.iter().filter_map(|s| s.max_rss_kb).max();
    (sum, overhead, peak)
}

fn json_target_result(
    id: String,
    label: String,
    optional: bool,
    skipped: bool,
    error: Option<String>,
    steps: Vec<JsonStepResult>,
    total_ms: u128,
) -> JsonTargetResult {
    let (step_sum_ms, overhead_ms, peak_rss_kb) = step_aggregates(&steps, total_ms);
    JsonTargetResult {
        id,
        label,
        optional,
        skipped,
        error,
        steps,
        total_ms,
        step_sum_ms,
        overhead_ms,
        peak_rss_kb,
    }
}

fn compute_run_summary(suites: &[JsonSuite]) -> JsonRunSummary {
    let mut skipped_target_count = 0usize;
    let mut rows: Vec<JsonInsightRow> = Vec::new();
    for suite in suites {
        for t in &suite.targets {
            if t.skipped {
                skipped_target_count += 1;
                continue;
            }
            rows.push(JsonInsightRow {
                suite: suite.id.clone(),
                id: t.id.clone(),
                label: t.label.clone(),
                wall_ms: t.total_ms,
                step_sum_ms: t.step_sum_ms,
                overhead_ms: t.overhead_ms,
                share_of_completed_wall_pct: 0.0,
                peak_rss_kb: t.peak_rss_kb,
            });
        }
    }
    rows.sort_by(|a, b| b.wall_ms.cmp(&a.wall_ms));
    let total_wall_ms_completed: u128 = rows.iter().map(|r| r.wall_ms).sum();
    let completed_target_count = rows.len();
    for r in &mut rows {
        r.share_of_completed_wall_pct = if total_wall_ms_completed > 0 {
            (r.wall_ms as f64) * 100.0 / (total_wall_ms_completed as f64)
        } else {
            0.0
        };
    }
    JsonRunSummary {
        total_wall_ms_completed,
        completed_target_count,
        skipped_target_count,
        by_wall_time: rows,
    }
}

fn print_human_insights(summary: &JsonRunSummary, dry_run: bool) {
    eprintln!();
    if dry_run {
        eprintln!(
            "  {}",
            style("insight: dry-run — re-run without --dry-run for wall times and RSS.").dim()
        );
        return;
    }
    if summary.completed_target_count == 0 {
        eprintln!(
            "  {}",
            style("insight: no completed targets — check skips or failures above.").dim()
        );
        return;
    }
    eprintln!(
        "{}",
        style("insight — where time went (completed targets, slowest first)")
            .cyan()
            .bold()
    );
    eprintln!(
        "  {}",
        style("% of sum(wall) · suite · wall / Σ steps / overhead — overhead is shell/setup outside step clocks")
            .dim()
    );
    let show = summary.by_wall_time.len().min(25);
    for row in summary.by_wall_time.iter().take(show) {
        let wall = fmt_ms(row.wall_ms);
        let steps_lbl = fmt_ms(row.step_sum_ms);
        let oh = fmt_ms(row.overhead_ms);
        let rss = row.peak_rss_kb.map_or(String::new(), |k| {
            format!("  peak {:.1} MiB", k as f64 / 1024.0)
        });
        eprintln!(
            "  {:>5.1}%  {:<8}  wall {:>8}  Σ {:>8}  OH {:>8}{}",
            row.share_of_completed_wall_pct,
            style(&row.suite).dim(),
            style(wall).white(),
            style(steps_lbl).cyan(),
            style(oh).yellow(),
            style(rss).dim(),
        );
        eprintln!("          {}", style(&row.label).white().bold());
    }
    if summary.by_wall_time.len() > 25 {
        eprintln!(
            "  {}",
            style(format!(
                "… {} more rows in --json (summary.by_wall_time)",
                summary.by_wall_time.len() - 25
            ))
            .dim()
        );
    }
    eprintln!(
        "  {}",
        style("speed up Crepus: slimmer WASM graph, `cargo build --timings`, reuse CARGO_TARGET_DIR, or LTO settings in fixture crates.")
            .dim()
    );
    eprintln!(
        "  {}",
        style("compare stacks: match cold/warm cache policy (--clean) so % shares are meaningful.")
            .dim()
    );
}

#[derive(Serialize, Clone)]
struct JsonStepResult {
    name: String,
    duration_ms: u128,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_rss_kb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stderr_tail: Option<String>,
}

pub fn run(args: &[String]) {
    let args = normalize_benchmark_invocation_args(args);

    let mut config_path: Option<PathBuf> = None;
    let mut suite_filter: Option<String> = None;
    let mut only: Option<Vec<String>> = None;
    let mut json_out = false;
    let mut dry_run = false;
    let mut clean = false;
    let mut repo_override: Option<PathBuf> = None;
    let mut work_override: Option<PathBuf> = None;
    // None: default verbose for human runs; Some: explicit --verbose / --quiet.
    let mut verbose_override: Option<bool> = None;
    let mut measure_memory = true;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--config" | "-c" => {
                i += 1;
                config_path = args.get(i).map(PathBuf::from);
            }
            "--suite" | "-s" => {
                i += 1;
                suite_filter = args.get(i).cloned();
            }
            "--only" => {
                i += 1;
                if let Some(s) = args.get(i) {
                    only = Some(
                        s.split(',')
                            .map(|x| x.trim().to_string())
                            .filter(|x| !x.is_empty())
                            .collect(),
                    );
                }
            }
            "--json" => json_out = true,
            "--dry-run" | "-n" => dry_run = true,
            "--verbose" | "-v" => verbose_override = Some(true),
            "--quiet" | "-q" => verbose_override = Some(false),
            "--memory" => measure_memory = true,
            "--no-memory" => measure_memory = false,
            "--clean" => clean = true,
            "--repo" => {
                i += 1;
                repo_override = args.get(i).map(PathBuf::from);
            }
            "--work-root" => {
                i += 1;
                work_override = args.get(i).map(PathBuf::from);
            }
            "--help" | "-h" => {
                print_benchmark_usage();
                return;
            }
            _ => {}
        }
        i += 1;
    }

    let verbose = if json_out {
        false
    } else {
        verbose_override.unwrap_or(true)
    };

    let cfg_path = resolve_config_path(config_path);
    let raw = std::fs::read_to_string(&cfg_path).unwrap_or_else(|e| {
        ui::error(&format!("cannot read {}: {e}", cfg_path.display()));
    });

    let file: BenchmarkFile = toml::from_str(&raw).unwrap_or_else(|e| {
        ui::error(&format!("benchmark.toml parse error: {e}"));
    });

    if file.version != 1 {
        ui::error(&format!(
            "unsupported benchmark.toml version {}",
            file.version
        ));
    }

    let repo = repo_override.unwrap_or_else(|| find_repo_root(&cfg_path));
    let work_base = work_override.unwrap_or_else(|| {
        let rel = file
            .settings
            .work_root
            .as_deref()
            .unwrap_or("examples/benchmarks/.work");
        resolve_path(&repo, rel)
    });

    if clean && !dry_run {
        let _ = std::fs::remove_dir_all(&work_base);
    }

    if !dry_run {
        std::fs::create_dir_all(&work_base).unwrap_or_else(|e| {
            ui::error(&format!("could not create {}: {e}", work_base.display()));
        });
    }

    let mut by_suite: HashMap<String, Vec<&BenchTarget>> = HashMap::new();
    for t in &file.targets {
        if let Some(ref f) = suite_filter {
            if f != "all" && t.suite != *f {
                continue;
            }
        }
        if let Some(ref ids) = only {
            if !ids.contains(&t.id) {
                continue;
            }
        }
        by_suite.entry(t.suite.clone()).or_default().push(t);
    }

    let mut json_suites: Vec<JsonSuite> = Vec::new();
    let mut suite_keys: Vec<String> = by_suite.keys().cloned().collect();
    suite_keys.sort();

    let total_targets: usize = by_suite.values().map(|v| v.len()).sum();
    if verbose && !json_out {
        print_run_header(
            &cfg_path,
            &repo,
            &work_base,
            clean,
            dry_run,
            measure_memory,
            total_targets,
            suite_keys.len(),
        );
    }

    for suite_id in suite_keys {
        let targets = by_suite.get(&suite_id).unwrap();
        if !json_out {
            eprintln!();
            eprintln!(
                "{} {}",
                style("suite").cyan().bold(),
                style(&suite_id).white().bold()
            );
        }

        let mut json_targets: Vec<JsonTargetResult> = Vec::new();

        for t in targets {
            let target_dir = work_base.join(&t.suite).join(&t.id);
            if !dry_run {
                std::fs::create_dir_all(&target_dir).unwrap_or_else(|e| {
                    ui::error(&format!("could not create {}: {e}", target_dir.display()));
                });
            }

            let step_total = planned_step_count(t);

            let mut vars = HashMap::new();
            vars.insert("REPO".into(), repo.display().to_string());
            vars.insert("WORK".into(), target_dir.display().to_string());
            vars.insert("TARGET".into(), t.id.clone());

            if !check_requires(&t.requires) {
                let msg = "missing prerequisite command";
                if t.optional {
                    if !json_out {
                        ui::warning(&format!("{} — skipped ({msg})", style(&t.label).yellow()));
                    }
                    json_targets.push(json_target_result(
                        t.id.clone(),
                        t.label.clone(),
                        t.optional,
                        true,
                        Some(msg.into()),
                        vec![],
                        0,
                    ));
                    continue;
                } else {
                    ui::error(msg);
                }
            }

            if !json_out {
                eprintln!("  {} {}", ui::arrow(), style(&t.label).cyan().bold());
                if verbose {
                    eprintln!(
                        "      {} id={}  work_dir={}",
                        style("target").dim(),
                        style(&t.id).dim(),
                        style(target_dir.display().to_string()).dim()
                    );
                }
            }

            let t0 = Instant::now();
            let ctx = StepRunCtx {
                target_dir: &target_dir,
                vars: &vars,
                dry_run,
                json_out,
                verbose,
                measure_memory,
                step_total,
            };
            let (ok, err, step_results) = if let Some(ref b) = t.builtin {
                run_builtin(
                    b,
                    t,
                    &repo,
                    &target_dir,
                    dry_run,
                    json_out,
                    verbose,
                    measure_memory,
                )
            } else {
                run_steps(&t.steps, ctx)
            };
            let total = t0.elapsed().as_millis();

            if !ok {
                let msg = err.unwrap_or_else(|| "failed".into());
                if t.optional {
                    if !json_out && !dry_run {
                        print_target_wall_footer(total, &step_results);
                    }
                    if !json_out {
                        ui::warning(&format!("{} — {msg}", style(&t.label).yellow()));
                    }
                    json_targets.push(json_target_result(
                        t.id.clone(),
                        t.label.clone(),
                        t.optional,
                        true,
                        Some(msg),
                        step_results,
                        total,
                    ));
                } else {
                    if !json_out && !dry_run {
                        print_target_wall_footer(total, &step_results);
                    }
                    if !json_out {
                        eprintln!("    {} {}", ui::err(), style(&t.label).red());
                    }
                    ui::error(&msg);
                }
                continue;
            }

            if !json_out && !dry_run {
                print_target_wall_footer(total, &step_results);
            }

            json_targets.push(json_target_result(
                t.id.clone(),
                t.label.clone(),
                t.optional,
                false,
                None,
                step_results,
                total,
            ));
        }

        json_suites.push(JsonSuite {
            id: suite_id,
            targets: json_targets,
        });
    }

    let summary = compute_run_summary(&json_suites);
    if json_out {
        let report = JsonReport {
            summary,
            suites: json_suites,
        };
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        print_human_insights(&summary, dry_run);
        eprintln!();
        ui::success("benchmark finished");
    }
}

/// Drop leading `all` / `run` so `crepus benchmark all` is an explicit full run.
fn normalize_benchmark_invocation_args(args: &[String]) -> Vec<String> {
    match args.first().map(|s| s.as_str()) {
        Some("all" | "run") => args[1..].to_vec(),
        _ => args.to_vec(),
    }
}

#[allow(clippy::too_many_arguments)]
fn print_run_header(
    cfg_path: &Path,
    repo: &Path,
    work_base: &Path,
    clean: bool,
    dry_run: bool,
    measure_memory: bool,
    total_targets: usize,
    suite_count: usize,
) {
    eprintln!();
    eprintln!("{}", style("crepus benchmark").cyan().bold());
    eprintln!("  {} {}", style("config").dim(), cfg_path.display());
    eprintln!("  {} {}", style("repo").dim(), repo.display());
    eprintln!("  {} {}", style("work").dim(), work_base.display());
    if clean {
        eprintln!("  {}", style("clean: removed previous work dir").yellow());
    }
    if dry_run {
        eprintln!("  {}", style("dry-run: no commands executed").yellow());
    }
    eprintln!(
        "  {} {} targets in {} suites",
        style("plan").dim(),
        style(total_targets.to_string()).white(),
        suite_count
    );
    eprintln!(
        "  {}",
        style("verbose: subprocess stdin/stdout/stderr inherited (live cargo/npm logs) — --quiet buffers output").dim()
    );
    if measure_memory {
        eprintln!(
            "  {}",
            style("memory: peak RSS per step via ps(1) sampled while child runs (Unix) — --no-memory to skip").dim()
        );
    } else {
        eprintln!("  {}", style("memory: off (--no-memory)").dim());
    }
    eprintln!();
}

fn elide_shell_script(s: &str, max: usize) -> String {
    let t = s.trim();
    if t.len() <= max {
        t.to_string()
    } else {
        format!("{}…", &t[..max])
    }
}

fn print_benchmark_usage() {
    eprintln!("{} benchmark", style("crepus").cyan().bold());
    eprintln!();
    eprintln!("{}", style("USAGE").dim());
    eprintln!(
        "  crepus benchmark [all|run] [--config PATH] [--suite web|desktop|webext|all]\n\
                   [--only id,id] [--work-root DIR] [--repo DIR] [--clean] [--dry-run]\n\
                   [--json] [--verbose|-v] [--quiet|-q] [--memory] [--no-memory]"
    );
    eprintln!();
    eprintln!(
        "{}",
        style("Default: verbose (stream subprocess output). --json or --quiet reduces noise.")
            .dim()
    );
    eprintln!();
    eprintln!(
        "{}",
        style("With no --suite / --only, every target in benchmark.toml runs.").dim()
    );
    eprintln!();
    eprintln!("{}", style("EXAMPLES").dim());
    eprintln!("  crepus benchmark all");
    eprintln!("  crepus benchmark --config examples/benchmarks/benchmark.toml");
    eprintln!("  crepus benchmark --only crepus-web,crepus-webext --json > bench.json");
    eprintln!("  examples/benchmarks/run-all.sh --json   # same from repo root via shell");
    eprintln!(
        "{}",
        style("JSON includes summary.by_wall_time (ranked timings + % of total wall).").dim()
    );
}

fn planned_step_count(t: &BenchTarget) -> usize {
    if let Some(ref b) = t.builtin {
        return match b.as_str() {
            "crepus-webext" => 2,
            "crepus-web" | "crepus-desktop" => 1,
            _ => 1,
        };
    }
    t.steps.len().max(1)
}

fn fmt_ms(ms: u128) -> String {
    ui::format_duration(Duration::from_millis(u64::try_from(ms).unwrap_or(u64::MAX)))
}

/// One line per step as it completes (or planned in `--dry-run`).
/// `timed`: `(ms, ok, peak_rss_kb)` — RSS from sampled `ps` while child runs (Unix).
fn print_step_line(
    json_out: bool,
    one_based: usize,
    total: usize,
    name: &str,
    timed: Option<(u128, bool, Option<u64>)>,
) {
    if json_out || total == 0 {
        return;
    }
    let idx = style(format!("[{one_based}/{total}]")).dim();
    let nm = style(name);
    match timed {
        None => {
            eprintln!("      {idx}  {nm}  {}", style("(planned)").dim());
        }
        Some((ms, ok, rss)) => {
            let tim = style(fmt_ms(ms)).cyan();
            let st = if ok {
                style("ok").green()
            } else {
                style("FAIL").red().bold()
            };
            if let Some(kb) = rss {
                let mib = kb as f64 / 1024.0;
                eprintln!(
                    "      {idx}  {nm}  {tim}  {st}  {}",
                    style(format!("peak {mib:.1} MiB RSS")).dim()
                );
            } else {
                eprintln!("      {idx}  {nm}  {tim}  {st}");
            }
        }
    }
}

fn print_steps_share_footer(steps: &[JsonStepResult]) {
    if steps.len() < 2 {
        return;
    }
    let sum: u128 = steps.iter().map(|s| s.duration_ms).sum();
    if sum == 0 {
        return;
    }
    eprintln!(
        "      {}",
        style("share of summed step times (where to optimize)").dim()
    );
    for s in steps {
        let pct = (s.duration_ms as f64) * 100.0 / (sum as f64);
        eprintln!("         {:>5.1}%  {}", pct, style(&s.name).dim());
    }
}

fn print_target_wall_footer(wall_ms: u128, steps: &[JsonStepResult]) {
    if steps.is_empty() {
        return;
    }
    let sum: u128 = steps.iter().map(|s| s.duration_ms).sum();
    let wall = fmt_ms(wall_ms);
    let steps_lbl = fmt_ms(sum);
    let overhead_ms = wall_ms.saturating_sub(sum);
    let oh = fmt_ms(overhead_ms);
    eprintln!(
        "      {} wall {}  ·  step times sum {}  ·  overhead {}",
        style("--").dim(),
        style(wall).white().bold(),
        style(steps_lbl).dim(),
        style(oh).yellow(),
    );
    if let Some(peak_kb) = steps.iter().filter_map(|s| s.max_rss_kb).max() {
        eprintln!(
            "      {} peak RSS (max over steps) {:.1} MiB",
            style("--").dim(),
            peak_kb as f64 / 1024.0
        );
    }
    print_steps_share_footer(steps);
}

fn resolve_config_path(explicit: Option<PathBuf>) -> PathBuf {
    if let Some(p) = explicit {
        return p;
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for candidate in [
        cwd.join("benchmark.toml"),
        cwd.join("examples/benchmarks/benchmark.toml"),
    ] {
        if candidate.exists() {
            return candidate;
        }
    }
    ui::error("no benchmark.toml found; pass --config path");
}

fn find_repo_root(config_path: &Path) -> PathBuf {
    if let Ok(root) = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        if root.status.success() {
            let s = String::from_utf8_lossy(&root.stdout).trim().to_string();
            if !s.is_empty() {
                return PathBuf::from(s);
            }
        }
    }
    config_path
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn resolve_path(repo: &Path, rel: &str) -> PathBuf {
    let p = Path::new(rel);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        repo.join(rel)
    }
}

fn check_requires(requires: &[String]) -> bool {
    if requires.is_empty() {
        return true;
    }
    for r in requires {
        let (ok, _, _) = run_shell(Path::new("."), r, false, false);
        if !ok {
            return false;
        }
    }
    true
}

#[cfg(unix)]
fn sample_rss_kb(pid: u32) -> Option<u64> {
    let out = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "rss="])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    s.split_whitespace().next()?.parse().ok()
}

#[cfg(not(unix))]
fn sample_rss_kb(_pid: u32) -> Option<u64> {
    None
}

fn wait_with_max_rss(
    child: &mut std::process::Child,
    measure_memory: bool,
) -> std::io::Result<(std::process::ExitStatus, Option<u64>)> {
    if !measure_memory {
        let s = child.wait()?;
        return Ok((s, None));
    }
    #[cfg(unix)]
    {
        let pid = child.id();
        let mut max_kb = 0_u64;
        loop {
            match child.try_wait()? {
                Some(status) => {
                    return Ok((
                        status,
                        if max_kb > 0 {
                            Some(max_kb)
                        } else {
                            sample_rss_kb(pid)
                        },
                    ));
                }
                None => {
                    if let Some(kb) = sample_rss_kb(pid) {
                        max_kb = max_kb.max(kb);
                    }
                    thread::sleep(Duration::from_millis(80));
                }
            }
        }
    }
    #[cfg(not(unix))]
    {
        let s = child.wait()?;
        Ok((s, None))
    }
}

/// Run a configured [`Command`]: capture I/O unless `inherit_io`, optionally sample child RSS (Unix).
fn exec_tracked(
    cmd: &mut Command,
    inherit_io: bool,
    measure_memory: bool,
) -> Result<(bool, String, Option<u64>), String> {
    use std::process::Stdio;
    if inherit_io {
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
    }
    if !measure_memory {
        if inherit_io {
            return match cmd.status() {
                Ok(s) if s.success() => Ok((true, String::new(), None)),
                Ok(s) => Ok((false, format!("exited with {s}"), None)),
                Err(e) => Err(e.to_string()),
            };
        }
        let o = cmd.output().map_err(|e| e.to_string())?;
        let mut err = String::from_utf8_lossy(&o.stderr).to_string();
        if !o.stdout.is_empty() {
            if !err.is_empty() {
                err.push('\n');
            }
            err.push_str(&String::from_utf8_lossy(&o.stdout));
        }
        return Ok((o.status.success(), err, None));
    }

    if inherit_io {
        let mut child = cmd.spawn().map_err(|e| e.to_string())?;
        let (status, rss) = wait_with_max_rss(&mut child, true).map_err(|e| e.to_string())?;
        let ok = status.success();
        let msg = if ok {
            String::new()
        } else {
            format!("exited with {status}")
        };
        return Ok((ok, msg, rss));
    }

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| e.to_string())?;
    let stdout_pipe = child.stdout.take().unwrap();
    let stderr_pipe = child.stderr.take().unwrap();
    let t_out = thread::spawn(move || {
        let mut s = String::new();
        let _ = std::io::BufReader::new(stdout_pipe).read_to_string(&mut s);
        s
    });
    let t_err = thread::spawn(move || {
        let mut s = String::new();
        let _ = std::io::BufReader::new(stderr_pipe).read_to_string(&mut s);
        s
    });
    let (status, rss) = wait_with_max_rss(&mut child, true).map_err(|e| e.to_string())?;
    let stdout = t_out.join().unwrap_or_default();
    let mut err = t_err.join().unwrap_or_default();
    if !stdout.is_empty() {
        if !err.is_empty() {
            err.push('\n');
        }
        err.push_str(&stdout);
    }
    Ok((status.success(), err, rss))
}

fn run_shell(
    workdir: &Path,
    script: &str,
    inherit_io: bool,
    measure_memory: bool,
) -> (bool, String, Option<u64>) {
    #[cfg(unix)]
    {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(script).current_dir(workdir);
        match exec_tracked(&mut cmd, inherit_io, measure_memory) {
            Ok(x) => x,
            Err(e) => (false, e, None),
        }
    }
    #[cfg(not(unix))]
    {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", script]).current_dir(workdir);
        let _measure_memory = measure_memory; // RSS sampling not implemented on Windows
        if inherit_io {
            let st = cmd.status();
            return match st {
                Ok(s) if s.success() => (true, String::new(), None),
                Ok(s) => (false, format!("`cmd /C` exited with {s}"), None),
                Err(e) => (false, e.to_string(), None),
            };
        }
        let o = cmd.output();
        match o {
            Ok(out) => {
                let mut err = String::from_utf8_lossy(&out.stderr).to_string();
                if !out.stdout.is_empty() {
                    if !err.is_empty() {
                        err.push('\n');
                    }
                    err.push_str(&String::from_utf8_lossy(&out.stdout));
                }
                (out.status.success(), err, None)
            }
            Err(e) => (false, e.to_string(), None),
        }
    }
}

fn expand_vars(s: &str, vars: &HashMap<String, String>) -> String {
    let mut out = s.to_string();
    for (k, v) in vars {
        out = out.replace(&format!("${k}"), v);
        out = out.replace(&format!("${{{k}}}"), v);
    }
    out
}

fn run_steps(
    steps: &[BenchStep],
    ctx: StepRunCtx<'_>,
) -> (bool, Option<String>, Vec<JsonStepResult>) {
    let mut results = Vec::new();
    let n = ctx.step_total;
    let inherit_io = ctx.verbose && !ctx.json_out && !ctx.dry_run;
    for (i, step) in steps.iter().enumerate() {
        let one = i + 1;
        let cwd = step
            .cwd
            .as_ref()
            .map(|c| ctx.target_dir.join(c))
            .unwrap_or_else(|| ctx.target_dir.to_path_buf());
        let script = expand_vars(&step.shell, ctx.vars);

        if ctx.dry_run {
            print_step_line(ctx.json_out, one, n, &step.name, None);
            if !ctx.json_out {
                eprintln!("            {}", style(&script).dim());
            }
            results.push(JsonStepResult {
                name: step.name.clone(),
                duration_ms: 0,
                ok: true,
                max_rss_kb: None,
                stderr_tail: None,
            });
            continue;
        }

        std::fs::create_dir_all(&cwd).ok();

        if ctx.verbose && !ctx.json_out {
            eprintln!(
                "      {} step {}/{}: {}",
                style("···").dim(),
                one,
                n,
                style(&step.name).yellow()
            );
            eprintln!("          {} {}", style("cwd").dim(), cwd.display());
            eprintln!(
                "          {} {}",
                style("sh -c").dim(),
                style(elide_shell_script(&script, 400)).dim()
            );
            eprintln!("          {}", style("— running —").dim());
        }

        let t0 = Instant::now();
        let (ok, stderr, max_rss) = run_shell(&cwd, &script, inherit_io, ctx.measure_memory);
        let ms = t0.elapsed().as_millis();

        let tail = stderr_tail_opt(&stderr);

        results.push(JsonStepResult {
            name: step.name.clone(),
            duration_ms: ms,
            ok,
            max_rss_kb: max_rss,
            stderr_tail: tail.clone(),
        });

        print_step_line(ctx.json_out, one, n, &step.name, Some((ms, ok, max_rss)));

        if !ok {
            let detail = if inherit_io && stderr.is_empty() {
                "(output was streamed; see above)".into()
            } else {
                tail.unwrap_or_else(|| "(no output)".into())
            };
            return (
                false,
                Some(format!(
                    "step '{}' failed ({}ms): {}",
                    step.name, ms, detail
                )),
                results,
            );
        }
    }
    (true, None, results)
}

#[allow(clippy::too_many_arguments)]
fn run_builtin(
    builtin: &str,
    target: &BenchTarget,
    repo: &Path,
    work_dir: &Path,
    dry_run: bool,
    json_out: bool,
    verbose: bool,
    measure_memory: bool,
) -> (bool, Option<String>, Vec<JsonStepResult>) {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("crepus"));
    let mut results = Vec::new();
    let inherit_io = verbose && !json_out && !dry_run;

    match builtin {
        "crepus-web" => {
            let site = target
                .site
                .as_deref()
                .unwrap_or("examples/benchmarks/crepus-web");
            let site_path = resolve_path(repo, site);
            let out_dir = work_dir.join("out");
            if dry_run {
                print_step_line(
                    json_out,
                    1,
                    1,
                    "crepus web build (--site → --out-dir)",
                    None,
                );
                if !json_out {
                    eprintln!(
                        "            {}",
                        style(format!(
                            "crepus web build --site {} --out-dir {}",
                            site_path.display(),
                            out_dir.display()
                        ))
                        .dim()
                    );
                }
                return (
                    true,
                    None,
                    vec![JsonStepResult {
                        name: "crepus web build".into(),
                        duration_ms: 0,
                        ok: true,
                        max_rss_kb: None,
                        stderr_tail: None,
                    }],
                );
            }
            if verbose && !json_out {
                eprintln!(
                    "      {} {}",
                    style("···").dim(),
                    style("builtin: crepus web build").yellow()
                );
                eprintln!("          exe {}", exe.display());
                eprintln!("          --site {}", site_path.display());
                eprintln!("          --out-dir {}", out_dir.display());
                eprintln!("          {}", style("— running —").dim());
            }
            let t0 = Instant::now();
            let mut cmd = Command::new(&exe);
            cmd.args(["web", "build", "--site"])
                .arg(&site_path)
                .arg("--out-dir")
                .arg(&out_dir);
            let built = exec_tracked(&mut cmd, inherit_io, measure_memory);
            let ms = t0.elapsed().as_millis();
            let (ok, combined, rss) = match built {
                Ok(x) => x,
                Err(e) => (false, e, None),
            };
            let err = if ok {
                None
            } else if combined.is_empty() {
                Some("crepus web build failed".into())
            } else {
                Some(combined)
            };
            results.push(JsonStepResult {
                name: "crepus web build".into(),
                duration_ms: ms,
                ok,
                max_rss_kb: rss,
                stderr_tail: err.as_ref().map(|s| tail_str(s)),
            });
            print_step_line(
                json_out,
                1,
                1,
                "crepus web build (--site → --out-dir)",
                Some((ms, ok, rss)),
            );
            if !ok {
                return (
                    false,
                    Some(err.unwrap_or_else(|| "crepus web build failed".into())),
                    results,
                );
            }
        }
        "crepus-webext" => {
            let fixture = repo.join("examples/benchmarks/crepus-webext");
            let app_dir = work_dir.join("app");
            if dry_run {
                print_step_line(json_out, 1, 2, "sync fixture (copy to work dir)", None);
                print_step_line(json_out, 2, 2, "crepus webext build", None);
                return (
                    true,
                    None,
                    vec![
                        JsonStepResult {
                            name: "sync fixture".into(),
                            duration_ms: 0,
                            ok: true,
                            max_rss_kb: None,
                            stderr_tail: None,
                        },
                        JsonStepResult {
                            name: "crepus webext build".into(),
                            duration_ms: 0,
                            ok: true,
                            max_rss_kb: None,
                            stderr_tail: None,
                        },
                    ],
                );
            }
            if verbose && !json_out {
                eprintln!(
                    "      {} {}",
                    style("···").dim(),
                    style("sync fixture: recursive copy").yellow()
                );
                eprintln!("          {} {}", style("from").dim(), fixture.display());
                eprintln!("          {} {}", style("to").dim(), app_dir.display());
            }
            let t_copy = Instant::now();
            if let Err(e) = copy_dir_all(&fixture, &app_dir) {
                let ms = t_copy.elapsed().as_millis();
                results.push(JsonStepResult {
                    name: "sync fixture".into(),
                    duration_ms: ms,
                    ok: false,
                    max_rss_kb: None,
                    stderr_tail: Some(e.to_string()),
                });
                print_step_line(
                    json_out,
                    1,
                    2,
                    "sync fixture (copy to work dir)",
                    Some((ms, false, None)),
                );
                return (false, Some(format!("copy fixture: {e}")), results);
            }
            let copy_ms = t_copy.elapsed().as_millis();
            results.push(JsonStepResult {
                name: "sync fixture".into(),
                duration_ms: copy_ms,
                ok: true,
                max_rss_kb: None,
                stderr_tail: None,
            });
            print_step_line(
                json_out,
                1,
                2,
                "sync fixture (copy to work dir)",
                Some((copy_ms, true, None)),
            );

            if verbose && !json_out {
                eprintln!(
                    "      {} {}",
                    style("···").dim(),
                    style("builtin: crepus webext build").yellow()
                );
                eprintln!("          exe {}", exe.display());
                eprintln!("          --app {}", app_dir.display());
                eprintln!("          {}", style("— running —").dim());
            }
            let t_build = Instant::now();
            let mut wcmd = Command::new(&exe);
            wcmd.args(["webext", "build", "--app"]).arg(&app_dir);
            let built = exec_tracked(&mut wcmd, inherit_io, measure_memory);
            let ms = t_build.elapsed().as_millis();
            let (ok, combined, rss) = match built {
                Ok(x) => x,
                Err(e) => (false, e, None),
            };
            let err = if ok {
                None
            } else if combined.is_empty() {
                Some("webext build failed".into())
            } else {
                Some(combined)
            };
            results.push(JsonStepResult {
                name: "crepus webext build".into(),
                duration_ms: ms,
                ok,
                max_rss_kb: rss,
                stderr_tail: err.as_ref().map(|s| tail_str(s)),
            });
            print_step_line(json_out, 2, 2, "crepus webext build", Some((ms, ok, rss)));
            if !ok {
                return (
                    false,
                    Some(err.unwrap_or_else(|| "webext build failed".into())),
                    results,
                );
            }
        }
        "crepus-desktop" => {
            let crate_dir = repo.join("examples/benchmarks/crepus-desktop");
            if !crate_dir.join("Cargo.toml").exists() {
                return (
                    false,
                    Some(format!("missing desktop fixture: {}", crate_dir.display())),
                    results,
                );
            }
            let target_sub = work_dir.join("cargo-target");
            if dry_run {
                print_step_line(
                    json_out,
                    1,
                    1,
                    "cargo build --release (GPUI fixture, CARGO_TARGET_DIR in work)",
                    None,
                );
                return (
                    true,
                    None,
                    vec![JsonStepResult {
                        name: "cargo build --release (GPUI fixture)".into(),
                        duration_ms: 0,
                        ok: true,
                        max_rss_kb: None,
                        stderr_tail: None,
                    }],
                );
            }
            if verbose && !json_out {
                eprintln!(
                    "      {} {}",
                    style("···").dim(),
                    style("builtin: cargo build --release (GPUI)").yellow()
                );
                eprintln!("          cwd {}", crate_dir.display());
                eprintln!("          CARGO_TARGET_DIR {}", target_sub.display());
                eprintln!("          {}", style("— running —").dim());
            }
            let t0 = Instant::now();
            let mut cmd = Command::new("cargo");
            cmd.args(["build", "--release"])
                .current_dir(&crate_dir)
                .env("CARGO_TARGET_DIR", &target_sub);
            inject_sdkroot_env(&mut cmd);
            let built = exec_tracked(&mut cmd, inherit_io, measure_memory);
            let ms = t0.elapsed().as_millis();
            let (ok, combined, rss) = match built {
                Ok(x) => x,
                Err(e) => (false, e, None),
            };
            let err = if ok {
                None
            } else if combined.is_empty() {
                Some("desktop build failed".into())
            } else {
                Some(combined)
            };
            results.push(JsonStepResult {
                name: "cargo build --release (GPUI fixture)".into(),
                duration_ms: ms,
                ok,
                max_rss_kb: rss,
                stderr_tail: err.as_ref().map(|s| tail_str(s)),
            });
            print_step_line(
                json_out,
                1,
                1,
                "cargo build --release (GPUI fixture, CARGO_TARGET_DIR in work)",
                Some((ms, ok, rss)),
            );
            if !ok {
                return (
                    false,
                    Some(err.unwrap_or_else(|| "desktop build failed".into())),
                    results,
                );
            }
        }
        _ => {
            return (false, Some(format!("unknown builtin '{builtin}'")), results);
        }
    }
    (true, None, results)
}

fn stderr_tail_opt(stderr: &str) -> Option<String> {
    if stderr.trim().is_empty() {
        None
    } else {
        Some(tail_str(stderr))
    }
}

fn tail_str(s: &str) -> String {
    if s.len() > 2000 {
        s[s.len().saturating_sub(2000)..].to_string()
    } else {
        s.to_string()
    }
}

fn inject_sdkroot_env(cmd: &mut Command) {
    if std::env::var_os("SDKROOT").is_none() && cfg!(target_os = "macos") {
        if let Ok(out) = Command::new("xcrun").args(["--show-sdk-path"]).output() {
            if out.status.success() {
                let p = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !p.is_empty() {
                    cmd.env("SDKROOT", p);
                }
            }
        }
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    use std::fs;
    if dst.exists() {
        fs::remove_dir_all(dst)?;
    }
    copy_rec(src, dst)
}

fn copy_rec(src: &Path, dst: &Path) -> std::io::Result<()> {
    use std::fs;
    if src.is_dir() {
        fs::create_dir_all(dst)?;
        for e in fs::read_dir(src)? {
            let e = e?;
            copy_rec(&e.path(), &dst.join(e.file_name()))?;
        }
    } else {
        if let Some(p) = dst.parent() {
            fs::create_dir_all(p)?;
        }
        fs::copy(src, dst)?;
    }
    Ok(())
}
