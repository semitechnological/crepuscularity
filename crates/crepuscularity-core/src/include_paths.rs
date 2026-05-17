//! Safe resolution of `include` paths relative to a template `base_dir`.

use std::path::{Path, PathBuf};

/// Resolve an `include` path, rejecting escapes outside `base_dir`.
///
/// On native targets, paths are canonicalized and must stay under the canonical
/// base directory when one is set. On `wasm32`, only syntactic checks run and the
/// joined candidate path is returned (virtual bundles have no real filesystem).
pub fn resolve_include_path(base_dir: Option<&Path>, path: &str) -> Result<PathBuf, String> {
    let requested = Path::new(path);
    if requested.has_root()
        || requested.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(format!("include path outside base dir: {path}"));
    }

    let candidate = if let Some(base) = base_dir {
        base.join(requested)
    } else {
        requested.to_path_buf()
    };

    #[cfg(not(target_arch = "wasm32"))]
    {
        let resolved = std::fs::canonicalize(&candidate).unwrap_or(candidate);
        if let Some(base) = base_dir {
            if let Ok(base) = std::fs::canonicalize(base) {
                if !resolved.starts_with(&base) {
                    return Err(format!("include path outside base dir: {path}"));
                }
            }
        }
        Ok(resolved)
    }

    #[cfg(target_arch = "wasm32")]
    {
        Ok(candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_include_path;

    #[test]
    fn include_path_rejects_parent_dir() {
        let err = resolve_include_path(None, "../secret.crepus").unwrap_err();
        assert!(err.contains("include path outside base dir"));
    }

    #[test]
    fn include_path_rejects_absolute_path() {
        let path = std::env::temp_dir().join("secret.crepus");
        let err = resolve_include_path(None, path.to_str().unwrap()).unwrap_err();
        assert!(err.contains("include path outside base dir"));
    }
}
