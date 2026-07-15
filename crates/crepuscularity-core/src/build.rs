use std::fs;
use std::path::{Path, PathBuf};

pub fn compile_crepus(path: impl AsRef<Path>) -> Result<(), String> {
    let root = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").map_err(|e| format!("CARGO_MANIFEST_DIR: {e}"))?,
    );
    let path = root.join(path.as_ref());
    compile_path(&path)
}

fn compile_path(path: &Path) -> Result<(), String> {
    println!("cargo:rerun-if-changed={}", path.display());

    let metadata =
        fs::metadata(path).map_err(|e| format!("could not stat {}: {e}", path.display()))?;

    if metadata.is_dir() {
        let mut entries = fs::read_dir(path)
            .map_err(|e| format!("could not read directory {}: {e}", path.display()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("could not read directory entry in {}: {e}", path.display()))?;
        entries.sort_by_key(|entry| entry.path());
        for entry in entries {
            compile_path(&entry.path())?;
        }
        return Ok(());
    }

    if matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("crepus" | "csx" | "jsx" | "tsx")
    ) {
        let source = fs::read_to_string(path)
            .map_err(|e| format!("could not read {}: {e}", path.display()))?;
        if let Ok(file) = crate::parse_component_file(&source) {
            if !file.components.is_empty() {
                return Ok(());
            }
        }
        crate::parse_template_with_path(&source, Some(path))
            .map_err(|e| format!("could not parse {}: {e}", path.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_single_template_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("view.crepus");
        fs::write(&file, "div\n  \"hello\"").unwrap();
        compile_path(&file).unwrap();
    }

    #[test]
    fn accepts_nested_template_directory() {
        let dir = tempfile::tempdir().unwrap();
        let views = dir.path().join("views");
        fs::create_dir_all(views.join("nested")).unwrap();
        fs::write(views.join("app.crepus"), "div #app\n  \"hello\"").unwrap();
        fs::write(views.join("nested").join("card.crepus"), "div\n  \"card\"").unwrap();
        compile_path(&views).unwrap();
    }

    #[test]
    fn rejects_invalid_template_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("bad.crepus");
        fs::write(&file, "<div").unwrap();
        let err = compile_path(&file).unwrap_err();
        assert!(err.contains("bad.crepus"));
    }

    #[test]
    fn compile_crepus_success() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("view.crepus");
        fs::write(&file, "div\n  \"hello\"").unwrap();
        // PathBuf::join with an absolute path replaces the base path,
        // so this will ignore CARGO_MANIFEST_DIR and resolve correctly.
        compile_crepus(&file).unwrap();
    }

    #[test]
    fn compile_crepus_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("nonexistent.crepus");
        let err = compile_crepus(&file).unwrap_err();
        assert!(err.contains("could not stat"));
    }
}
