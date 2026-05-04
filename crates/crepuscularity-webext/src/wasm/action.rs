use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::core::{self, Result};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeTextDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab_id: Option<i64>,
    pub text: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab_id: Option<i64>,
    pub title: String,
}

pub fn namespace() -> Result<JsValue> {
    core::namespace("action")
}

pub async fn set_badge_text(details: &BadgeTextDetails) -> Result<()> {
    let action = namespace()?;
    core::call_method(
        &action,
        "action.setBadgeText",
        "setBadgeText",
        &[core::to_js(details)?],
    )
    .await?;
    Ok(())
}

pub async fn set_title(details: &TitleDetails) -> Result<()> {
    let action = namespace()?;
    core::call_method(
        &action,
        "action.setTitle",
        "setTitle",
        &[core::to_js(details)?],
    )
    .await?;
    Ok(())
}

pub async fn enable(tab_id: Option<i64>) -> Result<()> {
    let action = namespace()?;
    let args = tab_id
        .map(|id| vec![JsValue::from_f64(id as f64)])
        .unwrap_or_default();
    core::call_method(&action, "action.enable", "enable", &args).await?;
    Ok(())
}

pub async fn disable(tab_id: Option<i64>) -> Result<()> {
    let action = namespace()?;
    let args = tab_id
        .map(|id| vec![JsValue::from_f64(id as f64)])
        .unwrap_or_default();
    core::call_method(&action, "action.disable", "disable", &args).await?;
    Ok(())
}
