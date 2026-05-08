use crate::parser::{parse_component_file_inner, parse_template_raw, RawParseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrepusDiagnostic {
    pub message: String,
    pub start_line: u32,
    pub start_character: u32,
    pub end_line: u32,
    pub end_character: u32,
}

fn byte_offset_to_position(source: &str, byte: usize) -> (u32, u32) {
    let b = byte.min(source.len());
    let prefix = &source[..b];
    let line = prefix.as_bytes().iter().filter(|&&x| x == b'\n').count() as u32;
    let line_start = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let col_src = &source[line_start..b];
    let character = col_src.encode_utf16().count() as u32;
    (line, character)
}

fn range_for_byte_span(source: &str, start: usize, end: usize) -> (u32, u32, u32, u32) {
    let start = start.min(source.len());
    let end = end.max(start).min(source.len());
    let (sl, sc) = byte_offset_to_position(source, start);
    let (el, ec) = byte_offset_to_position(source, end);
    (sl, sc, el, ec)
}

fn diagnostic_from_offset(source: &str, message: String, byte: usize) -> CrepusDiagnostic {
    let (sl, sc) = byte_offset_to_position(source, byte);
    let line_start = source[..byte.min(source.len())]
        .rfind('\n')
        .map(|i| i + 1)
        .unwrap_or(0);
    let line_rest = source[line_start..]
        .find('\n')
        .unwrap_or(source.len() - line_start);
    let line_end_byte = (line_start + line_rest).min(source.len());
    let end_byte = (byte + 1).min(line_end_byte);
    let (el, ec) = byte_offset_to_position(source, end_byte);
    CrepusDiagnostic {
        message,
        start_line: sl,
        start_character: sc,
        end_line: el,
        end_character: ec,
    }
}

fn diagnostic_fallback_first_line(source: &str, message: String) -> CrepusDiagnostic {
    let end_byte = source.find('\n').unwrap_or(source.len());
    let (sl, sc, el, ec) = range_for_byte_span(source, 0, end_byte);
    CrepusDiagnostic {
        message,
        start_line: sl,
        start_character: sc,
        end_line: el,
        end_character: ec,
    }
}

fn raw_parse_error_to_diagnostic(source: &str, e: RawParseError) -> CrepusDiagnostic {
    match e.byte_offset {
        Some(b) => diagnostic_from_offset(source, e.message, b),
        None => diagnostic_fallback_first_line(source, e.message),
    }
}

pub fn diagnose_crepus_source(source: &str) -> Vec<CrepusDiagnostic> {
    let trimmed = source.trim_start();
    if trimmed.starts_with("+++") {
        match parse_component_file_inner(source) {
            Ok(_) => vec![],
            Err(e) => vec![raw_parse_error_to_diagnostic(source, e)],
        }
    } else {
        match parse_template_raw(source) {
            Ok(_) => vec![],
            Err(e) => vec![raw_parse_error_to_diagnostic(source, e)],
        }
    }
}

pub fn is_multi_component_file(source: &str) -> bool {
    source.trim_start().starts_with("+++")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsx_unclosed_tag_has_line() {
        let src = "<div class=\"x\"";
        let d = diagnose_crepus_source(src);
        assert_eq!(d.len(), 1);
        assert!(
            d[0].message.contains("unclosed") || d[0].message.contains("</"),
            "{}",
            d[0].message
        );
        assert_eq!(d[0].start_line, 0);
    }

    #[test]
    fn indent_only_ok() {
        assert!(diagnose_crepus_source("div\n  \"hi\"").is_empty());
    }
}
