//! Parse the `crepus-bundle.json` payload produced by `crepus web build` and render via
//! [`crate::render_from_files`]. Template variables come from `.crepus` literals and from Rust
//! in the site `runtime/` crate (call `render_from_files` directly with a `TemplateContext` if you
//! need dynamic data).

pub use crepuscularity_core::bundle::{parse_bundle, Bundle};
use crepuscularity_core::context::TemplateContext;
use crepuscularity_core::CrepusError;

use crate::render_from_files;

/// JSON bundle written by `crepus web build` as `crepus-bundle.json`.
///
/// ```json
/// {
///   "entry": "index.crepus",
///   "files": { "index.crepus": "div\\n  \\"hi\\"" }
/// }
/// ```
pub fn render_bundle(bundle_json: &str) -> Result<String, CrepusError> {
    let Bundle { files, entry } = parse_bundle(bundle_json)?;
    let ctx = TemplateContext::new();
    render_from_files(&files, &entry, &ctx)
}

/// Like `render_bundle` but allows passing a `TemplateContext` with dynamic variables.
pub fn render_bundle_with_context(
    bundle_json: &str,
    ctx: &TemplateContext,
) -> Result<String, CrepusError> {
    let Bundle { files, entry } = parse_bundle(bundle_json)?;
    render_from_files(&files, &entry, ctx)
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

    #[test]
    fn parse_bundle_errors() {
        // Invalid JSON
        let err = render_bundle("{ invalid json }").unwrap_err().to_string();
        assert!(err.contains("bundle JSON:"));

        // Missing entry
        let bundle = serde_json::json!({
            "files": { "index.crepus": "div" }
        })
        .to_string();
        let err = render_bundle(&bundle).unwrap_err().to_string();
        assert_eq!(err, "render error: bundle missing string field \"entry\"");

        // Missing files
        let bundle = serde_json::json!({
            "entry": "index.crepus"
        })
        .to_string();
        let err = render_bundle(&bundle).unwrap_err().to_string();
        assert_eq!(err, "render error: bundle missing \"files\" object");

        // Invalid files type
        let bundle = serde_json::json!({
            "entry": "index.crepus",
            "files": "not an object"
        })
        .to_string();
        let err = render_bundle(&bundle).unwrap_err().to_string();
        assert_eq!(err, "render error: \"files\" must be a JSON object");

        // Invalid file value type
        let bundle = serde_json::json!({
            "entry": "index.crepus",
            "files": { "index.crepus": 123 }
        })
        .to_string();
        let err = render_bundle(&bundle).unwrap_err().to_string();
        assert_eq!(
            err,
            "render error: files[\"index.crepus\"] must be a string"
        );
    }
}
