//! Lookup helpers for in-memory `TemplateContext::virtual_files` bundles.

use std::path::Path;

use crate::context::TemplateContext;

/// Find template source in `virtual_files` for a resolved filesystem path.
///
/// Tries, in order: exact path key, basename-only key, then a single unambiguous
/// virtual key whose path suffix matches the requested file name.
pub fn lookup_virtual_file(ctx: &TemplateContext, path: &Path) -> Option<String> {
    let key = path.to_string_lossy();
    if let Some(content) = ctx.virtual_files.get(key.as_ref()) {
        return Some(content.clone());
    }

    let file_name = path.file_name()?.to_str()?;
    if let Some(content) = ctx.virtual_files.get(file_name) {
        return Some(content.clone());
    }

    let suffix = format!("/{file_name}");
    let mut matches = ctx
        .virtual_files
        .iter()
        .filter(|(vkey, _)| vkey.ends_with(&suffix) || vkey.as_str() == file_name);
    let first = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    Some(first.1.clone())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::context::TemplateContext;
    use crate::virtual_files::lookup_virtual_file;

    #[test]
    fn lookup_exact_key() {
        let mut ctx = TemplateContext::new();
        ctx.virtual_files.insert("child.crepus".into(), "ok".into());
        assert_eq!(
            lookup_virtual_file(&ctx, Path::new("child.crepus")).as_deref(),
            Some("ok")
        );
    }

    #[test]
    fn lookup_rejects_ambiguous_suffix() {
        let mut ctx = TemplateContext::new();
        ctx.virtual_files
            .insert("a/child.crepus".into(), "1".into());
        ctx.virtual_files
            .insert("b/child.crepus".into(), "2".into());
        assert!(lookup_virtual_file(&ctx, Path::new("/virtual/child.crepus")).is_none());
    }
}
