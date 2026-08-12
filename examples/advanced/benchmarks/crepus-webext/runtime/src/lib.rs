use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn runtime_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn render_popup(_state: JsValue) -> Result<JsValue, JsValue> {
    let html = r#"<div class="popup">benchmark</div>"#;
    let result = serde_json::json!({ "html": html });
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}
