#[cfg(any(target_arch = "wasm32", test))]
use base64::Engine;
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

#[cfg(any(target_arch = "wasm32", test))]
fn decode_ctx_payload(raw: &str, encoding: Option<&str>) -> HashMap<String, Value> {
    let json = match encoding {
        Some("base64") => base64::engine::general_purpose::STANDARD
            .decode(raw.trim())
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok()),
        _ => Some(raw.to_string()),
    };
    json.and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
fn read_ctx() -> HashMap<String, Value> {
    let doc = web_sys::window().unwrap().document().unwrap();
    let Some(el) = doc.get_element_by_id("__crepus_ctx__") else {
        return Default::default();
    };
    let encoding = el.get_attribute("data-encoding");
    decode_ctx_payload(&el.text_content().unwrap_or_default(), encoding.as_deref())
}

#[cfg(test)]
mod tests {
    use super::decode_ctx_payload;
    use base64::Engine;

    #[test]
    fn decodes_base64_context_payload() {
        let encoded = base64::engine::general_purpose::STANDARD.encode(r#"{"name":"Ada","n":2}"#);
        let ctx = decode_ctx_payload(&encoded, Some("base64"));
        assert_eq!(ctx["name"], "Ada");
        assert_eq!(ctx["n"], 2);
    }

    #[test]
    fn keeps_raw_json_context_compatible() {
        let ctx = decode_ctx_payload(r#"{"name":"Grace"}"#, None);
        assert_eq!(ctx["name"], "Grace");
    }
}
