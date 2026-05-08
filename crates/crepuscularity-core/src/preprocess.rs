//! Indent-syntax decorators: top-of-file Google Font pragmas and trailing `.alias` class shortcuts.
//!
//! Font pragmas (only at the top of the file, before real template lines):
//! - `google-font Inter` or `google-font: Inter` — one family, unquoted (spaces allowed).
//! - `google-font "Inter"` — one family, quoted (use quotes when the name has edge cases).
//! - `google-fonts "Inter" "JetBrains Mono"` — several families in one line (each must be quoted).

use std::collections::{HashMap, HashSet};

use crate::ast::{ConditionalClass, Node, TextPart};

/// Result of stripping indent-only decorators before parse.
#[derive(Debug, Clone)]
pub struct IndentDecorators {
    /// Source with pragma lines removed (ready for `collect_lines` / `parse_template`).
    pub body: String,
    /// Google Font family names (e.g. `"Inter"`, `"JetBrains Mono"`).
    pub google_fonts: Vec<String>,
    /// Maps shortcut name (without leading dot) → expanded utility string.
    pub class_aliases: HashMap<String, String>,
    /// Raw CSS collected from trailing style blocks / CSS tails.
    pub inline_css: String,
}

/// Strip `google-font` / `google-fonts` lines from the top and `.name tokens…` alias lines from the bottom.
/// JSX mode templates are returned unchanged (no stripping).
pub fn strip_indent_decorators(raw: &str) -> IndentDecorators {
    let lines: Vec<&str> = raw.lines().collect();
    if lines.is_empty() {
        return IndentDecorators {
            body: raw.to_string(),
            google_fonts: Vec::new(),
            class_aliases: HashMap::new(),
            inline_css: String::new(),
        };
    }

    let mut google_fonts = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let t = lines[i].trim();
        if t.is_empty() || t.starts_with('#') {
            i += 1;
            continue;
        }
        if let Some(families) = parse_google_font_pragma(t) {
            google_fonts.extend(families);
            i += 1;
            continue;
        }
        break;
    }

    let mut end = lines.len();
    let mut alias_lines: Vec<(String, String)> = Vec::new();
    while end > i {
        let t = lines[end - 1].trim();
        if t.is_empty() {
            end -= 1;
            continue;
        }
        if let Some((name, expansion)) = parse_class_alias_line(t) {
            alias_lines.push((name, expansion));
            end -= 1;
            continue;
        }
        break;
    }

    let mut class_aliases = HashMap::new();
    for (name, exp) in alias_lines.into_iter().rev() {
        class_aliases.insert(name, exp);
    }

    let (end, inline_css) = strip_trailing_inline_css(&lines, i, end);
    let body = lines[i..end].join("\n");
    IndentDecorators {
        body,
        google_fonts,
        class_aliases,
        inline_css,
    }
}

fn strip_trailing_inline_css(lines: &[&str], start: usize, mut end: usize) -> (usize, String) {
    if end <= start {
        return (end, String::new());
    }

    // Explicit trailing <style>...</style> block.
    let mut cursor = end;
    while cursor > start && lines[cursor - 1].trim().is_empty() {
        cursor -= 1;
    }
    if cursor > start && lines[cursor - 1].trim() == "</style>" {
        let mut open = cursor - 1;
        while open > start {
            open -= 1;
            if lines[open].trim() == "<style>" {
                let css = lines[(open + 1)..(cursor - 1)].join("\n").trim().to_string();
                return (open, css);
            }
        }
    }

    // Trailing raw CSS lines without <style> wrappers.
    let mut css_start = end;
    while css_start > start {
        let t = lines[css_start - 1].trim();
        if t.is_empty() {
            if css_start == end {
                end -= 1;
                css_start -= 1;
                continue;
            }
            break;
        }
        if !looks_like_css_line(t) {
            break;
        }
        css_start -= 1;
    }
    if css_start < end {
        let css = lines[css_start..end].join("\n").trim().to_string();
        return (css_start, css);
    }
    (end, String::new())
}

fn looks_like_css_line(line: &str) -> bool {
    line.starts_with('@')
        || line.starts_with('}')
        || line.starts_with("/*")
        || line.ends_with('{')
        || line.contains('{')
        || line.contains('}')
        || line.ends_with(';')
}

