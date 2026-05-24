use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::json;

use crate::crepus_toml::DocsHookConfig;

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

    let status = Command::new(&hook.command)
        .current_dir(site_dir)
        .args(&hook.args)
        .args([
            "--docs-src",
            &docs_src_arg,
            "--out-dir",
            &out_docs_arg,
            "--site-name",
            site_name,
            "--theme-json",
            &theme_json,
        ])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("docs hook exited with {status}")))
    }
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
