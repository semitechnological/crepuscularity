use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use serde_json::json;

use crate::crepus_toml::DocsHookConfig;

#[derive(Deserialize)]
struct Manifest {
    package: Package,
}

#[derive(Deserialize)]
struct Package {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DocsHookInvocation {
    program: PathBuf,
    args: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct DocsHookTheme {
    pub(crate) accent: String,
    pub(crate) accent_soft: String,
    pub(crate) surface: String,
    pub(crate) text: String,
    pub(crate) muted: String,
    pub(crate) border: String,
}

pub(crate) fn docs_src_path(site_dir: &Path, hook: &DocsHookConfig) -> PathBuf {
    let src = hook.src.as_deref().unwrap_or("../docs");
    absolutize(site_dir, src)
}

pub(crate) fn run_docs_hook(
    site_dir: &Path,
    out_docs_dir: &Path,
    hook: &DocsHookConfig,
    site_name: &str,
    theme: &DocsHookTheme,
) -> io::Result<()> {
    let docs_src = docs_src_path(site_dir, hook);
    if !docs_src.is_dir() {
        return Ok(());
    }

    // `out_docs_dir` is relative to the process cwd during `crepus web build`, but the hook
    // subprocess runs with `current_dir = site_dir`. Resolve to an absolute path first.
    let out_docs_dir = absolutize_process_cwd(out_docs_dir);

    let theme_json = json!({
        "accent": theme.accent,
        "accent_soft": theme.accent_soft,
        "surface": theme.surface,
        "text": theme.text,
        "muted": theme.muted,
        "border": theme.border,
    })
    .to_string();

    let docs_src_arg = docs_src.to_string_lossy().into_owned();
    let out_docs_arg = out_docs_dir.to_string_lossy().into_owned();
    let runtime_args = [
        "--docs-src",
        docs_src_arg.as_str(),
        "--out-dir",
        out_docs_arg.as_str(),
        "--site-name",
        site_name,
        "--theme-json",
        theme_json.as_str(),
    ];
    let invocation = resolve_docs_hook_invocation(site_dir, hook);

    let status = Command::new(&invocation.program)
        .current_dir(site_dir)
        .args(&invocation.args)
        .args(runtime_args)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("docs hook exited with {status}")))
    }
}

fn resolve_docs_hook_invocation(site_dir: &Path, hook: &DocsHookConfig) -> DocsHookInvocation {
    if let Some(binary) = cargo_run_manifest_binary(site_dir, hook) {
        return DocsHookInvocation {
            program: binary,
            args: Vec::new(),
        };
    }

    DocsHookInvocation {
        program: PathBuf::from(&hook.command),
        args: hook.args.clone(),
    }
}

fn cargo_run_manifest_binary(site_dir: &Path, hook: &DocsHookConfig) -> Option<PathBuf> {
    if hook.command != "cargo" || hook.args.first().map(String::as_str) != Some("run") {
        return None;
    }

    let manifest_idx = hook.args.iter().position(|arg| arg == "--manifest-path")?;
    let manifest_raw = hook.args.get(manifest_idx + 1)?;
    let manifest_path = absolutize(site_dir, manifest_raw);
    let profile = if hook.args.iter().any(|arg| arg == "--release") {
        "release"
    } else {
        "debug"
    };
    let package_name = package_name_from_manifest(&manifest_path)?;
    let binary_name = if cfg!(windows) {
        format!("{package_name}.exe")
    } else {
        package_name
    };
    let binary = manifest_path
        .parent()?
        .join("target")
        .join(profile)
        .join(binary_name);
    binary.is_file().then_some(binary)
}

fn package_name_from_manifest(manifest_path: &Path) -> Option<String> {
    let contents = std::fs::read_to_string(manifest_path).ok()?;
    toml::from_str::<Manifest>(&contents)
        .ok()
        .map(|m| m.package.name)
}

fn absolutize(base: &Path, raw: &str) -> PathBuf {
    let path = Path::new(raw);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        let joined = base.join(path);
        std::fs::canonicalize(&joined).unwrap_or(joined)
    }
}

fn absolutize_process_cwd(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let joined = cwd.join(path);
    std::fs::canonicalize(&joined).unwrap_or(joined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_docs_hook_falls_back_to_configured_command() {
        let hook = DocsHookConfig {
            command: "echo".into(),
            args: vec!["docs".into()],
            src: None,
        };
        let invocation = resolve_docs_hook_invocation(Path::new("."), &hook);
        assert_eq!(invocation.program, PathBuf::from("echo"));
        assert_eq!(invocation.args, vec!["docs".to_string()]);
    }
}
