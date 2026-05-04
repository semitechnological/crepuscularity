use crepuscularity_core::context::{TemplateContext, TemplateValue};
use crepuscularity_web::render_component_file_to_html;
use crepuscularity_webext::wasm::{runtime as browser_runtime, storage, tabs};
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

const UI_CREPUS: &str = include_str!("../../views/ui.crepus");

fn json_to_template(value: Value) -> TemplateValue {
    match value {
        Value::Bool(value) => TemplateValue::Bool(value),
        Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                TemplateValue::Int(value)
            } else {
                TemplateValue::Float(value.as_f64().unwrap_or(0.0))
            }
        }
        Value::String(value) => TemplateValue::Str(value),
        Value::Array(values) => TemplateValue::List(
            values
                .into_iter()
                .map(|item| {
                    let mut ctx = TemplateContext::new();
                    if let Value::Object(fields) = item {
                        for (key, value) in fields {
                            ctx.set(key, json_to_template(value));
                        }
                    }
                    ctx
                })
                .collect(),
        ),
        _ => TemplateValue::Null,
    }
}

fn to_js(value: Value) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(&value).map_err(|error| JsValue::from_str(&error.to_string()))
}

fn from_js(value: JsValue) -> Value {
    serde_wasm_bindgen::from_value(value).unwrap_or(Value::Null)
}

fn wasm_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

#[wasm_bindgen]
pub fn runtime_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn render_popup(_state: JsValue) -> Result<JsValue, JsValue> {
    let mut ctx = TemplateContext::new();
    ctx.set("title", "vimium-crepus");
    ctx.set("status", "Enabled on normal web pages");
    ctx.set("groups", json_to_template(shortcut_groups()));

    let html = render_component_file_to_html(UI_CREPUS, "Popup", &ctx)
        .map_err(|error| JsValue::from_str(&error))?;
    let out = json!({ "html": html, "css": POPUP_CSS });
    to_js(out)
}

#[wasm_bindgen]
pub async fn settings_get() -> Result<JsValue, JsValue> {
    let mut settings = storage::sync()
        .get_json(json!({ "enabled": true }))
        .await
        .map_err(wasm_error)?;
    if let Value::Object(fields) = &mut settings {
        fields.entry("enabled".to_string()).or_insert(Value::Bool(true));
    }
    to_js(json!({ "ok": true, "settings": settings }))
}

#[wasm_bindgen]
pub async fn settings_seed() -> Result<JsValue, JsValue> {
    let settings = storage::sync()
        .get_json(json!({ "enabled": true }))
        .await
        .map_err(wasm_error)?;
    storage::sync().set(&settings).await.map_err(wasm_error)?;
    to_js(json!({ "ok": true, "settings": settings }))
}

#[wasm_bindgen]
pub async fn send_runtime_message(message: JsValue) -> Result<JsValue, JsValue> {
    browser_runtime::send_message_value(message)
        .await
        .map_err(wasm_error)
}

#[wasm_bindgen]
pub async fn handle_background_message(message: JsValue) -> Result<JsValue, JsValue> {
    let message_value = from_js(message);
    match message_value.get("type").and_then(Value::as_str) {
        Some("settings:get") => settings_get().await,
        Some("vimium-crepus") => {
            execute_background_command(&message_value)
                .await
                .map_err(wasm_error)?;
            to_js(json!({ "ok": true }))
        }
        _ => to_js(json!({ "ok": false, "error": "unknown message" })),
    }
}

fn shortcut_groups() -> Value {
    json!([
        {
            "name": "Page",
            "items": [
                { "keys": "j / k", "label": "Scroll down or up" },
                { "keys": "h / l", "label": "Scroll left or right" },
                { "keys": "gg / G", "label": "Jump to top or bottom" },
                { "keys": "d / u", "label": "Half-page scroll" }
            ]
        },
        {
            "name": "Links and find",
            "items": [
                { "keys": "f / F", "label": "Open link hints here or in a new tab" },
                { "keys": "/", "label": "Find text on the page" },
                { "keys": "n / N", "label": "Next or previous find match" },
                { "keys": "gi", "label": "Focus the first text field" }
            ]
        },
        {
            "name": "Tabs",
            "items": [
                { "keys": "t", "label": "Open a new tab" },
                { "keys": "x / X", "label": "Close tab or restore the last closed URL" },
                { "keys": "J / K", "label": "Move to the left or right tab" },
                { "keys": "yt", "label": "Duplicate the current tab" }
            ]
        }
    ])
}

