//! `crepus native` — Native mobile applications for iOS and Android.
//!
//! Scaffold and build native iOS (SwiftUI) and Android (Jetpack Compose) apps
//! that use **View IR** (`crepuscularity-native::render_template_to_ir`) to
//! render `.crepus` templates.
//!
//! The scaffold is the same source tree as `examples/native-shells/` — a
//! SwiftPM package under `<dir>/ios/` and a Gradle module under
//! `<dir>/android/`, sharing a common `fixture.json`. We *don't* embed the
//! Gradle wrapper jar; users run `gradle wrapper --gradle-version 8.10`
//! (or just open the project in Android Studio, which regenerates it
//! automatically) before the first `./gradlew` invocation.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use console::style;
use crepuscularity_core::context::{TemplateContext, TemplateValue};
use crepuscularity_core::{DriverCache, Fingerprint};
use crepuscularity_native::{
    generate_native_source, render_component_file_to_ir, render_from_files, render_template_to_ir,
    to_json, to_json_pretty, NativeCodegenTarget, ANDROID_ACCESSIBILITY_INFO, ANDROID_ACTION_SHEET,
    ANDROID_APP, ANDROID_APPEARANCE, ANDROID_APP_STATE, ANDROID_BATTERY, ANDROID_BIOMETRICS,
    ANDROID_BLUETOOTH, ANDROID_BLUETOOTH_BRIDGE, ANDROID_BROWSER, ANDROID_CALENDAR, ANDROID_CAMERA,
    ANDROID_CLIPBOARD, ANDROID_CONTACTS, ANDROID_DEEP_LINKS, ANDROID_DEVICE, ANDROID_DIALOG,
    ANDROID_DIMENSIONS, ANDROID_DOCUMENT_PICKER, ANDROID_GEOLOCATION, ANDROID_GEOLOCATION_BRIDGE,
    ANDROID_HAPTICS, ANDROID_IMAGE_PICKER, ANDROID_IN_APP_BROWSER, ANDROID_KEYBOARD,
    ANDROID_LOCAL_NOTIFICATIONS, ANDROID_MICROPHONE, ANDROID_NETWORK, ANDROID_PERMISSIONS,
    ANDROID_PHOTO_LIBRARY, ANDROID_PREFERENCES, ANDROID_PRIVACY_SCREEN,
    ANDROID_SCHEDULED_NOTIFICATION_RECEIVER, ANDROID_SCREEN_ORIENTATION, ANDROID_SECURE_STORAGE,
    ANDROID_SENSORS, ANDROID_SENSORS_BRIDGE, ANDROID_SETTINGS, ANDROID_SHARE, ANDROID_SYSTEM_BARS,
    ANDROID_TOAST, ANDROID_VIDEO, IOS_ACCESSIBILITY_INFO, IOS_ACTION_SHEET,
    IOS_ACTION_SHEET_BRIDGE, IOS_APP, IOS_APPEARANCE, IOS_APP_STATE, IOS_BATTERY, IOS_BIOMETRICS,
    IOS_BLUETOOTH, IOS_BLUETOOTH_BRIDGE, IOS_BROWSER, IOS_CALENDAR, IOS_CAMERA, IOS_CAMERA_BRIDGE,
    IOS_CLIPBOARD, IOS_CLIPBOARD_BRIDGE, IOS_CONTACTS, IOS_DEEP_LINKS, IOS_DEVICE, IOS_DIALOG,
    IOS_DIALOG_BRIDGE, IOS_DIMENSIONS, IOS_DOCUMENT_PICKER, IOS_GEOLOCATION,
    IOS_GEOLOCATION_BRIDGE, IOS_HAPTICS, IOS_IMAGE_PICKER, IOS_IMAGE_PICKER_BRIDGE,
    IOS_IN_APP_BROWSER, IOS_KEYBOARD, IOS_LOCAL_NOTIFICATIONS, IOS_MICROPHONE, IOS_NETWORK,
    IOS_PERMISSIONS, IOS_PHOTO_LIBRARY, IOS_PHOTO_LIBRARY_BRIDGE, IOS_PREFERENCES,
    IOS_PRIVACY_SCREEN, IOS_SCREEN_ORIENTATION, IOS_SECURE_STORAGE, IOS_SENSORS,
    IOS_SENSORS_BRIDGE, IOS_SETTINGS, IOS_SHARE, IOS_SYSTEM_BARS, IOS_TOAST, IOS_VIDEO,
    IOS_VIDEO_BRIDGE,
};
use serde::Deserialize;
use serde_json::Value;

use crate::cli::{
    NativeBuildCommands, NativeCodegenCliArgs, NativeCodegenPlatformArg, NativeCommands,
    NativeExtensionCommands, NativeIrArgs, NativeRunCommands, NativeSyncArgs,
};
use crate::dispatch::native_ios_target;
use crate::ui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IosBuildTarget {
    Simulator,
    Device,
}

pub fn execute(cmd: NativeCommands) {
    match cmd {
        NativeCommands::New { name } => scaffold_native_app(&name),
        NativeCommands::Add { capability, dir } => {
            add_capability(&capability, &dir).unwrap_or_else(|e| ui::error(&e))
        }
        NativeCommands::Extension { extension } => handle_extension(extension),
        NativeCommands::Ir { args } => run_ir_from(args),
        NativeCommands::Sync { args } => {
            if let Err(e) = sync_from_cli(args) {
                ui::error(&e);
            }
        }
        NativeCommands::Codegen { args } => {
            if let Err(e) = codegen_from_cli(args) {
                ui::error(&e);
            }
        }
        NativeCommands::Build { platform } => handle_build(platform),
        NativeCommands::Run { platform } => handle_run(platform),
    }
}

struct CapabilitySpec {
    name: &'static str,
    aliases: &'static [&'static str],
    cargo_feature: &'static str,
    android_manifest: &'static str,
    ios_project: &'static str,
}

const CAPABILITIES: &[CapabilitySpec] = &[
    CapabilitySpec {
        name: "sensors",
        aliases: &["motion", "gyro", "accelerometer"],
        cargo_feature: "sensors",
        android_manifest: "    <uses-feature android:name=\"android.hardware.sensor.accelerometer\" android:required=\"false\" />\n    <uses-feature android:name=\"android.hardware.sensor.gyroscope\" android:required=\"false\" />\n",
        ios_project: "",
    },
    CapabilitySpec {
        name: "bluetooth",
        aliases: &["ble"],
        cargo_feature: "bluetooth",
        android_manifest: "    <uses-permission android:name=\"android.permission.ACCESS_FINE_LOCATION\" android:maxSdkVersion=\"30\" />\n    <uses-permission android:name=\"android.permission.BLUETOOTH_SCAN\" />\n    <uses-permission android:name=\"android.permission.BLUETOOTH_CONNECT\" />\n    <uses-feature android:name=\"android.hardware.bluetooth_le\" android:required=\"false\" />\n",
        ios_project: "        OTHER_LDFLAGS: \"-lcrepus_mobile_actions -framework CoreBluetooth\"\n        INFOPLIST_KEY_NSBluetoothAlwaysUsageDescription: \"$(PRODUCT_NAME) uses Bluetooth for nearby device setup.\"\n",
    },
    CapabilitySpec {
        name: "haptics",
        aliases: &["vibration"],
        cargo_feature: "haptics",
        android_manifest: "    <uses-permission android:name=\"android.permission.VIBRATE\" />\n",
        ios_project: "",
    },
    CapabilitySpec { name: "clipboard", aliases: &[], cargo_feature: "clipboard", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "toast", aliases: &[], cargo_feature: "toast", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "privacy-screen", aliases: &["privacyscreen"], cargo_feature: "privacy-screen", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "video", aliases: &[], cargo_feature: "video", android_manifest: "    <uses-permission android:name=\"android.permission.CAMERA\" />\n    <uses-permission android:name=\"android.permission.RECORD_AUDIO\" />\n    <uses-feature android:name=\"android.hardware.camera\" android:required=\"false\" />\n", ios_project: "        INFOPLIST_KEY_NSCameraUsageDescription: \"$(PRODUCT_NAME) records video when you ask it to.\"\n        INFOPLIST_KEY_NSMicrophoneUsageDescription: \"$(PRODUCT_NAME) records video audio when you ask it to.\"\n" },
    CapabilitySpec { name: "browser", aliases: &["linking", "app-launcher", "applauncher", "phone", "sms"], cargo_feature: "browser", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "in-app-browser", aliases: &["inappbrowser", "web-browser"], cargo_feature: "in-app-browser", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "share", aliases: &[], cargo_feature: "share", android_manifest: "", ios_project: "" },
    CapabilitySpec {
        name: "documentpicker",
        aliases: &["document-picker", "documents"],
        cargo_feature: "documentpicker",
        android_manifest: "",
        ios_project: "",
    },
    CapabilitySpec {
        name: "image-picker",
        aliases: &["imagepicker", "media-picker"],
        cargo_feature: "image-picker",
        android_manifest: "",
        ios_project: "",
    },
    CapabilitySpec {
        name: "photo-library",
        aliases: &["photolibrary", "media-library"],
        cargo_feature: "photo-library",
        android_manifest: "    <uses-permission android:name=\"android.permission.READ_MEDIA_IMAGES\" />\n    <uses-permission android:name=\"android.permission.READ_MEDIA_VIDEO\" />\n    <uses-permission android:name=\"android.permission.READ_EXTERNAL_STORAGE\" android:maxSdkVersion=\"32\" />\n",
        ios_project: "        INFOPLIST_KEY_NSPhotoLibraryUsageDescription: \"$(PRODUCT_NAME) accesses your media library when you ask it to.\"\n",
    },
    CapabilitySpec {
        name: "camera",
        aliases: &[],
        cargo_feature: "camera",
        android_manifest: "    <uses-permission android:name=\"android.permission.CAMERA\" />\n    <uses-feature android:name=\"android.hardware.camera\" android:required=\"false\" />\n",
        ios_project: "        INFOPLIST_KEY_NSCameraUsageDescription: \"$(PRODUCT_NAME) uses the camera when you ask it to.\"\n",
    },
    CapabilitySpec { name: "dimensions", aliases: &[], cargo_feature: "dimensions", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "dialog", aliases: &[], cargo_feature: "dialog", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "action-sheet", aliases: &["actionsheet"], cargo_feature: "action-sheet", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "app-state", aliases: &["appstate"], cargo_feature: "app-state", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "app", aliases: &["app-info", "appinfo"], cargo_feature: "app", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "screen-orientation", aliases: &["screenorientation"], cargo_feature: "screen-orientation", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "accessibility-info", aliases: &["accessibilityinfo", "screen-reader", "screenreader"], cargo_feature: "accessibility-info", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "device", aliases: &["device-info", "deviceinfo", "platform"], cargo_feature: "device", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "preferences", aliases: &["storage", "async-storage"], cargo_feature: "preferences", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "network", aliases: &["net-info", "netinfo"], cargo_feature: "network", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "keyboard", aliases: &[], cargo_feature: "keyboard", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "settings", aliases: &["app-settings"], cargo_feature: "settings", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "local-notifications", aliases: &["localnotifications", "notifications"], cargo_feature: "local-notifications", android_manifest: "    <uses-permission android:name=\"android.permission.POST_NOTIFICATIONS\" />\n", ios_project: "" },
    CapabilitySpec { name: "secure-storage", aliases: &["securestorage"], cargo_feature: "secure-storage", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "biometrics", aliases: &["authentication"], cargo_feature: "biometrics", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "permissions", aliases: &["permission"], cargo_feature: "permissions", android_manifest: "", ios_project: "        OTHER_LDFLAGS: \"-lcrepus_mobile_actions -framework CoreBluetooth\"\n        INFOPLIST_KEY_NSBluetoothAlwaysUsageDescription: \"$(PRODUCT_NAME) uses Bluetooth when you ask it to.\"\n" },
    CapabilitySpec { name: "microphone", aliases: &["audio"], cargo_feature: "microphone", android_manifest: "    <uses-permission android:name=\"android.permission.RECORD_AUDIO\" />\n", ios_project: "        INFOPLIST_KEY_NSMicrophoneUsageDescription: \"$(PRODUCT_NAME) uses the microphone when you ask it to.\"\n" },
    CapabilitySpec { name: "calendar", aliases: &["calendars"], cargo_feature: "calendar", android_manifest: "    <uses-permission android:name=\"android.permission.READ_CALENDAR\" />\n    <uses-permission android:name=\"android.permission.WRITE_CALENDAR\" />\n", ios_project: "        INFOPLIST_KEY_NSCalendarsFullAccessUsageDescription: \"$(PRODUCT_NAME) accesses your calendar when you ask it to.\"\n" },
    CapabilitySpec {
        name: "contacts",
        aliases: &[],
        cargo_feature: "contacts",
        android_manifest: "    <uses-permission android:name=\"android.permission.READ_CONTACTS\" />\n",
        ios_project: "        INFOPLIST_KEY_NSContactsUsageDescription: \"$(PRODUCT_NAME) accesses your contacts when you ask it to.\"\n",
    },
    CapabilitySpec {
        name: "filesystem",
        aliases: &["files"],
        cargo_feature: "filesystem",
        android_manifest: "",
        ios_project: "",
    },
    CapabilitySpec {
        name: "geolocation",
        aliases: &["location"],
        cargo_feature: "geolocation",
        android_manifest: "    <uses-permission android:name=\"android.permission.ACCESS_FINE_LOCATION\" />\n    <uses-permission android:name=\"android.permission.ACCESS_COARSE_LOCATION\" />\n",
        ios_project: "        INFOPLIST_KEY_NSLocationWhenInUseUsageDescription: \"$(PRODUCT_NAME) uses your location while the app is open.\"\n",
    },
    CapabilitySpec { name: "battery", aliases: &[], cargo_feature: "battery", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "appearance", aliases: &[], cargo_feature: "appearance", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "system-bars", aliases: &["systembars", "status-bar", "statusbar"], cargo_feature: "system-bars", android_manifest: "", ios_project: "" },
    CapabilitySpec { name: "deep-links", aliases: &["deeplinks", "deep-link", "url-events"], cargo_feature: "deep-links", android_manifest: "", ios_project: "" },
];

