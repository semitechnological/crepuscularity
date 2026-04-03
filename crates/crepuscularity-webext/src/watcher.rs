//! File watcher for auto-detecting capability changes.
//!
//! Watches .crepus and .js/.ts files and webext.toml, emitting events when
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
    /// - `**/*.crepus`, `**/*.js`, `**/*.ts` files for API usage
    /// - `webext.toml` for declared capabilities
    pub fn new(project_dir: &Path) -> Result<Self, notify::Error> {
        let (tx, rx) = channel();
        let project_dir = project_dir.to_path_buf();

        let tx_clone = tx.clone();
        let project_dir_clone = project_dir.clone();

        let mut watcher = recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                handle_event(event, &project_dir_clone, &tx_clone);
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
    match event.kind {
        EventKind::Modify(_) | EventKind::Create(_) => {
            for path in event.paths {
                // Check if it's a relevant file
                let ext = path.extension().and_then(|e| e.to_str());
                let file_name = path.file_name().and_then(|n| n.to_str());

                let is_source = matches!(ext, Some("crepus") | Some("js") | Some("ts"));
                if is_source {
                    // A source file changed — check for new capabilities
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
