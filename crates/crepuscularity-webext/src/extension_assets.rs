//! Default MV3 UI assets for `crepus webext build`, embedded relative to this crate.

pub const POPUP_HTML: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/popup.html"));
pub const POPUP_JS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/popup.js"));
pub const POPUP_CSS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/popup.css"));
pub const BACKGROUND_JS: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/background.js"));
pub const CONTENT_JS: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/content.js"));
pub const CONTENT_CSS: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/content.css"));
pub const BROWSER_SHIM: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/browser-shim.js"
));
pub const RUNTIME_ADAPTER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/runtime-as-adapter.js"
));
pub static UNOCSS_JS: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/vendor/unocss.js"
));
