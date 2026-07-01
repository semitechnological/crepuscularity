//! Integration tests for `crepus benchmark`.

use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

#[test]
#[cfg_attr(
    windows,
    ignore = "crepus benchmark subprocess exits with Windows loader status in CI"
)]
fn benchmark_help_subcommand_prints_usage() {
    let out = crepus()
        .args(["benchmark", "--help"])
        .output()
        .expect("spawn crepus benchmark --help");
    assert!(
        out.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let help = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(help.to_lowercase().contains("benchmark"), "{help}");
}

fn crepus() -> Command {
    Command::new(env!("CARGO_BIN_EXE_crepus"))
}

fn repo_root() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root");
    // Windows `canonicalize` returns `\\?\` verbatim paths; those can break child `current_dir`
    // and path handling in some tools. Strip the prefix for drive and UNC paths.
    #[cfg(windows)]
    {
        strip_windows_verbatim_prefix(p)
    }
    #[cfg(not(windows))]
    {
        p
    }
}

#[cfg(windows)]
fn strip_windows_verbatim_prefix(path: PathBuf) -> PathBuf {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::ffi::OsStringExt;

    let wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    const PREFIX: &[u16] = &[b'\\' as u16, b'\\' as u16, b'?' as u16, b'\\' as u16];
    if wide.len() < PREFIX.len() || !wide.starts_with(PREFIX) {
        return path;
    }
    let rest = &wide[PREFIX.len()..];
    if rest.len() >= 3 && rest[0] == b'U' as u16 && rest[1] == b'N' as u16 && rest[2] == b'C' as u16
    {
        let after_unc = &rest[3..];
        if after_unc.first().copied() == Some(b'\\' as u16) {
            let mut out = vec![b'\\' as u16, b'\\' as u16];
            out.extend_from_slice(&after_unc[1..]);
            return PathBuf::from(OsString::from_wide(&out));
        }
    }
    PathBuf::from(OsString::from_wide(rest))
}

#[test]
#[cfg_attr(
    windows,
    ignore = "crepus benchmark subprocess exits with Windows loader status in CI"
)]
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
        "status={:?} stderr:\n{}\nstdout:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
#[cfg_attr(
    windows,
    ignore = "crepus benchmark subprocess exits with Windows loader status in CI"
)]
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
        "status={:?} stderr:\n{}\nstdout:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("crepus-web") || stderr.contains("suite"),
        "expected suite output: {stderr}"
    );
}

/// `crepus benchmark` desktop builtin expects this GPUI fixture to keep compiling.
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
#[cfg_attr(
    windows,
    ignore = "crepus benchmark subprocess exits with Windows loader status in CI"
)]
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

#[test]
#[cfg_attr(
    windows,
    ignore = "crepus benchmark subprocess exits with Windows loader status in CI"
)]
fn benchmark_check_json_for_crepus_web() {
    let config = repo_root().join("examples/benchmarks/benchmark.toml");
    let out = crepus()
        .current_dir(repo_root())
        .args([
            "benchmark",
            "check",
            "--config",
            config.to_str().unwrap(),
            "--only",
            "crepus-web",
            "--json",
        ])
        .output()
        .expect("spawn crepus benchmark check");

    assert!(
        out.status.success(),
        "status={:?} stderr:\n{}\nstdout:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("stdout is JSON");
    assert_eq!(v["required_targets_ok"], true);
    let tools = v["tools"].as_array().expect("tools array");
    let ids: Vec<&str> = tools.iter().filter_map(|t| t["id"].as_str()).collect();
    assert!(
        ids.contains(&"cargo"),
        "crepus-web should need cargo: {ids:?}"
    );
    let targets = v["targets"].as_array().expect("targets");
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0]["id"], "crepus-web");
    assert_eq!(targets[0]["ok"], true);
}

#[test]
#[cfg_attr(
    windows,
    ignore = "crepus benchmark subprocess exits with Windows loader status in CI"
)]
fn benchmark_check_no_matching_targets_json() {
    let config = repo_root().join("examples/benchmarks/benchmark.toml");
    let out = crepus()
        .current_dir(repo_root())
        .args([
            "benchmark",
            "check",
            "--config",
            config.to_str().unwrap(),
            "--only",
            "__no_such_target__",
            "--json",
        ])
        .output()
        .expect("spawn crepus benchmark check");

    assert!(!out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).expect("stdout is JSON");
    assert!(v.get("error").is_some(), "expected error field: {v}");
}
