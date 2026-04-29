//! Relative path resolution under a sandbox root (no `..`, no absolute paths).

use std::path::{Component, Path, PathBuf};

use crate::bridge::BridgeError;

pub fn resolve_under_sandbox(root: &Path, relative: &str) -> Result<PathBuf, BridgeError> {
    let rel_path = Path::new(relative);
    if rel_path.is_absolute() {
        return Err(BridgeError::new(
            "path_not_relative",
            "path must be relative to the sandbox root",
        ));
    }
    let mut out = root.to_path_buf();
    for c in rel_path.components() {
        match c {
            Component::Normal(s) => out.push(s),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(BridgeError::new(
                    "path_escape",
                    "only normal path segments are allowed (no ..)",
                ));
            }
        }
    }
    if !out.starts_with(root) {
        return Err(BridgeError::new(
            "path_escape",
            "resolved path left sandbox root",
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_parent_dir() {
        let root = Path::new("/tmp/sandbox-test-root");
        assert!(resolve_under_sandbox(root, "../etc/passwd").is_err());
    }

    #[test]
    fn accepts_nested() {
        let root = Path::new("/tmp/sandbox");
        let p = resolve_under_sandbox(root, "a/b/c.txt").unwrap();
        assert_eq!(p, PathBuf::from("/tmp/sandbox/a/b/c.txt"));
    }
}
