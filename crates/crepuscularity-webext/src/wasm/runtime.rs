use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::core::{self, EventListenerGuard, Result};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageSender {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub origin: Option<String>,
    #[serde(default)]
    pub frame_id: Option<i64>,
    #[serde(default)]
    pub document_id: Option<String>,
    #[serde(default)]
    pub tab: Option<crate::wasm::tabs::Tab>,
}

pub fn namespace() -> Result<JsValue> {
    core::namespace("runtime")
}

pub fn get_url(path: &str) -> Result<String> {
    let runtime = namespace()?;
    let value = core::call_method_blocking(
        &runtime,
        "runtime.getURL",
        "getURL",
        &[JsValue::from_str(path)],
    )?;
    Ok(value.as_string().unwrap_or_default())
}

pub async fn send_message_value(message: JsValue) -> Result<JsValue> {
    let runtime = namespace()?;
    core::call_method(&runtime, "runtime.sendMessage", "sendMessage", &[message]).await
}

pub async fn send_message<T, R>(message: &T) -> Result<R>
where
    T: Serialize,
    R: DeserializeOwned,
{
    let value = send_message_value(core::to_js(message)?).await?;
    core::from_js(value)
}

pub fn on_message_value<F>(mut handler: F) -> Result<EventListenerGuard>
where
    F: FnMut(JsValue, MessageSender) + 'static,
{
    let runtime = namespace()?;
    core::add_event_listener(
        &runtime,
        "runtime.onMessage",
        "onMessage",
        move |message, sender, _| {
            let sender = core::from_js(sender).unwrap_or_default();
            handler(message, sender);
        },
    )
}

pub fn on_message<T, F>(mut handler: F) -> Result<EventListenerGuard>
where
    T: DeserializeOwned + 'static,
    F: FnMut(T, MessageSender) + 'static,
{
    on_message_value(move |message, sender| {
        if let Ok(message) = core::from_js(message) {
            handler(message, sender);
        }
    })
}
