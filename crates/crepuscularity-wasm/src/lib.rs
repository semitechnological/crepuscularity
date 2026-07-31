//! WASM bindings over the same parser the `crepus` CLI uses.
//!
//! The IR crosses the boundary as a JSON string rather than a structured
//! `JsValue`: it avoids a `serde-wasm-bindgen` dependency and `JSON.parse` on a
//! single string beats field-by-field reflection for trees of any real size.

use crepuscularity_core::context::TemplateContext;
use crepuscularity_native::{render_template_to_ir, render_template_to_ir_with_path, IR_VERSION};
use wasm_bindgen::prelude::*;

/// Schema version of the IR this module emits; consumers should check it.
#[wasm_bindgen]
pub fn ir_version() -> u32 {
    IR_VERSION
}

/// Parse a template into View IR, choosing the frontend from `filename`.
///
/// The parser dispatches on the file extension, so `.crepus`, `.jsx`/`.tsx`,
/// `.svelte` and `.vue` all reach the same IR through their own frontend.
#[wasm_bindgen]
pub fn parse_template_json(
    source: &str,
    filename: Option<String>,
    context_json: Option<String>,
) -> Result<String, JsError> {
    let ctx = build_context(context_json)?;
    let path = filename.as_deref().map(std::path::Path::new);
    let ir = render_template_to_ir_with_path(source, &ctx, path)
        .map_err(|e| JsError::new(&format!("lower template to View IR: {e}")))?;
    serde_json::to_string(&ir).map_err(|e| JsError::new(&format!("serialize View IR: {e}")))
}

/// Parse `.crepus` source into View IR, serialized as JSON.
///
/// `context_json`, when present, must be a JSON object whose values are bound
/// as template variables.
#[wasm_bindgen]
pub fn parse_crepus_json(source: &str, context_json: Option<String>) -> Result<String, JsError> {
    let ctx = build_context(context_json)?;
    let ir = render_template_to_ir(source, &ctx)
        .map_err(|e| JsError::new(&format!("lower .crepus to View IR: {e}")))?;
    serde_json::to_string(&ir).map_err(|e| JsError::new(&format!("serialize View IR: {e}")))
}

fn build_context(context_json: Option<String>) -> Result<TemplateContext, JsError> {
    let mut ctx = TemplateContext::new();
    let Some(raw) = context_json.as_deref().map(str::trim) else {
        return Ok(ctx);
    };
    if raw.is_empty() || raw == "null" {
        return Ok(ctx);
    }
    let parsed: serde_json::Value = serde_json::from_str(raw)
        .map_err(|e| JsError::new(&format!("context is not valid JSON: {e}")))?;
    let obj = parsed
        .as_object()
        .ok_or_else(|| JsError::new("context must be a JSON object"))?;
    for (k, v) in obj {
        let s = match v {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        ctx.set(k, s.as_str());
    }
    Ok(ctx)
}
