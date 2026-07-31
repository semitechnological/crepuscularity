//! Vue single-file-component block splitting.

use super::super::RawParseError;

/// One `<script>` block of a Vue SFC. The contents are never executed or parsed.
#[derive(Debug, Clone, Default)]
pub struct VueScriptBlock {
    /// True when the block was declared as `<script setup>`.
    pub setup: bool,
    /// The `lang` attribute, if any (`ts`, `js`, …).
    pub lang: Option<String>,
    /// Raw source between the tags.
    pub content: String,
}

/// One `<style>` block of a Vue SFC, preserved verbatim for a caller to emit.
#[derive(Debug, Clone, Default)]
pub struct VueStyleBlock {
    pub scoped: bool,
    pub module: bool,
    pub lang: Option<String>,
    pub content: String,
}

/// A split Vue single-file component.
#[derive(Debug, Clone, Default)]
pub struct VueSfc {
    /// Raw contents of the `<template>` block, if present.
    pub template: Option<String>,
    pub scripts: Vec<VueScriptBlock>,
    pub styles: Vec<VueStyleBlock>,
}

pub(crate) fn split_sfc(src: &str) -> Result<VueSfc, RawParseError> {
    let mut sfc = VueSfc::default();
    let bytes = src.as_bytes();
    let mut i = 0usize;

    while i < src.len() {
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }
        if src[i..].starts_with("<!--") {
            i = match src[i + 4..].find("-->") {
                Some(end) => i + 4 + end + 3,
                None => src.len(),
            };
            continue;
        }
        let rest = &src[i..];
        let name = block_name(rest);
        let Some(name) = name else {
            i += 1;
            continue;
        };

        let Some(open_end) = rest.find('>') else {
            return Err(RawParseError {
                message: format!("unclosed <{name}> tag in Vue SFC"),
                byte_offset: Some(i),
            });
        };
        let attrs_src = &rest[1 + name.len()..open_end];
        if attrs_src.trim_end().ends_with('/') {
            i += open_end + 1;
            continue;
        }
        let body_start = i + open_end + 1;
        let body_end = find_block_end(src, body_start, name).ok_or_else(|| RawParseError {
            message: format!("missing </{name}> in Vue SFC"),
            byte_offset: Some(i),
        })?;
        let body = &src[body_start..body_end];

        match name {
            "template" => {
                if sfc.template.is_some() {
                    return Err(RawParseError {
                        message: "Vue SFC has more than one top-level <template> block".to_string(),
                        byte_offset: Some(i),
                    });
                }
                sfc.template = Some(body.to_string());
            }
            "script" => sfc.scripts.push(VueScriptBlock {
                setup: has_flag(attrs_src, "setup"),
                lang: attr_value(attrs_src, "lang"),
                content: body.to_string(),
            }),
            _ => sfc.styles.push(VueStyleBlock {
                scoped: has_flag(attrs_src, "scoped"),
                module: has_flag(attrs_src, "module"),
                lang: attr_value(attrs_src, "lang"),
                content: body.to_string(),
            }),
        }

        i = body_end + name.len() + 3;
    }

    Ok(sfc)
}

fn block_name(rest: &str) -> Option<&'static str> {
    for name in ["template", "script", "style"] {
        if let Some(after) = rest
            .strip_prefix('<')
            .and_then(|r| r.strip_prefix(name))
            .filter(|after| {
                after.starts_with(|c: char| c.is_whitespace()) || after.starts_with('>')
            })
        {
            let _ = after;
            return Some(name);
        }
    }
    None
}

/// Find the byte offset of the matching `</name>` starting at `from`, honouring
/// nested `<name` openings (Vue templates contain nested `<template>` elements).
fn find_block_end(src: &str, from: usize, name: &str) -> Option<usize> {
    let open = format!("<{name}");
    let close = format!("</{name}");
    let mut depth = 1usize;
    let mut i = from;
    while i < src.len() {
        let tail = &src[i..];
        if let Some(after) = tail.strip_prefix("<!--") {
            i += 4 + after.find("-->").map(|e| e + 3).unwrap_or(after.len());
            continue;
        }
        if tail.starts_with(&close) {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
            i += close.len();
            continue;
        }
        if tail.starts_with(&open) {
            let after = &tail[open.len()..];
            if after.starts_with(|c: char| c.is_whitespace()) || after.starts_with('>') {
                let self_closing = after
                    .find('>')
                    .map(|e| after[..e].trim_end().ends_with('/'))
                    .unwrap_or(false);
                if !self_closing {
                    depth += 1;
                }
            }
            i += open.len();
            continue;
        }
        i += tail.chars().next().map(|c| c.len_utf8()).unwrap_or(1);
    }
    None
}

fn has_flag(attrs: &str, flag: &str) -> bool {
    attrs.split_whitespace().any(|token| {
        token == flag || token.starts_with(&format!("{flag}=")) && !token.ends_with("=\"false\"")
    })
}

fn attr_value(attrs: &str, key: &str) -> Option<String> {
    let idx = attrs.find(&format!("{key}="))?;
    let rest = attrs[idx + key.len() + 1..].trim_start();
    let quote = rest.chars().next()?;
    if quote == '"' || quote == '\'' {
        let end = rest[1..].find(quote)? + 1;
        Some(rest[1..end].to_string())
    } else {
        let end = rest.find(|c: char| c.is_whitespace()).unwrap_or(rest.len());
        Some(rest[..end].to_string())
    }
}
