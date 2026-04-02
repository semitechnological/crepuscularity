//! Browser API abstraction for generating cross-browser compatible JavaScript.
//!
//! This module provides a builder pattern for constructing browser extension
//! API calls that work across Chrome, Firefox, and other browsers.

use std::collections::BTreeMap;

use serde_json::Value;

/// A program that interacts with browser APIs.
///
/// BrowserProgram provides a builder pattern for constructing sequences of
/// browser API operations (storage, messaging, etc.) that can be emitted
/// as JavaScript module code.
#[derive(Debug, Clone)]
pub struct BrowserProgram {
    bindings: Vec<BrowserBinding>,
    statements: Vec<BrowserStatement>,
}

/// A binding that fetches a value from a browser API.
#[derive(Debug, Clone)]
pub struct BrowserBinding {
    pub name: String,
    pub source: BrowserSource,
}

/// Source of a browser binding value.
#[derive(Debug, Clone)]
pub enum BrowserSource {
    /// Get value from storage
    StorageGet { area: StorageArea, key: String },
    /// Get value from runtime message
    RuntimeMessage(MessagePayload),
}

/// A statement that performs a browser API action.
#[derive(Debug, Clone)]
pub enum BrowserStatement {
    /// Set a value in storage
    StorageSet {
        area: StorageArea,
        key: String,
        value: JsExpr,
    },
    /// Send a runtime message
    RuntimeSend(MessagePayload),
    /// Log to console
    ConsoleLog(Vec<JsExpr>),
}

/// Browser storage area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageArea {
    /// Local storage (persists until cleared)
    Local,
    /// Sync storage (syncs across devices)
    Sync,
}

/// A message payload for runtime messaging.
#[derive(Debug, Clone)]
pub struct MessagePayload {
    fields: BTreeMap<String, JsExpr>,
}

/// A JavaScript expression.
#[derive(Debug, Clone)]
pub enum JsExpr {
    /// A literal JSON value
    Literal(Value),
    /// A variable reference
    Var(String),
    /// A member access (object.property)
    Member { object: String, property: String },
    /// Raw JavaScript code (use with caution)
    Raw(String),
}

impl BrowserProgram {
    /// Create a new empty browser program.
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
            statements: Vec::new(),
        }
    }

    /// Bind a value from storage to a variable.
    pub fn bind_storage(
        mut self,
        name: impl Into<String>,
        area: StorageArea,
        key: impl Into<String>,
    ) -> Self {
        self.bindings.push(BrowserBinding {
            name: name.into(),
            source: BrowserSource::StorageGet {
                area,
                key: key.into(),
            },
        });
        self
    }

    /// Bind a value from a runtime message to a variable.
    pub fn bind_runtime_message(mut self, name: impl Into<String>, payload: MessagePayload) -> Self {
        self.bindings.push(BrowserBinding {
            name: name.into(),
            source: BrowserSource::RuntimeMessage(payload),
        });
        self
    }

    /// Set a value in storage.
    pub fn set_storage(
        mut self,
        area: StorageArea,
        key: impl Into<String>,
        value: impl Into<JsExpr>,
    ) -> Self {
        self.statements.push(BrowserStatement::StorageSet {
            area,
            key: key.into(),
            value: value.into(),
        });
        self
    }

    /// Send a runtime message.
    pub fn send_runtime(mut self, payload: MessagePayload) -> Self {
        self.statements.push(BrowserStatement::RuntimeSend(payload));
        self
    }

    /// Log values to console.
    pub fn console_log<I, T>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<JsExpr>,
    {
        self.statements.push(BrowserStatement::ConsoleLog(
            values.into_iter().map(Into::into).collect(),
        ));
        self
    }

    /// Emit the program as a JavaScript ES module.
    pub fn emit_module(&self) -> String {
        let mut js = String::from("export async function runBrowserProgram(browserApi) {\n");

        for binding in &self.bindings {
            match &binding.source {
                BrowserSource::StorageGet { area, key } => {
                    js.push_str(&format!(
                        "  const {name} = (await browserApi.storage.{area}.get({{{quoted_key}: undefined}}))[{quoted_key}];\n",
                        name = sanitize_ident(&binding.name),
                        area = area.as_js_name(),
                        quoted_key = serde_json::to_string(key).unwrap_or_else(|_| "\"\"".to_string()),
                    ));
                }
                BrowserSource::RuntimeMessage(payload) => {
                    js.push_str(&format!(
                        "  const {name} = await browserApi.runtime.sendMessage({payload});\n",
                        name = sanitize_ident(&binding.name),
                        payload = payload.to_js(),
                    ));
                }
            }
        }

        for statement in &self.statements {
            match statement {
                BrowserStatement::StorageSet { area, key, value } => {
                    js.push_str(&format!(
                        "  await browserApi.storage.{area}.set({{{quoted_key}: {value}}});\n",
                        area = area.as_js_name(),
                        quoted_key = serde_json::to_string(key).unwrap_or_else(|_| "\"\"".to_string()),
                        value = value.to_js(),
                    ));
                }
                BrowserStatement::RuntimeSend(payload) => {
                    js.push_str(&format!(
                        "  await browserApi.runtime.sendMessage({payload});\n",
                        payload = payload.to_js(),
                    ));
                }
                BrowserStatement::ConsoleLog(values) => {
                    let joined = values.iter().map(JsExpr::to_js).collect::<Vec<_>>().join(", ");
                    js.push_str(&format!("  console.log({joined});\n"));
                }
            }
        }

        js.push_str("}\n");
        js
    }
}

