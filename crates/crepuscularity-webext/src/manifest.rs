//! Extension manifest parsing and generation (.crex format).

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::capabilities::{Capability, CapabilitySet};

/// Error type for manifest operations.
#[derive(Debug)]
pub enum ManifestError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    Serialize(toml::ser::Error),
}

impl From<std::io::Error> for ManifestError {
    fn from(e: std::io::Error) -> Self {
        ManifestError::Io(e)
    }
}

impl From<toml::de::Error> for ManifestError {
    fn from(e: toml::de::Error) -> Self {
        ManifestError::Parse(e)
    }
}

impl From<toml::ser::Error> for ManifestError {
    fn from(e: toml::ser::Error) -> Self {
        ManifestError::Serialize(e)
    }
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::Io(e) => write!(f, "IO error: {e}"),
            ManifestError::Parse(e) => write!(f, "Parse error: {e}"),
            ManifestError::Serialize(e) => write!(f, "Serialize error: {e}"),
        }
    }
}

impl std::error::Error for ManifestError {}

/// The extension manifest (.crex file).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub extension: ExtensionInfo,
    #[serde(default)]
    pub capabilities: CapabilitiesSection,
    #[serde(default)]
    pub content_scripts: Vec<ContentScriptEntry>,
    #[serde(default)]
    pub plugins: HashMap<String, PluginEntry>,
}

/// Basic extension information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtensionInfo {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
}

/// Capabilities section in the manifest.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CapabilitiesSection {
    #[serde(default, rename = "content-script")]
    pub content_script: bool,
    #[serde(default, rename = "background-script")]
    pub background_script: bool,
    #[serde(default)]
    pub storage: bool,
    #[serde(default)]
    pub messaging: bool,
    #[serde(default)]
    pub clipboard: bool,
    #[serde(default)]
    pub notifications: bool,
    #[serde(default)]
    pub tabs: bool,
    #[serde(default)]
    pub history: bool,
    #[serde(default)]
    pub bookmarks: bool,
    #[serde(default)]
    pub downloads: bool,
    #[serde(default, rename = "web-request")]
    pub web_request: bool,
    #[serde(default)]
    pub cookies: bool,
    #[serde(default)]
    pub geolocation: bool,
    #[serde(default)]
    pub identity: bool,
    #[serde(default, rename = "active-tab")]
    pub active_tab: bool,
    #[serde(default)]
    pub scripting: bool,
    #[serde(default)]
    pub alarms: bool,
    #[serde(default, rename = "context-menus")]
    pub context_menus: bool,
    #[serde(default, rename = "host-permissions")]
    pub host_permissions: Vec<String>,
}

/// Content script entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContentScriptEntry {
    pub matches: Vec<String>,
    #[serde(default)]
    pub js: Vec<String>,
    #[serde(default)]
    pub css: Vec<String>,
    #[serde(default)]
    pub run_at: Option<String>,
}

/// Plugin entry for custom functionality.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginEntry {
    pub path: String,
    #[serde(rename = "type")]
    pub plugin_type: String,
}

