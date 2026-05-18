//! Browser extension capabilities - the core permission types.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A browser extension capability (permission).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    /// Content script injection
    ContentScript,
    /// Background service worker
    BackgroundScript,
    /// Local/sync storage access
    Storage,
    /// Port/message APIs between contexts
    Messaging,
    /// Clipboard read/write
    Clipboard,
    /// Notifications API
    Notifications,
    /// Tab management
    Tabs,
    /// History access
    History,
    /// Bookmarks access
    Bookmarks,
    /// Downloads management
    Downloads,
    /// Web requests interception
    WebRequest,
    /// Cookie access
    Cookies,
    /// Geolocation
    Geolocation,
    /// Identity/OAuth
    Identity,
    /// Active tab permission
    ActiveTab,
    /// Scripting API (MV3)
    Scripting,
    /// Alarms API
    Alarms,
    /// Native Messaging (connect to a local native host)
    NativeMessaging,
    /// Context menus
    ContextMenus,
    Sessions,
    WebNavigation,
    Search,
    Favicon,
    /// Host permission pattern
    HostPermission(String),
    /// Custom capability (for future extensions)
    Custom(String),
}

impl Capability {
    /// Convert to Chrome/Firefox manifest permission string.
    pub fn to_permission_string(&self) -> String {
        match self {
            Capability::ContentScript => "content_scripts".to_string(),
            Capability::BackgroundScript => "background".to_string(),
            Capability::Storage => "storage".to_string(),
            Capability::Messaging => "runtime".to_string(),
            Capability::Clipboard => "clipboardRead".to_string(),
            Capability::Notifications => "notifications".to_string(),
            Capability::Tabs => "tabs".to_string(),
            Capability::History => "history".to_string(),
            Capability::Bookmarks => "bookmarks".to_string(),
            Capability::Downloads => "downloads".to_string(),
            Capability::WebRequest => "webRequest".to_string(),
            Capability::Cookies => "cookies".to_string(),
            Capability::Geolocation => "geolocation".to_string(),
            Capability::Identity => "identity".to_string(),
            Capability::ActiveTab => "activeTab".to_string(),
            Capability::Scripting => "scripting".to_string(),
            Capability::Alarms => "alarms".to_string(),
            Capability::NativeMessaging => "nativeMessaging".to_string(),
            Capability::ContextMenus => "contextMenus".to_string(),
            Capability::Sessions => "sessions".to_string(),
            Capability::WebNavigation => "webNavigation".to_string(),
            Capability::Search => "search".to_string(),
            Capability::Favicon => "favicon".to_string(),
            Capability::HostPermission(pattern) => pattern.clone(),
            Capability::Custom(name) => name.clone(),
        }
    }

    /// Check if this is a host permission.
    pub fn is_host_permission(&self) -> bool {
        matches!(self, Capability::HostPermission(_))
    }

    /// Returns false for capabilities that are expressed as top-level manifest
    /// fields (background, content_scripts) rather than entries in "permissions".
    pub fn is_permission(&self) -> bool {
        !matches!(
            self,
            Capability::ContentScript | Capability::BackgroundScript
        )
    }
}

/// A set of capabilities for an extension.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    capabilities: HashSet<Capability>,
}

impl CapabilitySet {
    /// Create an empty capability set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a capability.
    pub fn add(&mut self, cap: Capability) {
        self.capabilities.insert(cap);
    }

    /// Check if a capability is present.
    pub fn has(&self, cap: &Capability) -> bool {
        self.capabilities.contains(cap)
    }

    /// Get all capabilities.
    pub fn iter(&self) -> impl Iterator<Item = &Capability> {
        self.capabilities.iter()
    }

    /// Get capabilities as permission strings for manifest.json.
    pub fn to_permissions(&self) -> Vec<String> {
        let mut permissions: Vec<String> = self
            .capabilities
            .iter()
            .filter(|c| !c.is_host_permission() && c.is_permission())
            .map(|c| c.to_permission_string())
            .collect();
        if self.capabilities.contains(&Capability::Clipboard) {
            permissions.push("clipboardWrite".to_string());
        }
        permissions
    }

    /// Get host permissions for manifest.json.
    pub fn to_host_permissions(&self) -> Vec<String> {
        self.capabilities
            .iter()
            .filter(|c| c.is_host_permission())
            .map(|c| c.to_permission_string())
            .collect()
    }

    /// Find capabilities in this set that are not in another set.
    pub fn missing_from(&self, other: &CapabilitySet) -> Vec<&Capability> {
        self.capabilities.iter().filter(|c| !other.has(c)).collect()
    }

    /// Merge another capability set into this one.
    pub fn merge(&mut self, other: &CapabilitySet) {
        for cap in &other.capabilities {
            self.capabilities.insert(cap.clone());
        }
    }

    /// Number of capabilities.
    pub fn len(&self) -> usize {
        self.capabilities.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }
}

impl FromIterator<Capability> for CapabilitySet {
    fn from_iter<I: IntoIterator<Item = Capability>>(iter: I) -> Self {
        Self {
            capabilities: iter.into_iter().collect(),
        }
    }
}
