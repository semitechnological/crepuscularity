use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::core::{self, Result};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub last_visit_time: Option<f64>,
    #[serde(default)]
    pub visit_count: Option<i64>,
    #[serde(default)]
    pub typed_count: Option<i64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistorySearchQuery {
    #[serde(default)]
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i64>,
}

pub fn namespace() -> Result<JsValue> {
    core::namespace("history")
}

pub async fn search(query: &HistorySearchQuery) -> Result<Vec<HistoryItem>> {
    let hist = namespace()?;
    let value =
        core::call_method(&hist, "history.search", "search", &[core::to_js(query)?]).await?;
    core::from_js(value)
}

pub async fn delete_url(url: &str) -> Result<()> {
    let hist = namespace()?;
    core::call_method(
        &hist,
        "history.deleteUrl",
        "deleteUrl",
        &[JsValue::from_str(url)],
    )
    .await?;
    Ok(())
}
