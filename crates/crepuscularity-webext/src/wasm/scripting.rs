use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::core::{self, Result};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectionTarget {
    pub tab_id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_ids: Option<Vec<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub all_frames: Option<bool>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptInjection {
    pub target: InjectionTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world: Option<String>,
}

pub fn namespace() -> Result<JsValue> {
    core::namespace("scripting")
}

pub async fn execute_script_value(injection: JsValue) -> Result<JsValue> {
    let scripting = namespace()?;
    core::call_method(
        &scripting,
        "scripting.executeScript",
        "executeScript",
        &[injection],
    )
    .await
}

pub async fn execute_script(injection: &ScriptInjection) -> Result<JsValue> {
    execute_script_value(core::to_js(injection)?).await
}
