//! Headless guest evaluation for benchmark tooling (no GPUI).

use std::path::Path;
use std::sync::Arc;

use crate::bridge::Bridge;
use crate::config::CrepusLiteConfig;
use crate::guest_compiler::prepare_guest_source;
use crate::v8_host::V8Host;

/// Shared merged prelude text for a benchmark matrix when every config lists the same [`CrepusLiteConfig::guest_prelude`] paths.
///
/// Does **not** share a [`Bridge`]: parallel matrix evals each use [`CrepusLiteConfig::build_bridge`] so concurrent
/// `invoke` from multiple V8 threads never aliases one bridge instance.
#[derive(Clone)]
pub struct BenchMatrixShared {
    pub prelude_merged: String,
}

/// If every config shares the same prelude path list, read those files once and merge.
pub fn bench_matrix_shared_for_configs(
    base: &Path,
    configs: &[&CrepusLiteConfig],
) -> Option<BenchMatrixShared> {
    if configs.is_empty() {
        return None;
    }
    let prel0 = &configs[0].guest_prelude;
    if !configs.iter().all(|c| c.guest_prelude == *prel0) {
        return None;
    }
    let prelude_merged = configs[0].guest_prelude_merged(base)?;
    Some(BenchMatrixShared { prelude_merged })
}

/// Run guest JS against an existing bridge (new isolate per call — use for a single evaluation).
pub fn eval_guest_from_script(bridge: Arc<Bridge>, script: &str) -> Result<String, String> {
    let mut host = V8Host::new(bridge)?;
    host.eval(script)
}

/// Evaluate using an already-loaded [`CrepusLiteConfig`].
pub fn eval_guest_from_config(base: &Path, config: &CrepusLiteConfig) -> Result<String, String> {
    let script = config
        .guest_source(base)
        .ok_or_else(|| "guest_entry, prelude, or file read failed".to_string())?;
    let path = config
        .resolved_guest_path(base)
        .unwrap_or_else(|| base.join(config.active_guest_entry().unwrap_or("guest.js")));
    let script = prepare_guest_source(&path, &script)?;
    eval_guest_from_script(config.build_bridge(), &script)
}

/// Like [`eval_guest_from_config`] but uses [`BenchMatrixShared`] to skip re-reading prelude files.
pub fn eval_guest_from_config_with_shared(
    base: &Path,
    config: &CrepusLiteConfig,
    shared: &BenchMatrixShared,
) -> Result<String, String> {
    let script = config
        .guest_source_with_merged_prelude(base, &shared.prelude_merged)
        .ok_or_else(|| "guest_entry or file read failed".to_string())?;
    let path = config
        .resolved_guest_path(base)
        .unwrap_or_else(|| base.join(config.active_guest_entry().unwrap_or("guest.js")));
    let script = prepare_guest_source(&path, &script)?;
    eval_guest_from_script(config.build_bridge(), &script)
}

/// Evaluate the configured guest (prelude + entry) relative to `base`, the process working directory
/// that guest paths in TOML are resolved against.
pub fn eval_guest_from_config_file(base: &Path, config_path: &Path) -> Result<String, String> {
    let config = CrepusLiteConfig::load_from_path(config_path)?;
    eval_guest_from_config(base, &config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn eval_guest_from_config_file_missing() {
        let dir = tempdir().unwrap();
        let base = dir.path();
        let config_path = base.join("missing.toml");

        let result = eval_guest_from_config_file(base, &config_path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("No such file or directory")
                || err_msg.contains("The system cannot find the file specified")
        );
    }

    #[test]
    fn eval_guest_from_config_file_invalid_toml() {
        let dir = tempdir().unwrap();
        let base = dir.path();
        let config_path = base.join("invalid.toml");

        fs::write(&config_path, "invalid toml syntax = [[").unwrap();

        let result = eval_guest_from_config_file(base, &config_path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("parse error") || err_msg.contains("expected a boolean"));
    }

    #[test]
    fn eval_guest_from_config_file_missing_entry() {
        let dir = tempdir().unwrap();
        let base = dir.path();
        let config_path = base.join("missing_entry.toml");

        fs::write(&config_path, "guest_entry = \"nonexistent.js\"").unwrap();

        let result = eval_guest_from_config_file(base, &config_path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("guest_entry, prelude, or file read failed"));
    }

    #[test]
    fn eval_guest_from_config_file_parse_error() {
        let dir = tempdir().unwrap();
        let base = dir.path();
        let config_path = base.join("parse_error.toml");
        let entry_path = base.join("entry.ts"); // Must be .ts to trigger oxc

        fs::write(&config_path, "guest_entry = \"entry.ts\"").unwrap();
        fs::write(&entry_path, "const x: = 1;").unwrap(); // Invalid TypeScript

        let result = eval_guest_from_config_file(base, &config_path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("Oxc parse failed") || err_msg.contains("Expected"));
    }

    #[test]
    fn eval_guest_from_config_file_eval_error() {
        let dir = tempdir().unwrap();
        let base = dir.path();
        let config_path = base.join("eval_error.toml");
        let entry_path = base.join("entry.js");

        fs::write(&config_path, "guest_entry = \"entry.js\"").unwrap();
        fs::write(&entry_path, "throw new Error('test error');").unwrap();

        let result = eval_guest_from_config_file(base, &config_path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("test error") || err_msg.contains("Error:"));
    }
}
