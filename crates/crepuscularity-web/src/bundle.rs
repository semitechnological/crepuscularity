//! Parse the `crepus-bundle.json` payload produced by `crepus web build` and render via
//! [`crate::render_from_files`]. Template variables come from `.crepus` literals and from Rust
//! in the site `runtime/` crate (call `render_from_files` directly with a `TemplateContext` if you
//! need dynamic data).

use std::collections::HashMap;

use crepuscularity_core::context::TemplateContext;
use serde_json::Value;

use crate::render_from_files;

/// JSON bundle written by `crepus web build` as `crepus-bundle.json`.
///
/// ```json
/// {
///   "entry": "index.crepus",
///   "files": { "index.crepus": "div\\n  \\"hi\\"" }
/// }
/// ```
pub fn render_bundle(bundle_json: &str) -> Result<String, String> {
    let root: Value = serde_json::from_str(bundle_json).map_err(|e| format!("bundle JSON: {e}"))?;
    let entry = root
        .get("entry")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "bundle missing string field \"entry\"".to_string())?
        .to_string();

    let files_val = root
        .get("files")
        .ok_or_else(|| "bundle missing \"files\" object".to_string())?;
    let files_obj = files_val
        .as_object()
        .ok_or_else(|| "\"files\" must be a JSON object".to_string())?;
    let mut files = HashMap::new();
    for (k, v) in files_obj {
        let s = v
            .as_str()
            .ok_or_else(|| format!("files[{k:?}] must be a string"))?
            .to_string();
        files.insert(k.clone(), s);
    }

    let ctx = TemplateContext::new();
    render_from_files(&files, &entry, &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_minimal_bundle() {
        let bundle = serde_json::json!({
            "entry": "index.crepus",
            "files": {
                "index.crepus": "div class=\"x\"\n  \"hello\""
            }
        })
        .to_string();
        let html = render_bundle(&bundle).expect("render");
        assert!(html.contains("hello"));
        assert!(html.contains("class=\"x\""));
    }

    #[test]
    fn slot_rotate_emits_data_attributes() {
        let tpl = r#"slot-rotate interval={2800} text-green-400
  "one"
  "two"
"#;
        let bundle = serde_json::json!({
            "entry": "index.crepus",
            "files": { "index.crepus": tpl }
        })
        .to_string();
        let html = render_bundle(&bundle).expect("render");
        assert!(html.contains("data-slot-words="));
        assert!(html.contains("data-slot-interval=\"2800\""));
        assert!(html.contains("crepus-slot"));
        assert!(html.contains("text-green-400"));
    }
}
