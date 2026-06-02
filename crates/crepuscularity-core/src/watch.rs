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
