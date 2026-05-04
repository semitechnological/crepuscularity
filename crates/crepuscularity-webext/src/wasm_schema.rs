pub const PUBLIC_MV3_NAMESPACES: &[&str] = &[
    "accessibilityFeatures",
    "action",
    "alarms",
    "audio",
    "bookmarks",
    "browsingData",
    "certificateProvider",
    "commands",
    "contentSettings",
    "contextMenus",
    "cookies",
    "debugger",
    "declarativeContent",
    "declarativeNetRequest",
    "desktopCapture",
    "devtools.inspectedWindow",
    "devtools.network",
    "devtools.panels",
    "documentScan",
    "dom",
    "downloads",
    "enterprise.deviceAttributes",
    "enterprise.hardwarePlatform",
    "enterprise.login",
    "enterprise.networkingAttributes",
    "enterprise.platformKeys",
    "events",
    "extension",
    "extensionTypes",
    "fileBrowserHandler",
    "fileSystemProvider",
    "fontSettings",
    "gcm",
    "history",
    "i18n",
    "identity",
    "idle",
    "input.ime",
    "instanceID",
    "loginState",
    "management",
    "notifications",
    "offscreen",
    "omnibox",
    "pageCapture",
    "permissions",
    "platformKeys",
    "power",
    "printerProvider",
    "printing",
    "printingMetrics",
    "privacy",
    "proxy",
    "readingList",
    "runtime",
    "scripting",
    "search",
    "sessions",
    "sidePanel",
    "storage",
    "system.cpu",
    "system.display",
    "system.memory",
    "system.storage",
    "systemLog",
    "tabCapture",
    "tabGroups",
    "tabs",
    "topSites",
    "tts",
    "ttsEngine",
    "types",
    "userScripts",
    "vpnProvider",
    "wallpaper",
    "webAuthenticationProxy",
    "webNavigation",
    "webRequest",
    "windows",
];

#[cfg(test)]
mod tests {
    use super::PUBLIC_MV3_NAMESPACES;

    #[test]
    fn schema_snapshot_contains_core_namespaces() {
        for namespace in [
            "runtime",
            "storage",
            "tabs",
            "scripting",
            "commands",
            "webRequest",
        ] {
            assert!(PUBLIC_MV3_NAMESPACES.contains(&namespace));
        }
    }

    #[test]
    fn schema_snapshot_excludes_internal_namespaces() {
        for namespace in PUBLIC_MV3_NAMESPACES {
            assert!(!namespace.starts_with("test."));
            assert!(!namespace.contains("internal"));
        }
    }
}
