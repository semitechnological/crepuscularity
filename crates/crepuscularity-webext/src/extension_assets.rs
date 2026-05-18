//! Default MV3 UI assets for `crepus webext build`, embedded relative to this crate.

pub const POPUP_HTML: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/popup.html"));
pub const POPUP_JS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/popup.js"));
pub const OPTIONS_JS: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/options.js"));
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

#[cfg(test)]
mod tests {
    use super::{BACKGROUND_JS, CONTENT_JS};

    #[test]
    fn background_host_is_mv3_service_worker_safe() {
        assert!(BACKGROUND_JS.contains("import init, * as runtimeModule"));
        assert!(!BACKGROUND_JS.contains("await import("));
        assert!(!BACKGROUND_JS.contains("const runtimeModule = await"));
        assert!(!BACKGROUND_JS.contains("await fetch("));
    }

    #[test]
    fn content_host_cache_busts_runtime_assets() {
        assert!(CONTENT_JS.contains("?v=${cacheKey}"));
        assert!(CONTENT_JS.contains("cache: \"no-store\""));
    }

    #[test]
    fn content_host_skips_child_frame_wasm_compile_failures() {
        assert!(CONTENT_JS.contains("globalThis.top === globalThis"));
        assert!(CONTENT_JS.contains("if (topFrame) throw error"));
        assert!(CONTENT_JS.contains("error instanceof WebAssembly.CompileError"));
    }
}
