//! crepuscularity-webext — Browser extension capabilities with auto-detection
//!
//! This crate provides opt-in capability management for browser extensions,
//! inspired by Equilibrium's `.eqp` capability system. Extensions declare
//! capabilities in a manifest, and the CLI watches files to auto-detect
//! and suggest missing capabilities.
//!
//! # Example Manifest (manifest.crex)
//!
//! ```toml
//! [extension]
//! name = "my-extension"
//! version = "1.0.0"
//!
//! [capabilities]
//! content-script = true
//! storage = true
//! messaging = true
//! host-permissions = ["https://example.com/*"]
//! ```

mod capabilities;
mod manifest;
mod scanner;
mod watcher;

pub use capabilities::{Capability, CapabilitySet};
pub use manifest::{ExtensionManifest, ManifestError};
pub use scanner::{scan_crepus_for_capabilities, CapabilityUsage};
pub use watcher::{CapabilityWatcher, WatchEvent};
