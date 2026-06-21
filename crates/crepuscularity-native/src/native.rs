use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const NATIVE_CAPABILITIES: &[NativeCapability] = &[
    NativeCapability::ActionSheet,
    NativeCapability::AppLauncher,
    NativeCapability::App,
    NativeCapability::BackgroundRunner,
    NativeCapability::BarcodeScanner,
    NativeCapability::Browser,
    NativeCapability::Bluetooth,
    NativeCapability::Camera,
    NativeCapability::Clipboard,
    NativeCapability::Cookies,
    NativeCapability::Device,
    NativeCapability::Dialog,
    NativeCapability::Filesystem,
    NativeCapability::FileTransfer,
    NativeCapability::FileViewer,
    NativeCapability::Geolocation,
    NativeCapability::GoogleMaps,
    NativeCapability::Haptics,
    NativeCapability::Http,
    NativeCapability::InAppBrowser,
    NativeCapability::Keyboard,
    NativeCapability::LocalLlm,
    NativeCapability::LocalNotifications,
    NativeCapability::Motion,
    NativeCapability::Network,
    NativeCapability::Preferences,
    NativeCapability::PrivacyScreen,
    NativeCapability::PushNotifications,
    NativeCapability::ScreenOrientation,
    NativeCapability::ScreenReader,
    NativeCapability::Share,
    NativeCapability::SplashScreen,
    NativeCapability::StatusBar,
    NativeCapability::SystemBars,
    NativeCapability::Sync,
    NativeCapability::TextZoom,
    NativeCapability::Toast,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NativeCapability {
    ActionSheet,
    AppLauncher,
    App,
    BackgroundRunner,
    BarcodeScanner,
    Browser,
    Bluetooth,
    Camera,
    Clipboard,
    Cookies,
    Device,
    Dialog,
    Filesystem,
    FileTransfer,
    FileViewer,
    Geolocation,
    GoogleMaps,
    Haptics,
    Http,
    InAppBrowser,
    Keyboard,
    LocalLlm,
    LocalNotifications,
    Motion,
    Network,
    Preferences,
    PrivacyScreen,
    PushNotifications,
    ScreenOrientation,
    ScreenReader,
    Share,
    SplashScreen,
    StatusBar,
    SystemBars,
    Sync,
    TextZoom,
    Toast,
}

impl NativeCapability {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ActionSheet => "actionSheet",
            Self::AppLauncher => "appLauncher",
            Self::App => "app",
            Self::BackgroundRunner => "backgroundRunner",
            Self::BarcodeScanner => "barcodeScanner",
            Self::Browser => "browser",
            Self::Bluetooth => "bluetooth",
            Self::Camera => "camera",
            Self::Clipboard => "clipboard",
            Self::Cookies => "cookies",
            Self::Device => "device",
            Self::Dialog => "dialog",
            Self::Filesystem => "filesystem",
            Self::FileTransfer => "fileTransfer",
            Self::FileViewer => "fileViewer",
            Self::Geolocation => "geolocation",
            Self::GoogleMaps => "googleMaps",
            Self::Haptics => "haptics",
            Self::Http => "http",
            Self::InAppBrowser => "inAppBrowser",
            Self::Keyboard => "keyboard",
            Self::LocalLlm => "localLlm",
            Self::LocalNotifications => "localNotifications",
            Self::Motion => "motion",
            Self::Network => "network",
            Self::Preferences => "preferences",
            Self::PrivacyScreen => "privacyScreen",
            Self::PushNotifications => "pushNotifications",
            Self::ScreenOrientation => "screenOrientation",
            Self::ScreenReader => "screenReader",
            Self::Share => "share",
            Self::SplashScreen => "splashScreen",
            Self::StatusBar => "statusBar",
            Self::SystemBars => "systemBars",
            Self::Sync => "sync",
            Self::TextZoom => "textZoom",
            Self::Toast => "toast",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum NativeRequest {
    FilePicker(FilePickerRequest),
    Plugin(NativePluginRequest),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePickerRequest {
    pub accept: Vec<String>,
    pub multiple: bool,
}

impl FilePickerRequest {
    pub fn media() -> Self {
        Self {
            accept: vec!["image/*".to_string(), "video/*".to_string()],
            multiple: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativePluginRequest {
    pub capability: NativeCapability,
    pub method: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum NativeResponse {
    FilePicker(FilePickerResponse),
    Plugin(NativePluginResponse),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePickerResponse {
    pub files: Vec<PickedFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickedFile {
    pub name: String,
    pub mime_type: String,
    pub bytes: u64,
    pub data_base64: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativePluginResponse {
    pub capability: NativeCapability,
    pub value: Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn file_picker_request_is_portable_json() {
        let json = serde_json::to_value(NativeRequest::FilePicker(FilePickerRequest::media()))
            .expect("serialize request");
        assert_eq!(json["kind"], "filePicker");
        assert_eq!(json["accept"][0], "image/*");
        assert_eq!(json["multiple"], true);
    }

    #[test]
    fn native_capability_registry_has_unique_names() {
        let names = NATIVE_CAPABILITIES
            .iter()
            .map(|capability| capability.as_str())
            .collect::<HashSet<_>>();

        assert_eq!(NATIVE_CAPABILITIES.len(), 37);
        assert_eq!(names.len(), NATIVE_CAPABILITIES.len());
        assert!(names.contains("bluetooth"));
        assert!(names.contains("camera"));
        assert!(names.contains("clipboard"));
        assert!(names.contains("fileTransfer"));
        assert!(names.contains("sync"));
        assert!(names.contains("toast"));
    }

    #[test]
    fn generic_plugin_request_is_portable_json() {
        let request = NativeRequest::Plugin(NativePluginRequest {
            capability: NativeCapability::Share,
            method: "share".to_string(),
            payload: serde_json::json!({ "title": "Cupboard" }),
        });
        let json = serde_json::to_value(request).expect("serialize request");

        assert_eq!(json["kind"], "plugin");
        assert_eq!(json["capability"], "share");
        assert_eq!(json["method"], "share");
        assert_eq!(json["payload"]["title"], "Cupboard");
    }
}
