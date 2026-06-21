//! Lower `.crepus` templates to a JSON view tree for SwiftUI, Jetpack Compose, and future
//! `android-activity` / `objc2` shells.
//!
//! ## Crate layout
//! - [`ir`] — serializable `ViewIr` / `ViewNode` / `ViewStyle` (the **contract** with shells).
//! - [`style`] — Tailwind-like class → style hints (extend here for GPUI parity).
//! - `include_expand` — `include` → `(Vec<Node>, TemplateContext)` without IR cycles.
//! - `render` — AST lowering and public `render_*` entry points.
//! - [`mutations`] — renderer-agnostic, path-based IR mutations and diffing.
//! - [`hot_reload`] — structured hot-reload protocol + conservative AST gate.
//!
//! ## Sharing / codegen (bindgen, other repos)
//! - **[`bindgen`](https://docs.rs/bindgen)** generates **Rust FFI** from **C/C++ headers**. It does
//!   not extract types from Rust for Swift/Kotlin, and it does not import arbitrary “other project”
//!   sources—it wraps existing native (C) APIs.
//! - **Single source of truth** options that *do* fit native shells:
//!   - **`schemars`** on `ViewIr` + **JSON Schema** → *quicktype*, *OpenAPI generators*, etc. for Swift/Kotlin.
//!   - A tiny **build.rs** that emits `fixture.json` and optional bindings (same repo or **path dependency** / **git submodule**).
//!   - Publish **`crepuscularity-native`** to crates.io and depend on it from another Rust workspace as usual.
//!
//! ## Coverage vs GPUI / web
//! Not 100% parity with GPUI `styler.rs`—expand [`style`].

pub mod codegen;
pub mod colors;
pub mod hot_reload;
mod include_expand;
pub mod ir;
pub mod mutations;
pub mod native;
mod render;
pub mod style;

pub use codegen::{generate_native_source, NativeCodegenTarget};
pub use colors::resolve_rgba;
pub use crepuscularity_core::CrepusError;
pub use hot_reload::{ast_shape_compatible, plan_hot_reload, HotReloadEnvelope, HotReloadMessage};
pub use ir::{PickerOption, StackAxis, ViewIr, ViewNode, ViewStyle, IR_VERSION};
pub use mutations::{apply_mutations, diff_ir, IrMutation};
pub use native::{
    FilePickerRequest, FilePickerResponse, NativeCapability, NativePluginRequest,
    NativePluginResponse, NativeRequest, NativeResponse, PickedFile, NATIVE_CAPABILITIES,
};
pub use render::{
    render_component_file_to_ir, render_from_files, render_nodes_to_ir, render_template_to_ir,
};

/// Serialize IR to JSON (compact).
pub fn to_json(ir: &ViewIr) -> Result<String, serde_json::Error> {
    serde_json::to_string(ir)
}

/// Serialize IR to pretty-printed JSON (fixtures / debugging).
pub fn to_json_pretty(ir: &ViewIr) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(ir)
}

#[cfg(test)]
mod tests;