async fn execute_background_command(message: &Value) -> crepuscularity_webext::wasm::Result<()> {
    let command = message.get("command").and_then(Value::as_str).unwrap_or("");
    match command {
        "new-tab" => {
            tabs::create(&tabs::CreateProperties::default()).await?;
        }
        "open-url" => {
            tabs::create(&tabs::CreateProperties {
                url: message.get("url").and_then(Value::as_str).map(ToOwned::to_owned),
                active: Some(message.get("active").and_then(Value::as_bool).unwrap_or(true)),
                ..Default::default()
            })
            .await?;
        }
        "duplicate-tab" => {
            if let Some(tab) = active_tab().await?.and_then(|tab| tab.id) {
                tabs::duplicate(tab).await?;
            }
        }
        "close-tab" => {
            if let Some(tab) = active_tab().await? {
                if let Some(url) = tab.url {
                    storage::sync().set(&json!({ "lastClosedUrl": url })).await?;
                }
                if let Some(id) = tab.id {
                    tabs::remove(id).await?;
                }
            }
        }
        "restore-tab" => {
            let storage = storage::sync()
                .get_json(json!({ "lastClosedUrl": "" }))
                .await?;
            if let Some(url) = storage.get("lastClosedUrl").and_then(Value::as_str) {
                if !url.is_empty() {
                    tabs::create(&tabs::CreateProperties {
                        url: Some(url.to_string()),
                        active: Some(true),
                        ..Default::default()
                    })
                    .await?;
                }
            }
        }
        "left-tab" => activate_relative_tab(-1).await?,
        "right-tab" => activate_relative_tab(1).await?,
        "first-tab" => activate_edge_tab(false).await?,
        "last-tab" => activate_edge_tab(true).await?,
        _ => {}
    }
    Ok(())
}

async fn active_tab() -> crepuscularity_webext::wasm::Result<Option<tabs::Tab>> {
    let tabs = tabs::query(&tabs::QueryInfo {
        active: Some(true),
        current_window: Some(true),
        ..Default::default()
    })
    .await?;
    Ok(tabs.into_iter().next())
}

async fn activate_relative_tab(delta: i64) -> crepuscularity_webext::wasm::Result<()> {
    let all = tabs::query(&tabs::QueryInfo {
        current_window: Some(true),
        ..Default::default()
    })
    .await?;
    if all.is_empty() {
        return Ok(());
    }
    let current = all
        .iter()
        .find(|tab| tab.active.unwrap_or(false))
        .and_then(|tab| tab.index)
        .unwrap_or(0);
    let next_index = (current + delta).rem_euclid(all.len() as i64);
    if let Some(tab_id) = all
        .iter()
        .find(|tab| tab.index == Some(next_index))
        .and_then(|tab| tab.id)
    {
        tabs::update(
            tab_id,
            &tabs::UpdateProperties {
                active: Some(true),
                ..Default::default()
            },
        )
        .await?;
    }
    Ok(())
}

async fn activate_edge_tab(last: bool) -> crepuscularity_webext::wasm::Result<()> {
    let all = tabs::query(&tabs::QueryInfo {
        current_window: Some(true),
        ..Default::default()
    })
    .await?;
    let tab_id = if last {
        all.last().and_then(|tab| tab.id)
    } else {
        all.first().and_then(|tab| tab.id)
    };
    if let Some(tab_id) = tab_id {
        tabs::update(
            tab_id,
            &tabs::UpdateProperties {
                active: Some(true),
                ..Default::default()
            },
        )
        .await?;
    }
    Ok(())
}

#[wasm_bindgen]
pub fn shortcut_groups_json() -> Result<JsValue, JsValue> {
    to_js(shortcut_groups())
}

#[wasm_bindgen]
pub fn render_help_overlay() -> String {
    let mut rows = String::new();
    for group in shortcut_groups().as_array().into_iter().flatten() {
        if let Some(items) = group.get("items").and_then(Value::as_array) {
            for item in items {
                let keys = item.get("keys").and_then(Value::as_str).unwrap_or("");
                let label = item.get("label").and_then(Value::as_str).unwrap_or("");
                rows.push_str(&format!(
                    r#"<div class="vc-overlay-row"><span class="vc-overlay-key">{keys}</span><span class="vc-overlay-label">{label}</span></div>"#
                ));
            }
        }
    }
    format!(
        r#"<div class="vc-overlay-header"><span>vimium-crepus shortcuts</span><button class="vc-overlay-close" type="button">Esc</button></div><div class="vc-overlay-grid">{rows}</div>"#
    )
}

#[wasm_bindgen]
pub fn hint_label(index: usize) -> String {
    const CHARS: &[u8] = b"asdfghjklqwertyuiopzxcvbnm";
    let mut label = String::new();
    let mut value = index;
    loop {
        label.insert(0, CHARS[value % CHARS.len()] as char);
        if value < CHARS.len() {
            break;
        }
        value = value / CHARS.len() - 1;
    }
    label
}

