//! Optional `crepus-lite.toml` project file (guest entry path, flags, capabilities).

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Deserialize;

use crate::bridge::{Bridge, Capability};

const DEFAULT_CONFIG_FILENAME: &str = "crepus-lite.toml";

fn default_cap_true() -> bool {
    true
}

/// Optional toggles for plugin families (omitted keys default to **enabled**, except `bench`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CrepusLiteCapabilities {
    #[serde(default = "default_cap_true")]
    pub fs: bool,
    #[serde(default = "default_cap_true")]
    pub clipboard: bool,
    #[serde(default = "default_cap_true")]
    pub window: bool,
    #[serde(default = "default_cap_true")]
    pub host: bool,
    #[serde(default = "default_cap_true")]
    pub download: bool,
    /// Developer-only benchmark plugin (`"bench"`).  Off by default — opt in with
    /// `[capabilities] bench = true` in `crepus-lite.toml`.
    #[serde(default)]
    pub bench: bool,
}

impl Default for CrepusLiteCapabilities {
    fn default() -> Self {
        Self {
            fs: true,
            clipboard: true,
            window: true,
            host: true,
            download: true,
            bench: false,
        }
    }
}

/// Project-oriented settings for guest scripts. Parsed from TOML; safe defaults if the file is missing.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CrepusLiteConfig {
    /// Optional scripts (relative to the config base dir) concatenated in order **before** [`Self::guest_entry`].
    /// Used to load a shared library (for example `examples/capacitor/crepus-capacitor-api.js`) ahead of the guest body.
    #[serde(default)]
    pub guest_prelude: Vec<String>,
    /// Relative path (from config base dir) to a UTF-8 script executed with [`V8Host::eval`](crate::v8_host::V8Host).
    pub guest_entry: Option<String>,
    #[serde(default)]
    pub watch_guest: bool,
    /// Optional auto-rerender interval for apps that need native background progress updates.
    #[serde(default)]
    pub guest_refresh_interval_ms: Option<u64>,
    /// Which optional native capabilities are enabled (`core` and `app` are always on).
    #[serde(default)]
    pub capabilities: CrepusLiteCapabilities,
}

impl CrepusLiteConfig {
    /// Resolve config file path: `CREPUS_LITE_CONFIG` env, else `./crepus-lite.toml`.
    pub fn config_path() -> PathBuf {
        std::env::var("CREPUS_LITE_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_CONFIG_FILENAME))
    }

    /// Load from [`Self::config_path`] if it exists; otherwise [`Default::default`].
    pub fn load_discovered() -> Self {
        let path = Self::config_path();
        Self::load_from_path(&path).unwrap_or_default()
    }

    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let raw = std::fs::read_to_string(path).map_err(|e| format!("{e}"))?;
        toml::from_str(&raw).map_err(|e| format!("{e}"))
    }

    /// Capabilities used to build a [`Bridge`]: always `core` + `app`; optional families follow [`Self::capabilities`].
    pub fn allowed_capabilities(&self) -> HashSet<Capability> {
        let mut s = HashSet::new();
        s.insert(Capability::Core);
        s.insert(Capability::App);
        if self.capabilities.fs {
            s.insert(Capability::Fs);
        }
        if self.capabilities.clipboard {
            s.insert(Capability::Clipboard);
        }
        if self.capabilities.window {
            s.insert(Capability::Window);
        }
        if self.capabilities.host {
            s.insert(Capability::Host);
        }
        if self.capabilities.download {
            s.insert(Capability::Download);
        }
        if self.capabilities.bench {
            s.insert(Capability::Bench);
        }
        s
    }

    /// Bridge matching this config (stricter than [`Bridge::default_arc`] when capabilities are turned off).
    pub fn build_bridge(&self) -> Arc<Bridge> {
        Bridge::with_capabilities(self.allowed_capabilities())
    }

    /// Merged [`Self::guest_prelude`] text only (each file followed by `\n\n`). Empty when the list is empty.
    /// Returns `None` if any prelude path is missing or unreadable.
    pub fn guest_prelude_merged(&self, base: &Path) -> Option<String> {
        #[cfg(feature = "parallel")]
        {
            if self.guest_prelude.is_empty() {
                return Some(String::new());
            }
            use rayon::prelude::*;
            let mut chunks: Vec<(usize, String)> = self
                .guest_prelude
                .par_iter()
                .enumerate()
                .filter_map(|(i, pr)| std::fs::read_to_string(base.join(pr)).ok().map(|s| (i, s)))
                .collect();
            if chunks.len() != self.guest_prelude.len() {
                return None;
            }
            chunks.sort_by_key(|(i, _)| *i);
            let mut s = String::new();
            for (_, chunk) in chunks {
                s.push_str(&chunk);
                s.push_str("\n\n");
            }
            Some(s)
        }
        #[cfg(not(feature = "parallel"))]
        {
            let mut s = String::new();
            for pr in &self.guest_prelude {
                let chunk = std::fs::read_to_string(base.join(pr)).ok()?;
                s.push_str(&chunk);
                s.push_str("\n\n");
            }
            Some(s)
        }
    }

    /// Like [`Self::guest_source`] but uses `merged_prelude` from [`Self::guest_prelude_merged`] (or any identical
    /// bytes) so callers can read prelude files once for many configs. `merged_prelude` must match this config’s
    /// prelude list contents.
    pub fn guest_source_with_merged_prelude(
        &self,
        base: &Path,
        merged_prelude: &str,
    ) -> Option<String> {
        let rel = self.guest_entry.as_ref()?;
        let body = std::fs::read_to_string(base.join(rel)).ok()?;
        let mut out = merged_prelude.to_string();
        out.push_str(&body);
        Some(out)
    }

