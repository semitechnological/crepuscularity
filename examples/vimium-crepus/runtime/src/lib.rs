mod background;
pub mod commands;
pub mod key_handler;
pub mod settings;
pub mod vomnibar;

use commands::KeyMapRegistry;
use crepuscularity_core::context::{TemplateContext, TemplateValue};
use crepuscularity_web::render_component_file_to_html;
use crepuscularity_webext::wasm::storage;
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use settings::UserSettings;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

const UI_CREPUS: &str = include_str!("../../views/ui.crepus");

static COMMAND_REGISTRY: Lazy<KeyMapRegistry> = Lazy::new(KeyMapRegistry::from_defaults);
static USER_SETTINGS: Lazy<Mutex<UserSettings>> = Lazy::new(|| Mutex::new(UserSettings::new()));
static USER_MAPPINGS: Lazy<Mutex<std::collections::HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(std::collections::HashMap::new()));

fn json_to_template(value: Value) -> TemplateValue {
    match value {
        Value::Bool(v) => TemplateValue::Bool(v),
        Value::Number(v) => {
            if let Some(i) = v.as_i64() {
                TemplateValue::Int(i)
            } else {
                TemplateValue::Float(v.as_f64().unwrap_or(0.0))
            }
        }
        Value::String(v) => TemplateValue::Str(v),
        Value::Array(vals) => {
            TemplateValue::List(vals.into_iter().map(|item| {
                let mut ctx = TemplateContext::new();
                if let Value::Object(fields) = item {
                    for (k, v) in fields { ctx.set(k, json_to_template(v)); }
                }
                ctx
            }).collect())
        }
        _ => TemplateValue::Null,
    }
}

fn to_js(value: Value) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(&value).map_err(|e| JsValue::from_str(&e.to_string()))
}

fn from_js(value: JsValue) -> Value {
    serde_wasm_bindgen::from_value(value).unwrap_or(Value::Null)
}

#[wasm_bindgen]
pub fn runtime_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn render_popup(_state: JsValue) -> Result<JsValue, JsValue> {
    let mut ctx = TemplateContext::new();
    ctx.set("title", "vimium-crepus");
    ctx.set("status", "Enabled");
    ctx.set("groups", json_to_template(shortcut_groups_json_val()));
    let html = render_component_file_to_html(UI_CREPUS, "Popup", &ctx)
        .map_err(|e| JsValue::from_str(&e))?;
    to_js(json!({"html": html, "css": POPUP_CSS}))
}

#[wasm_bindgen]
pub fn shortcut_groups_json() -> Result<JsValue, JsValue> {
    to_js(shortcut_groups_json_val())
}

fn shortcut_groups_json_val() -> Value {
    let commands = commands::all_commands();
    let mut groups: Value = json!([
        {"name": "Navigation", "items": []},
        {"name": "Vomnibar", "items": []},
        {"name": "Find", "items": []},
        {"name": "History", "items": []},
        {"name": "Tabs", "items": []},
        {"name": "Misc", "items": []},
    ]);

    let group_map: std::collections::HashMap<_, _> = [
        ("navigation", 0), ("vomnibar", 1), ("find", 2),
        ("history", 3), ("tabs", 4), ("misc", 5),
    ].into_iter().collect();

    for cmd in &commands {
        if let Some(&idx) = group_map.get(cmd.group.as_str()) {
            let item = json!({
                "keys": command_keys_for_name(&cmd.name),
                "label": cmd.desc,
                "command": cmd.name,
                "advanced": cmd.advanced
            });
            if let Some(arr) = groups[idx].get_mut("items").and_then(Value::as_array_mut) {
                arr.push(item);
            }
        }
    }
    groups
}

fn command_keys_for_name(cmd_name: &str) -> String {
    let registry = &*COMMAND_REGISTRY;
    let mut keys: Vec<String> = registry.key_to_command.iter()
        .filter(|(_, v)| *v == cmd_name)
        .map(|(k, _)| k.clone())
        .collect();
    keys.sort_by_key(|k| k.len());
    if keys.is_empty() { "—".to_string() } else { keys.join(" / ") }
}

