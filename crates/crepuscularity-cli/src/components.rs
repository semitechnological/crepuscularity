//! `crepus components` — list / add / themes for **`crepuscularity-components`**.
//!
//! Catalog lives at `plugins/crepuscularity-components/catalog/`. Commands are
//! resilient when the catalog is missing (prints guidance instead of failing hard).

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::cli::{ComponentTarget, ComponentsCommands};
use crate::error::CrepusCliError;
use crate::ui;

const CATALOG_REL: &str = "plugins/crepuscularity-components/catalog/components.json";
const THEMES_REL: &str = "plugins/crepuscularity-components/catalog/themes";
const PKG_ROOT: &str = "plugins/crepuscularity-components";

#[derive(Debug, Deserialize)]
struct Catalog {
    #[serde(default)]
    components: Vec<CatalogComponent>,
}

#[derive(Debug, Deserialize)]
struct CatalogComponent {
    id: String,
    /// Display title (newer catalog schema).
    #[serde(default)]
    title: Option<String>,
    /// Alternate display name (older / minimal schema).
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    /// Spec file relative to the components package root.
    #[serde(default)]
    spec: Option<String>,
    /// Platforms that implement this component.
    #[serde(default)]
    platforms: Vec<String>,
    /// Explicit per-target path hints (minimal schema).
    #[serde(default)]
    targets: CatalogTargets,
}

#[derive(Debug, Default, Deserialize)]
struct CatalogTargets {
    flutter: Option<String>,
    svelte: Option<String>,
    moonshine: Option<String>,
    gpui: Option<String>,
}

impl CatalogComponent {
    fn display_name(&self) -> &str {
        self.title
            .as_deref()
            .or(self.name.as_deref())
            .unwrap_or(&self.id)
    }
}

pub fn execute(cmd: ComponentsCommands) -> Result<(), CrepusCliError> {
    match cmd {
        ComponentsCommands::List => list(),
        ComponentsCommands::Add { id, target } => add(&id, target),
        ComponentsCommands::Themes => themes(),
    }
}

fn repo_root() -> PathBuf {
    std::env::var("CREPUS_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        })
}

fn catalog_path(root: &Path) -> PathBuf {
    root.join(CATALOG_REL)
}

fn themes_dir(root: &Path) -> PathBuf {
    root.join(THEMES_REL)
}

fn load_catalog(root: &Path) -> Result<Option<Catalog>, CrepusCliError> {
    let path = catalog_path(root);
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).map_err(|err| CrepusCliError::io(err, path.clone()))?;
    let catalog: Catalog = serde_json::from_str(&raw).map_err(|err| {
        CrepusCliError::context(format!("parse {}: {err}", path.display()))
    })?;
    Ok(Some(catalog))
}

fn list() -> Result<(), CrepusCliError> {
    let root = repo_root();
    let path = catalog_path(&root);
    match load_catalog(&root)? {
        None => {
            eprintln!(
                "{} catalog not found at {} — create plugins/crepuscularity-components/catalog/components.json",
                ui::warn(),
                path.display()
            );
            Ok(())
        }
        Some(catalog) if catalog.components.is_empty() => {
            println!("(no components in catalog)");
            Ok(())
        }
        Some(catalog) => {
            for c in &catalog.components {
                let name = c.display_name();
                let desc = c.description.as_deref().unwrap_or("");
                if desc.is_empty() {
                    println!("{:<16} {}", c.id, name);
                } else {
                    println!("{:<16} {} — {}", c.id, name, desc);
                }
            }
            Ok(())
        }
    }
}

fn target_key(t: ComponentTarget) -> &'static str {
    match t {
        ComponentTarget::Flutter => "flutter",
        ComponentTarget::Svelte => "svelte",
        ComponentTarget::Moonshine => "moonshine",
        ComponentTarget::Gpui => "gpui",
    }
}

fn path_for_target(comp: &CatalogComponent, t: ComponentTarget) -> Option<String> {
    let explicit = match t {
        ComponentTarget::Flutter => comp.targets.flutter.clone(),
        ComponentTarget::Svelte => comp.targets.svelte.clone(),
        ComponentTarget::Moonshine => comp.targets.moonshine.clone(),
        ComponentTarget::Gpui => comp.targets.gpui.clone(),
    };
    if explicit.is_some() {
        return explicit;
    }

    let key = target_key(t);
    let supported = if comp.platforms.is_empty() {
        true
    } else {
        comp.platforms.iter().any(|p| p == key)
    };
    if !supported {
        return None;
    }

    // Conventional layout used by crepuscularity-components:
    //   packages/<target>/… and specs/<id>.json
    Some(format!("{PKG_ROOT}/packages/{key}"))
}

