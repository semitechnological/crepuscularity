use std::fs;
use std::path::{Component, Path, PathBuf};

use crepuscularity_core::bundle::{parse_bundle, Bundle};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TauriVersion {
    V1,
    V2,
}

#[derive(Debug, Clone)]
pub struct TauriProject {
    root: PathBuf,
    config: PathBuf,
    version: TauriVersion,
    frontend_dist: PathBuf,
    config_value: Value,
}

impl TauriProject {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, String> {
        let root = root.as_ref().canonicalize().map_err(|e| e.to_string())?;
        let config = config_paths(&root)
            .into_iter()
            .find(|path| path.is_file())
            .ok_or_else(|| format!("no Tauri config under {}", root.display()))?;
        let value = read_config(&config)?;
        let build = value
            .get("build")
            .and_then(Value::as_object)
            .ok_or_else(|| "tauri config missing build object".to_string())?;
        let (version, dist) = if let Some(dist) = build.get("frontendDist").and_then(Value::as_str)
        {
            (TauriVersion::V2, dist)
        } else if let Some(dist) = build.get("distDir").and_then(Value::as_str) {
            (TauriVersion::V1, dist)
        } else {
            return Err(
                "tauri config missing build.frontendDist (v2) or build.distDir (v1)".into(),
            );
        };
        let frontend_dist = config.parent().unwrap_or(&root).join(dist);
        Ok(Self {
            root,
            config,
            version,
            frontend_dist,
            config_value: value,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn config_path(&self) -> &Path {
        &self.config
    }
    pub fn version(&self) -> TauriVersion {
        self.version
    }
    pub fn frontend_dist(&self) -> &Path {
        &self.frontend_dist
    }

    pub fn audit(&self) -> AuditReport {
        let mut uses = config_uses(&self.config_value, &self.config);
        collect_project_uses(&self.root, &mut uses);
        uses.sort_by(|left, right| {
            left.source
                .cmp(&right.source)
                .then(left.api.cmp(&right.api))
        });
        uses.dedup();
        AuditReport { uses }
    }

    pub fn bundle(&self) -> Result<Bundle, String> {
        let path = self.frontend_dist.join("crepus-bundle.json");
        let json = fs::read_to_string(&path)
            .map_err(|_| format!("{} is required for native conversion", path.display()))?;
        let bundle = parse_bundle(&json).map_err(|e| e.to_string())?;
        validate_bundle(&bundle)?;
        Ok(bundle)
    }

    #[cfg(feature = "native")]
    pub fn native_ir(&self) -> Result<crepuscularity_native::ViewIr, String> {
        let bundle = self.bundle()?;
        crepuscularity_native::render_from_files(
            &bundle.files,
            &bundle.entry,
            &crepuscularity_core::context::TemplateContext::new(),
        )
        .map_err(|e| e.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Coverage {
    Native,
    Backend,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiUse {
    pub source: String,
    pub api: String,
    pub coverage: Coverage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditReport {
    pub uses: Vec<ApiUse>,
}

impl AuditReport {
    pub fn native_ready(&self) -> Result<(), String> {
        let blocked = self
            .uses
            .iter()
            .filter(|use_| use_.coverage != Coverage::Native)
            .map(|use_| format!("{} ({})", use_.api, use_.source))
            .collect::<Vec<_>>();
        if blocked.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "native conversion requires adapters for: {}",
                blocked.join(", ")
            ))
        }
    }
}

fn config_paths(root: &Path) -> Vec<PathBuf> {
    [root.join("src-tauri"), root.to_path_buf()]
        .into_iter()
        .flat_map(|dir| {
            ["tauri.conf.json", "tauri.conf.json5", "Tauri.toml"]
                .into_iter()
                .map(move |name| dir.join(name))
        })
        .collect()
}

fn read_config(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    if path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
        let value: toml::Value = toml::from_str(&text).map_err(|e| e.to_string())?;
        serde_json::to_value(value).map_err(|e| e.to_string())
    } else {
        json5::from_str(&text).map_err(|e| e.to_string())
    }
}

fn validate_bundle(bundle: &Bundle) -> Result<(), String> {
    if !bundle.files.contains_key(&bundle.entry) {
        return Err(format!("bundle entry {:?} is missing", bundle.entry));
    }
    for path in bundle.files.keys() {
        if Path::new(path).components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        }) {
            return Err(format!("bundle contains unsafe path {path:?}"));
        }
    }
    Ok(())
}

fn config_uses(config: &Value, config_path: &Path) -> Vec<ApiUse> {
    let source = config_path.display().to_string();
    let mut uses = Vec::new();
    if config.pointer("/app/trayIcon").is_some() || config.pointer("/tauri/systemTray").is_some() {
        uses.push(api_use(&source, "tray", Coverage::Unsupported));
    }
    if config
        .pointer("/app/windows")
        .and_then(Value::as_array)
        .is_some_and(|windows| windows.len() > 1)
    {
        uses.push(api_use(&source, "windows.multiple", Coverage::Unsupported));
    }
    if config.pointer("/tauri/allowlist").is_some()
        || config.pointer("/app/security/capabilities").is_some()
    {
        uses.push(api_use(&source, "permissions", Coverage::Unsupported));
    }
    if let Some(plugins) = config.get("plugins").and_then(Value::as_object) {
        for name in plugins.keys() {
            uses.push(api_use(
                &source,
                &format!("plugin.{name}"),
                plugin_coverage(name),
            ));
        }
    }
    uses
}

fn collect_project_uses(root: &Path, uses: &mut Vec<ApiUse>) {
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if !matches!(
                    path.file_name().and_then(|name| name.to_str()),
                    Some("target" | "node_modules" | "dist" | ".git")
                ) {
                    pending.push(path);
                }
                continue;
            }
            let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
                continue;
            };
            let source = path.display().to_string();
            match extension {
                "rs" => {
                    if let Ok(text) = fs::read_to_string(&path) {
                        if text.contains("#[tauri::command]") || text.contains("#[command]") {
                            uses.push(api_use(&source, "command", Coverage::Backend));
                        }
                        if text.contains(".emit(") || text.contains(".listen(") {
                            uses.push(api_use(&source, "event", Coverage::Backend));
                        }
                    }
                }
                "js" | "jsx" | "ts" | "tsx" => {
                    if let Ok(text) = fs::read_to_string(&path) {
                        for api in frontend_apis(&text) {
                            uses.push(api_use(&source, api, frontend_coverage(api)));
                        }
                    }
                }
                "toml" if path.file_name().and_then(|name| name.to_str()) == Some("Cargo.toml") => {
                    if let Ok(text) = fs::read_to_string(&path) {
                        for line in text.lines() {
                            let name = line.split('=').next().unwrap_or("").trim();
                            if let Some(plugin) = name.strip_prefix("tauri-plugin-") {
                                uses.push(api_use(
                                    &source,
                                    &format!("plugin.{plugin}"),
                                    plugin_coverage(plugin),
                                ));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn frontend_apis(text: &str) -> Vec<&'static str> {
    [
        ("@tauri-apps/api/core", "invoke"),
        ("@tauri-apps/api/event", "event"),
        ("@tauri-apps/api/window", "window"),
        ("@tauri-apps/api/webview", "webview"),
        ("@tauri-apps/api/menu", "menu"),
        ("@tauri-apps/api/tray", "tray"),
        ("@tauri-apps/plugin-", "plugin"),
    ]
    .into_iter()
    .filter_map(move |(needle, api)| text.contains(needle).then_some(api))
    .collect()
}

fn frontend_coverage(api: &str) -> Coverage {
    match api {
        "plugin" => Coverage::Backend,
        "invoke" | "event" => Coverage::Backend,
        _ => Coverage::Unsupported,
    }
}

fn plugin_coverage(plugin: &str) -> Coverage {
    match plugin {
        "clipboard-manager" | "dialog" | "opener" | "haptics" | "share" => Coverage::Native,
        "fs" | "http" | "store" => Coverage::Backend,
        _ => Coverage::Unsupported,
    }
}

fn api_use(source: &str, api: &str, coverage: Coverage) -> ApiUse {
    ApiUse {
        source: source.to_string(),
        api: api.to_string(),
        coverage,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(config_name: &str, config: &str, dist: &str) -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("src-tauri")).unwrap();
        fs::create_dir_all(root.path().join(dist)).unwrap();
        fs::write(root.path().join("src-tauri").join(config_name), config).unwrap();
        fs::write(
            root.path().join(dist).join("crepus-bundle.json"),
            r#"{"entry":"main.crepus","files":{"main.crepus":"div\n  \"Hello\""}}"#,
        )
        .unwrap();
        root
    }

    #[test]
    fn reads_v1_dist_dir() {
        let root = project(
            "tauri.conf.json",
            r#"{"build":{"distDir":"../dist"}}"#,
            "dist",
        );
        let project = TauriProject::open(root.path()).unwrap();
        assert_eq!(project.version(), TauriVersion::V1);
        assert_eq!(project.bundle().unwrap().entry, "main.crepus");
    }

    #[test]
    fn reads_v2_frontend_dist() {
        let root = project(
            "tauri.conf.json",
            r#"{"build":{"frontendDist":"../dist"}}"#,
            "dist",
        );
        let project = TauriProject::open(root.path()).unwrap();
        assert_eq!(project.version(), TauriVersion::V2);
        assert_eq!(
            project.bundle().unwrap().files["main.crepus"],
            "div\n  \"Hello\""
        );
    }

    #[cfg(feature = "native")]
    #[test]
    fn renders_native_ir_from_static_bundle() {
        let root = project(
            "tauri.conf.json",
            r#"{"build":{"frontendDist":"../dist"}}"#,
            "dist",
        );
        let ir = TauriProject::open(root.path())
            .unwrap()
            .native_ir()
            .unwrap();
        assert_eq!(ir.root.len(), 1);
    }

    #[test]
    fn reads_json5_and_toml_configs() {
        let json5 = project(
            "tauri.conf.json5",
            "{ build: { frontendDist: '../dist' } }",
            "dist",
        );
        assert_eq!(
            TauriProject::open(json5.path()).unwrap().version(),
            TauriVersion::V2
        );
        let toml = project("Tauri.toml", "[build]\ndistDir = '../dist'\n", "dist");
        assert_eq!(
            TauriProject::open(toml.path()).unwrap().version(),
            TauriVersion::V1
        );
    }

    #[test]
    fn rejects_unsafe_bundle_paths() {
        let root = project(
            "tauri.conf.json",
            r#"{"build":{"frontendDist":"../dist"}}"#,
            "dist",
        );
        fs::write(
            root.path().join("dist/crepus-bundle.json"),
            r#"{"entry":"../index.crepus","files":{"../index.crepus":"div"}}"#,
        )
        .unwrap();
        assert!(TauriProject::open(root.path()).unwrap().bundle().is_err());
    }

    #[test]
    fn audit_classifies_plugins_commands_and_windows() {
        let root = project(
            "tauri.conf.json",
            r#"{"build":{"frontendDist":"../dist"},"app":{"windows":[{},{}]}}"#,
            "dist",
        );
        fs::write(
            root.path().join("src-tauri/Cargo.toml"),
            "tauri-plugin-dialog = \"2\"\ntauri-plugin-updater = \"2\"\n",
        )
        .unwrap();
        fs::write(
            root.path().join("src-tauri/lib.rs"),
            "#[tauri::command]\nfn greet() {}\n",
        )
        .unwrap();
        let report = TauriProject::open(root.path()).unwrap().audit();
        assert!(report
            .uses
            .iter()
            .any(|use_| use_.api == "plugin.dialog" && use_.coverage == Coverage::Native));
        assert!(report
            .uses
            .iter()
            .any(|use_| use_.api == "plugin.updater" && use_.coverage == Coverage::Unsupported));
        assert!(report
            .uses
            .iter()
            .any(|use_| use_.api == "command" && use_.coverage == Coverage::Backend));
        assert!(report.native_ready().is_err());
    }
}
