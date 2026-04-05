use serde_json::Value;
use std::collections::HashMap;

/// Read SSR context and wire signals to DOM.
/// `setup` receives the deserialized server context and should call bind_* functions.
/// On non-WASM targets this is a no-op (useful for native tests).
pub fn hydrate_root<F: FnOnce(HashMap<String, Value>)>(setup: F) {
    #[cfg(target_arch = "wasm32")]
    {
        let ctx = read_ctx();
        setup(ctx);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = setup;
    }
}

#[cfg(target_arch = "wasm32")]
fn read_ctx() -> HashMap<String, Value> {
    let doc = web_sys::window().unwrap().document().unwrap();
    let Some(el) = doc.get_element_by_id("__crepus_ctx__") else {
        return Default::default();
    };
    serde_json::from_str(&el.text_content().unwrap_or_default()).unwrap_or_default()
}
