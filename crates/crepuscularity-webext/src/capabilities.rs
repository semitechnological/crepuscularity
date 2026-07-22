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
        self.capabilities.extend(other.capabilities.iter().cloned());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_from() {
        let set1: CapabilitySet = vec![Capability::Storage, Capability::Tabs].into_iter().collect();
        let set2: CapabilitySet = vec![Capability::Storage].into_iter().collect();

        // Elements in set1 missing from set2 (Tabs)
        let missing = set1.missing_from(&set2);
        assert_eq!(missing.len(), 1);
        assert!(missing.contains(&&Capability::Tabs));

        // Elements in set2 missing from set1 (None)
        let missing_from_1 = set2.missing_from(&set1);
        assert!(missing_from_1.is_empty());

        // Missing from empty set
        let empty_set: CapabilitySet = vec![].into_iter().collect();
        let missing_from_empty = set1.missing_from(&empty_set);
        assert_eq!(missing_from_empty.len(), 2);
        assert!(missing_from_empty.contains(&&Capability::Storage));
        assert!(missing_from_empty.contains(&&Capability::Tabs));
    }

    #[test]
    fn test_to_permission_string() {
        assert_eq!(
            Capability::ContentScript.to_permission_string(),
            "content_scripts"
        );
        assert_eq!(
            Capability::BackgroundScript.to_permission_string(),
            "background"
        );
        assert_eq!(Capability::Storage.to_permission_string(), "storage");
        assert_eq!(Capability::Messaging.to_permission_string(), "runtime");
        assert_eq!(
            Capability::Clipboard.to_permission_string(),
            "clipboardRead"
        );
        assert_eq!(
            Capability::Notifications.to_permission_string(),
            "notifications"
        );
        assert_eq!(Capability::Tabs.to_permission_string(), "tabs");
        assert_eq!(Capability::History.to_permission_string(), "history");
        assert_eq!(Capability::Bookmarks.to_permission_string(), "bookmarks");
        assert_eq!(Capability::Downloads.to_permission_string(), "downloads");
        assert_eq!(Capability::WebRequest.to_permission_string(), "webRequest");
        assert_eq!(Capability::Cookies.to_permission_string(), "cookies");
        assert_eq!(
            Capability::Geolocation.to_permission_string(),
            "geolocation"
        );
        assert_eq!(Capability::Identity.to_permission_string(), "identity");
        assert_eq!(Capability::ActiveTab.to_permission_string(), "activeTab");
        assert_eq!(Capability::Scripting.to_permission_string(), "scripting");
        assert_eq!(Capability::Alarms.to_permission_string(), "alarms");
        assert_eq!(
            Capability::NativeMessaging.to_permission_string(),
            "nativeMessaging"
        );
        assert_eq!(
            Capability::ContextMenus.to_permission_string(),
            "contextMenus"
        );
        assert_eq!(Capability::Sessions.to_permission_string(), "sessions");
        assert_eq!(
            Capability::WebNavigation.to_permission_string(),
            "webNavigation"
        );
        assert_eq!(Capability::Search.to_permission_string(), "search");
        assert_eq!(Capability::Favicon.to_permission_string(), "favicon");
        assert_eq!(
            Capability::HostPermission("https://*.example.com/*".to_string())
                .to_permission_string(),
            "https://*.example.com/*"
        );
        assert_eq!(
            Capability::Custom("custom_perm".to_string()).to_permission_string(),
            "custom_perm"
        );
    }
}
