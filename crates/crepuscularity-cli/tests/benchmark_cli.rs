//! Integration tests for `crepus benchmark`.

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
