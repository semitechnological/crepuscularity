use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::core::{self, EventListenerGuard, Result};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub shortcut: Option<String>,
}

pub fn namespace() -> Result<JsValue> {
    core::namespace("commands")
}

pub async fn get_all() -> Result<Vec<Command>> {
    let commands = namespace()?;
    let value = core::call_method(&commands, "commands.getAll", "getAll", &[]).await?;
    core::from_js(value)
}

pub fn on_command<F>(mut handler: F) -> Result<EventListenerGuard>
where
    F: FnMut(String) + 'static,
{
    let commands = namespace()?;
    core::add_event_listener(
        &commands,
        "commands.onCommand",
        "onCommand",
        move |command, _, _| {
            if let Some(command) = command.as_string() {
                handler(command);
            }
        },
    )
}
