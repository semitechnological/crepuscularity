use std::collections::HashMap;

use serde_json::Value;

use crate::CrepusError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bundle {
    pub entry: String,
    pub files: HashMap<String, String>,
}

pub fn parse_bundle(bundle_json: &str) -> Result<Bundle, CrepusError> {
    let root: Value = serde_json::from_str(bundle_json)
        .map_err(|e| CrepusError::render(format!("bundle JSON: {e}")))?;
    let entry = root
        .get("entry")
        .and_then(Value::as_str)
        .ok_or_else(|| CrepusError::render("bundle missing string field \"entry\""))?
        .to_string();
    let files_obj = root
        .get("files")
        .ok_or_else(|| CrepusError::render("bundle missing \"files\" object"))?
        .as_object()
        .ok_or_else(|| CrepusError::render("\"files\" must be a JSON object"))?;
    let mut files = HashMap::new();
    for (path, source) in files_obj {
        files.insert(
            path.clone(),
            source
                .as_str()
                .ok_or_else(|| CrepusError::render(format!("files[{path:?}] must be a string")))?
                .to_string(),
        );
    }
    Ok(Bundle { entry, files })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_bundle() {
        let json = r#"{
            "entry": "main.crepus",
            "files": {
                "main.crepus": "hello",
                "other.crepus": "world"
            }
        }"#;
        let bundle = parse_bundle(json).unwrap();
        assert_eq!(bundle.entry, "main.crepus");
        assert_eq!(bundle.files.len(), 2);
        assert_eq!(bundle.files["main.crepus"], "hello");
        assert_eq!(bundle.files["other.crepus"], "world");
    }

    #[test]
    fn test_parse_invalid_json() {
        let err = parse_bundle("{ invalid json }").unwrap_err();
        assert!(err.to_string().contains("bundle JSON:"));
    }

    #[test]
    fn test_missing_entry_errors() {
        let json = r#"{
            "files": {
                "main.crepus": "hello"
            }
        }"#;
        let err = parse_bundle(json).unwrap_err();
        assert_eq!(
            err.to_string(),
            "render error: bundle missing string field \"entry\""
        );
    }

    #[test]
    fn test_missing_files() {
        let json = r#"{
            "entry": "main.crepus"
        }"#;
        let err = parse_bundle(json).unwrap_err();
        assert_eq!(
            err.to_string(),
            "render error: bundle missing \"files\" object"
        );
    }

    #[test]
    fn test_files_not_object() {
        let json = r#"{
            "entry": "main.crepus",
            "files": []
        }"#;
        let err = parse_bundle(json).unwrap_err();
        assert_eq!(
            err.to_string(),
            "render error: \"files\" must be a JSON object"
        );
    }

    #[test]
    fn test_files_non_string_value() {
        let json = r#"{
            "entry": "main.crepus",
            "files": {
                "main.crepus": 42
            }
        }"#;
        let err = parse_bundle(json).unwrap_err();
        assert_eq!(
            err.to_string(),
            "render error: files[\"main.crepus\"] must be a string"
        );
    }
}