    /// Read guest file relative to `base` (usually [`std::env::current_dir`]), after any [`Self::guest_prelude`]
    /// files in order. Returns `None` if `guest_entry` is unset, any prelude path is missing, or the entry file
    /// is unreadable.
    pub fn guest_source(&self, base: &Path) -> Option<String> {
        let rel = self.guest_entry.as_ref()?;
        let prelude = self.guest_prelude_merged(base)?;
        let body = std::fs::read_to_string(base.join(rel)).ok()?;
        let mut out = prelude;
        out.push_str(&body);
        Some(out)
    }

    /// Absolute path to the guest script when it exists on disk (`base` + `guest_entry`, then canonicalize).
    pub fn resolved_guest_path(&self, base: &Path) -> Option<PathBuf> {
        let rel = self.guest_entry.as_ref()?;
        let p = base.join(rel);
        p.canonicalize().ok()
    }

    /// All guest file paths to watch for hot reload (entry + preludes).
    ///
    /// Paths are resolved relative to `base` and are returned only when the target exists on disk.
    pub fn guest_watch_paths(&self, base: &Path) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let mut seen = HashSet::new();
        if let Some(rel) = self.guest_entry.as_ref() {
            let path = base.join(rel);
            if path.exists() {
                let path = path.canonicalize().unwrap_or(path);
                if seen.insert(path.clone()) {
                    paths.push(path);
                }
            }
        }
        for rel in &self.guest_prelude {
            let path = base.join(rel);
            if path.exists() {
                let path = path.canonicalize().unwrap_or(path);
                if seen.insert(path.clone()) {
                    paths.push(path);
                }
            }
        }
        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_default_all_optional_on() {
        let c = CrepusLiteConfig::default();
        // fs / clipboard / window / host / download default on; bench defaults OFF (developer opt-in).
        assert!(
            c.capabilities.fs
                && c.capabilities.clipboard
                && c.capabilities.window
                && c.capabilities.host
                && c.capabilities.download
        );
        assert!(!c.capabilities.bench);
        // Core + App + Fs + Clipboard + Window + Host + Download = 7; Bench excluded by default.
        assert_eq!(c.allowed_capabilities().len(), 7);
    }

    #[test]
    fn capabilities_disable_fs() {
        let raw = r#"
        guest_entry = "x.js"
        [capabilities]
        fs = false
        "#;
        let c: CrepusLiteConfig = toml::from_str(raw).unwrap();
        assert!(!c.capabilities.fs);
        assert!(c.capabilities.clipboard);
        let a = c.allowed_capabilities();
        assert!(a.contains(&Capability::Core));
        assert!(!a.contains(&Capability::Fs));
    }

    #[test]
    fn guest_prelude_toml() {
        let raw = r#"
        guest_prelude = ["a.js", "b.js"]
        guest_entry = "main.js"
        "#;
        let c: CrepusLiteConfig = toml::from_str(raw).unwrap();
        assert_eq!(
            c.guest_prelude,
            vec!["a.js".to_string(), "b.js".to_string()]
        );
        assert_eq!(c.guest_entry.as_deref(), Some("main.js"));
    }

    #[test]
    fn guest_source_concatenates_prelude() {
        use std::fs;
        let dir =
            std::env::temp_dir().join(format!("crepus-lite-guest-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("p.js"), "const __pre = 40;").unwrap();
        fs::write(dir.join("g.js"), "const __sum = __pre + 2;").unwrap();
        let c = CrepusLiteConfig {
            guest_prelude: vec!["p.js".into()],
            guest_entry: Some("g.js".into()),
            ..Default::default()
        };
        let s = c.guest_source(&dir).unwrap();
        assert!(s.contains("const __pre = 40;"));
        assert!(s.contains("const __sum = __pre + 2;"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn guest_source_matches_split_prelude_and_entry() {
        use std::fs;
        let dir =
            std::env::temp_dir().join(format!("crepus-lite-guest-split-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("a.js"), "aa").unwrap();
        fs::write(dir.join("b.js"), "bb").unwrap();
        fs::write(dir.join("main.js"), "cc").unwrap();
        let c = CrepusLiteConfig {
            guest_prelude: vec!["a.js".into(), "b.js".into()],
            guest_entry: Some("main.js".into()),
            ..Default::default()
        };
        let merged = c.guest_prelude_merged(&dir).unwrap();
        let with = c.guest_source_with_merged_prelude(&dir, &merged).unwrap();
        assert_eq!(with, c.guest_source(&dir).unwrap());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn guest_watch_paths_include_entry_and_preludes() {
        use std::fs;
        let dir = std::env::temp_dir().join(format!("crepus-lite-watch-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("a.js"), "aa").unwrap();
        fs::write(dir.join("main.js"), "cc").unwrap();
        let c = CrepusLiteConfig {
            guest_prelude: vec!["a.js".into()],
            guest_entry: Some("main.js".into()),
            ..Default::default()
        };
        let paths = c.guest_watch_paths(&dir);
        assert_eq!(paths.len(), 2);
        assert!(paths.iter().any(|p| p.ends_with("main.js")));
        assert!(paths.iter().any(|p| p.ends_with("a.js")));
        let _ = fs::remove_dir_all(&dir);
    }
}
