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
