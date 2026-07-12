use std::fs;
use std::path::{Path, PathBuf};

use crepuscularity_web::{parse_bundle, Bundle};
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
}

impl TauriProject {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, String> {
        let root = root.as_ref().canonicalize().map_err(|e| e.to_string())?;
        let config = [
            root.join("src-tauri/tauri.conf.json"),
            root.join("tauri.conf.json"),
        ]
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| format!("no tauri.conf.json under {}", root.display()))?;
        let text = fs::read_to_string(&config).map_err(|e| e.to_string())?;
        let value: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
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

    pub fn bundle(&self) -> Result<Bundle, String> {
        let path = self.frontend_dist.join("crepus-bundle.json");
        let json = fs::read_to_string(&path)
            .map_err(|_| format!("{} is required for native conversion", path.display()))?;
        parse_bundle(&json).map_err(|e| e.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn project(config: &str, dist: &str) -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("src-tauri")).unwrap();
        fs::create_dir_all(root.path().join(dist)).unwrap();
        fs::write(root.path().join("src-tauri/tauri.conf.json"), config).unwrap();
        fs::write(
            root.path().join(dist).join("crepus-bundle.json"),
            r#"{"entry":"main.crepus","files":{"main.crepus":"div\n  \"Hello\""}}"#,
        )
        .unwrap();
        root
    }

    #[test]
    fn reads_v1_dist_dir() {
        let root = project(r#"{"build":{"distDir":"../dist"}}"#, "dist");
        let project = TauriProject::open(root.path()).unwrap();
        assert_eq!(project.version(), TauriVersion::V1);
        assert_eq!(project.bundle().unwrap().entry, "main.crepus");
    }

    #[test]
    fn reads_v2_frontend_dist() {
        let root = project(r#"{"build":{"frontendDist":"../dist"}}"#, "dist");
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
        let root = project(r#"{"build":{"frontendDist":"../dist"}}"#, "dist");
        let ir = TauriProject::open(root.path())
            .unwrap()
            .native_ir()
            .unwrap();
        assert_eq!(ir.root.len(), 1);
    }
}