fn add_capability(capability: &str, root: &Path) -> Result<(), String> {
    let capability = capability.to_ascii_lowercase();
    let spec = CAPABILITIES
        .iter()
        .find(|spec| capability == spec.name || spec.aliases.contains(&capability.as_str()))
        .ok_or_else(|| {
            format!(
                "unknown native capability '{capability}'; available: {}",
                CAPABILITIES
                    .iter()
                    .map(|spec| spec.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
    let cargo = root.join("rust/Cargo.toml");
    let manifest = root.join("android/app/src/main/AndroidManifest.xml");
    let project = root.join("ios/project.yml");
    if !cargo.is_file() || !manifest.is_file() || !project.is_file() {
        return Err(format!("'{}' is not a native scaffold", root.display()));
    }
    add_feature(&cargo, spec.cargo_feature)?;
    if spec.name == "filesystem" {
        enable_default_feature(&cargo, spec.cargo_feature)?;
    }
    add_once(
        &manifest,
        spec.android_manifest,
        "    <uses-permission android:name=\"android.permission.INTERNET\" />\n",
    )?;
    if !spec.ios_project.is_empty() {
        let mut source = fs::read_to_string(&project)
            .map_err(|e| format!("read '{}': {e}", project.display()))?;
        if source.contains("INFOPLIST_FILE: App/Info.plist") {
            add_ios_info_plist_key(root, spec.ios_project)?;
        } else if (spec.name == "geolocation"
            && !source.contains("INFOPLIST_KEY_NSLocationWhenInUseUsageDescription"))
            || (spec.name == "photo-library"
                && !source.contains("INFOPLIST_KEY_NSPhotoLibraryUsageDescription"))
            || (spec.name == "camera" && !source.contains("INFOPLIST_KEY_NSCameraUsageDescription"))
            || (spec.name == "video"
                && !source.contains("INFOPLIST_KEY_NSMicrophoneUsageDescription"))
            || (spec.name == "microphone"
                && !source.contains("INFOPLIST_KEY_NSMicrophoneUsageDescription"))
            || (spec.name == "calendar"
                && !source.contains("INFOPLIST_KEY_NSCalendarsFullAccessUsageDescription"))
            || (spec.name == "contacts"
                && !source.contains("INFOPLIST_KEY_NSContactsUsageDescription"))
        {
            source = source.replace(
                "        INFOPLIST_KEY_UILaunchScreen_Generation: YES\n",
                &format!(
                    "        INFOPLIST_KEY_UILaunchScreen_Generation: YES\n{}",
                    spec.ios_project
                ),
            );
            fs::write(&project, source)
                .map_err(|e| format!("write '{}': {e}", project.display()))?;
        } else if (spec.name == "bluetooth" || spec.name == "permissions")
            && !source.contains("CoreBluetooth")
        {
            source = source.replace(
                "        OTHER_LDFLAGS: \"-lcrepus_mobile_actions\"\n",
                spec.ios_project,
            );
            fs::write(&project, source)
                .map_err(|e| format!("write '{}': {e}", project.display()))?;
        }
    }
    if spec.name == "sensors" {
        add_sensors_host(root)?;
    }
    if spec.name == "haptics" {
        add_haptics_host(root)?;
    }
    if spec.name == "clipboard" {
        add_clipboard_host(root)?;
    }
    if spec.name == "toast" {
        add_toast_host(root)?;
    }
    if spec.name == "privacy-screen" {
        add_privacy_screen_host(root)?;
    }
    if spec.name == "video" {
        add_video_host(root)?;
    }
    if spec.name == "browser" {
        add_browser_host(root)?;
    }
    if spec.name == "in-app-browser" {
        add_in_app_browser_host(root)?;
    }
    if spec.name == "share" {
        add_share_host(root)?;
    }
    if spec.name == "documentpicker" {
        add_document_picker_host(root)?;
    }
    if spec.name == "image-picker" {
        add_image_picker_host(root)?;
    }
    if spec.name == "photo-library" {
        add_photo_library_host(root)?;
    }
    if spec.name == "camera" {
        add_camera_host(root)?;
    }
    if spec.name == "dimensions" {
        add_dimensions_host(root)?;
    }
    if spec.name == "dialog" {
        add_dialog_host(root)?;
    }
    if spec.name == "action-sheet" {
        add_action_sheet_host(root)?;
    }
    if spec.name == "app-state" {
        add_app_state_host(root)?;
    }
    if spec.name == "app" {
        add_app_host(root)?;
    }
    if spec.name == "screen-orientation" {
        add_screen_orientation_host(root)?;
    }
    if spec.name == "accessibility-info" {
        add_accessibility_info_host(root)?;
    }
    if spec.name == "device" {
        add_device_host(root)?;
    }
    if spec.name == "preferences" {
        add_preferences_host(root)?;
    }
    if spec.name == "network" {
        add_network_host(root)?;
    }
    if spec.name == "keyboard" {
        add_keyboard_host(root)?;
    }
    if spec.name == "settings" {
        add_settings_host(root)?;
    }
    if spec.name == "local-notifications" {
        add_local_notifications_host(root)?;
    }
    if spec.name == "secure-storage" {
        add_secure_storage_host(root)?;
    }
    if spec.name == "biometrics" {
        add_biometrics_host(root)?;
    }
    if spec.name == "permissions" {
        add_permissions_host(root)?;
    }
    if spec.name == "microphone" {
        add_microphone_host(root)?;
    }
    if spec.name == "calendar" {
        add_calendar_host(root)?;
    }
    if spec.name == "contacts" {
        add_contacts_host(root)?;
    }
    if spec.name == "bluetooth" {
        add_bluetooth_host(root)?;
    }
    if spec.name == "geolocation" {
        add_geolocation_host(root)?;
    }
    if spec.name == "battery" {
        add_battery_host(root)?;
    }
    if spec.name == "appearance" {
        add_appearance_host(root)?;
    }
    if spec.name == "system-bars" {
        add_system_bars_host(root)?;
    }
    if spec.name == "deep-links" {
        add_deep_links_host(root)?;
    }
    dedupe_android_imports(root)?;
    ui::success(&format!(
        "added native capability '{}' to '{}'",
        spec.name,
        root.display()
    ));
    Ok(())
}

fn add_deep_links_host(root: &Path) -> Result<(), String> {
    let manifest = root.join("android/app/src/main/AndroidManifest.xml");
    let mut source =
        fs::read_to_string(&manifest).map_err(|e| format!("read '{}': {e}", manifest.display()))?;
    if !source.contains("android.intent.action.VIEW") {
        source = source.replace(
            "        </activity>\n",
            "            <intent-filter>\n                <action android:name=\"android.intent.action.VIEW\" />\n                <category android:name=\"android.intent.category.DEFAULT\" />\n                <category android:name=\"android.intent.category.BROWSABLE\" />\n                <data android:scheme=\"crepus\" />\n            </intent-filter>\n        </activity>\n",
        );
        fs::write(&manifest, source).map_err(|e| format!("write '{}': {e}", manifest.display()))?;
    }
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("deepLinksValue") {
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"deepLinks\" -> deepLinksValue(method, payload)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_DEEP_LINKS}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let activity =
        root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/MainActivity.kt");
    let mut source =
        fs::read_to_string(&activity).map_err(|e| format!("read '{}': {e}", activity.display()))?;
    if !source.contains("receiveDeepLink(intent.data)") {
        source = source.replace(
            "import android.os.Bundle\n",
            "import android.content.Intent\nimport android.os.Bundle\n",
        );
        source = source.replace(
            "        CrepusRustActions.install(this)\n",
            "        CrepusRustActions.install(this)\n        CrepusRustActions.receiveDeepLink(intent.data)\n",
        );
        source = source.replace(
            "\n    override fun onDestroy()",
            "\n    override fun onNewIntent(intent: Intent) {\n        super.onNewIntent(intent)\n        setIntent(intent)\n        CrepusRustActions.receiveDeepLink(intent.data)\n    }\n\n    override fun onDestroy()",
        );
        fs::write(&activity, source).map_err(|e| format!("write '{}': {e}", activity.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("deepLinksValue") {
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"deepLinks\":\n            return try deepLinksValue(method: method, payload: payload)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_DEEP_LINKS}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    let app = root.join("ios/App/CrepusMobileApp.swift");
    let mut source =
        fs::read_to_string(&app).map_err(|e| format!("read '{}': {e}", app.display()))?;
    if !source.contains(".onOpenURL") {
        source = source.replace(
            "        WindowGroup {\n            ContentView()\n        }\n",
            "        WindowGroup {\n            ContentView()\n                .onOpenURL { CrepusRustActions.receiveDeepLink($0) }\n        }\n",
        );
        fs::write(&app, source).map_err(|e| format!("write '{}': {e}", app.display()))?;
    }
    let project = root.join("ios/project.yml");
    let mut source =
        fs::read_to_string(&project).map_err(|e| format!("read '{}': {e}", project.display()))?;
    if source.contains("GENERATE_INFOPLIST_FILE: YES") {
        source = source.replace(
            "        GENERATE_INFOPLIST_FILE: YES\n",
            "        GENERATE_INFOPLIST_FILE: NO\n        INFOPLIST_FILE: App/Info.plist\n",
        );
        fs::write(&project, &source).map_err(|e| format!("write '{}': {e}", project.display()))?;
    }
    let info = root.join("ios/App/Info.plist");
    if !info.exists() {
        fs::write(
            &info,
            format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n<dict>\n    <key>CFBundleURLTypes</key>\n    <array>\n        <dict>\n            <key>CFBundleTypeRole</key>\n            <string>Editor</string>\n            <key>CFBundleURLName</key>\n            <string>$(PRODUCT_BUNDLE_IDENTIFIER)</string>\n            <key>CFBundleURLSchemes</key>\n            <array>\n                <string>crepus</string>\n            </array>\n        </dict>\n    </array>\n{}    <key>UILaunchScreen</key>\n    <dict/>\n</dict>\n</plist>\n", ios_info_plist_keys(&source)),
        )
        .map_err(|e| format!("write '{}': {e}", info.display()))?;
    }
    Ok(())
}

fn add_ios_info_plist_key(root: &Path, setting: &str) -> Result<(), String> {
    let keys = setting
        .lines()
        .filter_map(ios_info_plist_key)
        .collect::<Vec<_>>();
    if keys.is_empty() {
        return Ok(());
    }
    let path = root.join("ios/App/Info.plist");
    let mut source =
        fs::read_to_string(&path).map_err(|e| format!("read '{}': {e}", path.display()))?;
    let mut changed = false;
    for (key, value) in keys {
        if !source.contains(&format!("<key>{key}</key>")) {
            source = source.replace(
                "    <key>UILaunchScreen</key>",
                &format!("    <key>{key}</key>\n    <string>{value}</string>\n    <key>UILaunchScreen</key>"),
            );
            changed = true;
        }
    }
    if changed {
        fs::write(&path, source).map_err(|e| format!("write '{}': {e}", path.display()))?;
    }
    Ok(())
}

fn ios_info_plist_keys(project: &str) -> String {
    project
        .lines()
        .filter_map(|line| ios_info_plist_key(line))
        .map(|(key, value)| format!("    <key>{key}</key>\n    <string>{value}</string>\n"))
        .collect()
}

fn ios_info_plist_key(setting: &str) -> Option<(String, &str)> {
    let setting = setting.trim();
    let (key, value) = setting.split_once(": ")?;
    let key = key.strip_prefix("INFOPLIST_KEY_NS")?;
    Some((format!("NS{key}"), value.trim_matches('"')))
}

fn add_system_bars_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("systemBarsValue") {
        source = source.replace("import android.content.ClipData\n", "import android.content.ClipData\nimport android.graphics.Color\nimport android.os.Build\nimport android.view.View\n");
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"systemBars\" -> systemBarsValue(method, payload)\n",
        );
        source = source.replace(
            "\n}\n\nobject CrepusActionState",
            &format!("{ANDROID_SYSTEM_BARS}\n}}\n\nobject CrepusActionState"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("systemBarsValue") {
        source = source.replace("        switch capability {\n", "        switch capability {\n        case \"systemBars\":\n            return try systemBarsValue(method: method, payload: payload)\n");
        source = source.replace(
            "\n    fileprivate static func emit",
            &format!("{IOS_SYSTEM_BARS}\n\n    fileprivate static func emit"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_appearance_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("appearanceValue") {
        source = source.replace(
            "import android.content.Context\n",
            "import android.content.Context\nimport android.content.res.Configuration\n",
        );
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"appearance\" -> appearanceValue(method)\n",
        );
        source = source.replace(
            "\n}\n\nobject CrepusActionState",
            &format!("{ANDROID_APPEARANCE}\n}}\n\nobject CrepusActionState"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("appearanceValue") {
        source = source.replace("        switch capability {\n", "        switch capability {\n        case \"appearance\":\n            return try appearanceValue(method: method)\n");
        source = source.replace(
            "\n    fileprivate static func emit",
            &format!("{IOS_APPEARANCE}\n\n    fileprivate static func emit"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_battery_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("batteryValue") {
        source = source.replace("import android.content.Context\n", "import android.content.BroadcastReceiver\nimport android.content.Context\nimport android.content.IntentFilter\nimport android.os.BatteryManager\n");
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"battery\" -> batteryValue(method)\n",
        );
        source = source.replace(
            "\n}\n\nobject CrepusActionState",
            &format!("{ANDROID_BATTERY}\n}}\n\nobject CrepusActionState"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("batteryValue") {
        source = source.replace("        switch capability {\n", "        switch capability {\n        case \"battery\":\n            return try batteryValue(method: method)\n");
        source = source.replace(
            "\n    fileprivate static func emit",
            &format!("{IOS_BATTERY}\n\n    fileprivate static func emit"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_geolocation_ios_host(root: &Path) -> Result<(), String> {
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if source.contains("GeolocationBridge") {
        return Ok(());
    }
    source = source.replace(
        "import Foundation\n",
        "import Foundation\nimport CoreLocation\n",
    );
    source = source.replace(
        "        switch capability {\n",
        "        switch capability {\n        case \"geolocation\":\n            return try geolocationValue(method: method)\n",
    );
    source = source.replace(
        "\n    fileprivate static func emit",
        &format!("{IOS_GEOLOCATION}\n\n    fileprivate static func emit"),
    );
    source = source.replace(
        "    private static func emit(_ result: String)",
        "    static func emit(_ result: String)",
    );
    source = source.replace(
        "    private static func successJson(action: String, capability: String, method: String, value: Any) -> String",
        "    static func successJson(action: String, capability: String, method: String, value: Any) -> String",
    );
    source.push_str(IOS_GEOLOCATION_BRIDGE);
    fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))
}

fn add_geolocation_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if source.contains("GeolocationBridge") {
        return Ok(());
    }
    source = source.replace(
        "import android.content.ClipData\n",
        "import android.content.ClipData\nimport android.Manifest\nimport android.content.pm.PackageManager\nimport android.location.Location\nimport android.location.LocationListener\nimport android.location.LocationManager\n",
    );
    source = source.replace(
        "        when (capability) {\n",
        "        when (capability) {\n            \"geolocation\" -> geolocationValue(method)\n",
    );
    source = source.replace(
        "\n}\n\nobject CrepusActionState",
        &format!("{ANDROID_GEOLOCATION}\n}}\n\nobject CrepusActionState"),
    );
    source.push_str(ANDROID_GEOLOCATION_BRIDGE);
    fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    add_geolocation_ios_host(root)
}

fn add_feature(path: &Path, feature: &str) -> Result<(), String> {
    let mut source =
        fs::read_to_string(path).map_err(|e| format!("read '{}': {e}", path.display()))?;
    if source
        .lines()
        .any(|line| line.trim_start().starts_with(&format!("{feature} =")))
    {
        return Ok(());
    }
    if let Some(features) = source.find("[features]\n") {
        let start = features + "[features]\n".len();
        let end = source[start..]
            .find("\n[")
            .map_or(source.len(), |offset| start + offset);
        source.insert_str(end, &format!("{feature} = []\n"));
    } else {
        source.push_str(&format!("\n[features]\n{feature} = []\n"));
    }
    fs::write(path, source).map_err(|e| format!("write '{}': {e}", path.display()))
}

fn enable_default_feature(path: &Path, feature: &str) -> Result<(), String> {
    let mut source =
        fs::read_to_string(path).map_err(|e| format!("read '{}': {e}", path.display()))?;
    let default = "default = []";
    if source.contains(&format!("\"{feature}\"")) {
        return Ok(());
    }
    if source.contains(default) {
        source = source.replacen(default, &format!("default = [\"{feature}\"]"), 1);
    } else if source.contains("default = [") {
        source = source.replacen("default = [", &format!("default = [\"{feature}\", "), 1);
    } else if source.contains("[features]\n") {
        source = source.replacen(
            "[features]\n",
            &format!("[features]\ndefault = [\"{feature}\"]\n"),
            1,
        );
    } else {
        return Err(format!("missing feature table in '{}'", path.display()));
    }
    fs::write(path, source).map_err(|e| format!("write '{}': {e}", path.display()))
}

fn add_bluetooth_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if source.contains("BluetoothBridge") {
        return Ok(());
    }
    source = source.replace(
        "import android.content.Context\n",
        "import android.Manifest\nimport android.bluetooth.BluetoothAdapter\nimport android.bluetooth.le.ScanCallback\nimport android.bluetooth.le.ScanResult\nimport android.content.Context\nimport android.content.pm.PackageManager\n",
    );
    source = source.replace(
        "        when (capability) {\n",
        "        when (capability) {\n            \"bluetooth\" -> bluetoothValue(method)\n",
    );
    source = source.replace(
        "\n}\n\nobject CrepusActionState",
        &format!("{ANDROID_BLUETOOTH}\n}}\n\nobject CrepusActionState"),
    );
    source = source.replace(
        "    private fun emit(result: String)",
        "    internal fun emit(result: String)",
    );
    source.push_str(ANDROID_BLUETOOTH_BRIDGE);
    fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    add_bluetooth_ios_host(root)
}

fn add_bluetooth_ios_host(root: &Path) -> Result<(), String> {
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if source.contains("BluetoothBridge") {
        return Ok(());
    }
    source = source.replace(
        "import Foundation\n",
        "import Foundation\nimport CoreBluetooth\n",
    );
    source = source.replace(
        "        switch capability {\n",
        "        switch capability {\n        case \"bluetooth\":\n            return try bluetoothValue(method: method)\n",
    );
    source = source.replace(
        "    private static func successJson",
        "    fileprivate static func successJson",
    );
    source = source.replace(
        "\n    fileprivate static func emit",
        &format!("{IOS_BLUETOOTH}\n\n    fileprivate static func emit"),
    );
    source.push_str(IOS_BLUETOOTH_BRIDGE);
    fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))
}

fn add_haptics_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("hapticsValue") {
        source = source.replace(
            "import android.net.Uri\n",
            "import android.net.Uri\nimport android.os.Build\nimport android.os.VibrationEffect\nimport android.os.Vibrator\nimport android.os.VibratorManager\n",
        );
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"haptics\", \"vibration\" -> hapticsValue(method, payload)\n",
        );
        source = source.replace(
            "\n}\n\nobject CrepusActionState",
            &format!("{ANDROID_HAPTICS}\n}}\n\nobject CrepusActionState"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("hapticsValue") {
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"haptics\", \"vibration\":\n            return try hapticsValue(method: method, payload: payload)\n",
        );
        source = source.replace(
            "\n    fileprivate static func emit",
            &format!("{IOS_HAPTICS}\n\n    fileprivate static func emit"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_clipboard_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("clipboardValue") {
        source = source.replace(
            "import android.content.Context\n",
            "import android.content.ClipData\nimport android.content.ClipboardManager\nimport android.content.Context\n",
        );
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"clipboard\" -> clipboardValue(method, payload)\n",
        );
        source = source.replace(
            "\n}\n\nobject CrepusActionState",
            &format!("{ANDROID_CLIPBOARD}\n}}\n\nobject CrepusActionState"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("clipboardValue") {
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"clipboard\":\n            return try clipboardValue(method: method, payload: payload)\n",
        );
        source = source.replace(
            "\n    fileprivate static func emit",
            &format!("{IOS_CLIPBOARD}\n\n    fileprivate static func emit"),
        );
        source.push_str(IOS_CLIPBOARD_BRIDGE);
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_toast_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("toastValue") {
        source = source.replace(
            "import android.content.Context\n",
            "import android.content.Context\nimport android.widget.Toast\n",
        );
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"toast\" -> toastValue(method, payload)\n",
        );
        source = source.replace(
            "\n}\n\nobject CrepusActionState",
            &format!("{ANDROID_TOAST}\n}}\n\nobject CrepusActionState"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("toastValue") {
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"toast\":\n            return try toastValue(method: method, payload: payload)\n",
        );
        source = source.replace(
            "\n    fileprivate static func emit",
            &format!("{IOS_TOAST}\n\n    fileprivate static func emit"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_video_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("videoValue") {
        source = source.replace("import android.content.Context\n", "import android.app.Activity\nimport android.content.Context\nimport android.provider.MediaStore\n");
        source = source.replace("    private var openDocuments: (() -> Unit)? = null\n", "    private var openDocuments: (() -> Unit)? = null\n    private var pendingVideoAction: String? = null\n    private var captureVideo: (() -> Unit)? = null\n");
        source = source.replace("        openDocuments = { filePicker.launch(arrayOf(\"*/*\")) }\n", "        openDocuments = { filePicker.launch(arrayOf(\"*/*\")) }\n        val video = activity.registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->\n            val action = pendingVideoAction ?: return@registerForActivityResult\n            pendingVideoAction = null\n            val uri = result.data?.data\n            if (result.resultCode != Activity.RESULT_OK || uri == null) emit(videoCancelledJson(action)) else emit(videoResultJson(action, uri))\n        }\n        captureVideo = { video.launch(Intent(MediaStore.ACTION_VIDEO_CAPTURE)) }\n");
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"video\" -> videoValue(method)\n",
        );
        source = source.replace("\n    private fun dispatchHostAction", &format!("{ANDROID_VIDEO}\n    private fun videoCancelledJson(action: String): String = JSONObject().put(\"ok\", true).put(\"action\", action).put(\"value\", JSONObject().put(\"files\", JSONArray())).toString()\n\n    private fun dispatchHostAction"));
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("videoValue") {
        source = source.replace("        switch capability {\n", "        switch capability {\n        case \"video\":\n            return try videoValue(method: method)\n");
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_VIDEO}\n\n    private static func dispatchHostAction"),
        );
        source.push_str(IOS_VIDEO_BRIDGE);
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_privacy_screen_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("privacyScreenValue") {
        source = source.replace(
            "import android.content.Context\n",
            "import android.content.Context\nimport android.view.WindowManager\n",
        );
        source = source.replace("        when (capability) {\n", "        when (capability) {\n            \"privacyScreen\" -> privacyScreenValue(method)\n");
        source = source.replace(
            "\n}\n\nobject CrepusActionState",
            &format!("{ANDROID_PRIVACY_SCREEN}\n}}\n\nobject CrepusActionState"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("privacyScreenValue") {
        source = source.replace("        switch capability {\n", "        switch capability {\n        case \"privacyScreen\":\n            return try privacyScreenValue(method: method)\n");
        source = source.replace(
            "\n    fileprivate static func emit",
            &format!("{IOS_PRIVACY_SCREEN}\n\n    fileprivate static func emit"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_browser_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("openUrlValue") {
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"browser\", \"linking\", \"appLauncher\", \"phone\", \"sms\" -> openUrlValue(capability, method, payload)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_BROWSER}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("openUrlValue") {
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"browser\", \"linking\", \"appLauncher\", \"phone\", \"sms\":\n            return try openUrlValue(capability: capability, method: method, payload: payload)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_BROWSER}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_in_app_browser_host(root: &Path) -> Result<(), String> {
    let gradle = root.join("android/app/build.gradle.kts");
    add_once(
        &gradle,
        "    implementation(\"androidx.browser:browser:1.8.0\")\n",
        "dependencies {\n",
    )?;
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("inAppBrowserValue") {
        source = source.replace(
            "import androidx.activity.ComponentActivity\n",
            "import androidx.activity.ComponentActivity\nimport androidx.browser.customtabs.CustomTabsIntent\n",
        );
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"inAppBrowser\" -> inAppBrowserValue(method, payload)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_IN_APP_BROWSER}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("inAppBrowserValue") {
        source = source.replace(
            "import Foundation\n",
            "import Foundation\nimport SafariServices\n",
        );
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"inAppBrowser\":\n            return try inAppBrowserValue(method: method, payload: payload)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_IN_APP_BROWSER}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_share_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("shareValue") {
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"share\" -> shareValue(method, payload)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_SHARE}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("shareValue") {
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"share\":\n            return try shareValue(method: method, payload: payload)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_SHARE}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_document_picker_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("documentPickerValue") {
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"documentPicker\" -> documentPickerValue(method)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_DOCUMENT_PICKER}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("documentPickerValue") {
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"documentPicker\":\n            return try documentPickerValue(method: method)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_DOCUMENT_PICKER}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_image_picker_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("imagePickerValue") {
        source = source.replace(
            "    private var openDocuments: (() -> Unit)? = null\n",
            "    private var openDocuments: (() -> Unit)? = null\n    private var openMedia: (() -> Unit)? = null\n",
        );
        source = source.replace(
            "        openDocuments = { filePicker.launch(arrayOf(\"*/*\")) }\n",
            "        openDocuments = { filePicker.launch(arrayOf(\"*/*\")) }\n        openMedia = { filePicker.launch(arrayOf(\"image/*\", \"video/*\")) }\n",
        );
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"imagePicker\" -> imagePickerValue(method)\n",
        );
        source = source.replace(
            "        when (action) {\n            \"import_files\" -> {\n",
            "        when (action) {\n            \"pick_media\" -> {\n                pendingPickerAction = action\n                openMedia?.invoke() ?: emit(errorJson(action, \"media picker unavailable\"))\n                pendingJson(action)\n            }\n            \"import_files\" -> {\n",
        );
        source = source.replace(
            "    private fun dispatchHostAction",
            &format!("{ANDROID_IMAGE_PICKER}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("imagePickerValue") {
        source = source.replace("import UIKit\n", "import PhotosUI\nimport UIKit\n");
        source = source.replace(
            "        switch action {\n        case \"import_files\":\n",
            "        switch action {\n        case \"pick_media\":\n            presentMediaPicker(action: action)\n            return pendingJson(action: action)\n        case \"import_files\":\n",
        );
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"imagePicker\":\n            return try imagePickerValue(method: method)\n",
        );
        source = source.replace(
            "    private static func dispatchHostAction",
            &format!("{IOS_IMAGE_PICKER}\n\n    private static func dispatchHostAction"),
        );
        source.push_str(IOS_IMAGE_PICKER_BRIDGE);
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_photo_library_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("photoLibraryValue") {
        source = source.replace(
            "import android.content.Context\n",
            "import android.content.Context\nimport android.content.pm.PackageManager\nimport android.os.Build\n",
        );
        source = source.replace(
            "import android.net.Uri\n",
            "import android.net.Uri\nimport android.provider.MediaStore\n",
        );
        source = source.replace(
            "import java.io.File\n",
            "import java.io.File\nimport java.time.Instant\n",
        );
        source = source.replace(
            "    private var openDocuments: (() -> Unit)? = null\n",
            "    private var openDocuments: (() -> Unit)? = null\n    private var pendingPhotoAction: String? = null\n    private var requestPhotoAccess: ((String) -> Unit)? = null\n",
        );
        source = source.replace(
            "        openDocuments = { filePicker.launch(arrayOf(\"*/*\")) }\n",
            "        openDocuments = { filePicker.launch(arrayOf(\"*/*\")) }\n        val photoPermissions = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) arrayOf(android.Manifest.permission.READ_MEDIA_IMAGES, android.Manifest.permission.READ_MEDIA_VIDEO) else arrayOf(android.Manifest.permission.READ_EXTERNAL_STORAGE)\n        val photoAccess = activity.registerForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) { grants ->\n            val action = pendingPhotoAction ?: return@registerForActivityResult\n            pendingPhotoAction = null\n            if (grants.values.any { it }) scanPhotoLibrary(action) else emit(errorJson(action, \"photo access denied\"))\n        }\n        requestPhotoAccess = { action ->\n            if (photoPermissions.any { activity.checkSelfPermission(it) == PackageManager.PERMISSION_GRANTED }) scanPhotoLibrary(action)\n            else {\n                pendingPhotoAction = action\n                photoAccess.launch(photoPermissions)\n            }\n        }\n",
        );
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"photoLibrary\" -> photoLibraryValue(method)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_PHOTO_LIBRARY}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("photoLibraryValue") {
        source = source.replace(
            "import SwiftUI\n#if canImport(UIKit)\n",
            "import SwiftUI\n#if canImport(UIKit)\nimport Photos\n",
        );
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"photoLibrary\":\n            return try photoLibraryValue(method: method)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_PHOTO_LIBRARY}\n\n    private static func dispatchHostAction"),
        );
        source.push_str(IOS_PHOTO_LIBRARY_BRIDGE);
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_camera_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("cameraValue") {
        source = source.replace(
            "import android.content.Context\n",
            "import android.content.Context\nimport android.graphics.Bitmap\n",
        );
        source = source.replace(
            "    private var openDocuments: (() -> Unit)? = null\n",
            "    private var openDocuments: (() -> Unit)? = null\n    private var pendingCameraAction: String? = null\n    private var captureCameraPhoto: (() -> Unit)? = null\n",
        );
        source = source.replace(
            "        openDocuments = { filePicker.launch(arrayOf(\"*/*\")) }\n",
            "        openDocuments = { filePicker.launch(arrayOf(\"*/*\")) }\n        val camera = activity.registerForActivityResult(ActivityResultContracts.TakePicturePreview()) { bitmap ->\n            val action = pendingCameraAction ?: return@registerForActivityResult\n            pendingCameraAction = null\n            if (bitmap == null) emit(cameraCancelledJson(action)) else emit(cameraResultJson(action, bitmap))\n        }\n        captureCameraPhoto = { camera.launch(null) }\n",
        );
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"camera\" -> cameraValue(method)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_CAMERA}\n    private fun cameraCancelledJson(action: String): String = JSONObject().put(\"ok\", true).put(\"action\", action).put(\"value\", JSONObject().put(\"files\", JSONArray())).toString()\n\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("cameraValue") {
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"camera\":\n            return try cameraValue(method: method)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_CAMERA}\n\n    private static func dispatchHostAction"),
        );
        source.push_str(IOS_CAMERA_BRIDGE);
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_dimensions_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("dimensionsValue") {
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"dimensions\" -> dimensionsValue(method)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_DIMENSIONS}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("dimensionsValue") {
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"dimensions\":\n            return try dimensionsValue(method: method)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_DIMENSIONS}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_dialog_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("dialogValue") {
        source = source.replace(
            "import android.content.Context\n",
            "import android.content.Context\nimport android.app.AlertDialog\n",
        );
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"dialog\" -> dialogValue(method, payload)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_DIALOG}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("dialogValue") {
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"dialog\":\n            return try dialogValue(method: method, payload: payload)\n",
        );
        source = source.replace(
            "    private static func emit(_ result: String)",
            "    fileprivate static func emit(_ result: String)",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_DIALOG}\n\n    private static func dispatchHostAction"),
        );
        source.push_str(IOS_DIALOG_BRIDGE);
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_action_sheet_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("actionSheetValue") {
        source = source.replace(
            "import android.content.Context\n",
            "import android.content.Context\nimport android.app.AlertDialog\n",
        );
        source = source.replace("        when (capability) {\n", "        when (capability) {\n            \"actionSheet\" -> actionSheetValue(method, payload)\n");
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_ACTION_SHEET}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("actionSheetValue") {
        source = source.replace("        switch capability {\n", "        switch capability {\n        case \"actionSheet\":\n            return try actionSheetValue(method: method, payload: payload)\n");
        source = source.replace(
            "    private static func emit(_ result: String)",
            "    fileprivate static func emit(_ result: String)",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_ACTION_SHEET}\n\n    private static func dispatchHostAction"),
        );
        source.push_str(IOS_ACTION_SHEET_BRIDGE);
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_app_state_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("appStateValue") {
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"appState\" -> appStateValue(method)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_APP_STATE}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("appStateValue") {
        source = source.replace("        switch capability {\n", "        switch capability {\n        case \"appState\":\n            return try appStateValue(method: method)\n");
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_APP_STATE}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_app_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("appValue") {
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"app\" -> appValue(method)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_APP}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("appValue") {
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"app\":\n            return try appValue(method: method)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_APP}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_screen_orientation_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("screenOrientationValue") {
        source = source.replace("        when (capability) {\n", "        when (capability) {\n            \"screenOrientation\" -> screenOrientationValue(method, payload)\n");
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_SCREEN_ORIENTATION}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("screenOrientationValue") {
        source = source.replace("        switch capability {\n", "        switch capability {\n        case \"screenOrientation\":\n            return try screenOrientationValue(method: method, payload: payload)\n");
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_SCREEN_ORIENTATION}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_accessibility_info_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("accessibilityInfoValue") {
        source = source.replace(
            "import android.net.Uri\n",
            "import android.net.Uri\nimport android.provider.Settings\nimport android.view.accessibility.AccessibilityManager\n",
        );
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"accessibilityInfo\", \"screenReader\" -> accessibilityInfoValue(method)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_ACCESSIBILITY_INFO}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("accessibilityInfoValue") {
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"accessibilityInfo\", \"screenReader\":\n            return try accessibilityInfoValue(method: method)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_ACCESSIBILITY_INFO}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_device_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("deviceValue") {
        source = source.replace(
            "import android.net.Uri\n",
            "import android.net.Uri\nimport android.os.Build\n",
        );
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"device\", \"platform\" -> deviceValue(method)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_DEVICE}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("deviceValue") {
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"device\", \"platform\":\n            return try deviceValue(method: method)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_DEVICE}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_preferences_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("preferencesValue") {
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"preferences\" -> preferencesValue(method, payload)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_PREFERENCES}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("preferencesValue") {
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"preferences\":\n            return try preferencesValue(method: method, payload: payload)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_PREFERENCES}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_network_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("networkValue") {
        source = source.replace(
            "import android.net.Uri\n",
            "import android.net.Uri\nimport android.net.ConnectivityManager\nimport android.net.Network\nimport android.net.NetworkCapabilities\n",
        );
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"network\" -> networkValue(method)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_NETWORK}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("networkValue") {
        source = source.replace("import Foundation\n", "import Foundation\nimport Network\n");
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"network\":\n            return try networkValue(method: method)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_NETWORK}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_keyboard_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("keyboardValue") {
        source = source.replace(
            "import android.net.Uri\n",
            "import android.net.Uri\nimport android.view.inputmethod.InputMethodManager\n",
        );
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"keyboard\" -> keyboardValue(method)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_KEYBOARD}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("keyboardValue") {
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"keyboard\":\n            return try keyboardValue(method: method)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_KEYBOARD}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_settings_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("settingsValue") {
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"settings\" -> settingsValue(method)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_SETTINGS}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("settingsValue") {
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"settings\":\n            return try settingsValue(method: method)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_SETTINGS}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_local_notifications_host(root: &Path) -> Result<(), String> {
    let receiver = root.join(
        "android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusNotificationReceiver.kt",
    );
    if !receiver.exists() {
        fs::write(&receiver, ANDROID_SCHEDULED_NOTIFICATION_RECEIVER)
            .map_err(|e| format!("write '{}': {e}", receiver.display()))?;
    }
    let manifest = root.join("android/app/src/main/AndroidManifest.xml");
    add_once(
        &manifest,
        "        <receiver android:name=\".CrepusNotificationReceiver\" android:exported=\"false\" />\n",
        "        </activity>\n",
    )?;
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("localNotificationsValue") {
        source = source.replace(
            "import android.content.Context\n",
            "import android.Manifest\nimport android.app.AlarmManager\nimport android.app.Notification\nimport android.app.NotificationChannel\nimport android.app.NotificationManager\nimport android.app.PendingIntent\nimport android.content.Context\nimport android.content.pm.PackageManager\nimport android.os.Build\n",
        );
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"localNotifications\" -> localNotificationsValue(method, payload)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_LOCAL_NOTIFICATIONS}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("localNotificationsValue") {
        source = source.replace(
            "import Foundation\n",
            "import Foundation\nimport UserNotifications\n",
        );
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"localNotifications\":\n            return try localNotificationsValue(method: method, payload: payload)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_LOCAL_NOTIFICATIONS}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_secure_storage_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("secureStorageValue") {
        source = source.replace(
            "import android.content.Context\n",
            "import android.content.Context\nimport android.security.keystore.KeyGenParameterSpec\nimport android.security.keystore.KeyProperties\nimport android.util.Base64\nimport java.security.KeyStore\nimport javax.crypto.Cipher\nimport javax.crypto.KeyGenerator\nimport javax.crypto.SecretKey\nimport javax.crypto.spec.GCMParameterSpec\n",
        );
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"secureStorage\" -> secureStorageValue(method, payload)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_SECURE_STORAGE}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("secureStorageValue") {
        source = source.replace(
            "import Foundation\n",
            "import Foundation\nimport Security\n",
        );
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"secureStorage\":\n            return try secureStorageValue(method: method, payload: payload)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_SECURE_STORAGE}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_biometrics_host(root: &Path) -> Result<(), String> {
    let gradle = root.join("android/app/build.gradle.kts");
    add_once(
        &gradle,
        "    implementation(\"androidx.biometric:biometric:1.1.0\")\n",
        "dependencies {\n",
    )?;
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("biometricsValue") {
        source = source.replace(
            "import androidx.activity.ComponentActivity\n",
            "import androidx.activity.ComponentActivity\nimport androidx.biometric.BiometricManager\nimport androidx.biometric.BiometricPrompt\nimport androidx.core.content.ContextCompat\n",
        );
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"biometrics\", \"authentication\" -> biometricsValue(method, payload)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_BIOMETRICS}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
        let activity =
            root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/MainActivity.kt");
        let source = fs::read_to_string(&activity)
            .map_err(|e| format!("read '{}': {e}", activity.display()))?
            .replace(
                "import androidx.activity.ComponentActivity\n",
                "import androidx.fragment.app.FragmentActivity\n",
            )
            .replace(
                "class MainActivity : ComponentActivity()",
                "class MainActivity : FragmentActivity()",
            );
        fs::write(&activity, source).map_err(|e| format!("write '{}': {e}", activity.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("biometricsValue") {
        source = source.replace(
            "import Foundation\n",
            "import Foundation\nimport LocalAuthentication\n",
        );
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"biometrics\", \"authentication\":\n            return try biometricsValue(method: method, payload: payload)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_BIOMETRICS}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_permissions_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("permissionsValue") {
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"permissions\" -> permissionsValue(method, payload)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_PERMISSIONS}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("permissionsValue") {
        source = source.replace(
            "import Foundation\n",
            "import Foundation\nimport AVFoundation\nimport Contacts\nimport CoreBluetooth\nimport CoreLocation\nimport Photos\nimport UserNotifications\n",
        );
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"permissions\":\n            return try permissionsValue(method: method, payload: payload)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_PERMISSIONS}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_microphone_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("microphoneValue") {
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"microphone\" -> microphoneValue(method)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_MICROPHONE}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("microphoneValue") {
        if !source.contains("import AVFoundation\n") {
            source = source.replace(
                "import Foundation\n",
                "import Foundation\nimport AVFoundation\n",
            );
        }
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"microphone\":\n            return try microphoneValue(method: method)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_MICROPHONE}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_contacts_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("contactsValue") {
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"contacts\" -> contactsValue(method, payload)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_CONTACTS}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("contactsValue") {
        if !source.contains("import Contacts\n") {
            source = source.replace(
                "import Foundation\n",
                "import Foundation\nimport Contacts\n",
            );
        }
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"contacts\":\n            return try contactsValue(method: method, payload: payload)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_CONTACTS}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_calendar_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let mut source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !source.contains("calendarValue") {
        source = source.replace(
            "import android.content.ClipData\n",
            "import android.content.ClipData\nimport android.content.ContentValues\nimport android.provider.CalendarContract\nimport java.util.TimeZone\n",
        );
        source = source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"calendar\", \"calendars\" -> calendarValue(method, payload)\n",
        );
        source = source.replace(
            "\n    private fun dispatchHostAction",
            &format!("{ANDROID_CALENDAR}\n    private fun dispatchHostAction"),
        );
        fs::write(&android, source).map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !source.contains("calendarValue") {
        source = source.replace(
            "import Foundation\n",
            "import Foundation\nimport EventKit\n",
        );
        source = source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"calendar\", \"calendars\":\n            return try calendarValue(method: method, payload: payload)\n",
        );
        source = source.replace(
            "\n    private static func dispatchHostAction",
            &format!("{IOS_CALENDAR}\n\n    private static func dispatchHostAction"),
        );
        fs::write(&ios, source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn add_sensors_host(root: &Path) -> Result<(), String> {
    let android = android_actions_path(root)?;
    let ios = root.join("ios/Sources/NativeShell/CrepusRustActions.swift");
    let mut android_source =
        fs::read_to_string(&android).map_err(|e| format!("read '{}': {e}", android.display()))?;
    if !android_source.contains("SensorBridge") {
        android_source = android_source.replace(
            "import android.net.Uri\n",
            "import android.net.Uri\nimport android.hardware.Sensor\nimport android.hardware.SensorEvent\nimport android.hardware.SensorEventListener\nimport android.hardware.SensorManager\n",
        );
        android_source = android_source.replace(
            "        when (capability) {\n",
            "        when (capability) {\n            \"sensors\", \"motion\" -> sensorsValue(method)\n",
        );
        android_source = android_source.replace(
            "\n}\n\nobject CrepusActionState",
            &format!("{ANDROID_SENSORS}\n}}\n\nobject CrepusActionState"),
        );
        android_source.push_str(ANDROID_SENSORS_BRIDGE);
        fs::write(&android, android_source)
            .map_err(|e| format!("write '{}': {e}", android.display()))?;
    }
    let mut ios_source =
        fs::read_to_string(&ios).map_err(|e| format!("read '{}': {e}", ios.display()))?;
    if !ios_source.contains("SensorBridge") {
        ios_source = ios_source.replace(
            "import Foundation\n",
            "import Foundation\nimport CoreMotion\n",
        );
        ios_source = ios_source.replace(
            "        switch capability {\n",
            "        switch capability {\n        case \"sensors\", \"motion\":\n            return try sensorsValue(method: method)\n",
        );
        ios_source = ios_source.replace(
            "\n    fileprivate static func emit",
            &format!("{IOS_SENSORS}\n\n    fileprivate static func emit"),
        );
        ios_source.push_str(IOS_SENSORS_BRIDGE);
        fs::write(&ios, ios_source).map_err(|e| format!("write '{}': {e}", ios.display()))?;
    }
    Ok(())
}

fn android_actions_path(root: &Path) -> Result<PathBuf, String> {
    let packages = root.join("android/app/src/main/java/dev/crepuscularity");
    fs::read_dir(&packages)
        .map_err(|e| format!("read '{}': {e}", packages.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("CrepusRustActions.kt"))
        .find(|path| path.is_file())
        .ok_or_else(|| {
            format!(
                "Android Rust actions not found under '{}'",
                packages.display()
            )
        })
}

fn dedupe_android_imports(root: &Path) -> Result<(), String> {
    let path = android_actions_path(root)?;
    let source =
        fs::read_to_string(&path).map_err(|e| format!("read '{}': {e}", path.display()))?;
    let mut imports = HashSet::new();
    let deduped = source
        .lines()
        .filter(|line| !line.starts_with("import ") || imports.insert(*line))
        .collect::<Vec<_>>()
        .join("\n");
    if deduped != source.trim_end() {
        fs::write(&path, format!("{deduped}\n"))
            .map_err(|e| format!("write '{}': {e}", path.display()))?;
    }
    Ok(())
}

fn add_once(path: &Path, addition: &str, anchor: &str) -> Result<(), String> {
    let mut source =
        fs::read_to_string(path).map_err(|e| format!("read '{}': {e}", path.display()))?;
    if addition.starts_with("    <uses-") {
        let mut missing = String::new();
        for line in addition.lines() {
            let Some(name) = line
                .split("android:name=\"")
                .nth(1)
                .and_then(|part| part.split('"').next())
            else {
                missing.push_str(line);
                missing.push('\n');
                continue;
            };
            let identity = format!("android:name=\"{name}\"");
            let existing = source
                .lines()
                .find(|candidate| candidate.contains(&identity))
                .map(str::to_owned);
            match existing {
                Some(existing)
                    if existing.contains("android:maxSdkVersion")
                        && !line.contains("android:maxSdkVersion") =>
                {
                    source = source.replacen(&existing, line, 1);
                }
                Some(_) => {}
                None => {
                    missing.push_str(line);
                    missing.push('\n');
                }
            }
        }
        if missing.is_empty() {
            return fs::write(path, source).map_err(|e| format!("write '{}': {e}", path.display()));
        }
        source = source.replacen(anchor, &format!("{anchor}{missing}"), 1);
        return fs::write(path, source).map_err(|e| format!("write '{}': {e}", path.display()));
    }
    if source.contains(addition.lines().next().unwrap_or_default()) {
        return Ok(());
    }
    if addition.starts_with("[features]") {
        source.push('\n');
        source.push_str(addition);
    } else {
        source = source.replacen(anchor, &format!("{anchor}{addition}"), 1);
    }
    fs::write(path, source).map_err(|e| format!("write '{}': {e}", path.display()))
}

fn handle_extension(extension: NativeExtensionCommands) {
    match extension {
        NativeExtensionCommands::IosShare { dir, name } => {
            scaffold_share_extension(&dir, &name, ShareExtensionPlatform::Ios)
        }
        NativeExtensionCommands::MacosShare { dir, name } => {
            scaffold_share_extension(&dir, &name, ShareExtensionPlatform::Macos)
        }
    }
}

fn handle_build(platform: NativeBuildCommands) {
    match platform {
        NativeBuildCommands::Ios {
            dir,
            target,
            configuration,
        } => build_ios(
            &dir.unwrap_or_else(default_native_dir),
            native_ios_target(target),
            &configuration,
        ),
        NativeBuildCommands::Android { dir, flavor } => {
            build_android(&dir.unwrap_or_else(default_native_dir), &flavor)
        }
    }
}

fn handle_run(platform: NativeRunCommands) {
    match platform {
        NativeRunCommands::Ios { dir } => {
            run_ios_help(&dir.unwrap_or_else(default_native_dir));
        }
        NativeRunCommands::Android { dir, flavor } => {
            run_android(&dir.unwrap_or_else(default_native_dir), &flavor);
        }
    }
}

fn default_native_dir() -> PathBuf {
    PathBuf::from(".")
}

fn run_ir_from(a: NativeIrArgs) {
    let parsed = IrArgs {
        path: a.file,
        component: a.component,
        ctx_file: a.ctx,
        vars: parse_kv_vars(&a.vars),
        pretty: a.pretty,
        stdin: a.stdin,
        stdin_json: a.stdin_json,
        base_dir: a.base_dir,
    };
    match run_ir_parsed(parsed) {
        Ok(out) => print!("{out}"),
        Err(e) => {
            let payload = serde_json::json!({ "error": e });
            eprintln!("{payload}");
            std::process::exit(1);
        }
    }
}

fn sync_from_cli(a: NativeSyncArgs) -> Result<(), String> {
    let template = a
        .template
        .ok_or_else(|| "expected <file.crepus>".to_string())?;
    sync_native_fixture_inner(SyncArgs {
        template,
        dir: a.dir,
        outputs: a.out,
        defaults: !a.no_defaults,
        component: a.component,
        ctx_file: a.ctx,
        vars: parse_kv_vars(&a.vars),
        pretty: a.pretty,
    })
}

fn codegen_from_cli(a: NativeCodegenCliArgs) -> Result<PathBuf, String> {
    let template = a.template.ok_or_else(|| {
        "Usage: crepus native codegen <file.crepus> --platform swiftui|compose --out DIR --view-name NAME".to_string()
    })?;
    let platform = a
        .platform
        .ok_or_else(|| "--platform swiftui|compose is required".to_string())?;
    let out_dir = a.out.ok_or_else(|| "--out DIR is required".to_string())?;
    let view_name = a
        .view_name
        .ok_or_else(|| "--view-name NAME is required".to_string())?;
    codegen_native_source_inner(CodegenArgs {
        template,
        platform: match platform {
            NativeCodegenPlatformArg::SwiftUi => NativeCodegenTarget::SwiftUi,
            NativeCodegenPlatformArg::Compose => NativeCodegenTarget::Compose,
        },
        out_dir,
        view_name,
        component: a.component,
        ctx_file: a.ctx,
        vars: parse_kv_vars(&a.vars),
    })
}

fn parse_kv_vars(vars: &[String]) -> Vec<(String, String)> {
    vars.iter()
        .map(|raw| {
            raw.split_once('=')
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .ok_or_else(|| format!("--var expects key=value, got: {raw}"))
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|e| ui::error(&e))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IrEnvelope {
    entry: Option<String>,
    files: Option<HashMap<String, String>>,
    template: Option<String>,
    context: Option<Value>,
    component: Option<String>,
    pretty: Option<bool>,
    base_dir: Option<PathBuf>,
}

struct IrArgs {
    path: Option<PathBuf>,
    component: Option<String>,
    ctx_file: Option<PathBuf>,
    vars: Vec<(String, String)>,
    pretty: bool,
    stdin: bool,
    stdin_json: bool,
    base_dir: Option<PathBuf>,
}

struct SyncArgs {
    template: PathBuf,
    dir: PathBuf,
    outputs: Vec<PathBuf>,
    defaults: bool,
    component: Option<String>,
    ctx_file: Option<PathBuf>,
    vars: Vec<(String, String)>,
    pretty: bool,
}

struct CodegenArgs {
    template: PathBuf,
    platform: NativeCodegenTarget,
    out_dir: PathBuf,
    view_name: String,
    component: Option<String>,
    ctx_file: Option<PathBuf>,
    vars: Vec<(String, String)>,
}

impl IosBuildTarget {
    fn sdk(self) -> &'static str {
        match self {
            Self::Simulator => "iphonesimulator",
            Self::Device => "iphoneos",
        }
    }

    fn destination(self) -> &'static str {
        match self {
            Self::Simulator => "generic/platform=iOS Simulator",
            Self::Device => "generic/platform=iOS",
        }
    }
}

struct MobileIosConfig {
    scheme: String,
    bundle_id: String,
    development_team: Option<String>,
    code_sign_style: Option<String>,
    allow_provisioning_updates: bool,
}

struct MobileAndroidConfig {
    application_id: Option<String>,
    namespace: Option<String>,
}

fn run_ir_parsed(parsed: IrArgs) -> Result<String, String> {
    if parsed.stdin && parsed.stdin_json {
        return Err("--stdin and --stdin-json are mutually exclusive".to_string());
    }

    let mut ctx = TemplateContext::new();
    if let Some(path) = &parsed.ctx_file {
        load_json_ctx(path, &mut ctx)?;
    }
    for (key, raw) in parsed.vars {
        ctx.set(key, parse_var_value(&raw));
    }

    if parsed.stdin_json {
        let mut raw = String::new();
        std::io::stdin()
            .read_to_string(&mut raw)
            .map_err(|e| format!("read stdin: {e}"))?;
        check_template_size(raw.len())?;
        let env: IrEnvelope = serde_json::from_str(&raw).map_err(|e| format!("stdin JSON: {e}"))?;
        if let Some(value) = env.context {
            merge_json_ctx(&value, &mut ctx)?;
        }
        if let Some(base_dir) = env.base_dir {
            ctx.base_dir = Some(base_dir);
        }
        let pretty = env.pretty.unwrap_or(parsed.pretty);
        let ir = if let (Some(files), Some(entry)) = (env.files, env.entry) {
            render_from_files(&files, &entry, &ctx).map_err(|e| e.to_string())?
        } else if let Some(template) = env.template {
            if let Some(component) = env.component {
                render_component_file_to_ir(&template, &component, &ctx)
                    .map_err(|e| e.to_string())?
            } else {
                render_template_to_ir(&template, &ctx).map_err(|e| e.to_string())?
            }
        } else {
            return Err("stdin JSON must include files+entry or template".to_string());
        };
        return stringify_ir(&ir, pretty);
    }

    if parsed.stdin {
        let mut template = String::new();
        std::io::stdin()
            .read_to_string(&mut template)
            .map_err(|e| format!("read stdin: {e}"))?;
        check_template_size(template.len())?;
        ctx.base_dir = parsed.base_dir;
        let ir = if let Some(component) = parsed.component {
            render_component_file_to_ir(&template, &component, &ctx).map_err(|e| e.to_string())?
        } else {
            render_template_to_ir(&template, &ctx).map_err(|e| e.to_string())?
        };
        return stringify_ir(&ir, parsed.pretty);
    }

    let path = parsed.path.ok_or_else(|| {
        "Usage: crepus native ir <file.crepus> [--component Name] [--ctx FILE] [--var k=v] [--pretty]".to_string()
    })?;
    let content = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    check_template_size(content.len())?;
    ctx.base_dir = path.parent().map(Path::to_path_buf);
    let ir = if let Some(component) = parsed.component {
        render_component_file_to_ir(&content, &component, &ctx).map_err(|e| e.to_string())?
    } else {
        render_template_to_ir(&content, &ctx).map_err(|e| e.to_string())?
    };
    stringify_ir(&ir, parsed.pretty)
}

fn load_json_ctx(path: &Path, ctx: &mut TemplateContext) -> Result<(), String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let value: Value =
        serde_json::from_str(&raw).map_err(|e| format!("context JSON {}: {e}", path.display()))?;
    merge_json_ctx(&value, ctx)
}

fn merge_json_ctx(value: &Value, ctx: &mut TemplateContext) -> Result<(), String> {
    let Some(obj) = value.as_object() else {
        return Err("context must be a JSON object".to_string());
    };
    for (key, value) in obj {
        ctx.set(key.clone(), json_to_template_value(value)?);
    }
    Ok(())
}

fn json_to_template_value(value: &Value) -> Result<TemplateValue, String> {
    match value {
        Value::Null => Ok(TemplateValue::Null),
        Value::Bool(v) => Ok(TemplateValue::Bool(*v)),
        Value::Number(v) => {
            if let Some(n) = v.as_i64() {
                Ok(TemplateValue::Int(n))
            } else if let Some(n) = v.as_f64() {
                Ok(TemplateValue::Float(n))
            } else {
                Err(format!("unsupported number: {v}"))
            }
        }
        Value::String(v) => Ok(TemplateValue::Str(v.clone())),
        Value::Array(values) => {
            let mut items = Vec::new();
            for item in values {
                match item {
                    Value::Object(obj) => {
                        // Each object becomes a child context
                        let mut child = TemplateContext::new();
                        for (key, value) in obj {
                            child.set(key.clone(), json_to_template_scalar(value)?);
                        }
                        items.push(child);
                    }
                    Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null => {
                        // Scalar values become item contexts with "value" key
                        let mut child = TemplateContext::new();
                        child.set("value", json_to_template_scalar(item)?);
                        items.push(child);
                    }
                    _ => return Err("unsupported array item type".to_string()),
                }
            }
            Ok(TemplateValue::List(items))
        }
        Value::Object(_) => {
            Err("context object values are only supported inside arrays".to_string())
        }
    }
}

fn json_to_template_scalar(value: &Value) -> Result<TemplateValue, String> {
    match value {
        Value::Array(_) | Value::Object(_) => {
            Err("loop item fields must be scalar JSON values".to_string())
        }
        _ => json_to_template_value(value),
    }
}

fn parse_var_value(raw: &str) -> TemplateValue {
    match raw {
        "true" => TemplateValue::Bool(true),
        "false" => TemplateValue::Bool(false),
        "null" => TemplateValue::Null,
        _ => raw
            .parse::<i64>()
            .map(TemplateValue::Int)
            .or_else(|_| raw.parse::<f64>().map(TemplateValue::Float))
            .unwrap_or_else(|_| TemplateValue::Str(raw.to_string())),
    }
}

fn stringify_ir(ir: &crepuscularity_native::ViewIr, pretty: bool) -> Result<String, String> {
    if pretty {
        to_json_pretty(ir).map_err(|e| format!("serialize IR: {e}"))
    } else {
        to_json(ir).map_err(|e| format!("serialize IR: {e}"))
    }
}

fn sync_native_fixture_inner(parsed: SyncArgs) -> Result<(), String> {
    if !parsed.dir.exists() {
        return Err(format!(
            "native scaffold dir not found: {}",
            parsed.dir.display()
        ));
    }
    let root = fs::canonicalize(&parsed.dir).unwrap_or(parsed.dir);
    let template_path = resolve_template_path(&root, &parsed.template);
    let template = fs::read_to_string(&template_path)
        .map_err(|e| format!("read {}: {e}", template_path.display()))?;

    let mut ctx = TemplateContext::new();
    if let Some(path) = &parsed.ctx_file {
        load_json_ctx(path, &mut ctx)?;
    }
    for (key, raw) in parsed.vars {
        ctx.set(key, parse_var_value(&raw));
    }
    ctx.base_dir = template_path.parent().map(Path::to_path_buf);

    let component_ref = parsed.component.clone();
    let ir = if let Some(component) = &parsed.component {
        render_component_file_to_ir(&template, component, &ctx).map_err(|e| e.to_string())?
    } else {
        render_template_to_ir(&template, &ctx).map_err(|e| e.to_string())?
    };
    let mut json = stringify_ir(&ir, parsed.pretty)?;
    if !json.ends_with('\n') {
        json.push('\n');
    }

    // Cache skip: avoid rewriting unchanged fixtures (prevents git dirty
    // markers on `crepus native sync` re-runs).
    let cache = DriverCache::open(&root);
    let fp = Fingerprint::new(&template, component_ref.as_deref(), "native-ir");

    let mut written = 0;
    if parsed.defaults {
        for path in default_fixture_output_paths(&root) {
            if let Some(parent) = path.parent() {
                if parent.exists() {
                    if path.is_file() && cache.is_up_to_date(&fp, &json) {
                        written += 1;
                        continue;
                    }
                    fs::write(&path, &json)
                        .map_err(|e| format!("write {}: {e}", path.display()))?;
                    written += 1;
                }
            }
        }
    }
    for path in explicit_fixture_output_paths(&root, &parsed.outputs) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
        }
        if path.is_file() && cache.is_up_to_date(&fp, &json) {
            written += 1;
            continue;
        }
        fs::write(&path, &json).map_err(|e| format!("write {}: {e}", path.display()))?;
        written += 1;
    }
    cache.record(&fp, &json);
    if written == 0 {
        return Err(format!(
            "no native fixture directories found under {}",
            root.display()
        ));
    }
    ui::success(&format!(
        "synced View IR fixture from {}",
        template_path.display()
    ));
    Ok(())
}

fn codegen_native_source_inner(parsed: CodegenArgs) -> Result<PathBuf, String> {
    let template = fs::read_to_string(&parsed.template)
        .map_err(|e| format!("read {}: {e}", parsed.template.display()))?;

    let mut ctx = TemplateContext::new();
    if let Some(path) = &parsed.ctx_file {
        load_json_ctx(path, &mut ctx)?;
    }
    for (key, raw) in parsed.vars {
        ctx.set(key, parse_var_value(&raw));
    }
    ctx.base_dir = parsed.template.parent().map(Path::to_path_buf);

    let ir = if let Some(component) = &parsed.component {
        render_component_file_to_ir(&template, component, &ctx).map_err(|e| e.to_string())?
    } else {
        render_template_to_ir(&template, &ctx).map_err(|e| e.to_string())?
    };
    let mut source = generate_native_source(&ir, parsed.platform, &parsed.view_name);
    if !source.ends_with('\n') {
        source.push('\n');
    }
    fs::create_dir_all(&parsed.out_dir)
        .map_err(|e| format!("create {}: {e}", parsed.out_dir.display()))?;
    let ext = match parsed.platform {
        NativeCodegenTarget::SwiftUi => "swift",
        NativeCodegenTarget::Compose => "kt",
    };
    let path = parsed.out_dir.join(format!("{}.{}", parsed.view_name, ext));
    fs::write(&path, source).map_err(|e| format!("write {}: {e}", path.display()))?;
    if parsed.platform == NativeCodegenTarget::Compose
        && is_native_shell_android_generated_dir(&parsed.out_dir)
    {
        prepend_kotlin_package(&path);
    }
    ui::success(&format!("generated native source at {}", path.display()));
    Ok(path)
}

fn is_native_shell_android_generated_dir(path: &Path) -> bool {
    path.ends_with("android/app/src/main/java/dev/crepuscularity/nativeshell/generated")
}

fn resolve_template_path(root: &Path, template: &Path) -> PathBuf {
    if template.is_absolute() {
        return template.to_path_buf();
    }
    let rooted = root.join(template);
    if rooted.exists() {
        rooted
    } else {
        template.to_path_buf()
    }
}

fn default_fixture_output_paths(root: &Path) -> Vec<PathBuf> {
    vec![
        root.join("fixture.json"),
        root.join("ios/Sources/NativeShell/fixture.json"),
        root.join("NativeShell/Sources/NativeShell/fixture.json"),
        root.join("android/app/src/main/assets/fixture.json"),
    ]
}

fn explicit_fixture_output_paths(root: &Path, explicit: &[PathBuf]) -> Vec<PathBuf> {
    explicit
        .iter()
        .map(|path| {
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                root.join(path)
            }
        })
        .collect()
}

/// Each entry is `(relative path within the scaffold root, file content)`.
///
/// Embedding via `include_str!` keeps the templates next to the published
/// crate source without an explicit `[package].include` list — `cargo
/// publish` walks the source tree by default and picks them up.
const TEMPLATE_FILES: &[(&str, &str)] = &[
    ("README.md", include_str!("../templates/native/README.md")),
    ("crepus.toml", include_str!("../templates/native/crepus.toml")),
    (
        "views/main.crepus",
        include_str!("../templates/native/views/main.crepus"),
    ),
    ("fixture.json", include_str!("../templates/native/fixture.json")),
    ("ios/Package.swift", include_str!("../templates/native/ios/Package.swift")),
    (
        "ios/project.yml",
        include_str!("../templates/native/ios/project.yml"),
    ),
    (
        "ios/crepus.toml",
        include_str!("../templates/native/ios/crepus.toml"),
    ),
    (
        "ios/App/CrepusMobileApp.swift",
        include_str!("../templates/native/ios/App/CrepusMobileApp.swift"),
    ),
    (
        "ios/App/ContentView.swift",
        include_str!("../templates/native/ios/App/ContentView.swift"),
    ),
    (
        "ios/Sources/NativeShell/CrepusRustActions.swift",
        include_str!("../templates/native/ios/Sources/NativeShell/CrepusRustActions.swift"),
    ),
    (
        "ios/Sources/NativeShell/Generated/CrepusGeneratedView.swift",
        include_str!(
            "../templates/native/ios/Sources/NativeShell/Generated/CrepusGeneratedView.swift"
        ),
    ),
    (
        "android/build.gradle.kts",
        include_str!("../templates/native/android/build.gradle.kts"),
    ),
    (
        "android/settings.gradle.kts",
        include_str!("../templates/native/android/settings.gradle.kts"),
    ),
    (
        "android/gradle.properties",
        include_str!("../templates/native/android/gradle.properties"),
    ),
    (
        "android/gradle/wrapper/gradle-wrapper.properties",
        include_str!("../templates/native/android/gradle/wrapper/gradle-wrapper.properties"),
    ),
    (
        "android/app/build.gradle.kts",
        include_str!("../templates/native/android/app/build.gradle.kts"),
    ),
    (
        "android/app/src/main/AndroidManifest.xml",
        include_str!("../templates/native/android/app/src/main/AndroidManifest.xml"),
    ),
    (
        "android/app/src/main/res/values/themes.xml",
        include_str!("../templates/native/android/app/src/main/res/values/themes.xml"),
    ),
    (
        "android/app/src/main/assets/fixture.json",
        include_str!("../templates/native/android/app/src/main/assets/fixture.json"),
    ),
    (
        "android/app/src/main/java/dev/crepuscularity/nativeshell/MainActivity.kt",
        include_str!(
            "../templates/native/android/app/src/main/java/dev/crepuscularity/nativeshell/MainActivity.kt"
        ),
    ),
    (
        "android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt",
        include_str!(
            "../templates/native/android/app/src/main/java/dev/crepuscularity/nativeshell/CrepusRustActions.kt"
        ),
    ),
    (
        "android/app/src/main/java/dev/crepuscularity/nativeshell/generated/CrepusGeneratedView.kt",
        include_str!(
            "../templates/native/android/app/src/main/java/dev/crepuscularity/nativeshell/generated/CrepusGeneratedView.kt"
        ),
    ),
    (
        "rust/Cargo.toml",
        include_str!("../templates/native/rust/Cargo.toml.template"),
    ),
    (
        "rust/src/lib.rs",
        include_str!("../templates/native/rust/src/lib.rs"),
    ),
];

fn scaffold_native_app(name: &str) {
    let root = PathBuf::from(name);
    if root.exists() {
        ui::error(&format!(
            "destination '{}' already exists; pick a fresh name or remove it first",
            root.display()
        ));
    }

    for (rel, content) in TEMPLATE_FILES {
        let target = root.join(rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|e| {
                ui::error(&format!("failed to create '{}': {e}", parent.display()));
            });
        }
        fs::write(&target, content).unwrap_or_else(|e| {
            ui::error(&format!("failed to write '{}': {e}", target.display()));
        });
    }

    let gitignore = "# Build outputs and IDE caches kept out of source control.\n\
                     ios/.build/\n\
                     ios/build/\n\
                     ios/*.xcodeproj/\n\
                     ios/*.xcworkspace/\n\
                     ios/xcuserdata/\n\
                     android/.gradle/\n\
                     android/build/\n\
                     android/app/build/\n\
                     android/local.properties\n\
                     .idea/\n\
                     *.iml\n";
    fs::write(root.join(".gitignore"), gitignore).unwrap_or_else(|e| {
        ui::error(&format!("failed to write .gitignore: {e}"));
    });

    ui::success(&format!(
        "scaffolded native app '{}' at '{}'",
        name,
        root.display()
    ));
    eprintln!();
    eprintln!("{}", style("Next steps").dim());
    eprintln!("  iOS:     crepus mobile build --platform ios --dir {name} --target simulator");
    eprintln!(
        "  Android: cd {dir}/android && gradle wrapper --gradle-version 8.10 && \\\n           ./gradlew :app:assembleDebug",
        dir = name
    );
    eprintln!(
        "  Build via crepus: crepus native build ios --dir {dir} --target simulator",
        dir = name
    );
    eprintln!(
        "                    crepus native build android --dir {dir}",
        dir = name
    );
}

pub(crate) fn scaffold_native_app_at(root: &Path) -> Result<(), String> {
    if root.exists() {
        return Err(format!("destination '{}' already exists", root.display()));
    }
    for (rel, content) in TEMPLATE_FILES {
        let target = root.join(rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(target, content).map_err(|e| e.to_string())?;
    }
    fs::write(
        root.join(".gitignore"),
        "ios/.build/\nandroid/.gradle/\nandroid/build/\nandroid/app/build/\n",
    )
    .map_err(|e| e.to_string())
}

const IOS_SHARE_INFO_PLIST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>NSExtension</key>
	<dict>
		<key>NSExtensionPointIdentifier</key>
		<string>com.apple.share-services</string>
		<key>NSExtensionPrincipalClass</key>
		<string>$(PRODUCT_MODULE_NAME).ShareViewController</string>
	</dict>
</dict>
</plist>
"#;

const IOS_SHARE_VIEW_CONTROLLER: &str = r#"import NativeShell
import UIKit
import UniformTypeIdentifiers

final class ShareViewController: UIViewController {
    override func viewDidLoad() {
        super.viewDidLoad()
        Task {
            let payload = await Self.payload(from: extensionContext)
            let data = try? JSONSerialization.data(withJSONObject: [
                "action": "share.receive",
                "value": payload,
            ])
            if let data, let json = String(data: data, encoding: .utf8) {
                _ = CrepusRustActions.dispatchStored(json)
            }
            extensionContext?.completeRequest(returningItems: nil)
        }
    }

    private static func payload(from context: NSExtensionContext?) async -> [String: Any] {
        var items: [[String: String]] = []
        for case let item as NSExtensionItem in context?.inputItems ?? [] {
            for provider in item.attachments ?? [] {
                if let text = await loadString(provider, .plainText) {
                    items.append(["kind": "text", "value": text])
                } else if let url = await loadString(provider, .url) {
                    items.append(["kind": "url", "value": url])
                }
            }
        }
        return ["items": items]
    }

    private static func loadString(_ provider: NSItemProvider, _ type: UTType) async -> String? {
        guard provider.hasItemConformingToTypeIdentifier(type.identifier) else {
            return nil
        }
        return await withCheckedContinuation { continuation in
            provider.loadItem(forTypeIdentifier: type.identifier) { item, _ in
                if let text = item as? String {
                    continuation.resume(returning: text)
                } else if let url = item as? URL {
                    continuation.resume(returning: url.absoluteString)
                } else {
                    continuation.resume(returning: nil)
                }
            }
        }
    }
}
"#;

const MACOS_SHARE_VIEW_CONTROLLER: &str = r#"import AppKit
import NativeShell
import UniformTypeIdentifiers

final class ShareViewController: NSViewController {
    override func viewDidLoad() {
        super.viewDidLoad()
        Task {
            let payload = await Self.payload(from: extensionContext)
            let data = try? JSONSerialization.data(withJSONObject: [
                "action": "share.receive",
                "value": payload,
            ])
            if let data, let json = String(data: data, encoding: .utf8) {
                _ = CrepusRustActions.dispatchStored(json)
            }
            extensionContext?.completeRequest(returningItems: nil, completionHandler: nil)
        }
    }

    private static func payload(from context: NSExtensionContext?) async -> [String: Any] {
        var items: [[String: String]] = []
        for case let item as NSExtensionItem in context?.inputItems ?? [] {
            for provider in item.attachments ?? [] {
                if let text = await loadString(provider, .plainText) {
                    items.append(["kind": "text", "value": text])
                } else if let url = await loadString(provider, .url) {
                    items.append(["kind": "url", "value": url])
                }
            }
        }
        return ["items": items]
    }

    private static func loadString(_ provider: NSItemProvider, _ type: UTType) async -> String? {
        guard provider.hasItemConformingToTypeIdentifier(type.identifier) else {
            return nil
        }
        return await withCheckedContinuation { continuation in
            provider.loadItem(forTypeIdentifier: type.identifier) { item, _ in
                if let text = item as? String {
                    continuation.resume(returning: text)
                } else if let url = item as? URL {
                    continuation.resume(returning: url.absoluteString)
                } else {
                    continuation.resume(returning: nil)
                }
            }
        }
    }
}
"#;

#[derive(Clone, Copy)]
enum ShareExtensionPlatform {
    Ios,
    Macos,
}

impl ShareExtensionPlatform {
    fn xcode_platform(self) -> &'static str {
        match self {
            Self::Ios => "iOS",
            Self::Macos => "macOS",
        }
    }

    fn source_dir(self) -> &'static str {
        match self {
            Self::Ios => "ShareExtension",
            Self::Macos => "MacShareExtension",
        }
    }

    fn controller(self) -> &'static str {
        match self {
            Self::Ios => IOS_SHARE_VIEW_CONTROLLER,
            Self::Macos => MACOS_SHARE_VIEW_CONTROLLER,
        }
    }

    fn rust_target_script(self) -> &'static str {
        match self {
            Self::Ios => {
                "if [ \"${PLATFORM_NAME:-iphonesimulator}\" = \"iphoneos\" ]; then\n            rust_target=aarch64-apple-ios\n          else\n            rust_target=aarch64-apple-ios-sim\n          fi"
            }
            Self::Macos => {
                "case \"${CURRENT_ARCH:-$(uname -m)}\" in\n            arm64) rust_target=aarch64-apple-darwin ;;\n            x86_64) rust_target=x86_64-apple-darwin ;;\n            *) echo \"unsupported macOS arch: ${CURRENT_ARCH:-$(uname -m)}\" >&2; exit 1 ;;\n          esac"
            }
        }
    }

    fn library_search_paths(self) -> &'static str {
        match self {
            Self::Ios => {
                "        LIBRARY_SEARCH_PATHS[sdk=iphoneos*]: \"$(PROJECT_DIR)/build/rust/aarch64-apple-ios\"\n        LIBRARY_SEARCH_PATHS[sdk=iphonesimulator*]: \"$(PROJECT_DIR)/build/rust/aarch64-apple-ios-sim\""
            }
            Self::Macos => {
                "        LIBRARY_SEARCH_PATHS: \"$(PROJECT_DIR)/build/rust/aarch64-apple-darwin $(PROJECT_DIR)/build/rust/x86_64-apple-darwin\""
            }
        }
    }

    fn output_files(self) -> &'static str {
        match self {
            Self::Ios => {
                "          - \"$(PROJECT_DIR)/build/rust/aarch64-apple-ios/libcrepus_mobile_actions.a\"\n          - \"$(PROJECT_DIR)/build/rust/aarch64-apple-ios-sim/libcrepus_mobile_actions.a\""
            }
            Self::Macos => {
                "          - \"$(PROJECT_DIR)/build/rust/aarch64-apple-darwin/libcrepus_mobile_actions.a\"\n          - \"$(PROJECT_DIR)/build/rust/x86_64-apple-darwin/libcrepus_mobile_actions.a\""
            }
        }
    }
}

fn scaffold_share_extension(dir: &Path, name: &str, platform: ShareExtensionPlatform) {
    let ios_dir = dir.join("ios");
    if !ios_dir.is_dir() {
        ui::error(&format!(
            "{} is not a native scaffold with ios/",
            dir.display()
        ));
    }
    let extension_dir = ios_dir.join(platform.source_dir());
    fs::create_dir_all(&extension_dir).unwrap_or_else(|e| {
        ui::error(&format!(
            "failed to create '{}': {e}",
            extension_dir.display()
        ));
    });
    write_new_file(&extension_dir.join("Info.plist"), IOS_SHARE_INFO_PLIST);
    write_new_file(
        &extension_dir.join("ShareViewController.swift"),
        platform.controller(),
    );
    let project = ios_dir.join("project.yml");
    let mut text = fs::read_to_string(&project)
        .unwrap_or_else(|e| ui::error(&format!("read {}: {e}", project.display())));
    if !text.contains(&format!("  {name}:")) {
        text.push_str(&share_extension_target(
            name,
            &main_bundle_id(&text),
            platform,
        ));
        fs::write(&project, text)
            .unwrap_or_else(|e| ui::error(&format!("write {}: {e}", project.display())));
    }
    ui::success(&format!(
        "added {} share extension target '{name}'",
        platform.xcode_platform()
    ));
}

fn write_new_file(path: &Path, content: &str) {
    if path.exists() {
        return;
    }
    fs::write(path, content)
        .unwrap_or_else(|e| ui::error(&format!("failed to write '{}': {e}", path.display())));
}

fn main_bundle_id(project_yml: &str) -> String {
    project_yml
        .lines()
        .find_map(|line| line.trim().strip_prefix("PRODUCT_BUNDLE_IDENTIFIER: "))
        .unwrap_or("dev.crepuscularity.mobile")
        .trim_matches('"')
        .to_string()
}

fn share_extension_target(
    name: &str,
    main_bundle_id: &str,
    platform: ShareExtensionPlatform,
) -> String {
    format!(
        "\n  {name}:\n    type: app-extension\n    platform: {xcode_platform}\n    sources:\n      - path: {source_dir}\n    settings:\n      base:\n{library_search_paths}\n        OTHER_LDFLAGS: \"-lcrepus_mobile_actions\"\n        EXCLUDED_ARCHS[sdk=iphonesimulator*]: \"x86_64\"\n        INFOPLIST_FILE: {source_dir}/Info.plist\n        PRODUCT_BUNDLE_IDENTIFIER: {main_bundle_id}.share\n    dependencies:\n      - package: NativeShell\n        product: NativeShell\n    preBuildScripts:\n      - name: Build Rust Actions\n        outputFiles:\n{output_files}\n        script: |\n          set -euo pipefail\n          export PATH=\"$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$PATH\"\n          {rust_target_script}\n          rustup target add \"$rust_target\" >/dev/null\n          cargo_profile=debug\n          if [ \"${{CONFIGURATION:-Debug}}\" = \"Release\" ]; then\n            cargo_profile=release\n          fi\n          cargo build --manifest-path \"$PROJECT_DIR/../rust/Cargo.toml\" \\\n            --target \"$rust_target\" \\\n            $([ \"$cargo_profile\" = release ] && echo \"--release\") \\\n            --no-default-features\n          mkdir -p \"$PROJECT_DIR/build/rust/$rust_target\"\n          cp \"$PROJECT_DIR/../rust/target/$rust_target/$cargo_profile/libcrepus_mobile_actions.a\" \"$PROJECT_DIR/build/rust/$rust_target/\"\n",
        xcode_platform = platform.xcode_platform(),
        source_dir = platform.source_dir(),
        library_search_paths = platform.library_search_paths(),
        output_files = platform.output_files(),
        rust_target_script = platform.rust_target_script(),
    )
}

fn build_ios(dir: &Path, target: IosBuildTarget, configuration: &str) {
    let ios_dir = dir.join("ios");
    if ios_dir.join("project.yml").exists() {
        build_ios_app(&ios_dir, target, configuration);
        return;
    }
    if !ios_dir.join("Package.swift").exists() {
        ui::error(&format!(
            "no Package.swift at '{}'. Pass --dir <path-to-scaffold-root> if the project lives elsewhere.",
            ios_dir.display()
        ));
    }
    let mut cmd = Command::new("swift");
    cmd.arg("build").current_dir(&ios_dir);
    if configuration == "Release" {
        cmd.args(["-c", "release"]);
    } else {
        cmd.args(["-c", "debug"]);
    }
    delegate(cmd, "swift build");
}

fn build_ios_app(ios_dir: &Path, target: IosBuildTarget, configuration: &str) {
    let spec = ios_dir.join("project.yml");
    if !spec.exists() {
        ui::error(&format!("no project.yml at '{}'", spec.display()));
    }
    if let Some(root) = ios_dir.parent() {
        sync_default_mobile_artifacts(root, true, false);
    }
    let cfg = load_mobile_ios_config(ios_dir);
    let mut xcodegen = Command::new("xcodegen");
    xcodegen
        .current_dir(ios_dir)
        .args(["generate", "--spec", "project.yml"]);
    delegate(xcodegen, "xcodegen generate");

    let project = ios_dir.join(format!("{}.xcodeproj", cfg.scheme));
    let project_name = if project.exists() {
        project
    } else {
        find_xcodeproj(ios_dir).unwrap_or_else(|| {
            ui::error(&format!(
                "no .xcodeproj generated in '{}'",
                ios_dir.display()
            ));
        })
    };
    let mut build = Command::new("xcodebuild");
    build.current_dir(ios_dir).args([
        "-project",
        project_name
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("CrepusMobileApp.xcodeproj"),
        "-target",
        &cfg.scheme,
        "-sdk",
        target.sdk(),
        "-configuration",
        configuration,
        "-destination",
        target.destination(),
        "build",
        "SYMROOT=build",
    ]);
    build.arg(format!("PRODUCT_BUNDLE_IDENTIFIER={}", cfg.bundle_id));
    if cfg.allow_provisioning_updates {
        build.arg("-allowProvisioningUpdates");
    }
    if let Some(development_team) = &cfg.development_team {
        build.arg(format!("DEVELOPMENT_TEAM={development_team}"));
    }
    if let Some(code_sign_style) = &cfg.code_sign_style {
        build.arg(format!("CODE_SIGN_STYLE={code_sign_style}"));
    }
    delegate(build, "xcodebuild");
}

fn build_android(dir: &Path, flavor: &str) {
    sync_default_mobile_artifacts(dir, false, true);
    let android_dir = dir.join("android");
    apply_android_config(&android_dir, &load_mobile_android_config(dir));
    let gradlew = android_dir.join("gradlew");
    if !android_dir.join("settings.gradle.kts").exists() {
        ui::error(&format!(
            "no settings.gradle.kts at '{}'. Pass --dir <path-to-scaffold-root> if the project lives elsewhere.",
            android_dir.display()
        ));
    }

    let task = format!(":app:assemble{}", capitalize_ascii(flavor));
    let mut cmd = if gradlew.exists() {
        let mut c = Command::new("./gradlew");
        c.current_dir(&android_dir);
        c.arg(&task);
        c
    } else {
        // Fall back to system `gradle`. Print a hint either way so users know
        // why we're not invoking ./gradlew.
        eprintln!(
            "{} no ./gradlew at {}; using system `gradle` (run `gradle wrapper --gradle-version 8.10` to generate the wrapper)",
            style("note:").yellow(),
            gradlew.display()
        );
        let mut c = Command::new("gradle");
        c.current_dir(&android_dir);
        c.arg(&task);
        c
    };
    configure_gradle_java(&mut cmd);
    cmd.arg("--quiet"); // don't drown the user in gradle log spam
    delegate(cmd, "gradle build");
}

fn run_ios_help(dir: &Path) {
    let ios_dir = dir.join("ios");
    if !ios_dir.join("project.yml").exists() {
        eprintln!(
            "{}",
            style("crepus native run ios — open in Xcode").cyan().bold()
        );
        eprintln!();
        eprintln!("  open {dir}/ios/Package.swift", dir = dir.display());
        eprintln!();
        eprintln!(
            "{} SwiftPM-only scaffold has no installable app target.",
            style("note:").yellow()
        );
        return;
    }
    build_ios_app(&ios_dir, IosBuildTarget::Simulator, "Debug");
    run_ios_app(&ios_dir);
}

fn run_ios_app(ios_dir: &Path) {
    let cfg = load_mobile_ios_config(ios_dir);
    let app = find_built_ios_app(ios_dir, &cfg).unwrap_or_else(|| {
        ui::error(&format!(
            "no built .app under '{}'",
            ios_dir.join("build").display()
        ));
    });
    let device = booted_or_available_ios_device().unwrap_or_else(|| {
        ui::error("no available iOS simulator device found");
    });
    let _ = Command::new("xcrun")
        .args(["simctl", "boot", &device])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let bootstatus = Command::new("xcrun")
        .args(["simctl", "bootstatus", &device, "-b"])
        .status()
        .unwrap_or_else(|e| ui::error(&format!("simctl bootstatus: {e}")));
    if !bootstatus.success() {
        ui::error("iOS simulator failed to boot");
    }
    let install = Command::new("xcrun")
        .args(["simctl", "install", &device])
        .arg(&app)
        .status()
        .unwrap_or_else(|e| ui::error(&format!("simctl install: {e}")));
    if !install.success() {
        ui::error("simctl install failed");
    }
    let launch = Command::new("xcrun")
        .args(["simctl", "launch", &device, &cfg.bundle_id])
        .status()
        .unwrap_or_else(|e| ui::error(&format!("simctl launch: {e}")));
    if !launch.success() {
        ui::error("simctl launch failed");
    }
    eprintln!(
        "{} installed and launched {} on {}",
        style("ios:").green(),
        app.display(),
        device
    );
}

fn load_mobile_ios_config(ios_dir: &Path) -> MobileIosConfig {
    let toml = fs::read_to_string(ios_dir.join("crepus.toml")).unwrap_or_default();
    let root_toml = ios_dir
        .parent()
        .map(|root| fs::read_to_string(root.join("crepus.toml")).unwrap_or_default())
        .unwrap_or_default();
    let project = fs::read_to_string(ios_dir.join("project.yml")).unwrap_or_default();
    let root_manifest = crate::crepus_toml::CrepusManifest::parse(&root_toml)
        .ok()
        .and_then(|manifest| manifest.ios);
    let manifest = crate::crepus_toml::CrepusManifest::parse(&toml)
        .ok()
        .and_then(|manifest| manifest.ios);
    MobileIosConfig {
        scheme: manifest
            .as_ref()
            .map(|ios| ios.scheme.clone())
            .or_else(|| root_manifest.as_ref().map(|ios| ios.scheme.clone()))
            .or_else(|| toml_value(&toml, "scheme"))
            .or_else(|| project_name(&project))
            .unwrap_or_else(|| "CrepusMobileApp".to_string()),
        bundle_id: manifest
            .as_ref()
            .and_then(|ios| ios.bundle_id.clone())
            .or_else(|| root_manifest.as_ref().and_then(|ios| ios.bundle_id.clone()))
            .or_else(|| project_bundle_id(&project))
            .unwrap_or_else(|| "dev.crepuscularity.mobile".to_string()),
        development_team: manifest
            .as_ref()
            .and_then(|ios| ios.development_team.clone())
            .or_else(|| {
                root_manifest
                    .as_ref()
                    .and_then(|ios| ios.development_team.clone())
            }),
        code_sign_style: manifest
            .as_ref()
            .and_then(|ios| ios.code_sign_style.clone())
            .or_else(|| {
                root_manifest
                    .as_ref()
                    .and_then(|ios| ios.code_sign_style.clone())
            }),
        allow_provisioning_updates: manifest
            .as_ref()
            .is_some_and(|ios| ios.allow_provisioning_updates)
            || root_manifest
                .as_ref()
                .is_some_and(|ios| ios.allow_provisioning_updates),
    }
}

fn load_mobile_android_config(root: &Path) -> MobileAndroidConfig {
    let toml = fs::read_to_string(root.join("crepus.toml")).unwrap_or_default();
    let manifest = crate::crepus_toml::CrepusManifest::parse(&toml).ok();
    let android = manifest.and_then(|manifest| manifest.android);
    MobileAndroidConfig {
        application_id: android
            .as_ref()
            .and_then(|android| android.application_id.clone()),
        namespace: android.and_then(|android| android.namespace),
    }
}

fn apply_android_config(android_dir: &Path, cfg: &MobileAndroidConfig) {
    if cfg.application_id.is_none() && cfg.namespace.is_none() {
        return;
    }
    let gradle_path = android_dir.join("app/build.gradle.kts");
    let Ok(mut gradle) = fs::read_to_string(&gradle_path) else {
        return;
    };
    if let Some(namespace) = &cfg.namespace {
        gradle = replace_kotlin_string_assignment(&gradle, "namespace", namespace);
    }
    if let Some(application_id) = &cfg.application_id {
        gradle = replace_kotlin_string_assignment(&gradle, "applicationId", application_id);
    }
    fs::write(&gradle_path, gradle).unwrap_or_else(|e| {
        ui::error(&format!("failed to write '{}': {e}", gradle_path.display()));
    });
}

fn replace_kotlin_string_assignment(src: &str, key: &str, value: &str) -> String {
    src.lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with(key) {
                let indent_len = line.len() - trimmed.len();
                format!("{}{} = \"{}\"", &line[..indent_len], key, value)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn toml_value(src: &str, key: &str) -> Option<String> {
    src.lines().find_map(|line| {
        let line = line.trim();
        let rest = line.strip_prefix(key)?.trim_start();
        let rest = rest.strip_prefix('=')?.trim();
        Some(rest.trim_matches('"').to_string())
    })
}

fn project_name(src: &str) -> Option<String> {
    src.lines().find_map(|line| {
        let line = line.trim();
        let rest = line.strip_prefix("name:")?.trim();
        if rest.is_empty() {
            None
        } else {
            Some(rest.to_string())
        }
    })
}

fn project_bundle_id(src: &str) -> Option<String> {
    src.lines().find_map(|line| {
        let line = line.trim();
        let rest = line.strip_prefix("PRODUCT_BUNDLE_IDENTIFIER:")?.trim();
        if rest.is_empty() {
            None
        } else {
            Some(rest.to_string())
        }
    })
}

fn find_xcodeproj(dir: &Path) -> Option<PathBuf> {
    fs::read_dir(dir).ok()?.flatten().find_map(|entry| {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "xcodeproj") {
            Some(path)
        } else {
            None
        }
    })
}

fn find_built_ios_app(ios_dir: &Path, cfg: &MobileIosConfig) -> Option<PathBuf> {
    let direct = ios_dir
        .join("build/Debug-iphonesimulator")
        .join(format!("{}.app", cfg.scheme));
    if direct.exists() {
        return Some(direct);
    }
    find_app_under(&ios_dir.join("build"))
}

fn find_app_under(dir: &Path) -> Option<PathBuf> {
    for entry in fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "app") {
            return Some(path);
        }
        if path.is_dir() {
            if let Some(app) = find_app_under(&path) {
                return Some(app);
            }
        }
    }
    None
}

fn booted_or_available_ios_device() -> Option<String> {
    let output = Command::new("xcrun")
        .args(["simctl", "list", "devices", "available"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let mut first = None;
    for line in text.lines() {
        if !line.contains("(Booted)") && !line.contains("(Shutdown)") {
            continue;
        }
        let Some(id) = simulator_id_from_line(line) else {
            continue;
        };
        if line.contains("(Booted)") {
            return Some(id);
        }
        if first.is_none() {
            first = Some(id);
        }
    }
    first
}

fn simulator_id_from_line(line: &str) -> Option<String> {
    let start = line.find('(')? + 1;
    let rest = &line[start..];
    let end = rest.find(')')?;
    let candidate = &rest[..end];
    if candidate.chars().filter(|c| *c == '-').count() == 4 {
        Some(candidate.to_string())
    } else {
        None
    }
}

fn run_android(dir: &Path, flavor: &str) {
    sync_default_mobile_artifacts(dir, false, true);
    let android_dir = dir.join("android");
    let gradlew = android_dir.join("gradlew");
    let task = format!(":app:install{}", capitalize_ascii(flavor));
    let application_id = load_android_application_id(&android_dir)
        .unwrap_or_else(|| "dev.crepuscularity.nativeshell".to_string());

    let mut cmd = if gradlew.exists() {
        let mut c = Command::new("./gradlew");
        c.current_dir(&android_dir);
        c.arg(&task);
        c
    } else {
        let mut c = Command::new("gradle");
        c.current_dir(&android_dir);
        c.arg(&task);
        c
    };
    configure_gradle_java(&mut cmd);
    cmd.arg("--quiet");
    delegate(cmd, "gradle install");

    let component = android_main_activity_component(&application_id);
    let mut launch = Command::new("adb");
    launch.args(["shell", "am", "start", "-n", &component]);
    delegate(launch, "adb launch");

    eprintln!(
        "\n{} installed and launched {}",
        style("android:").green(),
        component
    );
}

fn load_android_application_id(android_dir: &Path) -> Option<String> {
    let gradle = fs::read_to_string(android_dir.join("app/build.gradle.kts")).ok()?;
    gradle_kts_value(&gradle, "applicationId")
}

fn android_main_activity_component(application_id: &str) -> String {
    format!("{application_id}/dev.crepuscularity.nativeshell.MainActivity")
}

fn gradle_kts_value(src: &str, key: &str) -> Option<String> {
    src.lines().find_map(|line| {
        let line = line.trim();
        let rest = line.strip_prefix(key)?.trim_start();
        let rest = rest.strip_prefix('=')?.trim();
        Some(rest.trim_matches('"').to_string())
    })
}

fn sync_default_mobile_artifacts(root: &Path, ios: bool, android: bool) {
    let template = root.join("views/main.crepus");
    if !template.exists() {
        return;
    }
    sync_native_fixture_inner(SyncArgs {
        template: template.clone(),
        dir: root.to_path_buf(),
        outputs: Vec::new(),
        defaults: true,
        component: None,
        ctx_file: None,
        vars: Vec::new(),
        pretty: true,
    })
    .unwrap_or_else(|e| ui::error(&e));
    if ios {
        codegen_native_source_inner(CodegenArgs {
            template: template.clone(),
            platform: NativeCodegenTarget::SwiftUi,
            out_dir: root.join("ios/Sources/NativeShell/Generated"),
            view_name: "CrepusGeneratedView".to_string(),
            component: None,
            ctx_file: None,
            vars: Vec::new(),
        })
        .unwrap_or_else(|e| ui::error(&e));
    }
    if android {
        let out_dir =
            root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/generated");
        codegen_native_source_inner(CodegenArgs {
            template: template.clone(),
            platform: NativeCodegenTarget::Compose,
            out_dir: out_dir.clone(),
            view_name: "CrepusGeneratedView".to_string(),
            component: None,
            ctx_file: None,
            vars: Vec::new(),
        })
        .unwrap_or_else(|e| ui::error(&e));
        prepend_kotlin_package(&out_dir.join("CrepusGeneratedView.kt"));
    }
}

fn prepend_kotlin_package(path: &Path) {
    let Ok(source) = fs::read_to_string(path) else {
        return;
    };
    if source.starts_with("package ") {
        return;
    }
    let updated = format!("package dev.crepuscularity.nativeshell\n\n{source}");
    let _ = fs::write(path, updated);
}

fn delegate(mut cmd: Command, label: &str) {
    match cmd.status() {
        Ok(status) if status.success() => {}
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(e) => ui::error(&format!(
            "failed to invoke `{label}`: {e}. Is the toolchain installed and on PATH?"
        )),
    }
}

fn configure_gradle_java(cmd: &mut Command) {
    match std::env::var("JAVA_HOME") {
        Ok(raw) if PathBuf::from(&raw).join("bin/java").exists() => {}
        _ => {
            if let Some(java_home) = discover_java_home() {
                cmd.env("JAVA_HOME", java_home);
            } else {
                cmd.env_remove("JAVA_HOME");
            }
        }
    }
}

fn discover_java_home() -> Option<String> {
    for candidate in [
        "/opt/homebrew/opt/openjdk@17/libexec/openjdk.jdk/Contents/Home",
        "/opt/homebrew/opt/openjdk/libexec/openjdk.jdk/Contents/Home",
    ] {
        let path = Path::new(candidate);
        if path.join("bin/java").exists() {
            return Some(candidate.to_string());
        }
    }
    if let Ok(out) = Command::new("/usr/libexec/java_home")
        .args(["-v", "17"])
        .output()
    {
        if out.status.success() {
            let path = String::from_utf8(out.stdout).ok()?;
            let trimmed = path.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    let java = fs::canonicalize("/opt/homebrew/bin/java")
        .or_else(|_| fs::canonicalize("/usr/bin/java"))
        .ok()?;
    let home = java.parent()?.parent()?;
    if home.join("bin/java").exists() {
        Some(home.display().to_string())
    } else {
        None
    }
}

fn capitalize_ascii(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    if let Some(c) = chars.next() {
        out.extend(c.to_uppercase());
    }
    out.extend(chars);
    out
}

/// ponytail: cap template source at 10 MB
const MAX_TEMPLATE_SIZE: usize = 10_000_000;

fn check_template_size(len: usize) -> Result<(), String> {
    if len > MAX_TEMPLATE_SIZE {
        return Err(format!(
            "template too large ({} bytes, max {})",
            len, MAX_TEMPLATE_SIZE
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capitalize_ascii_basic() {
        assert_eq!(capitalize_ascii("debug"), "Debug");
        assert_eq!(capitalize_ascii("Release"), "Release");
        assert_eq!(capitalize_ascii(""), "");
        assert_eq!(capitalize_ascii("a"), "A");
    }

    #[test]
    fn gradle_kts_value_reads_application_id() {
        let src = r#"
android {
    defaultConfig {
        applicationId = "dev.crepuscularity.nativeshell"
    }
}
"#;
        assert_eq!(
            gradle_kts_value(src, "applicationId"),
            Some("dev.crepuscularity.nativeshell".to_string())
        );
    }

    #[test]
    fn android_component_uses_generated_shell_package() {
        assert_eq!(
            android_main_activity_component("hk.tsc.acme"),
            "hk.tsc.acme/dev.crepuscularity.nativeshell.MainActivity"
        );
    }

    #[test]
    fn ios_share_target_uses_main_bundle_suffix() {
        let project = r#"
targets:
  App:
    settings:
      base:
        PRODUCT_BUNDLE_IDENTIFIER: hk.tsc.acme
"#;
        assert_eq!(main_bundle_id(project), "hk.tsc.acme");
        let target = share_extension_target(
            "AcmeShare",
            &main_bundle_id(project),
            ShareExtensionPlatform::Ios,
        );
        assert!(target.contains("  AcmeShare:"));
        assert!(target.contains("type: app-extension"));
        assert!(target.contains("platform: iOS"));
        assert!(target.contains("path: ShareExtension"));
        assert!(target.contains("PRODUCT_BUNDLE_IDENTIFIER: hk.tsc.acme.share"));
        assert!(target.contains("Build Rust Actions"));
    }

    #[test]
    fn macos_share_target_uses_macos_sources_and_rust_target() {
        let target =
            share_extension_target("AcmeMacShare", "hk.tsc.acme", ShareExtensionPlatform::Macos);
        assert!(target.contains("  AcmeMacShare:"));
        assert!(target.contains("platform: macOS"));
        assert!(target.contains("path: MacShareExtension"));
        assert!(target.contains("aarch64-apple-darwin"));
        assert!(target.contains("x86_64-apple-darwin"));
    }

    #[test]
    fn mobile_ios_config_reads_root_manifest_identity() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("ios")).unwrap();
        fs::write(
            root.join("crepus.toml"),
            r#"
[ios]
bundle_id = "hk.tsc.acme"
development_team = "LZ3NL5434Q"
code_sign_style = "Automatic"
allow_provisioning_updates = true
"#,
        )
        .unwrap();
        fs::write(
            root.join("ios/crepus.toml"),
            r#"
[ios]
scheme = "CrepusMobileApp"
"#,
        )
        .unwrap();
        fs::write(
            root.join("ios/project.yml"),
            "name: CrepusMobileApp\nPRODUCT_BUNDLE_IDENTIFIER: dev.crepuscularity.mobile\n",
        )
        .unwrap();

        let cfg = load_mobile_ios_config(&root.join("ios"));
        assert_eq!(cfg.scheme, "CrepusMobileApp");
        assert_eq!(cfg.bundle_id, "hk.tsc.acme");
        assert_eq!(cfg.development_team.as_deref(), Some("LZ3NL5434Q"));
        assert_eq!(cfg.code_sign_style.as_deref(), Some("Automatic"));
        assert!(cfg.allow_provisioning_updates);
    }

    #[test]
    fn apply_android_config_rewrites_gradle_identity() {
        let temp = tempfile::tempdir().unwrap();
        let android_dir = temp.path().join("android");
        fs::create_dir_all(android_dir.join("app")).unwrap();
        fs::write(
            android_dir.join("app/build.gradle.kts"),
            r#"android {
    namespace = "dev.crepuscularity.nativeshell"
    defaultConfig {
        applicationId = "dev.crepuscularity.nativeshell"
    }
}
"#,
        )
        .unwrap();

        apply_android_config(
            &android_dir,
            &MobileAndroidConfig {
                application_id: Some("hk.tsc.acme".to_string()),
                namespace: Some("hk.tsc.acme".to_string()),
            },
        );
        let gradle = fs::read_to_string(android_dir.join("app/build.gradle.kts")).unwrap();
        assert!(gradle.contains("namespace = \"hk.tsc.acme\""));
        assert!(gradle.contains("applicationId = \"hk.tsc.acme\""));
    }

    #[test]
    fn sync_fixture_writes_native_shell_outputs() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("app");
        fs::create_dir_all(root.join("views")).unwrap();
        fs::create_dir_all(root.join("ios/Sources/NativeShell")).unwrap();
        fs::create_dir_all(root.join("android/app/src/main/assets")).unwrap();
        fs::write(
            root.join("views/main.crepus"),
            "div flex flex-col\n  span\n    \"Hello {name}\"",
        )
        .unwrap();

        sync_native_fixture_inner(SyncArgs {
            template: root.join("views/main.crepus"),
            dir: root.clone(),
            outputs: vec![root.join("linux/share/dashboard.view-ir.json")],
            defaults: true,
            component: None,
            ctx_file: None,
            vars: vec![("name".into(), "Acme".into())],
            pretty: true,
        })
        .unwrap();

        let root_fixture = fs::read_to_string(root.join("fixture.json")).unwrap();
        let ios_fixture =
            fs::read_to_string(root.join("ios/Sources/NativeShell/fixture.json")).unwrap();
        let android_fixture =
            fs::read_to_string(root.join("android/app/src/main/assets/fixture.json")).unwrap();
        let linux_fixture =
            fs::read_to_string(root.join("linux/share/dashboard.view-ir.json")).unwrap();

        assert_eq!(root_fixture, ios_fixture);
        assert_eq!(root_fixture, android_fixture);
        assert_eq!(root_fixture, linux_fixture);
        assert!(root_fixture.contains("Hello Acme"));
        assert!(root_fixture.contains("\"kind\": \"stack\""));
    }

    #[test]
    fn sync_fixture_can_write_only_explicit_outputs() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("app");
        fs::create_dir_all(root.join("views")).unwrap();
        fs::write(
            root.join("views/main.crepus"),
            "div flex flex-col\n  span\n    \"Hello {name}\"",
        )
        .unwrap();

        sync_native_fixture_inner(SyncArgs {
            template: root.join("views/main.crepus"),
            dir: root.clone(),
            outputs: vec![root.join("desktop/dashboard.view-ir.json")],
            defaults: false,
            component: None,
            ctx_file: None,
            vars: vec![("name".into(), "Acme".into())],
            pretty: true,
        })
        .unwrap();

        let desktop_fixture =
            fs::read_to_string(root.join("desktop/dashboard.view-ir.json")).unwrap();
        assert!(!root.join("fixture.json").exists());
        assert!(desktop_fixture.contains("Hello Acme"));
    }

    #[test]
    fn template_files_present() {
        // Smoke-test: every embedded file is non-empty so we know `include_str!`
        // is wired correctly to existing template files.
        for (rel, content) in TEMPLATE_FILES {
            assert!(!content.is_empty(), "empty template content at {rel}");
        }
        assert!(!TEMPLATE_FILES.iter().any(|(rel, content)| {
            rel.contains("btleplug")
                || rel.contains("gedgygedgy")
                || content.contains("android.permission.BLUETOOTH_SCAN")
                || content.contains("CoreBluetooth")
        }));
        assert!(TEMPLATE_FILES.iter().any(|(rel, _)| {
            *rel == "android/app/src/main/java/dev/crepuscularity/nativeshell/MainActivity.kt"
        }));
        assert!(TEMPLATE_FILES
            .iter()
            .any(|(rel, _)| { *rel == "ios/Sources/NativeShell/CrepusRustActions.swift" }));
    }

    #[test]
    fn template_files_have_unique_paths() {
        use std::collections::BTreeSet;
        let mut seen = BTreeSet::new();
        for (rel, _) in TEMPLATE_FILES {
            assert!(seen.insert(*rel), "duplicate template entry: {rel}");
        }
    }
}
