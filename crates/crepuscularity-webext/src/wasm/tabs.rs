use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::core::{self, Result};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tab {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub index: Option<i64>,
    #[serde(default)]
    pub window_id: Option<i64>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub highlighted: Option<bool>,
    #[serde(default)]
    pub pinned: Option<bool>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub fav_icon_url: Option<String>,
    #[serde(default)]
    pub incognito: Option<bool>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_window: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlighted: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_id: Option<i64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProperties {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProperties {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlighted: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,
}

pub fn namespace() -> Result<JsValue> {
    core::namespace("tabs")
}

pub async fn query(info: &QueryInfo) -> Result<Vec<Tab>> {
    let tabs = namespace()?;
    let value = core::call_method(&tabs, "tabs.query", "query", &[core::to_js(info)?]).await?;
    core::from_js(value)
}

pub async fn create(properties: &CreateProperties) -> Result<Tab> {
    let tabs = namespace()?;
    let value =
        core::call_method(&tabs, "tabs.create", "create", &[core::to_js(properties)?]).await?;
    core::from_js(value)
}

pub async fn update(tab_id: i64, properties: &UpdateProperties) -> Result<Tab> {
    let tabs = namespace()?;
    let value = core::call_method(
        &tabs,
        "tabs.update",
        "update",
        &[JsValue::from_f64(tab_id as f64), core::to_js(properties)?],
    )
    .await?;
    core::from_js(value)
}

pub async fn remove(tab_id: i64) -> Result<()> {
    let tabs = namespace()?;
    core::call_method(
        &tabs,
        "tabs.remove",
        "remove",
        &[JsValue::from_f64(tab_id as f64)],
    )
    .await?;
    Ok(())
}

pub async fn duplicate(tab_id: i64) -> Result<Tab> {
    let tabs = namespace()?;
    let value = core::call_method(
        &tabs,
        "tabs.duplicate",
        "duplicate",
        &[JsValue::from_f64(tab_id as f64)],
    )
    .await?;
    core::from_js(value)
}

pub async fn send_message_value(tab_id: i64, message: JsValue) -> Result<JsValue> {
    let tabs = namespace()?;
    core::call_method(
        &tabs,
        "tabs.sendMessage",
        "sendMessage",
        &[JsValue::from_f64(tab_id as f64), message],
    )
    .await
}

pub async fn send_message<T, R>(tab_id: i64, message: &T) -> Result<R>
where
    T: Serialize,
    R: DeserializeOwned,
{
    let value = send_message_value(tab_id, core::to_js(message)?).await?;
    core::from_js(value)
}

pub async fn move_tab(tab_id: i64, index: i64) -> Result<Tab> {
    let tabs = namespace()?;
    let value = core::call_method(
        &tabs,
        "tabs.move",
        "move",
        &[
            JsValue::from_f64(tab_id as f64),
            core::to_js(&serde_json::json!({"index": index}))?,
        ],
    )
    .await?;
    core::from_js(value)
}

pub async fn get_zoom(tab_id: i64) -> Result<f64> {
    let tabs = namespace()?;
    let value = core::call_method(
        &tabs,
        "tabs.getZoom",
        "getZoom",
        &[JsValue::from_f64(tab_id as f64)],
    )
    .await?;
    Ok(value.as_f64().unwrap_or(1.0))
}

pub async fn set_zoom(tab_id: i64, zoom_factor: f64) -> Result<()> {
    let tabs = namespace()?;
    core::call_method(
        &tabs,
        "tabs.setZoom",
        "setZoom",
        &[
            JsValue::from_f64(tab_id as f64),
            JsValue::from_f64(zoom_factor),
        ],
    )
    .await?;
    Ok(())
}

pub async fn reload(tab_id: i64) -> Result<()> {
    let tabs = namespace()?;
    core::call_method(
        &tabs,
        "tabs.reload",
        "reload",
        &[JsValue::from_f64(tab_id as f64)],
    )
    .await?;
    Ok(())
}
