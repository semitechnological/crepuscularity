/// Cargo build runner + JSON diagnostic parser.
///
/// Runs `cargo build --message-format=json-diagnostic-rendered-ansi`,
/// streams the JSON output, and returns a `BuildOutcome`.
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use serde::Deserialize;

use crate::build_options::BuildOptions;
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
pub fn cargo_build(
    cwd: &Path,
    options: BuildOptions,
    hud: Option<Arc<Mutex<HudState>>>,
) -> BuildOutcome {
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--message-format=json-diagnostic-rendered-ansi")
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::null()) // diagnostics come through JSON stdout
        .env("CARGO_BUILD_INCREMENTAL", "true");

    if options.release() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_build_finished_msg() {
        let json = r#"{"reason":"build-finished","success":true}"#;
        let msg: CargoLine = serde_json::from_str(json).unwrap();
        assert_eq!(msg.reason, "build-finished");
        assert_eq!(msg.success, Some(true));
    }

    #[test]
    fn parse_compiler_error_msg() {
        let json = r#"{
            "reason": "compiler-message",
            "package_id": "foo 0.1.0",
            "target": {},
            "message": {
                "level": "error",
                "message": "expected `;`",
                "rendered": "error: expected `;`\n --> src/main.rs:1:1\n",
                "spans": [{"file_name": "src/main.rs", "line_start": 1, "line_end": 1, "byte_start": 0, "byte_end": 1, "column_start": 1, "column_end": 1, "text": [], "is_primary": true}]
            }
        }"#;
        let msg: CargoLine = serde_json::from_str(json).unwrap();
        assert_eq!(msg.reason, "compiler-message");
        let cm = msg.message.unwrap();
        assert_eq!(cm.level, "error");
        assert_eq!(cm.message, "expected `;`");
        assert!(cm.rendered.unwrap().contains("expected `;`"));
        assert_eq!(cm.spans[0].file_name, "src/main.rs");
        assert_eq!(cm.spans[0].line_start, 1);
    }

    #[test]
    fn parse_compiler_warning_msg() {
        let json = r#"{
            "reason": "compiler-message",
            "package_id": "bar 0.2.0",
            "target": {},
            "message": {
                "level": "warning",
                "message": "unused variable",
                "rendered": "warning: unused variable `x`\n --> src/lib.rs:5:9\n",
                "spans": [{"file_name": "src/lib.rs", "line_start": 5, "line_end": 5, "byte_start": 0, "byte_end": 1, "column_start": 9, "column_end": 10, "text": [], "is_primary": true}]
            }
        }"#;
        let msg: CargoLine = serde_json::from_str(json).unwrap();
        let cm = msg.message.unwrap();
        assert_eq!(cm.level, "warning");
        assert_eq!(cm.message, "unused variable");
    }

    #[test]
    fn parse_invalid_json_does_not_panic() {
        let result: Result<CargoLine, _> = serde_json::from_str("not json");
        assert!(result.is_err());
    }

    #[test]
    fn find_bin_name_prefers_bin_over_package() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("Cargo.toml");
        std::fs::write(
            &toml_path,
            r#"[package]
name = "my-lib"

[[bin]]
name = "my-app"
"#,
        )
        .unwrap();
        assert_eq!(find_bin_name(dir.path(), None), Some("my-app".into()));
    }

    #[test]
    fn find_bin_name_falls_back_to_package() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("Cargo.toml");
        std::fs::write(
            &toml_path,
            r#"[package]
name = "my-lib"
"#,
        )
        .unwrap();
        assert_eq!(find_bin_name(dir.path(), None), Some("my-lib".into()));
    }

    #[test]
    fn find_bin_name_override_wins() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            find_bin_name(dir.path(), Some("custom-bin")),
            Some("custom-bin".into())
        );
    }

    #[test]
    fn find_bin_name_no_toml_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(find_bin_name(dir.path(), None), None);
    }
}
