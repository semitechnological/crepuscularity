//! Indent-syntax decorators: top-of-file `google-font` lines and trailing `.alias` class shortcuts.

use std::collections::{HashMap, HashSet};

use crate::ast::{Node, TextPart};

/// Result of stripping indent-only decorators before parse.
#[derive(Debug, Clone)]
pub struct IndentDecorators {
    /// Source with pragma lines removed (ready for `collect_lines` / `parse_template`).
    pub body: String,
    /// Google Font family names (e.g. `"Inter"`, `"JetBrains Mono"`).
    pub google_fonts: Vec<String>,
    /// Maps shortcut name (without leading dot) → expanded utility string.
    pub class_aliases: HashMap<String, String>,
}

/// Strip `google-font …` lines from the top and `.name tokens…` alias lines from the bottom.
/// JSX mode templates are returned unchanged (no stripping).
pub fn strip_indent_decorators(raw: &str) -> IndentDecorators {
    let lines: Vec<&str> = raw.lines().collect();
    if lines.is_empty() {
        return IndentDecorators {
            body: raw.to_string(),
            google_fonts: Vec::new(),
            class_aliases: HashMap::new(),
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
        if let Some(family) = parse_google_font_line(t) {
            google_fonts.push(family);
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

    let body = lines[i..end].join("\n");
    IndentDecorators {
        body,
        google_fonts,
        class_aliases,
    }
}

fn parse_google_font_line(line: &str) -> Option<String> {
    let t = line.trim();
    let rest = t.strip_prefix("google-font")?;
    let rest = rest.trim_start();
    if rest.is_empty() {
        return None;
    }
    // `google-font: Inter` or `google-font Inter`
    let name = rest.strip_prefix(':').map(str::trim).unwrap_or(rest).trim();
    if name.is_empty() {
        return None;
    }
    Some(name.to_string())
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
    }

    #[test]
    fn google_fonts_head_markup_smoke() {
        let s = google_fonts_head_markup(&["JetBrains Mono".into(), "Inter".into()]);
        assert!(s.contains("fonts.googleapis.com"));
        assert!(s.contains("JetBrains+Mono"));
        assert!(s.contains("family=Inter"));
    }
}
