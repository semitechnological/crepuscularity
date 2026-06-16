use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use crate::ast::Node;
use crate::error::CrepusError;
use crate::parser::parse_template;

thread_local! {
    static FILE_CACHE: RefCell<HashMap<PathBuf, FileCacheEntry>> = RefCell::new(HashMap::new());
    static CONTENT_CACHE: RefCell<HashMap<u64, Arc<Vec<Node>>>> = RefCell::new(HashMap::new());
}

struct FileCacheEntry {
    mtime: SystemTime,
    len: u64,
    nodes: Arc<Vec<Node>>,
}

/// Parse a `.crepus` file from disk, returning a shared AST snapshot.
pub fn parse_file(path: &Path) -> Result<Arc<Vec<Node>>, CrepusError> {
    let meta = std::fs::metadata(path).map_err(|e| {
        CrepusError::Io(std::io::Error::new(
            e.kind(),
            format!("read {:?}: {e}", path),
        ))
    })?;
    let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let len = meta.len();

    let cached = FILE_CACHE.with(|cache| {
        let cache = cache.borrow();
        cache.get(path).and_then(|entry| {
            if entry.mtime == mtime && entry.len == len {
                Some(Arc::clone(&entry.nodes))
            } else {
                None
            }
        })
    });

    if let Some(nodes) = cached {
        return Ok(nodes);
    }

    let content = std::fs::read_to_string(path).map_err(|e| {
        CrepusError::Io(std::io::Error::new(
            e.kind(),
            format!("read {:?}: {e}", path),
        ))
    })?;
    let nodes = Arc::new(parse_template(&content)?);

    FILE_CACHE.with(|cache| {
        cache.borrow_mut().insert(
            path.to_path_buf(),
            FileCacheEntry {
                mtime,
                len,
                nodes: Arc::clone(&nodes),
            },
        );
    });

    Ok(nodes)
}

/// Parse in-memory template content, returning a shared AST snapshot.
pub fn parse_content(content: &str) -> Result<Arc<Vec<Node>>, CrepusError> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    let key = hasher.finish();

    let cached = CONTENT_CACHE.with(|cache| cache.borrow().get(&key).map(Arc::clone));

    if let Some(nodes) = cached {
        return Ok(nodes);
    }

    let nodes = Arc::new(parse_template(content)?);
    CONTENT_CACHE.with(|cache| {
        cache.borrow_mut().insert(key, Arc::clone(&nodes));
    });

    Ok(nodes)
}

pub fn invalidate_file(path: &Path) {
    FILE_CACHE.with(|cache| {
        cache.borrow_mut().remove(path);
    });
}

pub fn clear() {
    FILE_CACHE.with(|cache| cache.borrow_mut().clear());
    CONTENT_CACHE.with(|cache| cache.borrow_mut().clear());
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_content_cache() {
        clear();
        let content1 = "div\n  span Hello";
        let content2 = "div\n  span World";

        let ast1 = parse_content(content1).unwrap();
        let ast1_cached = parse_content(content1).unwrap();
        assert!(Arc::ptr_eq(&ast1, &ast1_cached), "Cache hit should return the same Arc");

        let ast2 = parse_content(content2).unwrap();
        assert!(!Arc::ptr_eq(&ast1, &ast2), "Different content should have different Arc");
    }

    #[test]
    fn test_parse_file_cache() {
        clear();
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "div\n  span File").unwrap();
        file.flush().unwrap();

        let path = file.path().to_path_buf();

        let ast1 = parse_file(&path).unwrap();
        let ast1_cached = parse_file(&path).unwrap();
        assert!(Arc::ptr_eq(&ast1, &ast1_cached), "Cache hit should return the same Arc");

        // Wait a bit and modify file to change mtime and length
        std::thread::sleep(std::time::Duration::from_millis(10));
        write!(file, " Extra content").unwrap();
        file.flush().unwrap();

        let ast2 = parse_file(&path).unwrap();
        assert!(!Arc::ptr_eq(&ast1, &ast2), "Cache should miss after file modification");
    }

    #[test]
    fn test_invalidate_file() {
        clear();
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "div").unwrap();
        file.flush().unwrap();

        let path = file.path().to_path_buf();

        let ast1 = parse_file(&path).unwrap();

        invalidate_file(&path);

        let ast2 = parse_file(&path).unwrap();
        assert!(!Arc::ptr_eq(&ast1, &ast2), "Cache should miss after invalidation");
    }

    #[test]
    fn test_clear() {
        clear();
        let content = "span Test";
        let ast1 = parse_content(content).unwrap();

        clear();

        let ast2 = parse_content(content).unwrap();
        assert!(!Arc::ptr_eq(&ast1, &ast2), "Cache should miss after clear");
    }
}
