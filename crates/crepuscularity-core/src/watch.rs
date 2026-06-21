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
