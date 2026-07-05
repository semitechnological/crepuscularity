use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::build_options::BuildOptions;
use crate::crepus_toml::{pick_targets, ResolvedTarget};
use crate::target_builder;
use crate::ui;

pub struct ManifestBuildArgs {
    pub options: BuildOptions,
    pub target_id: Option<String>,
    pub selector: Option<String>,
    pub manifest: Option<PathBuf>,
    pub all: bool,
}

pub(crate) fn has_manifest_targets(manifest: Option<PathBuf>) -> bool {
    matches!(
        crate::crepus_toml::load_manifest_targets(manifest),
        Ok(Some(targets)) if !targets.is_empty()
    )
}

pub(crate) fn execute(args: ManifestBuildArgs) {
    let ManifestBuildArgs {
        options,
        target_id,
        selector,
        manifest,
        all,
    } = args;
    if target_id.is_some() && selector.is_some() {
        ui::error("use either --target ID or a positional selector, not both");
    }
    if all && (target_id.is_some() || selector.is_some()) {
        ui::error("use either --all or a target selector, not both");
    }
    let targets = crate::crepus_toml::load_manifest_targets(manifest)
        .unwrap_or_else(|e| ui::error(&e))
        .unwrap_or_else(|| ui::error("no crepus.toml found"));
    if targets.is_empty() {
        ui::error("crepus.toml has no [[targets]] entries");
    }
    let picked = if all {
        targets
    } else if let Some(sel) = selector.as_deref() {
        pick_targets_by_selector(&targets, sel).unwrap_or_else(|e| ui::error(&e))
    } else {
        pick_targets(&targets, target_id.as_deref()).unwrap_or_else(|e| ui::error(&e))
    };
    for target in picked {
        build_target(&target, options);
    }
}

fn pick_targets_by_selector(
    targets: &[ResolvedTarget],
    selector: &str,
) -> Result<Vec<ResolvedTarget>, String> {
    if let Ok(picked) = pick_targets(targets, Some(selector)) {
        return Ok(picked);
    }
    let normalized = normalize_selector(selector);
    let picked: Vec<ResolvedTarget> = targets
        .iter()
        .filter(|target| normalize_selector(&target.target_type) == normalized)
        .cloned()
        .collect();
    if !picked.is_empty() {
        return Ok(picked);
    }
    let ids: Vec<&str> = targets.iter().map(|t| t.id.as_str()).collect();
    let types: BTreeSet<&str> = targets.iter().map(|t| t.target_type.as_str()).collect();
    Err(format!(
        "no target id or type {selector:?} (ids: {ids:?}; types: {types:?})"
    ))
}

fn normalize_selector(selector: &str) -> &str {
    match selector {
        "extension" | "browser-extension" | "web-extension" => "webext",
        "ir" => "native",
        other => other,
    }
}

fn build_target(target: &ResolvedTarget, options: BuildOptions) {
    let builder = target_builder::builder_for(target);
    if let Err(e) = builder.build(&options) {
        ui::error(&e);
    }
}