#[wasm_bindgen]
pub async fn settings_get() -> Result<JsValue, JsValue> {
    let stored = storage::sync()
        .get_json(json!({"enabled": true}))
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    if let Ok(mut s) = USER_SETTINGS.lock() {
        s.merge(stored);
    }
    let settings = USER_SETTINGS.lock().map(|s| s.settings.clone()).unwrap_or_default();
    to_js(json!({"ok": true, "settings": settings}))
}

#[wasm_bindgen]
pub async fn settings_seed() -> Result<JsValue, JsValue> {
    let stored = storage::sync()
        .get_json(json!({"enabled": true}))
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    if let Ok(mut s) = USER_SETTINGS.lock() {
        s.merge(stored);

        {
            let mut mappings = USER_MAPPINGS.lock().unwrap();
            *mappings = COMMAND_REGISTRY.parse_user_mappings(&s.get_str("keyMappings"));
        }

        let pruned = settings::prune_defaults(&s.settings);
        storage::sync().set(&pruned).await.map_err(|e| JsValue::from_str(&e.to_string()))?;
    }
    to_js(json!({"ok": true}))
}

#[wasm_bindgen]
pub fn content_key(state_val: JsValue, key: &str, editable: bool) -> Result<JsValue, JsValue> {
    let incoming = from_js(state_val);
    let state = key_handler::KeyState {
        mode: incoming.get("mode").and_then(Value::as_str).unwrap_or("normal").to_string(),
        sequence: incoming.get("sequence").and_then(Value::as_str).unwrap_or("").to_string(),
        count_text: incoming.get("countText").and_then(Value::as_str).unwrap_or("").to_string(),
        input: incoming.get("input").and_then(Value::as_str).unwrap_or("").to_string(),
    };

    let mappings = USER_MAPPINGS.lock().unwrap();
    let result = key_handler::handle_key(
        &state, key, editable, &COMMAND_REGISTRY, &mappings,
    );
    to_js(result)
}

#[wasm_bindgen]
pub fn render_help_overlay(show_advanced: bool) -> String {
    let groups = shortcut_groups_json_val();
    let mut html = String::from(
        r#"<div class="vc-overlay-header"><span>vimium-crepus shortcuts</span><button class="vc-overlay-close" type="button">Esc</button></div><div class="vc-overlay-grid">"#
    );
    for group in groups.as_array().into_iter().flatten() {
        let name = group.get("name").and_then(Value::as_str).unwrap_or("");
        let items = group.get("items").and_then(Value::as_array);
        if let Some(items) = items {
            let visible: Vec<_> = items.iter().filter(|item| {
                show_advanced || !item.get("advanced").and_then(Value::as_bool).unwrap_or(false)
            }).collect();
            if visible.is_empty() { continue; }
            html.push_str(&format!(
                r#"<div class="vc-group"><div class="vc-group-title">{name}</div>"#
            ));
            for item in &visible {
                let keys = item.get("keys").and_then(Value::as_str).unwrap_or("");
                let label = item.get("label").and_then(Value::as_str).unwrap_or("");
                html.push_str(&format!(
                    r#"<div class="vc-overlay-row"><span class="vc-overlay-key">{keys}</span><span class="vc-overlay-label">{label}</span></div>"#
                ));
            }
            html.push_str("</div>");
        }
    }
    html.push_str("</div>");
    html
}

#[wasm_bindgen]
pub fn command_list() -> Result<JsValue, JsValue> {
    to_js(shortcut_groups_json_val())
}

#[wasm_bindgen]
pub fn hint_label(index: usize) -> String {
    const CHARS: &[u8] = b"asdfghjklqwertyuiopzxcvbnm";
    let mut label = String::new();
    let mut value = index;
    loop {
        label.insert(0, CHARS[value % CHARS.len()] as char);
        if value < CHARS.len() { break; }
        value = value / CHARS.len() - 1;
    }
    label
}