#[wasm_bindgen]
pub fn update_hint_state(labels: JsValue, current: &str, key: &str) -> Result<JsValue, JsValue> {
    let labels = from_js(labels);
    let next_input = format!("{}{}", current, key.to_lowercase());
    let labels = labels.as_array().cloned().unwrap_or_default();
    let mut exact = None;
    let mut remaining = Vec::new();
    let mut dim = Vec::new();

    for (index, label) in labels.iter().enumerate() {
        let label = label.as_str().unwrap_or("");
        let matched = label.starts_with(&next_input);
        dim.push(!matched);
        if matched {
            remaining.push(index);
        }
        if label == next_input {
            exact = Some(index);
        }
    }

    let selected = exact.or_else(|| {
        if remaining.len() == 1 {
            remaining.first().copied()
        } else {
            None
        }
    });

    to_js(json!({
        "input": next_input,
        "dim": dim,
        "selected": selected
    }))
}

#[wasm_bindgen]
pub fn content_key(state: JsValue, key: &str, editable: bool) -> Result<JsValue, JsValue> {
    let state = from_js(state);
    let mode = state
        .get("mode")
        .and_then(Value::as_str)
        .unwrap_or("normal");
    let mut sequence = state
        .get("sequence")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let mut count_text = state
        .get("countText")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    if key == "Esc" {
        return to_js(json!({
            "state": { "mode": "normal", "sequence": "", "countText": "" },
            "effect": { "kind": "clear-overlays" },
            "prevent": false
        }));
    }

    if mode == "insert" || editable {
        return to_js(json!({ "state": state, "effect": null, "prevent": false }));
    }

    if key.len() == 1
        && key.as_bytes()[0].is_ascii_digit()
        && sequence.is_empty()
        && (key != "0" || !count_text.is_empty())
    {
        count_text.push_str(key);
        return to_js(json!({
            "state": { "mode": "normal", "sequence": "", "countText": count_text },
            "effect": null,
            "prevent": true
        }));
    }

    sequence.push_str(key);
    let complete = is_complete_command(&sequence);
    let prefix = matches!(sequence.as_str(), "g" | "y");

    if complete {
        let count = parse_count(&count_text);
        let effect = command_effect(&sequence, count);
        return to_js(json!({
            "state": { "mode": effect_mode(&effect), "sequence": "", "countText": "" },
            "effect": effect,
            "prevent": true
        }));
    }

    if prefix {
        return to_js(json!({
            "state": { "mode": "normal", "sequence": sequence, "countText": count_text },
            "effect": null,
            "prevent": true
        }));
    }

    to_js(json!({
        "state": { "mode": "normal", "sequence": "", "countText": "" },
        "effect": null,
        "prevent": false
    }))
}

#[wasm_bindgen]
pub fn background_plan(message: JsValue, context: JsValue) -> Result<JsValue, JsValue> {
    let message = from_js(message);
    let context = from_js(context);
    let command = message.get("command").and_then(Value::as_str).unwrap_or("");
    let tab = context.get("tab").cloned().unwrap_or(Value::Null);
    let storage = context.get("storage").cloned().unwrap_or(Value::Null);

    let plan = match command {
        "new-tab" => json!({ "op": "create-tab" }),
        "open-url" => json!({
            "op": "create-tab",
            "url": message.get("url").and_then(Value::as_str).unwrap_or("about:blank"),
            "active": message.get("active").and_then(Value::as_bool).unwrap_or(true)
        }),
        "duplicate-tab" => tab_id(&tab)
            .map(|id| json!({ "op": "duplicate-tab", "tabId": id }))
            .unwrap_or_else(|| json!({ "op": "none" })),
        "close-tab" => {
            if let Some(id) = tab_id(&tab) {
                json!({
                    "op": "close-tab",
                    "tabId": id,
                    "lastClosedUrl": tab.get("url").and_then(Value::as_str).unwrap_or("")
                })
            } else {
                json!({ "op": "none" })
            }
        }
        "restore-tab" => {
            let url = storage
                .get("lastClosedUrl")
                .and_then(Value::as_str)
                .unwrap_or("");
            if url.is_empty() {
                json!({ "op": "none" })
            } else {
                json!({ "op": "create-tab", "url": url })
            }
        }
        "left-tab" => json!({ "op": "move-tab", "delta": -1 }),
        "right-tab" => json!({ "op": "move-tab", "delta": 1 }),
        "first-tab" => json!({ "op": "edge-tab", "edge": "first" }),
        "last-tab" => json!({ "op": "edge-tab", "edge": "last" }),
        _ => json!({ "op": "none" }),
    };

    to_js(plan)
}

fn parse_count(count_text: &str) -> i64 {
    count_text
        .parse::<i64>()
        .ok()
        .filter(|value| *value > 0)
        .unwrap_or(1)
}

