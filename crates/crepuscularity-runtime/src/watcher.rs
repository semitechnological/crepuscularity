//! File watcher for the GPUI hot-reload path.
//!
//! Thin re-export of [`crepuscularity_core::watch::create_watcher`].

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use notify::Watcher;

/// Create and start a file-system watcher rooted at the parent directory of
/// `path`.
///
/// See [`crepuscularity_core::watch::create_watcher`] for details.
pub fn create_watcher(
    path: PathBuf,
    changed: Arc<Mutex<bool>>,
) -> Result<Box<dyn Watcher + Send>, String> {
    crepuscularity_core::watch::create_watcher(path, changed, "runtime")
}
