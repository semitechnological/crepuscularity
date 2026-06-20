use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum NativeRequest {
    FilePicker(FilePickerRequest),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePickerRequest {
    pub accept: Vec<String>,
    pub multiple: bool,
}

impl FilePickerRequest {
    pub fn media() -> Self {
        Self {
            accept: vec!["image/*".to_string(), "video/*".to_string()],
            multiple: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum NativeResponse {
    FilePicker(FilePickerResponse),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePickerResponse {
    pub files: Vec<PickedFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickedFile {
    pub name: String,
    pub mime_type: String,
    pub bytes: u64,
    pub data_base64: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_picker_request_is_portable_json() {
        let json = serde_json::to_value(NativeRequest::FilePicker(FilePickerRequest::media()))
            .expect("serialize request");
        assert_eq!(json["kind"], "filePicker");
        assert_eq!(json["accept"][0], "image/*");
        assert_eq!(json["multiple"], true);
    }
}
