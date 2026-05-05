use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::core::{self, Result};
use super::tabs::Tab;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Window {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub focused: Option<bool>,
    #[serde(default)]
    pub top: Option<i64>,
    #[serde(default)]
    pub left: Option<i64>,
    #[serde(default)]
    pub width: Option<i64>,
    #[serde(default)]
    pub height: Option<i64>,
    #[serde(default)]
    pub tabs: Option<Vec<Tab>>,
    #[serde(default)]
    pub incognito: Option<bool>,
    #[serde(default)]
    pub always_on_top: Option<bool>,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focused: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incognito: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focused: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draw_attention: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

pub fn namespace() -> Result<JsValue> {
    core::namespace("windows")
}

pub async fn create(data: &CreateData) -> Result<Window> {
    let win = namespace()?;
    let value = core::call_method(&win, "windows.create", "create", &[core::to_js(data)?]).await?;
    core::from_js(value)
}

pub async fn get_current(populate: Option<bool>) -> Result<Window> {
    let win = namespace()?;
    let value = core::call_method(
        &win,
        "windows.getCurrent",
        "getCurrent",
        &[core::to_js(&serde_json::json!({"populate": populate}))?],
    )
    .await?;
    core::from_js(value)
}

pub async fn get(window_id: i64, populate: Option<bool>) -> Result<Window> {
    let win = namespace()?;
    let value = core::call_method(
        &win,
        "windows.get",
        "get",
        &[
            JsValue::from_f64(window_id as f64),
            core::to_js(&serde_json::json!({"populate": populate}))?,
        ],
    )
    .await?;
    core::from_js(value)
}

pub async fn update(window_id: i64, info: &UpdateInfo) -> Result<Window> {
    let win = namespace()?;
    let value = core::call_method(
        &win,
        "windows.update",
        "update",
        &[JsValue::from_f64(window_id as f64), core::to_js(info)?],
    )
    .await?;
    core::from_js(value)
}

pub async fn remove(window_id: i64) -> Result<()> {
    let win = namespace()?;
    core::call_method(
        &win,
        "windows.remove",
        "remove",
        &[JsValue::from_f64(window_id as f64)],
    )
    .await?;
    Ok(())
}

pub async fn get_all(populate: Option<bool>) -> Result<Vec<Window>> {
    let win = namespace()?;
    let value = core::call_method(
        &win,
        "windows.getAll",
        "getAll",
        &[core::to_js(&serde_json::json!({"populate": populate}))?],
    )
    .await?;
    core::from_js(value)
}