impl ExtensionManifest {
    /// Load manifest from a .crex file.
    pub fn load(path: &Path) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(path)?;
        let manifest: ExtensionManifest = toml::from_str(&content)?;
        Ok(manifest)
    }

    /// Save manifest to a .crex file.
    pub fn save(&self, path: &Path) -> Result<(), ManifestError> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Convert capabilities section to a CapabilitySet.
    pub fn to_capability_set(&self) -> CapabilitySet {
        let mut set = CapabilitySet::new();

        if self.capabilities.content_script {
            set.add(Capability::ContentScript);
        }
        if self.capabilities.background_script {
            set.add(Capability::BackgroundScript);
        }
        if self.capabilities.storage {
            set.add(Capability::Storage);
        }
        if self.capabilities.messaging {
            set.add(Capability::Messaging);
        }
        if self.capabilities.clipboard {
            set.add(Capability::Clipboard);
        }
        if self.capabilities.notifications {
            set.add(Capability::Notifications);
        }
        if self.capabilities.tabs {
            set.add(Capability::Tabs);
        }
        if self.capabilities.history {
            set.add(Capability::History);
        }
        if self.capabilities.bookmarks {
            set.add(Capability::Bookmarks);
        }
        if self.capabilities.downloads {
            set.add(Capability::Downloads);
        }
        if self.capabilities.web_request {
            set.add(Capability::WebRequest);
        }
        if self.capabilities.cookies {
            set.add(Capability::Cookies);
        }
        if self.capabilities.geolocation {
            set.add(Capability::Geolocation);
        }
        if self.capabilities.identity {
            set.add(Capability::Identity);
        }
        if self.capabilities.active_tab {
            set.add(Capability::ActiveTab);
        }
        if self.capabilities.scripting {
            set.add(Capability::Scripting);
        }
        if self.capabilities.alarms {
            set.add(Capability::Alarms);
        }
        if self.capabilities.context_menus {
            set.add(Capability::ContextMenus);
        }

        for pattern in &self.capabilities.host_permissions {
            set.add(Capability::HostPermission(pattern.clone()));
        }

        set
    }

    /// Generate a Chrome/Firefox manifest.json from this manifest.
    pub fn to_browser_manifest(&self) -> serde_json::Value {
        let caps = self.to_capability_set();

        let mut manifest = serde_json::json!({
            "manifest_version": 3,
            "name": self.extension.name,
            "version": self.extension.version,
        });

        if let Some(desc) = &self.extension.description {
            manifest["description"] = serde_json::json!(desc);
        }

        // Permissions
        let permissions = caps.to_permissions();
        if !permissions.is_empty() {
            manifest["permissions"] = serde_json::json!(permissions);
        }

        // Host permissions
        let host_permissions = caps.to_host_permissions();
        if !host_permissions.is_empty() {
            manifest["host_permissions"] = serde_json::json!(host_permissions);
        }

        // Content scripts
        if !self.content_scripts.is_empty() {
            let scripts: Vec<serde_json::Value> = self
                .content_scripts
                .iter()
                .map(|cs| {
                    let mut entry = serde_json::json!({
                        "matches": cs.matches,
                    });
                    if !cs.js.is_empty() {
                        entry["js"] = serde_json::json!(cs.js);
                    }
                    if !cs.css.is_empty() {
                        entry["css"] = serde_json::json!(cs.css);
                    }
                    if let Some(run_at) = &cs.run_at {
                        entry["run_at"] = serde_json::json!(run_at);
                    }
                    entry
                })
                .collect();
            manifest["content_scripts"] = serde_json::json!(scripts);
        }

        manifest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let toml = r#"
[extension]
name = "test-ext"
version = "1.0.0"

[capabilities]
storage = true
messaging = true
host-permissions = ["https://example.com/*"]
"#;

        let manifest: ExtensionManifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.extension.name, "test-ext");
        assert!(manifest.capabilities.storage);
        assert!(manifest.capabilities.messaging);
        assert_eq!(manifest.capabilities.host_permissions.len(), 1);
    }

    #[test]
    fn test_to_browser_manifest() {
        let manifest = ExtensionManifest {
            extension: ExtensionInfo {
                name: "Test".to_string(),
                version: "1.0.0".to_string(),
                description: Some("A test extension".to_string()),
                author: None,
                homepage: None,
            },
            capabilities: CapabilitiesSection {
                storage: true,
                tabs: true,
                host_permissions: vec!["https://*.example.com/*".to_string()],
                ..Default::default()
            },
            content_scripts: vec![],
            plugins: HashMap::new(),
        };

        let browser = manifest.to_browser_manifest();
        assert_eq!(browser["manifest_version"], 3);
        assert_eq!(browser["name"], "Test");
    }
}