/// Returns font families declared on this line, or `None` if the line is not a font pragma.
fn parse_google_font_pragma(line: &str) -> Option<Vec<String>> {
    let t = line.trim();
    // `google-font` is a prefix of `google-fonts` — match plural first.
    let (plural, after_kw) = if let Some(r) = t.strip_prefix("google-fonts") {
        (true, r.trim_start())
    } else if let Some(r) = t.strip_prefix("google-font") {
        (false, r.trim_start())
    } else {
        return None;
    };

    let rest = after_kw
        .strip_prefix(':')
        .map(str::trim)
        .unwrap_or(after_kw)
        .trim();
    if rest.is_empty() {
        return None;
    }

    let quoted = parse_quoted_font_names(rest);
    if !quoted.is_empty() {
        if plural {
            return Some(quoted);
        }
        return Some(vec![quoted[0].clone()]);
    }

    if plural {
        // `google-fonts` requires quoted family names so multi-word names are unambiguous.
        return None;
    }

    Some(vec![rest.to_string()])
}

/// Parses consecutive `"..."` tokens (supports `\"` and `\\` inside quotes).
fn parse_quoted_font_names(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let b = s.as_bytes();
    let mut i = 0usize;
    while i < b.len() {
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= b.len() {
            break;
        }
        if b[i] != b'"' {
            return out;
        }
        i += 1;
        let start = i;
        while i < b.len() {
            match b[i] {
                b'\\' if i + 1 < b.len() => i += 2,
                b'"' => break,
                _ => i += 1,
            }
        }
        if i >= b.len() {
            break;
        }
        let inner = &s[start..i];
        let decoded = inner.replace("\\\\", "\\").replace("\\\"", "\"");
        out.push(decoded);
        i += 1;
    }
    out
}

fn parse_class_alias_line(line: &str) -> Option<(String, String)> {
    let t = line.trim();
    let rest = t.strip_prefix('.')?;
    let mut parts = rest.splitn(2, char::is_whitespace);
    let name = parts.next()?.trim();
    if name.is_empty() {
        return None;
    }
    let expansion = parts.next()?.trim();
    if expansion.is_empty() {
        return None;
    }
    Some((name.to_string(), expansion.to_string()))
}

/// Expand `.shortcut` tokens in `classes` using `aliases` (one level).
pub fn expand_class_token(token: &str, aliases: &HashMap<String, String>) -> Vec<String> {
    if let Some(exp) = aliases.get(token) {
        return exp.split_whitespace().map(|s| s.to_string()).collect();
    }
    vec![token.to_string()]
}

/// Recursively expand class shortcuts on every element.
pub fn expand_class_aliases_in_nodes(nodes: &mut [Node], aliases: &HashMap<String, String>) {
    if aliases.is_empty() {
        return;
    }
    for node in nodes.iter_mut() {
        match node {
            Node::Element(el) => {
                let mut out = Vec::new();
                for c in std::mem::take(&mut el.classes) {
                    out.extend(expand_class_token(&c, aliases));
                }
                el.classes = out;
                let mut out_cc: Vec<ConditionalClass> = Vec::new();
                for cc in std::mem::take(&mut el.conditional_classes) {
                    for c in expand_class_token(&cc.class, aliases) {
                        out_cc.push(ConditionalClass {
                            class: c,
                            condition: cc.condition.clone(),
                        });
                    }
                }
                el.conditional_classes = out_cc;
                expand_class_aliases_in_nodes(&mut el.children, aliases);
            }
            Node::If(b) => {
                expand_class_aliases_in_nodes(&mut b.then_children, aliases);
                if let Some(else_c) = &mut b.else_children {
                    expand_class_aliases_in_nodes(else_c, aliases);
                }
            }
            Node::For(b) => {
                expand_class_aliases_in_nodes(&mut b.body, aliases);
            }
            Node::Match(b) => {
                for arm in &mut b.arms {
                    expand_class_aliases_in_nodes(&mut arm.body, aliases);
                }
            }
            Node::Include(inc) => {
                expand_class_aliases_in_nodes(&mut inc.slot, aliases);
            }
            Node::LetDecl(_) | Node::Text(_) | Node::RawText(_) => {}
        }
    }
}

/// Deduplicate font family names (case-insensitive), preserving first-seen order.
pub fn merge_unique_font_families<I: IntoIterator<Item = String>>(iter: I) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for f in iter {
        let t = f.trim().to_string();
        if t.is_empty() {
            continue;
        }
        let k = t.to_lowercase();
        if seen.insert(k) {
            out.push(t);
        }
    }
    out
}

