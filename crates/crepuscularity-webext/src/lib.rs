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

pub mod extension_assets;

mod api;
mod capabilities;
mod manifest;
mod scanner;
#[cfg(any(feature = "wasm", test))]
mod wasm_schema;
mod watcher;
pub mod widgets;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use api::{
    BrowserProgram, BrowserSource, BrowserStatement, JsExpr, MessagePayload, StorageArea,
};
pub use capabilities::{Capability, CapabilitySet};
pub use manifest::{
    ActionSpec, BackgroundSpec, BrowserTarget, CapabilitiesSection, ContentScriptEntry,
    ContentScriptSpec, ExtensionInfo, ExtensionManifest, ExtensionManifestCommand, ManifestError,
    ManifestOptions, ManifestV3, OptionsUiSpec, PluginEntry, WebAccessibleResources,
    WebAccessibleResourcesOptions,
};
pub use scanner::{scan_crepus_for_capabilities, CapabilityUsage};
pub use watcher::{
    check_project_capabilities, check_project_capabilities_with_manifest, CapabilityWatcher,
    WatchEvent,
};
pub use widgets::{build_frame_doc, json_to_template};