fn is_complete_command(command: &str) -> bool {
    matches!(
        command,
        "j" | "k"
            | "h"
            | "l"
            | "d"
            | "u"
            | "G"
            | "r"
            | "H"
            | "L"
            | "?"
            | "f"
            | "F"
            | "/"
            | "n"
            | "N"
            | "i"
            | "t"
            | "x"
            | "X"
            | "J"
            | "K"
            | "p"
            | "P"
            | "gg"
            | "gi"
            | "yt"
            | "yy"
            | "g0"
            | "g$"
    )
}

fn command_effect(command: &str, count: i64) -> Value {
    match command {
        "j" => json!({ "kind": "scroll", "x": 0, "y": 80 * count }),
        "k" => json!({ "kind": "scroll", "x": 0, "y": -80 * count }),
        "h" => json!({ "kind": "scroll", "x": -120 * count, "y": 0 }),
        "l" => json!({ "kind": "scroll", "x": 120 * count, "y": 0 }),
        "d" => json!({ "kind": "half-scroll", "direction": 1, "count": count }),
        "u" => json!({ "kind": "half-scroll", "direction": -1, "count": count }),
        "gg" => json!({ "kind": "scroll-top" }),
        "G" => json!({ "kind": "scroll-bottom" }),
        "r" => json!({ "kind": "reload" }),
        "H" => json!({ "kind": "history-back" }),
        "L" => json!({ "kind": "history-forward" }),
        "?" => json!({ "kind": "help" }),
        "f" => json!({ "kind": "hints", "newTab": false }),
        "F" => json!({ "kind": "hints", "newTab": true }),
        "/" => json!({ "kind": "find" }),
        "n" => json!({ "kind": "find-next", "reverse": false }),
        "N" => json!({ "kind": "find-next", "reverse": true }),
        "gi" => json!({ "kind": "focus-input" }),
        "i" => json!({ "kind": "insert-mode" }),
        "t" => json!({ "kind": "background", "command": "new-tab" }),
        "x" => json!({ "kind": "background", "command": "close-tab" }),
        "X" => json!({ "kind": "background", "command": "restore-tab" }),
        "yt" => json!({ "kind": "background", "command": "duplicate-tab" }),
        "J" => json!({ "kind": "background", "command": "left-tab" }),
        "K" => json!({ "kind": "background", "command": "right-tab" }),
        "g0" => json!({ "kind": "background", "command": "first-tab" }),
        "g$" => json!({ "kind": "background", "command": "last-tab" }),
        "yy" => json!({ "kind": "copy-url" }),
        "p" => json!({ "kind": "open-clipboard", "newTab": false }),
        "P" => json!({ "kind": "open-clipboard", "newTab": true }),
        _ => json!({ "kind": "none" }),
    }
}

fn effect_mode(effect: &Value) -> &'static str {
    match effect.get("kind").and_then(Value::as_str).unwrap_or("") {
        "hints" => "hints",
        "insert-mode" => "insert",
        _ => "normal",
    }
}

fn tab_id(tab: &Value) -> Option<i64> {
    tab.get("id").and_then(Value::as_i64)
}

const POPUP_CSS: &str = r#"
body{margin:0;min-width:360px;font-family:Inter,ui-sans-serif,system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;background:#111315;color:#f5f0e6}
.vc-popup{display:flex;flex-direction:column;gap:18px;padding:18px;background:linear-gradient(180deg,#16191d 0%,#101214 100%)}
.vc-header{display:flex;align-items:flex-start;justify-content:space-between;gap:16px}
.vc-title{margin:0;font-size:22px;line-height:1;font-weight:800;letter-spacing:0}
.vc-status{font-size:12px;color:#9ca58d;white-space:nowrap}
.vc-grid{display:grid;grid-template-columns:1fr;gap:12px}
.vc-group{border:1px solid rgba(245,240,230,.11);border-radius:8px;background:rgba(255,255,255,.035);overflow:hidden}
.vc-group-title{padding:9px 11px;font-size:12px;font-weight:700;text-transform:uppercase;color:#c7b46a;border-bottom:1px solid rgba(245,240,230,.09)}
.vc-row{display:grid;grid-template-columns:78px 1fr;gap:12px;align-items:center;padding:9px 11px;border-bottom:1px solid rgba(245,240,230,.06)}
.vc-row:last-child{border-bottom:0}
.vc-keys{font-family:"SFMono-Regular",Consolas,monospace;font-size:12px;color:#141414;background:#d8c66f;border-radius:5px;padding:4px 6px;text-align:center}
.vc-label{font-size:13px;color:#ddd7c9;line-height:1.35}
.vc-footer{font-size:12px;line-height:1.45;color:#8d9483}
"#;