fn add(id: &str, target: Option<ComponentTarget>) -> Result<(), CrepusCliError> {
    let root = repo_root();
    let Some(catalog) = load_catalog(&root)? else {
        eprintln!(
            "{} catalog missing — cannot resolve component {id:?}",
            ui::warn()
        );
        eprintln!("  expected {}", catalog_path(&root).display());
        return Ok(());
    };

    let Some(comp) = catalog.components.iter().find(|c| c.id == id) else {
        let ids: Vec<&str> = catalog.components.iter().map(|c| c.id.as_str()).collect();
        return Err(CrepusCliError::context(format!(
            "unknown component {id:?}; known: {}",
            ids.join(", ")
        )));
    };

    let wanted: Vec<ComponentTarget> = match target {
        Some(t) => vec![t],
        None => vec![
            ComponentTarget::Flutter,
            ComponentTarget::Svelte,
            ComponentTarget::Moonshine,
            ComponentTarget::Gpui,
        ],
    };

    let selected: Vec<(ComponentTarget, String)> = wanted
        .into_iter()
        .filter_map(|t| path_for_target(comp, t).map(|p| (t, p)))
        .collect();

    if selected.is_empty() {
        return Err(CrepusCliError::context(format!(
            "component {id:?} has no path for target {:?}",
            target.unwrap_or(ComponentTarget::Moonshine)
        )));
    }

    println!("component: {}", comp.id);
    println!("name:      {}", comp.display_name());
    if let Some(desc) = &comp.description {
        println!("about:     {desc}");
    }
    if let Some(spec) = &comp.spec {
        let spec_path = format!("{PKG_ROOT}/{spec}");
        let abs = root.join(&spec_path);
        let status = if abs.is_file() { "ok" } else { "missing" };
        println!("spec:      {spec_path}  [{status}]");
    }
    println!();
    println!("Install / copy path hints (relative to repo root):");
    for (t, path) in &selected {
        let abs = root.join(path);
        let status = if abs.exists() { "ok" } else { "missing" };
        println!("  {:<10} {}  [{status}]", target_key(*t), path);
    }
    println!();
    println!("Guidance:");
    println!("  • Copy or symlink the target package into your app, or depend on it once published.");
    println!("  • Spec JSON under plugins/crepuscularity-components/specs/ is the source of truth.");
    println!("  • Moonshine/Svelte: see `crepus moonshine dep` for @crepuscularity/components.");
    println!("  • Flutter: wire via crepuscularity_flutter + the flutter package path.");
    println!("  • GPUI: include the gpui package module from your desktop crate.");
    Ok(())
}

fn themes() -> Result<(), CrepusCliError> {
    let root = repo_root();
    let dir = themes_dir(&root);
    if !dir.is_dir() {
        eprintln!(
            "{} themes directory not found at {} — create catalog/themes/",
            ui::warn(),
            dir.display()
        );
        return Ok(());
    }

    let mut names: Vec<String> = fs::read_dir(&dir)
        .map_err(|err| CrepusCliError::io(err, dir.clone()))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return None;
            }
            let ext = path.extension()?.to_str()?;
            if matches!(ext, "json" | "toml" | "yaml" | "yml") {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();
    names.sort();

    if names.is_empty() {
        println!("(no themes in {})", dir.display());
    } else {
        for name in names {
            println!("{name}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_deserializes_minimal_targets() {
        let raw = r#"{"components":[{"id":"button","name":"Button","targets":{"moonshine":"x.ts"}}]}"#;
        let catalog: Catalog = serde_json::from_str(raw).unwrap();
        assert_eq!(catalog.components.len(), 1);
        assert_eq!(catalog.components[0].id, "button");
        assert_eq!(
            catalog.components[0].targets.moonshine.as_deref(),
            Some("x.ts")
        );
    }

    #[test]
    fn catalog_deserializes_registry_schema() {
        let raw = r#"{
          "components":[{
            "id":"button",
            "title":"Button",
            "description":"Pressable",
            "spec":"specs/button.json",
            "platforms":["flutter","moonshine"]
          }]
        }"#;
        let catalog: Catalog = serde_json::from_str(raw).unwrap();
        let c = &catalog.components[0];
        assert_eq!(c.display_name(), "Button");
        assert_eq!(
            path_for_target(c, ComponentTarget::Moonshine).as_deref(),
            Some("plugins/crepuscularity-components/packages/moonshine")
        );
        assert!(path_for_target(c, ComponentTarget::Svelte).is_none());
    }
}