#[wasm_bindgen]
pub fn update_hint_state(labels: JsValue, current: &str, key: &str) -> Result<JsValue, JsValue> {
    let labels = from_js(labels);
    let next_input = format!("{}{}", current, key.to_lowercase());
    let labels_arr = labels.as_array().cloned().unwrap_or_default();
    let mut exact = None;
    let mut remaining = Vec::new();
    let mut dim = Vec::new();

    for (i, label) in labels_arr.iter().enumerate() {
        let label = label.as_str().unwrap_or("");
        let matched = label.starts_with(&next_input);
        dim.push(!matched);
        if matched { remaining.push(i); }
        if label == next_input { exact = Some(i); }
    }

    let selected = exact.or_else(|| if remaining.len() == 1 { remaining.first().copied() } else { None });
    to_js(json!({"input": next_input, "dim": dim, "selected": selected}))
}

#[wasm_bindgen]
pub fn resolve_navigable(query: &str) -> Result<JsValue, JsValue> {
    let settings = USER_SETTINGS.lock().unwrap();
    let engines = vomnibar::SearchEngines::from_settings(&settings);
    to_js(vomnibar::resolve_navigable(query, &engines))
}

#[wasm_bindgen]
pub fn key_name(event_key: &str) -> String {
    key_handler::key_name(event_key)
}

#[wasm_bindgen]
pub fn is_search_query(query: &str) -> bool {
    vomnibar::SearchEngines::is_search_query(query)
}

#[wasm_bindgen]
pub async fn handle_background_message(message: JsValue) -> Result<JsValue, JsValue> {
    let msg = from_js(message);
    let msg_type = msg.get("type").and_then(Value::as_str).unwrap_or("");

    match msg_type {
        "settings:get" => return settings_get().await,
        "vimium-crepus" | "" => {
            let command = msg.get("command").and_then(Value::as_str).unwrap_or("");
            background::execute_background_command(command, &msg)
                .await
                .map_err(|e| JsValue::from_str(&e))?;
            return to_js(json!({"ok": true}));
        }
        _ => {
            let command = msg.get("handler").and_then(Value::as_str).unwrap_or("");
            if !command.is_empty() {
                match command {
                    "runBackgroundCommand" => {
                        let empty = json!({});
                        let registry_entry = msg.get("registryEntry").unwrap_or(&empty);
                        let cmd_name = registry_entry.get("command").and_then(Value::as_str).unwrap_or("");
                        background::execute_background_command(cmd_name, &msg)
                            .await
                            .map_err(|e| JsValue::from_str(&e))?;
                        return to_js(json!({"ok": true}));
                    }
                    _ => {}
                }
            }
            return to_js(json!({"ok": false, "error": format!("unknown message: {}", msg_type)}));
        }
    }
}

pub const POPUP_CSS: &str = r#"
body{margin:0;min-width:400px;font-family:Inter,ui-sans-serif,system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;background:#111315;color:#f5f0e6}
.vc-popup{display:flex;flex-direction:column;gap:16px;padding:16px;background:linear-gradient(180deg,#16191d 0%,#101214 100%)}
.vc-header{display:flex;align-items:flex-start;justify-content:space-between;gap:12px}
.vc-title{margin:0;font-size:20px;line-height:1;font-weight:800;letter-spacing:0}
.vc-status{font-size:12px;color:#9ca58d;white-space:nowrap}
.vc-grid{display:grid;grid-template-columns:1fr;gap:10px}
.vc-group{border:1px solid rgba(245,240,230,.11);border-radius:8px;background:rgba(255,255,255,.035);overflow:hidden}
.vc-group-title{padding:8px 10px;font-size:11px;font-weight:700;text-transform:uppercase;color:#c7b46a;border-bottom:1px solid rgba(245,240,230,.09)}
.vc-row{display:grid;grid-template-columns:72px 1fr;gap:10px;align-items:center;padding:8px 10px;border-bottom:1px solid rgba(245,240,230,.06)}
.vc-row:last-child{border-bottom:0}
.vc-keys{font-family:"SFMono-Regular",Consolas,monospace;font-size:11px;color:#141414;background:#d8c66f;border-radius:4px;padding:3px 5px;text-align:center}
.vc-label{font-size:12px;color:#ddd7c9;line-height:1.35}
.vc-footer{font-size:11px;line-height:1.4;color:#8d9483}
"#;