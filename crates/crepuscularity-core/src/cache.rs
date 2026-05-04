//! Turbopack-style content-addressed output cache for the `.crepus` driver.
//!
//! [`DriverCache`] stores a SHA-256 digest of each previously written output
//! under `.crepus-cache/`. Before the driver writes a generated file it calls
//! [`DriverCache::is_up_to_date`]; if the new output would be byte-for-byte
//! identical to the last write it skips the write entirely. This prevents
//! `rustc` from seeing a bumped mtime on generated `.rs` files and triggering
//! an unnecessary incremental rebuild.
//!
//! The cache is purely additive and safe to delete at any time — a cold cache
//! just means one extra write per entry on the next run.

#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

#[cfg(not(target_arch = "wasm32"))]
use sha2::{Digest, Sha256};

#[cfg(not(target_arch = "wasm32"))]
use crate::analysis::Fingerprint;
#[cfg(not(target_arch = "wasm32"))]
use crate::util::bytes_to_hex;

/// An on-disk content-addressed cache for driver pipeline outputs.
///
/// Not available on `wasm32` targets (no filesystem).
#[cfg(not(target_arch = "wasm32"))]
pub struct DriverCache {
    cache_dir: PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
impl DriverCache {
    /// Open (or create) a cache rooted at `project_root/.crepus-cache/`.
    pub fn open(project_root: &Path) -> Self {
        let cache_dir = project_root.join(".crepus-cache");
        std::fs::create_dir_all(&cache_dir).ok();
        Self { cache_dir }
    }

    /// Returns `true` if `output` is byte-for-byte identical to the last
    /// recorded output for `fp`. Returns `false` on any I/O error or cache miss.
    pub fn is_up_to_date(&self, fp: &Fingerprint, output: &str) -> bool {
        let entry_path = self.entry_path(fp);
        match std::fs::read_to_string(&entry_path) {
            Ok(stored) => stored == self.output_hash(output),
            Err(_) => false,
        }
    }

    /// Record the output hash for `fp` so future calls to [`Self::is_up_to_date`]
    /// can detect unchanged output.
    ///
    /// Call this **after** successfully writing the actual output file.
    pub fn record(&self, fp: &Fingerprint, output: &str) {
        let entry_path = self.entry_path(fp);
        let _ = std::fs::write(&entry_path, self.output_hash(output));
    }

    fn entry_path(&self, fp: &Fingerprint) -> PathBuf {
        self.cache_dir.join(fp.cache_key())
    }

    fn output_hash(&self, output: &str) -> String {
        let mut h = Sha256::new();
        h.update(output.as_bytes());
        bytes_to_hex(h.finalize().as_ref())
    }
}
