//! Root **`crepus.toml`** — shared manifest for **`[ios]`** and one or more **`[[targets]]`** (`type = "web"`).
//!
//! Paths under each web target are relative to the directory containing **`crepus.toml`**.

use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CrepusManifest {
    #[serde(default)]
    pub ios: Option<IosTomlSection>,
    #[serde(default)]
    pub targets: Vec<ManifestTarget>,
    /// Single-site shorthand when you do not use `[[targets]]`.
    #[serde(default)]
    pub web: Option<WebShorthand>,
}

#[derive(Debug, Deserialize)]
pub struct IosTomlSection {
    pub scheme: String,
    #[serde(default = "default_xcodegen_spec")]
    pub xcodegen_spec: String,
    #[serde(default = "default_ios_destination")]
    pub destination: String,
}

#[derive(Debug, Deserialize)]
pub struct ManifestTarget {
    #[serde(rename = "type")]
    pub target_type: String,
    #[serde(default)]
    pub id: Option<String>,
    /// Site root directory, relative to the manifest folder.
    pub site: String,
    #[serde(default)]
    pub out: Option<String>,
    #[serde(default)]
    pub entry: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WebShorthand {
    /// Site root (relative to manifest dir). Default `"."`.
    #[serde(default = "default_dot")]
    pub site: String,
    #[serde(default)]
    pub out: Option<String>,
    #[serde(default)]
    pub entry: Option<String>,
}

fn default_dot() -> String {
    ".".into()
}

pub(crate) fn default_xcodegen_spec() -> String {
    "project.yml".into()
}

pub(crate) fn default_ios_destination() -> String {
    "platform=iOS Simulator,name=iPhone 16,OS=latest".into()
}

/// Resolved web site for `crepus web build` / `serve`.
#[derive(Debug, Clone)]
pub struct ResolvedWebTarget {
    pub id: String,
    pub site_dir: PathBuf,
    pub out_dir: PathBuf,
    pub entry: String,
}

type WebTargetRow = (Option<String>, String, Option<String>, Option<String>);

impl CrepusManifest {
    pub fn parse(src: &str) -> Result<Self, String> {
        toml::from_str(src).map_err(|e| e.to_string())
    }

    /// All `type = "web"` entries, plus `[web]` when there are no `[[targets]]` web rows.
    pub fn web_targets(&self, manifest_dir: &Path) -> Result<Vec<ResolvedWebTarget>, String> {
        let mut raw: Vec<WebTargetRow> = Vec::new();

        for t in &self.targets {
            if !t.target_type.eq_ignore_ascii_case("web") {
                continue;
            }
            raw.push((t.id.clone(), t.site.clone(), t.out.clone(), t.entry.clone()));
        }

        if raw.is_empty() {
            if let Some(w) = &self.web {
                raw.push((None, w.site.clone(), w.out.clone(), w.entry.clone()));
            }
        }

        if raw.is_empty() {
            return Ok(Vec::new());
        }

        let mut out = Vec::with_capacity(raw.len());
        for (id, site, od, ent) in raw {
            let site_dir = manifest_dir.join(&site);
            let site_dir = std::fs::canonicalize(&site_dir).unwrap_or(site_dir);
            let default_out = site_dir.join("dist");
            let out_dir = od
                .map(|r| {
                    let p = manifest_dir.join(r);
                    std::fs::canonicalize(&p).unwrap_or(p)
                })
                .unwrap_or(default_out);
            let entry = ent
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "index.crepus".into());
            let id = id.unwrap_or_else(|| {
                site_dir
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "default".into())
            });
            out.push(ResolvedWebTarget {
                id,
                site_dir,
                out_dir,
                entry,
            });
        }

        Ok(out)
    }
}

pub fn find_manifest_upward(start: &Path) -> Option<PathBuf> {
    let mut dir = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        let p = dir.join("crepus.toml");
        if p.is_file() {
            return Some(p);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

pub fn try_load_ios(manifest_path: &Path) -> Option<IosTomlSection> {
    let raw = std::fs::read_to_string(manifest_path).ok()?;
    let m: CrepusManifest = CrepusManifest::parse(&raw).ok()?;
    m.ios
}

/// Load web targets from `crepus.toml`, walking up from cwd when `manifest` is `None`.
pub fn load_web_targets(manifest: Option<PathBuf>) -> Option<Vec<ResolvedWebTarget>> {
    let cwd = std::env::current_dir().ok()?;
    let mpath = manifest.or_else(|| find_manifest_upward(&cwd))?;
    let md = mpath.parent()?.to_path_buf();
    let raw = std::fs::read_to_string(&mpath).ok()?;
    let man = CrepusManifest::parse(&raw).ok()?;
    let targets = man.web_targets(&md).ok()?;
    if targets.is_empty() {
        None
    } else {
        Some(targets)
    }
}

pub fn resolve_pick(
    targets: &[ResolvedWebTarget],
    id: Option<&str>,
) -> Result<ResolvedWebTarget, String> {
    let ids: Vec<&str> = targets.iter().map(|t| t.id.as_str()).collect();
    match (id, targets.len()) {
        (Some(id), _) => targets
            .iter()
            .find(|t| t.id == id)
            .cloned()
            .ok_or_else(|| format!("no web target with id {id:?} (available: {ids:?})")),
        (None, 1) => Ok(targets[0].clone()),
        (None, n) => Err(format!(
            "crepus.toml defines {n} web targets {ids:?}; pass --target ID"
        )),
    }
}
