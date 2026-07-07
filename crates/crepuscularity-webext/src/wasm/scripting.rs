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

#[cfg(all(test, target_arch = "wasm32", feature = "wasm"))]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn test_execute_script_api_unavailable() {
        let injection = ScriptInjection {
            target: InjectionTarget {
                tab_id: 1,
                ..Default::default()
            },
            files: Some(vec!["test.js".to_string()]),
            ..Default::default()
        };

        let result = execute_script(&injection).await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(matches!(err, super::super::core::BrowserError::ApiUnavailable(_)));
    }
}
