use std::collections::BTreeMap;

use crepuscularity_core::context::{TemplateContext, TemplateValue};
use crepuscularity_web::render_component_file_to_html;
use crepuscularity_webext::{BrowserProgram, JsExpr, StorageArea};
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

const UI_CREPUS: &str = include_str!("../../views/ui.crepus");

/// Convert a serde_json Value into a TemplateValue.
/// Object arrays become List(Vec<TemplateContext>), each entry keyed by the object fields.
fn json_to_template(v: Value) -> TemplateValue {
    match v {
        Value::Bool(b) => TemplateValue::Bool(b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                TemplateValue::Int(i)
            } else {
                TemplateValue::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        Value::String(s) => TemplateValue::Str(s),
        Value::Array(arr) => {
            let list = arr
                .into_iter()
                .map(|item| {
                    let mut ctx = TemplateContext::new();
                    if let Value::Object(obj) = item {
                        for (k, v) in obj {
                            ctx.set(k, json_to_template(v));
                        }
                    }
                    ctx
                })
                .collect();
            TemplateValue::List(list)
        }
        _ => TemplateValue::Null,
    }
}

/// Render a named component from ui.crepus with the given props.
fn render_component(
    component_name: &str,
    props: &[(&str, TemplateValue)],
) -> Result<String, String> {
    let mut ctx = TemplateContext::new();
    for (k, v) in props {
        ctx.set(*k, v.clone());
    }
    render_component_file_to_html(UI_CREPUS, component_name, &ctx)
}

// ── WASM exports ─────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn runtime_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Called by popup.js on startup and after every action.
/// `state` is `browser.storage.local` contents as a JS object.
/// Returns `{ html, css }`.
#[wasm_bindgen]
pub fn render_popup(state: JsValue) -> Result<JsValue, JsValue> {
    let state_map: BTreeMap<String, Value> =
        serde_wasm_bindgen::from_value(state).unwrap_or_default();

    let notes: Vec<Value> = state_map
        .get("quicknotes")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    let note_count = notes.len() as i64;
    let empty = notes.is_empty();

    let html = render_component(
        "NoteList",
        &[
            ("notes", json_to_template(json!(notes))),
            ("note_count", TemplateValue::Int(note_count)),
            ("empty", TemplateValue::Bool(empty)),
        ],
    )
    .map_err(|e| JsValue::from_str(&e))?;

    let out = json!({ "html": html, "css": POPUP_CSS });
    serde_wasm_bindgen::to_value(&out).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Called by popup.js event delegation on `[data-action]` clicks.
/// Returns `{ storage_op? }` for popup.js to apply.
#[wasm_bindgen]
pub fn handle_popup_action(action: &str, data: JsValue) -> Result<JsValue, JsValue> {
    let data_map: BTreeMap<String, Value> =
        serde_wasm_bindgen::from_value(data).unwrap_or_default();

    let response = match action {
        "add-note" => {
            let text = data_map
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if text.is_empty() {
                json!({ "noop": true })
            } else {
                json!({
                    "storage_op": {
                        "type": "push",
                        "key": "quicknotes",
                        "item": { "text": text }
                    }
                })
            }
        }
        "delete-note" => {
            let id = data_map
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            json!({ "storage_op": { "type": "remove", "key": "quicknotes", "id": id } })
        }
        "clear-notes" => {
            json!({ "storage_op": { "type": "set", "key": "quicknotes", "value": [] } })
        }
        _ => json!({ "noop": true }),
    };

    serde_wasm_bindgen::to_value(&response).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Returns the browser API interaction program as a self-contained ES module string.
#[wasm_bindgen]
pub fn browser_program() -> String {
    BrowserProgram::new()
        .bind_storage("notes", StorageArea::Local, "quicknotes")
        .set_storage(StorageArea::Local, "quicknoteBooted", JsExpr::bool(true))
        .console_log([
            JsExpr::string("quicknote booted"),
            JsExpr::var("notes"),
            JsExpr::Literal(json!({ "framework": "crepuscularity", "app": "quicknote" })),
        ])
        .emit_module()
}

const POPUP_CSS: &str = r#"
body{margin:0;min-width:280px;font-family:system-ui,"Segoe UI",sans-serif;background:#1a1a2e;color:#e8e8f0}
.qn-popup{display:flex;flex-direction:column}
.qn-header{display:flex;align-items:center;justify-content:space-between;padding:10px 14px;background:#16213e;border-bottom:1px solid rgba(255,255,255,.08)}
.qn-brand{font-size:12px;font-weight:700;text-transform:uppercase;letter-spacing:.1em;color:#7b8cde}
.qn-count{font-size:11px;color:#666688}
.qn-body{padding:6px;overflow-y:auto;max-height:260px}
.qn-empty{padding:18px;text-align:center;color:#555577;font-size:13px}
.qn-note{display:flex;align-items:flex-start;gap:6px;padding:7px 9px;margin-bottom:5px;background:rgba(255,255,255,.05);border-radius:8px;border:1px solid rgba(255,255,255,.07)}
.qn-note-text{flex:1;font-size:13px;line-height:1.4;word-break:break-word}
.qn-delete{flex-shrink:0;background:none;border:none;color:#555577;cursor:pointer;font-size:15px;padding:0 2px;line-height:1}
.qn-delete:hover{color:#cc4444}
.qn-footer{border-top:1px solid rgba(255,255,255,.08);background:#16213e}
.qn-form{display:flex;gap:5px;padding:8px 10px}
.qn-input{flex:1;padding:7px 9px;background:rgba(255,255,255,.08);border:1px solid rgba(255,255,255,.12);border-radius:7px;color:#e8e8f0;font-size:13px;outline:none}
.qn-input:focus{border-color:#7b8cde}
.qn-add{padding:7px 12px;background:#7b8cde;color:#fff;border:none;border-radius:7px;font-size:13px;font-weight:600;cursor:pointer}
.qn-add:hover{background:#6a7bcf}
.qn-clear{display:block;width:100%;padding:6px 10px;background:none;border:none;border-top:1px solid rgba(255,255,255,.06);color:#555577;font-size:12px;cursor:pointer;text-align:center}
.qn-clear:hover{color:#cc4444}
"#;
