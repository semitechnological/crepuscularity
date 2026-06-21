use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const NATIVE_CAPABILITIES: &[NativeCapability] = &[
    NativeCapability::ActionSheet,
    NativeCapability::AccessibilityInfo,
    NativeCapability::AppLauncher,
    NativeCapability::App,
    NativeCapability::AppState,
    NativeCapability::Appearance,
    NativeCapability::Audio,
    NativeCapability::Authentication,
    NativeCapability::BackgroundRunner,
    NativeCapability::BarcodeScanner,
    NativeCapability::Battery,
    NativeCapability::Biometrics,
    NativeCapability::Browser,
    NativeCapability::Bluetooth,
    NativeCapability::Calendar,
    NativeCapability::Camera,
    NativeCapability::Clipboard,
    NativeCapability::Contacts,
    NativeCapability::Cookies,
    NativeCapability::Device,
    NativeCapability::Dialog,
    NativeCapability::Dimensions,
    NativeCapability::DocumentPicker,
    NativeCapability::Filesystem,
    NativeCapability::FileTransfer,
    NativeCapability::FileViewer,
    NativeCapability::Geolocation,
    NativeCapability::GoogleMaps,
    NativeCapability::Haptics,
    NativeCapability::Health,
    NativeCapability::Http,
    NativeCapability::ImagePicker,
    NativeCapability::InAppBrowser,
    NativeCapability::Keyboard,
    NativeCapability::Linking,
    NativeCapability::LocalLlm,
    NativeCapability::LocalNotifications,
    NativeCapability::Motion,
    NativeCapability::Network,
    NativeCapability::Nfc,
    NativeCapability::Payments,
    NativeCapability::Permissions,
    NativeCapability::Phone,
    NativeCapability::PhotoLibrary,
    NativeCapability::Platform,
    NativeCapability::Preferences,
    NativeCapability::PrivacyScreen,
    NativeCapability::PushNotifications,
    NativeCapability::ScreenOrientation,
    NativeCapability::ScreenReader,
    NativeCapability::Share,
    NativeCapability::SplashScreen,
    NativeCapability::StatusBar,
    NativeCapability::SystemBars,
    NativeCapability::Settings,
    NativeCapability::Shortcuts,
    NativeCapability::Sms,
    NativeCapability::SecureStorage,
    NativeCapability::Sync,
    NativeCapability::TextZoom,
    NativeCapability::Toast,
    NativeCapability::Vibration,
    NativeCapability::Video,
    NativeCapability::Wallet,
    NativeCapability::Widgets,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NativeCapability {
    ActionSheet,
    AccessibilityInfo,
    AppLauncher,
    App,
    AppState,
    Appearance,
    Audio,
    Authentication,
    BackgroundRunner,
    BarcodeScanner,
    Battery,
    Biometrics,
    Browser,
    Bluetooth,
    Calendar,
    Camera,
    Clipboard,
    Contacts,
    Cookies,
    Device,
    Dialog,
    Dimensions,
    DocumentPicker,
    Filesystem,
    FileTransfer,
    FileViewer,
    Geolocation,
    GoogleMaps,
    Haptics,
    Health,
    Http,
    ImagePicker,
    InAppBrowser,
    Keyboard,
    Linking,
    LocalLlm,
    LocalNotifications,
    Motion,
    Network,
    Nfc,
    Payments,
    Permissions,
    Phone,
    PhotoLibrary,
    Platform,
    Preferences,
    PrivacyScreen,
    PushNotifications,
    ScreenOrientation,
    ScreenReader,
    Share,
    SplashScreen,
    StatusBar,
    SystemBars,
    Settings,
    Shortcuts,
    Sms,
    SecureStorage,
    Sync,
    TextZoom,
    Toast,
    Vibration,
    Video,
    Wallet,
    Widgets,
}

impl NativeCapability {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ActionSheet => "actionSheet",
            Self::AccessibilityInfo => "accessibilityInfo",
            Self::AppLauncher => "appLauncher",
            Self::App => "app",
            Self::AppState => "appState",
            Self::Appearance => "appearance",
            Self::Audio => "audio",
            Self::Authentication => "authentication",
            Self::BackgroundRunner => "backgroundRunner",
            Self::BarcodeScanner => "barcodeScanner",
            Self::Battery => "battery",
            Self::Biometrics => "biometrics",
            Self::Browser => "browser",
            Self::Bluetooth => "bluetooth",
            Self::Calendar => "calendar",
            Self::Camera => "camera",
            Self::Clipboard => "clipboard",
            Self::Contacts => "contacts",
            Self::Cookies => "cookies",
            Self::Device => "device",
            Self::Dialog => "dialog",
            Self::Dimensions => "dimensions",
            Self::DocumentPicker => "documentPicker",
            Self::Filesystem => "filesystem",
            Self::FileTransfer => "fileTransfer",
            Self::FileViewer => "fileViewer",
            Self::Geolocation => "geolocation",
            Self::GoogleMaps => "googleMaps",
            Self::Haptics => "haptics",
            Self::Health => "health",
            Self::Http => "http",
            Self::ImagePicker => "imagePicker",
            Self::InAppBrowser => "inAppBrowser",
            Self::Keyboard => "keyboard",
            Self::Linking => "linking",
            Self::LocalLlm => "localLlm",
            Self::LocalNotifications => "localNotifications",
            Self::Motion => "motion",
            Self::Network => "network",
            Self::Nfc => "nfc",
            Self::Payments => "payments",
            Self::Permissions => "permissions",
            Self::Phone => "phone",
            Self::PhotoLibrary => "photoLibrary",
            Self::Platform => "platform",
            Self::Preferences => "preferences",
            Self::PrivacyScreen => "privacyScreen",
            Self::PushNotifications => "pushNotifications",
            Self::ScreenOrientation => "screenOrientation",
            Self::ScreenReader => "screenReader",
            Self::Share => "share",
            Self::SplashScreen => "splashScreen",
            Self::StatusBar => "statusBar",
            Self::SystemBars => "systemBars",
            Self::Settings => "settings",
            Self::Shortcuts => "shortcuts",
            Self::Sms => "sms",
            Self::SecureStorage => "secureStorage",
            Self::Sync => "sync",
            Self::TextZoom => "textZoom",
            Self::Toast => "toast",
            Self::Vibration => "vibration",
            Self::Video => "video",
            Self::Wallet => "wallet",
            Self::Widgets => "widgets",
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

        assert_eq!(NATIVE_CAPABILITIES.len(), 65);
        assert_eq!(names.len(), NATIVE_CAPABILITIES.len());
        assert!(names.contains("accessibilityInfo"));
        assert!(names.contains("appState"));
        assert!(names.contains("appearance"));
        assert!(names.contains("authentication"));
        assert!(names.contains("battery"));
        assert!(names.contains("biometrics"));
        assert!(names.contains("bluetooth"));
        assert!(names.contains("calendar"));
        assert!(names.contains("camera"));
        assert!(names.contains("clipboard"));
        assert!(names.contains("contacts"));
        assert!(names.contains("dimensions"));
        assert!(names.contains("documentPicker"));
        assert!(names.contains("fileTransfer"));
        assert!(names.contains("health"));
        assert!(names.contains("imagePicker"));
        assert!(names.contains("linking"));
        assert!(names.contains("nfc"));
        assert!(names.contains("payments"));
        assert!(names.contains("permissions"));
        assert!(names.contains("photoLibrary"));
        assert!(names.contains("platform"));
        assert!(names.contains("secureStorage"));
        assert!(names.contains("settings"));
        assert!(names.contains("sync"));
        assert!(names.contains("toast"));
        assert!(names.contains("vibration"));
        assert!(names.contains("wallet"));
        assert!(names.contains("widgets"));
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
