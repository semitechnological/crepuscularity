//! Shared clipboard backend used by the Rust plugin and the host UI.

pub fn read_text() -> Result<String, String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.get_text().map_err(|e| e.to_string())
}

pub fn write_text(text: &str) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_empty_clipboard() {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.clear();
        }

        let result = read_text();
        assert!(
            result.is_err(),
            "Expected error when reading empty clipboard, got: {:?}",
            result
        );
    }
}