/// `<link rel="preconnect">` and one Google Fonts `css2` stylesheet for the given families.
pub fn google_fonts_head_markup(families: &[String]) -> String {
    if families.is_empty() {
        return String::new();
    }
    let mut q = String::new();
    for (i, f) in families.iter().enumerate() {
        if i > 0 {
            q.push('&');
        }
        let slug = f.split_whitespace().collect::<Vec<_>>().join("+");
        q.push_str("family=");
        q.push_str(&slug);
        q.push_str(":wght@400;500;600;700");
    }
    q.push_str("&display=swap");
    format!(
        r#"  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?{q}" rel="stylesheet">"#
    )
}

/// Plain-text lines from a `slot-rotate` element's children (web + native renderers).
pub fn slot_rotate_child_phrases(children: &[Node]) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for c in children {
        match c {
            Node::Text(parts) => {
                let mut s = String::new();
                for p in parts {
                    match p {
                        TextPart::Literal(l) => s.push_str(l),
                        TextPart::Expr(_) => {
                            return Err(
                                "slot-rotate children must be plain text (no `{…}` expressions)"
                                    .into(),
                            );
                        }
                    }
                }
                let t = s.trim();
                if !t.is_empty() {
                    out.push(t.to_string());
                }
            }
            _ => return Err("slot-rotate only allows quoted text lines as children".into()),
        }
    }
    Ok(out)
}

/// JSON array for `data-slot-words` (avoids `|` collisions in phrases).
pub fn slot_rotate_words_json_attr(phrases: &[String]) -> String {
    let mut s = String::from('[');
    for (i, p) in phrases.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push('"');
        for ch in p.chars() {
            match ch {
                '\\' => s.push_str(r"\\"),
                '"' => s.push_str("\\\""),
                c if c.is_control() => {
                    s.push_str(&format!("\\u{:04x}", ch as u32));
                }
                c => s.push(c),
            }
        }
        s.push('"');
    }
    s.push(']');
    s
}

/// Expand alias tokens in one element's class list (for the `view!` proc-macro AST).
pub fn expand_class_list_in_place(classes: &mut Vec<String>, aliases: &HashMap<String, String>) {
    if aliases.is_empty() {
        return;
    }
    let mut out = Vec::new();
    for c in std::mem::take(classes) {
        out.extend(expand_class_token(&c, aliases));
    }
    *classes = out;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_fonts_and_aliases() {
        let s = r#"google-font Inter
google-font JetBrains Mono

div center
  "hi"
.center items-center justify-center flex
.body-text text-sm text-black
"#;
        let d = strip_indent_decorators(s);
        assert_eq!(d.google_fonts, vec!["Inter", "JetBrains Mono"]);
        assert_eq!(
            d.class_aliases.get("center").map(String::as_str),
            Some("items-center justify-center flex")
        );
        assert!(d.body.contains("div center"));
        assert!(!d.body.contains("google-font"));
        assert!(!d.body.contains(".center"));
        assert!(d.inline_css.is_empty());
    }

    #[test]
    fn google_fonts_one_line_quoted() {
        let s = r#"google-fonts "Inter" "JetBrains Mono"

div
  "x"
"#;
        let d = strip_indent_decorators(s);
        assert_eq!(d.google_fonts, vec!["Inter", "JetBrains Mono"]);
    }

    #[test]
    fn google_font_quoted_single() {
        let s = "google-font \"IBM Plex Sans\"\ndiv\n";
        let d = strip_indent_decorators(s);
        assert_eq!(d.google_fonts, vec!["IBM Plex Sans"]);
    }

    #[test]
    fn strips_trailing_style_block_into_inline_css() {
        let s = r#"div p-4
  "hello"
<style>
  @keyframes sunset {
    0% { opacity: .6; }
    100% { opacity: 1; }
  }
</style>
"#;
        let d = strip_indent_decorators(s);
        assert!(d.body.contains("div p-4"));
        assert!(!d.body.contains("<style>"));
        assert!(d.inline_css.contains("@keyframes sunset"));
    }

    #[test]
    fn strips_trailing_raw_css_tail() {
        let s = r#"div
  "x"
@keyframes fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}
.animate-fade-in {
  animation: fade-in 1s ease-in-out;
}
"#;
        let d = strip_indent_decorators(s);
        assert_eq!(d.body.trim(), "div\n  \"x\"");
        assert!(d.inline_css.contains(".animate-fade-in"));
    }

    #[test]
    fn google_fonts_head_markup_smoke() {
        let s = google_fonts_head_markup(&["JetBrains Mono".into(), "Inter".into()]);
        assert!(s.contains("fonts.googleapis.com"));
        assert!(s.contains("JetBrains+Mono"));
        assert!(s.contains("family=Inter"));
    }
}
