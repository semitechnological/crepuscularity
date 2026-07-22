//! File watcher for auto-detecting capability changes.
//!
//! Watches .crepus files and webext.toml, emitting events when
//! capabilities change or new ones are detected.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};

use crate::capabilities::{Capability, CapabilitySet};
use crate::manifest::ExtensionManifest;
use crate::scanner::scan_directory_for_capabilities;

/// Events emitted by the capability watcher.
#[derive(Clone, Debug)]
pub enum WatchEvent {
    /// A file was changed.
    FileChanged { path: PathBuf },

    /// New capabilities were detected that aren't in the manifest.
    MissingCapabilities {
        capabilities: Vec<Capability>,
        manifest_path: PathBuf,
    },

    /// The manifest was updated.
    ManifestUpdated { path: PathBuf },

    /// An error occurred.
    Error { message: String },
}

/// Watches files for capability changes.
pub struct CapabilityWatcher {
    rx: Receiver<WatchEvent>,
    _watcher: Box<dyn Watcher + Send>,
}

impl CapabilityWatcher {
    /// Create a new watcher for the given project directory.
    ///
    /// Watches:
    /// - `**/*.crepus` files for API usage in expression slots
    /// - `webext.toml` for declared capabilities
    pub fn new(project_dir: &Path) -> Result<Self, notify::Error> {
        let (tx, rx) = channel();
        let project_dir = project_dir.to_path_buf();

        let tx_clone = tx.clone();
        let project_dir_clone = project_dir.clone();

        let mut watcher = recommended_watcher(move |res: notify::Result<Event>| match res {
            Ok(event) => handle_event(event, &project_dir_clone, &tx_clone),
            Err(e) => {
                let _ = tx_clone.send(WatchEvent::Error {
                    message: format!("watcher error: {e}"),
                });
            }
        })?;

        // Watch the entire project directory recursively
        watcher.watch(&project_dir, RecursiveMode::Recursive)?;

        Ok(Self {
            rx,
            _watcher: Box::new(watcher),
        })
    }

    /// Receive the next event, blocking until one is available.
    pub fn recv(&self) -> Option<WatchEvent> {
        self.rx.recv().ok()
    }

    /// Try to receive an event with a timeout.
    pub fn recv_timeout(&self, timeout: Duration) -> Option<WatchEvent> {
        self.rx.recv_timeout(timeout).ok()
    }

    /// Get the receiver for custom event handling.
    pub fn receiver(&self) -> &Receiver<WatchEvent> {
        &self.rx
    }
}

fn handle_event(event: Event, project_dir: &Path, tx: &Sender<WatchEvent>) {
    // Accept Remove too: editor "atomic saves" (write-temp-then-rename) deliver
    // a Remove on the original path before a Modify/Create on the new file. If
    // we drop Remove, capability scanning misses the save on Linux/inotify
    // because the underlying inode is gone.
    match event.kind {
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
            for path in event.paths {
                let ext = path.extension().and_then(|e| e.to_str());
                let file_name = path.file_name().and_then(|n| n.to_str());

                if ext == Some("crepus") {
                    let _ = tx.send(WatchEvent::FileChanged { path: path.clone() });

                    if let Err(e) = check_capabilities(project_dir, tx) {
                        let _ = tx.send(WatchEvent::Error {
                            message: format!("Failed to check capabilities: {e}"),
                        });
                    }
                } else if file_name == Some("webext.toml") {
                    let _ = tx.send(WatchEvent::ManifestUpdated { path });
                }
            }
        }
        _ => {}
    }
}

fn check_capabilities(project_dir: &Path, tx: &Sender<WatchEvent>) -> Result<(), String> {
    let manifest_path = project_dir.join("webext.toml");

    // Load manifest if it exists
    let declared_caps = if manifest_path.exists() {
        ExtensionManifest::load(&manifest_path)
            .map(|m| m.to_capability_set())
            .unwrap_or_default()
    } else {
        CapabilitySet::new()
    };

    // Scan source files
    let used_caps =
        scan_directory_for_capabilities(project_dir).map_err(|e| format!("Scan failed: {e}"))?;

    // Find missing capabilities
    let missing: Vec<Capability> = used_caps
        .missing_from(&declared_caps)
        .into_iter()
        .cloned()
        .collect();

    if !missing.is_empty() {
        let _ = tx.send(WatchEvent::MissingCapabilities {
            capabilities: missing,
            manifest_path,
        });
    }

    Ok(())
}

pub fn check_project_capabilities_with_manifest(
    project_dir: &Path,
    manifest: &ExtensionManifest,
) -> Result<Vec<Capability>, String> {
    let used_caps =
        scan_directory_for_capabilities(project_dir).map_err(|e| format!("Scan failed: {e}"))?;
    Ok(used_caps
        .missing_from(&manifest.to_capability_set())
        .into_iter()
        .cloned()
        .collect())
}

/// Check a project for missing capabilities (one-shot, no watching).
pub fn check_project_capabilities(project_dir: &Path) -> Result<Vec<Capability>, String> {
    let manifest_path = project_dir.join("webext.toml");

    let declared_caps = if manifest_path.exists() {
        ExtensionManifest::load(&manifest_path)
            .map(|m| m.to_capability_set())
            .map_err(|e| format!("Failed to load manifest: {e}"))?
    } else {
        CapabilitySet::new()
    };

    let used_caps =
        scan_directory_for_capabilities(project_dir).map_err(|e| format!("Scan failed: {e}"))?;

    Ok(used_caps
        .missing_from(&declared_caps)
        .into_iter()
        .cloned()
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use std::time::Duration;

    #[test]
    fn test_watcher_creation() {
        let dir = tempdir().unwrap();
        let watcher = CapabilityWatcher::new(dir.path()).unwrap();
        assert!(watcher.recv_timeout(Duration::from_millis(10)).is_none());
    }

    #[test]
    fn test_watcher_file_changed() {
        let dir = tempdir().unwrap();
        let watcher = CapabilityWatcher::new(dir.path()).unwrap();

        let file_path = dir.path().join("test.crepus");
        fs::write(&file_path, "div").unwrap();

        let mut received = false;
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            if let Some(event) = watcher.recv_timeout(Duration::from_millis(100)) {
                if let WatchEvent::FileChanged { path } = event {
                    if path == file_path {
                        received = true;
                        break;
                    }
                }
            }
        }
        assert!(received, "Did not receive FileChanged event");
    }

    #[test]
    fn test_watcher_manifest_updated() {
        let dir = tempdir().unwrap();
        let watcher = CapabilityWatcher::new(dir.path()).unwrap();

        let file_path = dir.path().join("webext.toml");
        fs::write(&file_path, "name = \"test\"").unwrap();

        let mut received = false;
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            if let Some(event) = watcher.recv_timeout(Duration::from_millis(100)) {
                if let WatchEvent::ManifestUpdated { path } = event {
                    if path == file_path {
                        received = true;
                        break;
                    }
                }
            }
        }
        assert!(received, "Did not receive ManifestUpdated event");
    }

    #[test]
    fn test_check_project_capabilities() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.crepus");
        fs::write(&file_path, "div on-click={browser.storage.local.get()}").unwrap();

        let missing = check_project_capabilities(dir.path()).unwrap();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], Capability::Storage);
    }
}
