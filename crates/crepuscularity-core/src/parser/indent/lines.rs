//! Line collection and structural indent handling.

/// Strip exactly `n` leading spaces from `s`, leaving any additional
/// whitespace intact (it is content, not structural indent).
pub(crate) fn strip_structural_indent(s: &str, n: usize) -> &str {
    for (count, (byte_pos, ch)) in s.char_indices().enumerate() {
        if count >= n || ch != ' ' {
            return &s[byte_pos..];
        }
    }
    // Entire string was spaces (≤ n of them).
    ""
}

pub(crate) fn collect_lines(template: &str) -> Vec<(usize, String)> {
    let source_lines: Vec<&str> = template.lines().collect();
    let mut raw: Vec<(usize, String)> = Vec::new();
    let mut i = 0;

    while i < source_lines.len() {
        let line = source_lines[i];
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        i += 1;

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Multi-line strings: a `"` that opens but doesn't close on the same line
        // is merged with subsequent lines until the closing `"` is found.
        // Continuation lines have exactly `indent` leading spaces stripped (the
        // structural indent of the opening `"` line); any additional whitespace
        // is part of the string content and must be preserved.
        if trimmed.starts_with('"') && !string_is_closed(trimmed) {
            let mut combined = trimmed.to_string();
            while i < source_lines.len() {
                combined.push('\n');
                combined.push_str(strip_structural_indent(source_lines[i], indent));
                i += 1;
                if string_is_closed(&combined) {
                    break;
                }
            }
            raw.push((indent, combined));
        } else {
            raw.push((indent, trimmed.to_string()));
        }
    }

    // Normalize indentation so root elements always start at column 0.
    let min_indent = raw.iter().map(|(i, _)| *i).min().unwrap_or(0);
    if min_indent == 0 {
        return raw;
    }
    raw.into_iter().map(|(i, l)| (i - min_indent, l)).collect()
}

/// Returns true when `s` is a properly closed double-quoted string
/// (starts with `"` and has a matching unescaped closing `"`).
fn string_is_closed(s: &str) -> bool {
    if !s.starts_with('"') {
        return false;
    }
    let mut escaped = false;
    let mut closes = 0usize;
    for ch in s.chars().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            closes += 1;
        }
    }
    closes > 0
}
