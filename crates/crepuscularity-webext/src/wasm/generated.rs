use wasm_bindgen::prelude::*;

use super::core::{self, EventListenerGuard, RawNamespace, Result};

macro_rules! raw_namespace_module {
    ($module:ident, $path:literal) => {
        pub mod $module {
            use super::*;

            pub const PATH: &str = $path;

            pub fn namespace() -> Result<RawNamespace> {
                core::raw_namespace(PATH)
            }

            pub async fn call(method: &str, args: &[JsValue]) -> Result<JsValue> {
                namespace()?.call(method, args).await
            }

            pub async fn call_callback(method: &str, args: &[JsValue]) -> Result<JsValue> {
                namespace()?.call_callback(method, args).await
            }

            pub fn on_raw<F>(event: &str, handler: F) -> Result<EventListenerGuard>
            where
                F: FnMut(JsValue, JsValue, JsValue) + 'static,
            {
                namespace()?.on_raw(event, handler)
            }
        }
    };
}

raw_namespace_module!(accessibility_features, "accessibilityFeatures");
raw_namespace_module!(alarms, "alarms");
raw_namespace_module!(audio, "audio");
raw_namespace_module!(bookmarks, "bookmarks");
raw_namespace_module!(browsing_data, "browsingData");
raw_namespace_module!(certificate_provider, "certificateProvider");
raw_namespace_module!(content_settings, "contentSettings");
raw_namespace_module!(context_menus, "contextMenus");
raw_namespace_module!(cookies, "cookies");
raw_namespace_module!(debugger, "debugger");
raw_namespace_module!(declarative_content, "declarativeContent");
raw_namespace_module!(declarative_net_request, "declarativeNetRequest");
raw_namespace_module!(desktop_capture, "desktopCapture");
raw_namespace_module!(devtools_inspected_window, "devtools.inspectedWindow");
raw_namespace_module!(devtools_network, "devtools.network");
raw_namespace_module!(devtools_panels, "devtools.panels");
raw_namespace_module!(document_scan, "documentScan");
raw_namespace_module!(dom, "dom");
raw_namespace_module!(downloads, "downloads");
raw_namespace_module!(enterprise_device_attributes, "enterprise.deviceAttributes");
raw_namespace_module!(enterprise_hardware_platform, "enterprise.hardwarePlatform");
raw_namespace_module!(enterprise_login, "enterprise.login");
raw_namespace_module!(
    enterprise_networking_attributes,
    "enterprise.networkingAttributes"
);
raw_namespace_module!(enterprise_platform_keys, "enterprise.platformKeys");
raw_namespace_module!(events, "events");
raw_namespace_module!(extension, "extension");
raw_namespace_module!(extension_types, "extensionTypes");
raw_namespace_module!(file_browser_handler, "fileBrowserHandler");
raw_namespace_module!(file_system_provider, "fileSystemProvider");
raw_namespace_module!(font_settings, "fontSettings");
raw_namespace_module!(gcm, "gcm");
raw_namespace_module!(history, "history");
raw_namespace_module!(i18n, "i18n");
raw_namespace_module!(identity, "identity");
raw_namespace_module!(idle, "idle");
raw_namespace_module!(input_ime, "input.ime");
raw_namespace_module!(instance_id, "instanceID");
raw_namespace_module!(login_state, "loginState");
raw_namespace_module!(management, "management");
raw_namespace_module!(notifications, "notifications");
raw_namespace_module!(offscreen, "offscreen");
raw_namespace_module!(omnibox, "omnibox");
raw_namespace_module!(page_capture, "pageCapture");
raw_namespace_module!(permissions, "permissions");
raw_namespace_module!(platform_keys, "platformKeys");
raw_namespace_module!(power, "power");
raw_namespace_module!(printer_provider, "printerProvider");
raw_namespace_module!(printing, "printing");
raw_namespace_module!(printing_metrics, "printingMetrics");
raw_namespace_module!(privacy, "privacy");
raw_namespace_module!(proxy, "proxy");
raw_namespace_module!(reading_list, "readingList");
raw_namespace_module!(search, "search");
raw_namespace_module!(sessions, "sessions");
raw_namespace_module!(side_panel, "sidePanel");
raw_namespace_module!(system_cpu, "system.cpu");
raw_namespace_module!(system_display, "system.display");
raw_namespace_module!(system_memory, "system.memory");
raw_namespace_module!(system_storage, "system.storage");
raw_namespace_module!(system_log, "systemLog");
raw_namespace_module!(tab_capture, "tabCapture");
raw_namespace_module!(tab_groups, "tabGroups");
raw_namespace_module!(top_sites, "topSites");
raw_namespace_module!(tts, "tts");
raw_namespace_module!(tts_engine, "ttsEngine");
raw_namespace_module!(types, "types");
raw_namespace_module!(user_scripts, "userScripts");
raw_namespace_module!(vpn_provider, "vpnProvider");
raw_namespace_module!(wallpaper, "wallpaper");
raw_namespace_module!(web_authentication_proxy, "webAuthenticationProxy");
raw_namespace_module!(web_navigation, "webNavigation");
raw_namespace_module!(web_request, "webRequest");
raw_namespace_module!(windows, "windows");
