use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::ast::Node;
use crate::parser::parse_template;

thread_local! {
    static FILE_CACHE: RefCell<HashMap<PathBuf, FileCacheEntry>> = RefCell::new(HashMap::new());
    static CONTENT_CACHE: RefCell<HashMap<u64, Vec<Node>>> = RefCell::new(HashMap::new());
}

struct FileCacheEntry {
    mtime: SystemTime,
    len: u64,
    nodes: Vec<Node>,
}

pub fn parse_file(path: &Path) -> Result<Vec<Node>, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("read {:?}: {}", path, e))?;
    let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let len = meta.len();

    let cached = FILE_CACHE.with(|cache| {
        let cache = cache.borrow();
        cache.get(path).and_then(|entry| {
            if entry.mtime == mtime && entry.len == len {
                Some(entry.nodes.clone())
            } else {
                None
            }
        })
    });

    if let Some(nodes) = cached {
        return Ok(nodes);
    }

    let content = std::fs::read_to_string(path).map_err(|e| format!("read {:?}: {}", path, e))?;
    let nodes = parse_template(&content)?;

    FILE_CACHE.with(|cache| {
        cache.borrow_mut().insert(
            path.to_path_buf(),
            FileCacheEntry {
                mtime,
                len,
                nodes: nodes.clone(),
            },
        );
    });

    Ok(nodes)
}

pub fn parse_content(content: &str) -> Result<Vec<Node>, String> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    let key = hasher.finish();

    let cached = CONTENT_CACHE.with(|cache| cache.borrow().get(&key).cloned());

    if let Some(nodes) = cached {
        return Ok(nodes);
    }

    let nodes = parse_template(content)?;
    CONTENT_CACHE.with(|cache| {
        cache.borrow_mut().insert(key, nodes.clone());
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
