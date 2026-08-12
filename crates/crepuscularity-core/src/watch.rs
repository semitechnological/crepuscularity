//! Shared file-watch primitives for all Crepuscularity dev servers.
//!
//! Every backend that hot-reloads `.crepus` files should use these constants
//! to keep debounce behaviour consistent across GPUI, web, and webext targets.

/// Minimum interval (ms) between consecutive rebuild triggers.
///
/// After the first file change event arrives, subsequent events within this
/// window are coalesced into a single rebuild. This avoids churn when an
/// editor writes several files in quick succession (e.g. `include`d
/// components together with the entry template).
pub const DEBOUNCE_MS: u64 = 50;

/// Cooldown period (ms) after the last received event before a rebuild fires.
///
/// The watcher waits this long after the most recent event before starting a
/// rebuild, so rapid successive saves only produce one rebuild.
pub const COOLDOWN_MS: u64 = 200;

/// Poll interval (ms) for extension-page hot reload (`dev.js`).
///
/// The content script polls `.reload-id` at this cadence. Must be fast
/// enough to feel instant but slow enough to avoid excessive network
/// requests inside the extension sandbox.
pub const EXTENSION_POLL_MS: u64 = 1500;

/// Create a `notify` watcher rooted at the *directory* containing `path`.
///
/// The watcher flips `*changed = true` when:
/// - the watched template is modified / created / removed (covers
///   atomic-save editor patterns), or
/// - any sibling/descendant `.crepus` file changes (`include`d components
///   trigger reload), or
/// - a `context.toml` next to the template is updated.
///
/// `label` appears in error messages to distinguish the source (e.g.
/// `"runtime"`, `"tui"`). Drop the returned handle to stop watching cleanly.
#[cfg(feature = "notify")]
use notify::Watcher;

#[cfg(feature = "notify")]
pub fn create_watcher(
    path: impl Into<std::path::PathBuf>,
    changed: std::sync::Arc<std::sync::Mutex<bool>>,
    label: &'static str,
) -> Result<Box<dyn notify::Watcher + Send>, String> {
    let path: std::path::PathBuf = path.into();
    let canonical_target = path.canonicalize().unwrap_or_else(|_| path.clone());
    let watch_dir = canonical_target
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    let target = canonical_target.clone();
    let root = watch_dir.clone();

    let mut watcher =
        notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            handle_watcher_event(res, &changed, &target, &root, label)
        })
        .map_err(|e| format!("could not create file watcher: {e}"))?;

    watcher
        .watch(&watch_dir, notify::RecursiveMode::Recursive)
        .map_err(|e| format!("failed to watch {}: {e}", watch_dir.display()))?;

    Ok(Box::new(watcher))
}

#[cfg(feature = "notify")]
pub fn handle_watcher_event(
    res: notify::Result<notify::Event>,
    changed: &std::sync::Arc<std::sync::Mutex<bool>>,
    target: &std::path::Path,
    root: &std::path::Path,
    label: &str,
) {
    match res {
        Ok(ev) => {
            if !is_relevant_kind(&ev.kind) {
                return;
            }
            if !event_touches_relevant_path(&ev, target, root) {
                return;
            }
            if let Ok(mut flag) = changed.lock() {
                *flag = true;
            }
        }
        Err(e) => eprintln!("[crepuscularity-{label}] watcher error: {e}"),
    }
}

/// Returns true for Modify, Create, or Remove event kinds.
#[cfg(feature = "notify")]
pub fn is_relevant_kind(kind: &notify::EventKind) -> bool {
    matches!(
        kind,
        notify::EventKind::Modify(_) | notify::EventKind::Create(_) | notify::EventKind::Remove(_)
    )
}

#[cfg(feature = "notify")]
pub fn event_touches_relevant_path(
    event: &notify::Event,
    target: &std::path::Path,
    watch_root: &std::path::Path,
) -> bool {
    for path in &event.paths {
        if path == target {
            return true;
        }
        let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if canon == *target {
            return true;
        }
        let canon_root = watch_root
            .canonicalize()
            .unwrap_or_else(|_| watch_root.to_path_buf());
        if !canon.starts_with(&canon_root) {
            continue;
        }
        match canon.extension().and_then(|e| e.to_str()) {
            Some("crepus") => return true,
            Some("toml") if canon.file_name().and_then(|n| n.to_str()) == Some("context.toml") => {
                return true
            }
            _ => continue,
        }
    }
    false
}
