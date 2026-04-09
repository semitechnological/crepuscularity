//! Integration tests for `crepus benchmark`.

use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

fn crepus() -> Command {
    Command::new(env!("CARGO_BIN_EXE_crepus"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn benchmark_all_alias_runs() {
    let config = repo_root().join("examples/benchmarks/benchmark.toml");
    let out = crepus()
        .current_dir(repo_root())
        .args([
            "benchmark",
            "all",
            "--config",
            config.to_str().unwrap(),
            "--dry-run",
            "--only",
            "crepus-web",
        ])
        .output()
        .expect("spawn crepus benchmark all");

    assert!(
        out.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn benchmark_dry_run_parses_config() {
    let config = repo_root().join("examples/benchmarks/benchmark.toml");
    assert!(config.is_file(), "{}", config.display());

    let out = crepus()
        .current_dir(repo_root())
        .args([
            "benchmark",
            "--config",
            config.to_str().unwrap(),
            "--dry-run",
            "--only",
            "crepus-web,nextjs",
        ])
        .output()
        .expect("spawn crepus benchmark");

    assert!(
        out.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("crepus-web") || stderr.contains("suite"),
        "expected suite output: {stderr}"
    );
}

/// `crepus benchmark` desktop builtin builds this crate; keep it compiling (see `parse_when`-style `view!` needs `cx`).
#[test]
fn benchmark_desktop_fixture_cargo_check() {
    let root = repo_root();
    let manifest = root.join("examples/benchmarks/crepus-desktop/Cargo.toml");
    assert!(manifest.is_file(), "missing {}", manifest.display());

    let mut cmd = Command::new("cargo");
    cmd.current_dir(&root).args([
        "check",
        "--manifest-path",
        "examples/benchmarks/crepus-desktop/Cargo.toml",
    ]);
    #[cfg(target_os = "macos")]
    {
        if std::env::var_os("SDKROOT").is_none() {
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

    let st = cmd
        .status()
        .expect("spawn cargo check for bench desktop fixture");
    assert!(
        st.success(),
        "examples/benchmarks/crepus-desktop must compile for `crepus benchmark` (set SDKROOT on macOS if needed)"
    );
}

#[test]
fn benchmark_json_includes_summary() {
    let config = repo_root().join("examples/benchmarks/benchmark.toml");
    let out = crepus()
        .current_dir(repo_root())
        .args([
            "benchmark",
            "--config",
            config.to_str().unwrap(),
            "--dry-run",
            "--only",
            "crepus-web",
            "--json",
        ])
        .output()
        .expect("spawn crepus benchmark --json");

    assert!(
        out.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("stdout is JSON");
    assert!(v.get("summary").is_some(), "expected summary: {v}");
    assert!(v.get("suites").is_some(), "expected suites: {v}");
    let summary = v.get("summary").unwrap();
    assert!(summary.get("by_wall_time").is_some());
    assert!(summary.get("total_wall_ms_completed").is_some());
}
