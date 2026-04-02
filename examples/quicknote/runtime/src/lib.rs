use std::collections::BTreeMap;

use anywhere_core::plugin::{FrontendRenderRequest, PluginHost, PluginId};
use anywhere_crepuscularity::{plugin as crepuscularity_plugin, PLUGIN_ID as CREPUSCULARITY_PLUGIN_ID};
use crepuscularity_anywhere_webext::api::{BrowserProgram, JsExpr, StorageArea};
use crepuscularity_anywhere_webext::manifest::{ExtensionApp, ManifestSpec};
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

fn app_definition() -> ExtensionApp {
    ExtensionApp::new(
        "quicknote",
        ManifestSpec {
            name: "quicknote".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Take quick notes on any page, stored locally in your browser.".to_string(),
        },
        CREPUSCULARITY_PLUGIN_ID,
    )
    .with_frontend(CREPUSCULARITY_PLUGIN_ID, "views/ui.crepus#NoteList")
}

fn plugin_host() -> Result<PluginHost, JsValue> {
    let mut host = PluginHost::new();
    host.register_frontend(crepuscularity_plugin())
        .map_err(|err| JsValue::from_str(&err))?;
    Ok(host)
}

fn render_entry(
    entry: &str,
    files: BTreeMap<String, String>,
    props: BTreeMap<String, Value>,
) -> Result<JsValue, JsValue> {
    let request = FrontendRenderRequest { entry: entry.to_string(), files, props };
    let host = plugin_host()?;
    let plugin_id = PluginId::new(CREPUSCULARITY_PLUGIN_ID);
    let rendered = host
        .render_frontend(&plugin_id, &request)
        .map_err(|err| JsValue::from_str(&err))?;
    serde_wasm_bindgen::to_value(&rendered).map_err(|err| JsValue::from_str(&err.to_string()))
}

// The note list template is compiled into the binary so the popup needs no
// network fetch. Editing the template requires a WASM rebuild.
const UI_CREPUS: &str = include_str!("../../views/ui.crepus");

fn template_files() -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    files.insert("views/ui.crepus".to_string(), UI_CREPUS.to_string());
    files
}

#[wasm_bindgen]
pub fn runtime_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn app_manifest() -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(&app_definition()).map_err(|err| JsValue::from_str(&err.to_string()))
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

/// Called by the framework `popup.js` on startup and after every action.
/// `state` is the raw `browser.storage.local` contents as a JS object.
/// Returns `{ html, css }` — `css` is injected into `<head>` by popup.js.
#[wasm_bindgen]
pub fn render_popup(state: JsValue) -> Result<JsValue, JsValue> {
    let state_map: BTreeMap<String, Value> =
        serde_wasm_bindgen::from_value(state).unwrap_or_default();

    let notes: Vec<Value> = state_map
        .get("quicknotes")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    let note_count = notes.len();
    let empty = notes.is_empty();

    let mut props: BTreeMap<String, Value> = BTreeMap::new();
    props.insert("notes".to_string(), json!(notes));
    props.insert("note_count".to_string(), json!(note_count));
    props.insert("empty".to_string(), json!(empty));

    let rendered = render_entry("views/ui.crepus#NoteList", template_files(), props)?;
    // Deserialise into a plain Value so we can add the css field.
    let mut out: BTreeMap<String, Value> =
        serde_wasm_bindgen::from_value(rendered).map_err(|e| JsValue::from_str(&e.to_string()))?;
    out.insert("css".to_string(), json!(POPUP_CSS));
    serde_wasm_bindgen::to_value(&out).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Called by `popup.js` event delegation on `[data-action]` clicks.
/// Returns `{ storage_op? }` — an optional storage mutation for `popup.js` to apply.
///
/// Supported actions:
/// - `add-note`    data: { text: String }
///   → `{ storage_op: { type: "push", key: "quicknotes", item: { text } } }`
/// - `delete-note` data: { id: String }
///   → `{ storage_op: { type: "remove", key: "quicknotes", id } }`
/// - `clear-notes`
///   → `{ storage_op: { type: "set", key: "quicknotes", value: [] } }`
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

    serde_wasm_bindgen::to_value(&response).map_err(|err| JsValue::from_str(&err.to_string()))
}

#[wasm_bindgen]
pub fn browser_program() -> String {
    BrowserProgram::new()
        .bind_storage("notes", StorageArea::Local, "quicknotes")
        .set_storage(StorageArea::Local, "quicknoteBooted", JsExpr::bool(true))
        .console_log([
            JsExpr::string("quicknote booted"),
            JsExpr::var("notes"),
            JsExpr::Literal(json!({ "framework": "anywhere", "app": "quicknote" })),
        ])
        .emit_module()
}
