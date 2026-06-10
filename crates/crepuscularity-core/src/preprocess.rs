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
    // Scan backwards from the bottom, pairing `.name` + indented expansion (two-line form)
    // or single-line `.name expansion` form.
    loop {
        if end <= i {
            break;
        }
        let t = lines[end - 1].trim();
        if t.is_empty() {
            end -= 1;
            continue;
        }
        // Single-line: `.name expansion tokens`
        if let Some((name, expansion)) = parse_class_alias_line(t) {
            alias_lines.push((name, expansion));
            end -= 1;
            continue;
        }
        // Two-line: `  expansion tokens` then `.name` (backward: expansion first, then name)
        if end >= 2 {
            let name_line = lines[end - 2].trim();
            let exp_line = lines[end - 1];
            if name_line.starts_with('.') && !name_line.contains(' ') {
                let name_indent = lines[end - 2].len() - lines[end - 2].trim_start().len();
                let exp_indent = exp_line.len() - exp_line.trim_start().len();
                if exp_indent > name_indent && !exp_line.trim().is_empty() {
                    let alias_name = name_line.strip_prefix('.').unwrap_or(name_line).trim().to_string();
                    alias_lines.push((alias_name, exp_line.trim().to_string()));
                    end -= 2;
                    continue;
                }
            }
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

/// If the template starts with a `head` block at indent zero, extract its indented
/// children as raw head content and return the remaining body. Returns `(head_raw, body_raw)`.
///
/// ```crepus
/// head
///   title "Notes"
///   meta charset="utf-8"
///   link rel="icon" href="./static/favicon.svg"
///
/// div wrap
///   ...
/// ```
pub fn extract_head_block(raw: &str) -> (Option<String>, String) {
    let lines: Vec<&str> = raw.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            i += 1;
            continue;
        }
        let trimmed_lower = trimmed.to_lowercase();
        if trimmed_lower == "head" || trimmed_lower.starts_with("head ") {
            let head_indent = lines[i].len() - lines[i].trim_start().len();
            let mut j = i + 1;
            let mut head_lines: Vec<&str> = Vec::new();
            while j < lines.len() {
                let line = lines[j];
                if line.trim().is_empty() {
                    head_lines.push("");
                    j += 1;
                    continue;
                }
                let line_indent = line.len() - line.trim_start().len();
                if line_indent > head_indent {
                    let dedented = &line[head_indent + 2..]; // strip one level
                    head_lines.push(dedented);
                    j += 1;
                } else {
                    break;
                }
            }
            let head_raw = if head_lines.is_empty() {
                String::new()
            } else {
                head_lines.join("\n")
            };
            let body = if j < lines.len() {
                let mut body_lines: Vec<&str> = Vec::new();
                body_lines.push("# head block removed"); // marker so parser knows this is the body
                body_lines.extend(&lines[j..]);
                body_lines.join("\n")
            } else {
                String::new()
            };
            return (
                if head_raw.is_empty() {
                    None
                } else {
                    Some(head_raw)
                },
                body,
            );
        }
        break;
    }
    (None, raw.to_string())
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
                let css = lines[(open + 1)..(cursor - 1)]
                    .join("\n")
                    .trim()
                    .to_string();
                return (open, css);
            }
        }
    }

    // Trailing raw CSS without `<style>` wrappers.
    //
    // The body and the CSS tail are not separated by a blank line in many
    // templates, so walking back through "CSS-shaped" lines alone is not
    // enough — `.crepus` element lines like `div bind:href={url}` and bare
    // expressions like `{score}` also end with `}`. We require an
    // **unambiguous CSS opener** at the top of the candidate trailing block
    // (`@`-rule, comment, or a selector line ending with `{`).
    while end > start && lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    if end <= start {
        return (end, String::new());
    }
    if !lines[end - 1].trim().ends_with('}') {
        return (end, String::new());
    }

    let mut css_start = end;
    while css_start > start {
        let t = lines[css_start - 1].trim();
        if t.is_empty() {
            if css_start > start + 1 && looks_like_css_line(lines[css_start - 2].trim()) {
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
    if css_start >= end {
        return (end, String::new());
    }

    // If the first line at css_start doesn't look like a CSS opener, try
    // the next line — the walk may have landed on a template-remnant line
    // (e.g. `div` placeholder) when the file is CSS-only.
    let mut actual_start = css_start;
    while actual_start < end {
        let candidate = lines[actual_start].trim();
        if candidate.starts_with('@')
            || candidate.starts_with("/*")
            || candidate.ends_with('{')
            || (candidate.ends_with('}') && candidate.contains('{') && candidate.contains(':'))
        {
            break;
        }
        actual_start += 1;
    }
    if actual_start >= end {
        return (end, String::new());
    }
    let css = lines[actual_start..end].join("\n").trim().to_string();
    (actual_start, css)
}

/// Heuristic: does this trimmed line look like a real CSS line (selector, rule
/// boundary, declaration, at-rule, or comment) rather than a `.crepus` template
/// line?
///
/// `.crepus` lines such as text nodes (`"Hello {name}"`), bare expressions
/// (`{score}`), `$:` declarations (`$: let x = {expr}`), bound elements
/// (`div bind:href={url}`), and control headers (`for x in {items}`,
/// `match {status}`) all contain braces but must be kept in the body — so we
/// only treat a line as CSS when it has an *unambiguous* CSS shape:
///
/// - starts with `@` (at-rule) or `/*` (comment) or `}` (block close)
/// - ends with `{` (selector opener — never appears in indent-mode `.crepus`)
/// - is a CSS declaration `prop: value;`
/// - is a complete inline CSS rule `selector { prop: value; }`
fn looks_like_css_line(line: &str) -> bool {
    if line.starts_with('@') || line.starts_with("/*") || line.starts_with('}') {
        return true;
    }
    if line.ends_with('{') || line.ends_with(',') {
        return true;
    }
    if line.ends_with(';') && line.contains(':') {
        return true;
    }
    if line.ends_with('}') && line.contains('{') && line.contains(':') && line.contains(';') {
        return true;
    }
    if line.contains(':') && line.contains(';') {
        return true;
    }
    false
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
            Node::LetDecl(_)
            | Node::Text(_)
            | Node::RawText(_)
            | Node::RawHtml(_)
            | Node::Embed(_) => {}
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
        let slug = encode_google_font_family(f);
        q.push_str("family=");
        q.push_str(&slug);
        if !slug.contains(':') {
            q.push_str(":wght@400;500;600;700");
        }
    }
    q.push_str("&display=swap");
    format!(
        r#"  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?{q}" rel="stylesheet">"#
    )
}

pub fn google_font_css_family_name(family: &str) -> &str {
    family
        .split_once(':')
        .map_or(family, |(name, _)| name)
        .trim()
}

fn encode_google_font_family(family: &str) -> String {
    let family = normalize_google_font_family(family);
    let mut out = String::new();
    for byte in family.as_bytes() {
        match byte {
            b' ' => out.push('+'),
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b':' | b',' | b'@' | b'.' | b'_' | b'-' => {
                out.push(*byte as char);
            }
            b => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn normalize_google_font_family(family: &str) -> String {
    match family.trim().to_lowercase().as_str() {
        "material symbols outlined" => {
            "Material Symbols Outlined:opsz,wght,FILL,GRAD@24,400,0,0".to_string()
        }
        "material symbols rounded" => {
            "Material Symbols Rounded:opsz,wght,FILL,GRAD@24,400,0,0".to_string()
        }
        "material symbols sharp" => {
            "Material Symbols Sharp:opsz,wght,FILL,GRAD@24,400,0,0".to_string()
        }
        _ => family.trim().to_string(),
    }
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
    fn strips_trailing_css_blocks_separated_by_blank_lines() {
        // Regression: multiple CSS blocks separated by blank lines must all be
        // stripped, not just the last block.
        let s = r#"div
  "x"
@keyframes sunset {
  0% { opacity: .5; }
  100% { opacity: 1; }
}
.animate-sunset {
  animation: sunset 24s ease-in-out infinite;
}

@media (prefers-reduced-motion: reduce) {
  .animate-sunset {
    animation: none;
  }
}
"#;
        let d = strip_indent_decorators(s);
        assert_eq!(d.body.trim(), "div\n  \"x\"");
        assert!(d.inline_css.contains("@keyframes sunset"));
        assert!(d.inline_css.contains(".animate-sunset"));
        assert!(d
            .inline_css
            .contains("@media (prefers-reduced-motion: reduce)"));
    }

    #[test]
    fn google_fonts_head_markup_smoke() {
        let s = google_fonts_head_markup(&["JetBrains Mono".into(), "Inter".into()]);
        assert!(s.contains("fonts.googleapis.com"));
        assert!(s.contains("JetBrains+Mono"));
        assert!(s.contains("family=Inter"));
    }

    #[test]
    fn google_fonts_head_markup_supports_material_symbols_shorthand() {
        let s = google_fonts_head_markup(&["Material Symbols Outlined".into()]);
        assert!(s.contains("Material+Symbols+Outlined:opsz,wght,FILL,GRAD@24,400,0,0"));
        assert!(!s.contains("Material+Symbols+Outlined:wght@400"));
    }

    #[test]
    fn google_fonts_head_markup_preserves_axis_suffix() {
        let s = google_fonts_head_markup(&[
            "Inter".into(),
            "Material Symbols Rounded:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200".into(),
        ]);
        assert!(s.contains("family=Inter:wght@400;500;600;700"));
        assert!(s.contains(
            "family=Material+Symbols+Rounded:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200"
        ));
    }

    #[test]
    fn does_not_strip_trailing_text_with_interpolation() {
        let s = "div w-full h-full flex-col\n  div\n    \"Hello {name}\"\n";
        let d = strip_indent_decorators(s);
        assert!(
            d.body.contains("Hello {name}"),
            "trailing text node was stripped: body={:?} css={:?}",
            d.body,
            d.inline_css
        );
        assert!(d.inline_css.is_empty(), "css={:?}", d.inline_css);
    }

    #[test]
    fn does_not_strip_trailing_bare_expression() {
        let s = "div\n  {score}\n";
        let d = strip_indent_decorators(s);
        assert!(
            d.body.contains("{score}"),
            "bare expression was stripped: body={:?} css={:?}",
            d.body,
            d.inline_css
        );
        assert!(d.inline_css.is_empty());
    }

    #[test]
    fn does_not_strip_trailing_let_decl() {
        let s = "div\n  $: let total = {price * qty}\n";
        let d = strip_indent_decorators(s);
        assert!(
            d.body.contains("$: let total"),
            "$: let was stripped: body={:?} css={:?}",
            d.body,
            d.inline_css
        );
        assert!(d.inline_css.is_empty());
    }

    #[test]
    fn still_strips_at_rule_tail_with_interpolation_above() {
        let s = "div\n  \"score: {score}\"\n@keyframes pulse {\n  0% { opacity: .5; }\n  100% { opacity: 1; }\n}\n";
        let d = strip_indent_decorators(s);
        assert!(d.body.contains("score: {score}"), "body={:?}", d.body);
        assert!(d.inline_css.contains("@keyframes pulse"));
    }

    #[test]
    fn does_not_strip_trailing_element_with_binding() {
        let s = "div bind:href={url}\n";
        let d = strip_indent_decorators(s);
        assert!(d.body.contains("bind:href={url}"), "body={:?}", d.body);
        assert!(d.inline_css.is_empty());
    }

    #[test]
    fn does_not_strip_trailing_class_binding() {
        let s = "div\n  span class:active={selected}\n";
        let d = strip_indent_decorators(s);
        assert!(
            d.body.contains("class:active={selected}"),
            "body={:?}",
            d.body
        );
        assert!(d.inline_css.is_empty());
    }

    #[test]
    fn strips_css_after_trailing_binding_line_without_blank_separator() {
        // Regression: a bound element on the last template line directly
        // followed by a CSS `@keyframes` block (no blank line in between)
        // must keep the binding in the body and strip only the CSS.
        let s = "div bind:href={url}\n@keyframes pulse {\n  0% { opacity: .5; }\n  100% { opacity: 1; }\n}\n";
        let d = strip_indent_decorators(s);
        assert!(d.body.contains("bind:href={url}"), "body={:?}", d.body);
        assert!(
            d.inline_css.contains("@keyframes pulse"),
            "css={:?}",
            d.inline_css
        );
        assert!(
            !d.body.contains("@keyframes"),
            "css leaked into body: body={:?}",
            d.body
        );
    }

    #[test]
    fn does_not_strip_trailing_match_header() {
        let s = "div\n  match {status}\n    \"a\" =>\n      div\n        \"A\"\n";
        let d = strip_indent_decorators(s);
        assert!(d.body.contains("match {status}"), "body={:?}", d.body);
        assert!(d.inline_css.is_empty());
    }

    #[test]
    fn does_not_strip_trailing_for_header() {
        let s = "div\n  for item in {items}\n    div\n      {item}\n";
        let d = strip_indent_decorators(s);
        assert!(d.body.contains("for item in {items}"), "body={:?}", d.body);
        assert!(d.inline_css.is_empty());
    }

    #[test]
    fn css_only_file_without_template_placeholder() {
        let s = ".foo {\n  color: red;\n}\n.bar {\n  color: blue;\n}\n";
        let d = strip_indent_decorators(s);
        assert!(
            d.body.trim().is_empty(),
            "body should be empty, got {:?}",
            d.body
        );
        assert!(
            d.inline_css.contains(".foo"),
            "css missing .foo: {:?}",
            d.inline_css
        );
        assert!(
            d.inline_css.contains(".bar"),
            "css missing .bar: {:?}",
            d.inline_css
        );
    }

    #[test]
    fn test_merge_unique_font_families() {
        let input = vec![
            "  Inter  ".to_string(), // Trimming
            "".to_string(),          // Empty string
            "   ".to_string(),       // Whitespace-only string
            "Roboto".to_string(),    // Normal
            "inter".to_string(),     // Duplicate, different case
            "INTER".to_string(),     // Duplicate, different case
            "Open Sans".to_string(), // Normal
            "roboto".to_string(),    // Duplicate, different case
        ];

        let result = merge_unique_font_families(input);

        assert_eq!(
            result,
            vec![
                "Inter".to_string(),
                "Roboto".to_string(),
                "Open Sans".to_string(),
            ]
        );
    }
}
