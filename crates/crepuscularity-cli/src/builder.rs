/// Cargo build runner + JSON diagnostic parser.
///
/// Runs `cargo build --message-format=json-diagnostic-rendered-ansi`,
/// streams the JSON output, and returns a `BuildOutcome`.
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use serde::Deserialize;

use crate::hud::{BuildError, DevStatus, HudState};

// ── Cargo JSON schema ──────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CargoLine {
    reason: String,
    message: Option<CompilerMsg>,
    success: Option<bool>,
}

#[derive(Deserialize)]
struct CompilerMsg {
    level: String,
    message: String,
    rendered: Option<String>,
    spans: Vec<MsgSpan>,
}

#[derive(Deserialize, Default, Clone)]
struct MsgSpan {
    file_name: String,
    line_start: u32,
}

// ── Public API ─────────────────────────────────────────────────────────────

pub struct BuildOutcome {
    pub success: bool,
    pub errors: Vec<BuildError>,
}

/// Run `cargo build` in `cwd`.
///
/// Diagnostics are streamed to stderr immediately (ANSI rendered).
/// Errors are also collected in the returned `BuildOutcome` for the HUD.
/// `hud` is optional — pass `None` when running outside the dev loop.
pub fn cargo_build(cwd: &Path, release: bool, hud: Option<Arc<Mutex<HudState>>>) -> BuildOutcome {
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--message-format=json-diagnostic-rendered-ansi")
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::null()) // diagnostics come through JSON stdout
        .env("CARGO_BUILD_INCREMENTAL", "true");

    if release {
        cmd.arg("--release");
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[crepus] Failed to spawn cargo: {e}");
            return BuildOutcome {
                success: false,
                errors: vec![],
            };
        }
    };

    let stdout = child.stdout.take().unwrap();
    let mut errors: Vec<BuildError> = Vec::new();
    let mut success = false;

    for line in BufReader::new(stdout).lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let msg: CargoLine = match serde_json::from_str(&line) {
            Ok(m) => m,
            Err(_) => continue,
        };

        match msg.reason.as_str() {
            "compiler-message" => {
                let Some(cm) = msg.message else { continue };

                // Always print rendered output to terminal
                if let Some(ref rendered) = cm.rendered {
                    eprint!("{rendered}");
                }

                if cm.level == "error" {
                    let span = cm.spans.first().cloned().unwrap_or_default();
                    let err = BuildError {
                        level: cm.level,
                        message: cm.message,
                        file: span.file_name,
                        line: span.line_start,
                        rendered: cm.rendered,
                    };
                    errors.push(err.clone());

                    // Live-update HUD so spinner shows errors as they arrive
                    if let Some(ref hud) = hud {
                        if let Ok(mut state) = hud.lock() {
                            let count = errors.len();
                            state.status = DevStatus::Failed {
                                errors: errors.clone(),
                                count,
                            };
                        }
                    }
                }
            }
            "build-finished" => {
                success = msg.success.unwrap_or(false);
            }
            _ => {}
        }
    }

    let _ = child.wait();
    BuildOutcome { success, errors }
}

/// Kill a child process (best-effort).
pub fn kill_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

/// Find the binary name to run after a build.
/// Prefers `[[bin]] name` over `[package] name` in Cargo.toml.
/// Falls back to `override_name` if provided.
pub fn find_bin_name(cwd: &Path, override_name: Option<&str>) -> Option<String> {
    if let Some(name) = override_name {
        return Some(name.to_string());
    }

    let content = std::fs::read_to_string(cwd.join("Cargo.toml")).ok()?;
    let mut in_bin = false;
    let mut in_package = false;
    let mut package_name: Option<String> = None;

    for line in content.lines() {
        let t = line.trim();
        if t == "[[bin]]" {
            in_bin = true;
            in_package = false;
            continue;
        }
        if t.starts_with('[') {
            in_bin = false;
            in_package = t == "[package]";
            continue;
        }
        if (in_bin || in_package) && t.starts_with("name") {
            if let Some(eq) = t.find('=') {
                let name = t[eq + 1..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                if !name.is_empty() {
                    if in_bin {
                        return Some(name); // [[bin]] wins immediately
                    } else if package_name.is_none() {
                        package_name = Some(name);
                    }
                }
            }
        }
    }

    package_name
}
