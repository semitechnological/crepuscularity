//! Vue single-file-component frontend.
//!
//! Compiles the `<template>` block of a `.vue` file into the same `Node`/`Element`
//! AST the indentation and JSX frontends produce, so every backend — web, TUI,
//! native View IR, LVGL — consumes Vue markup unchanged. `<script>` blocks are
//! never executed; `<style>` blocks are preserved for a caller to emit.

mod builders;
mod sfc;
mod template;

use super::RawParseError;
use crate::ast::Node;

pub use sfc::{VueScriptBlock, VueSfc, VueStyleBlock};

/// Split a `.vue` source file into its `<template>`, `<script>` and `<style>` blocks.
pub fn parse_vue_sfc(src: &str) -> Result<VueSfc, crate::error::CrepusError> {
    sfc::split_sfc(src).map_err(|e| crate::error::CrepusError::parse(e.message))
}

pub(crate) fn parse_vue_file(src: &str) -> Result<Vec<Node>, RawParseError> {
    let sfc = sfc::split_sfc(src)?;
    match sfc.template {
        Some(body) => parse_vue_template(&body),
        None => Ok(Vec::new()),
    }
}

/// Parse a bare Vue template fragment (no SFC blocks).
pub(crate) fn parse_vue_template(src: &str) -> Result<Vec<Node>, RawParseError> {
    let (nodes, rest) = template::parse_vue_nodes(src, src, 0)?;
    let rest = rest.trim();
    if !rest.is_empty() {
        return Err(RawParseError {
            message: format!(
                "unexpected trailing markup in Vue template: {}",
                &rest[..rest.len().min(40)]
            ),
            byte_offset: Some(super::subslice_byte_offset(src, rest)),
        });
    }
    Ok(nodes)
}

#[cfg(test)]
mod tests;
