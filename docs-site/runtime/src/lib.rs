use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn crepus_render(bundle_json: &str) -> Result<String, JsValue> {
    crepuscularity_web::render_bundle(bundle_json).map_err(|e| JsValue::from_str(&e))
}