impl Default for BrowserProgram {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageArea {
    fn as_js_name(self) -> &'static str {
        match self {
            StorageArea::Local => "local",
            StorageArea::Sync => "sync",
        }
    }
}

impl MessagePayload {
    /// Create a new empty message payload.
    pub fn new() -> Self {
        Self {
            fields: BTreeMap::new(),
        }
    }

    /// Add a field to the message.
    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<JsExpr>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    fn to_js(&self) -> String {
        let fields = self
            .fields
            .iter()
            .map(|(key, value)| {
                format!(
                    "{}: {}",
                    serde_json::to_string(key).unwrap_or_else(|_| "\"\"".to_string()),
                    value.to_js()
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("{{ {fields} }}")
    }
}

impl Default for MessagePayload {
    fn default() -> Self {
        Self::new()
    }
}

impl JsExpr {
    /// Create a string literal.
    pub fn string(value: impl Into<String>) -> Self {
        Self::Literal(Value::String(value.into()))
    }

    /// Create a boolean literal.
    pub fn bool(value: bool) -> Self {
        Self::Literal(Value::Bool(value))
    }

    /// Create a variable reference.
    pub fn var(name: impl Into<String>) -> Self {
        Self::Var(name.into())
    }

    /// Create a member access expression.
    pub fn member(object: impl Into<String>, property: impl Into<String>) -> Self {
        Self::Member {
            object: object.into(),
            property: property.into(),
        }
    }

    /// Create a raw JavaScript expression.
    pub fn raw(code: impl Into<String>) -> Self {
        Self::Raw(code.into())
    }

    fn to_js(&self) -> String {
        match self {
            JsExpr::Literal(value) => value.to_string(),
            JsExpr::Var(name) => sanitize_ident(name),
            JsExpr::Member { object, property } => {
                format!("{}.{}", sanitize_ident(object), sanitize_ident(property))
            }
            JsExpr::Raw(code) => code.clone(),
        }
    }
}

impl From<Value> for JsExpr {
    fn from(value: Value) -> Self {
        Self::Literal(value)
    }
}

impl From<&str> for JsExpr {
    fn from(value: &str) -> Self {
        Self::Literal(Value::String(value.to_string()))
    }
}

impl From<String> for JsExpr {
    fn from(value: String) -> Self {
        Self::Literal(Value::String(value))
    }
}

impl From<bool> for JsExpr {
    fn from(value: bool) -> Self {
        Self::Literal(Value::Bool(value))
    }
}

impl From<i64> for JsExpr {
    fn from(value: i64) -> Self {
        Self::Raw(value.to_string())
    }
}

fn sanitize_ident(input: &str) -> String {
    input
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() || ch == '_' { ch } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_program() {
        let program = BrowserProgram::new()
            .bind_storage("notes", StorageArea::Local, "myNotes")
            .set_storage(StorageArea::Local, "booted", JsExpr::bool(true))
            .console_log([JsExpr::string("Hello"), JsExpr::var("notes")]);

        let js = program.emit_module();
        assert!(js.contains("browserApi.storage.local.get"));
        assert!(js.contains("browserApi.storage.local.set"));
        assert!(js.contains("console.log"));
    }

    #[test]
    fn test_message_payload() {
        let payload = MessagePayload::new()
            .with_field("action", "test")
            .with_field("value", JsExpr::var("myVar"));

        let js = payload.to_js();
        assert!(js.contains("\"action\""));
        assert!(js.contains("myVar"));
    }
}
