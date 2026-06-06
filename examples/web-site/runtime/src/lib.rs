use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn crepus_render(bundle_json: &str) -> Result<String, JsValue> {
    crepuscularity_web::render_bundle(bundle_json).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn on_refresh_status() -> Result<(), JsValue> {
    let crepus = crepuscularity_web::crepus_refs!("../index.crepus");
    crepus.hero.text("Bye")
}
