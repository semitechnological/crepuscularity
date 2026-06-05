/// Runtime parser for the crepuscularity template DSL.
/// Mirrors the compile-time proc-macro parser but operates on strings at runtime.
use std::collections::HashMap;

use crate::ast::*;

#[derive(Debug, Clone)]
pub(crate) struct RawParseError {
    pub message: String,
    pub byte_offset: Option<usize>,
}

fn subslice_byte_offset(full: &str, tail: &str) -> usize {
    let fp = full.as_ptr() as usize;
    let tp = tail.as_ptr() as usize;
    debug_assert!(tp >= fp && tp <= fp.saturating_add(full.len()));
    tp.saturating_sub(fp)
}

// ── Multi-component files ─────────────────────────────────────────────────────

/// Metadata and default prop values for one component parsed from TOML frontmatter.
#[derive(Debug, Clone, Default)]
pub struct ComponentMeta {
    /// Description string (from `description = "..."` in TOML).
    pub description: Option<String>,
    /// Default prop values as evaluable expression strings.
    /// `[Card.defaults]` section: `title = "Untitled"` → `"title" → "\"Untitled\""`
    pub defaults: HashMap<String, String>,
}

/// A named component extracted from a multi-component file.
#[derive(Debug, Clone)]
pub struct ComponentDef {
    pub nodes: Vec<Node>,
    pub meta: ComponentMeta,
}

/// Parsed multi-component `.crepus` file.
///
/// A multi-component file starts with an optional `+++...+++` TOML frontmatter
/// block, followed by one or more component sections introduced by `--- Name`.
///
/// ```text
/// +++
/// [Card]
/// description = "A simple card"
///
/// [Card.defaults]
/// title = "Untitled"
/// subtitle = ""
///
/// [Button]
/// description = "A clickable button"
///
/// [Button.defaults]
/// label = "Click me"
/// variant = "primary"
/// +++
///
/// --- Card
/// div rounded-lg border p-4 mb-2
///   div font-bold text-lg
///     {title}
///   div text-sm text-gray-400
///     {subtitle}
///   slot
///
/// --- Button
/// $: default variant = "primary"
/// button px-4 py-2 rounded
///   {label}
/// ```
///
/// Components are then included with `include components.crepus#Card title="Hello"`.
pub struct ComponentFile {
    pub components: HashMap<String, ComponentDef>,
}

/// Parse a multi-component `.crepus` file into a [`ComponentFile`].
#[tracing::instrument(skip(content), fields(len = content.len()))]
pub fn parse_component_file(content: &str) -> Result<ComponentFile, String> {
    parse_component_file_inner(content).map_err(|e| e.message)
}

pub(crate) fn parse_component_file_inner(content: &str) -> Result<ComponentFile, RawParseError> {
    let (frontmatter_str, body, body_byte_in_file) = split_frontmatter_parts(content);

    let mut meta_map: HashMap<String, ComponentMeta> = HashMap::new();
    if let Some(toml_src) = frontmatter_str {
        let toml_block_start = subslice_byte_offset(content, toml_src);
        let value: toml::Value =
            toml::from_str(toml_src).map_err(|e: toml::de::Error| RawParseError {
                message: format!("TOML parse error in frontmatter: {e}"),
                byte_offset: e.span().map(|r| toml_block_start + r.start),
            })?;

        if let toml::Value::Table(table) = value {
            for (comp_name, comp_val) in &table {
                if let toml::Value::Table(comp_table) = comp_val {
                    let mut meta = ComponentMeta::default();

                    if let Some(toml::Value::String(desc)) = comp_table.get("description") {
                        meta.description = Some(desc.clone());
                    }

                    if let Some(toml::Value::Table(defs)) = comp_table.get("defaults") {
                        for (k, v) in defs {
                            meta.defaults.insert(k.clone(), toml_value_to_expr(v));
                        }
                    }

                    meta_map.insert(comp_name.clone(), meta);
                }
            }
        }
    }

    let sections = split_component_body_sections(body);
    let mut components = HashMap::new();

    for (name, section_content, sec_start_in_body) in sections {
        let nodes = parse_template_raw(&section_content).map_err(|mut e| {
            if let Some(off) = e.byte_offset {
                e.byte_offset = Some(body_byte_in_file + sec_start_in_body + off);
            }
            if e.byte_offset.is_some() {
                e.message = format!("component {name:?}: {}", e.message);
            }
            e
        })?;
        let meta = meta_map.remove(&name).unwrap_or_default();
        components.insert(name, ComponentDef { nodes, meta });
    }

    Ok(ComponentFile { components })
}

pub(crate) fn split_frontmatter_parts(content: &str) -> (Option<&str>, &str, usize) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("+++") {
        return (None, content, 0);
    }
    let after_open = &trimmed[3..];
    if let Some(close_pos) = after_open.find("\n+++") {
        let raw_toml = &after_open[..close_pos];
        let rest = &after_open[close_pos + 4..];
        let body_start = subslice_byte_offset(content, rest);
        (Some(raw_toml), rest, body_start)
    } else {
        (None, content, 0)
    }
}

pub(crate) fn split_component_body_sections(body: &str) -> Vec<(String, String, usize)> {
    let mut sections: Vec<(String, String, usize)> = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_lines: Vec<&str> = Vec::new();
    let mut content_start_byte: Option<usize> = None;
    let mut pending_content_start = 0usize;

    let mut byte_pos = 0usize;
    while byte_pos <= body.len() {
        let line_start = byte_pos;
        if line_start >= body.len() {
            break;
        }
        let nl = body[line_start..].find('\n');
        let line_end = nl.map(|n| line_start + n).unwrap_or(body.len());
        let raw_line = &body[line_start..line_end];
        byte_pos = if nl.is_some() {
            line_end + 1
        } else {
            body.len()
        };

        let line_content = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let trimmed = line_content.trim();

        if let Some(name) = trimmed.strip_prefix("--- ") {
            if let Some(prev) = current_name.take() {
                let joined = current_lines.join("\n");
                let start = content_start_byte.unwrap_or(pending_content_start);
                sections.push((prev, joined, start));
                current_lines.clear();
            }
            current_name = Some(name.trim().to_string());
            content_start_byte = None;
            pending_content_start = byte_pos;
        } else if current_name.is_some() {
            if content_start_byte.is_none() {
                content_start_byte = Some(line_start);
            }
            current_lines.push(line_content);
        }
    }

    if let Some(name) = current_name {
        let joined = current_lines.join("\n");
        let start = content_start_byte.unwrap_or(pending_content_start);
        sections.push((name, joined, start));
    }

    sections
}

/// Convert a TOML scalar to an expression string usable by the evaluator.
fn toml_value_to_expr(v: &toml::Value) -> String {
    match v {
        toml::Value::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(f) => f.to_string(),
        toml::Value::Boolean(b) => b.to_string(),
        _ => "\"\"".to_string(),
    }
}

#[tracing::instrument(skip(template), fields(len = template.len()))]
pub fn parse_template(template: &str) -> Result<Vec<Node>, String> {
    parse_template_raw(template).map_err(|e| e.message)
}

pub(crate) fn parse_template_raw(template: &str) -> Result<Vec<Node>, RawParseError> {
    if is_jsx_mode(template) {
        parse_jsx_template(template)
    } else {
        let dec = crate::preprocess::strip_indent_decorators(template);
        let lines = collect_lines(&dec.body);
        let (mut nodes, _) = parse_nodes(&lines, 0, 0);
        crate::preprocess::expand_class_aliases_in_nodes(&mut nodes, &dec.class_aliases);
        Ok(nodes)
    }
}

/// Returns true when the first non-blank, non-comment, non-`$:` line starts with `<`.
/// This activates the JSX/HTML tag-based input syntax.
fn is_jsx_mode(template: &str) -> bool {
    for line in template.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') || t.starts_with("$:") {
            continue;
        }
        return t.starts_with('<');
    }
    false
}

/// Strip exactly `n` leading spaces from `s`, leaving any additional
/// whitespace intact (it is content, not structural indent).
fn strip_structural_indent(s: &str, n: usize) -> &str {
    for (count, (byte_pos, ch)) in s.char_indices().enumerate() {
        if count >= n || ch != ' ' {
            return &s[byte_pos..];
        }
    }
    // Entire string was spaces (≤ n of them).
    ""
}

fn collect_lines(template: &str) -> Vec<(usize, String)> {
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

fn parse_nodes(
    lines: &[(usize, String)],
    start: usize,
    expected_indent: usize,
) -> (Vec<Node>, usize) {
    let mut nodes = Vec::new();
    let mut i = start;

    while i < lines.len() {
        let (indent, line) = &lines[i];

        if *indent < expected_indent {
            break;
        }
        if *indent > expected_indent {
            i += 1;
            continue;
        }

        // `else` and `else if` belong to the caller's `if`
        if line == "else" || line.starts_with("else if ") {
            break;
        }

        // Match arm terminators
        if line.ends_with(" =>") || line == "_ =>" {
            break;
        }

        if let Some(embed) = try_parse_embed(line) {
            nodes.push(Node::Embed(embed));
            i += 1;
            continue;
        }

        // include directive
        if let Some(mut inc) = try_parse_include(line) {
            i += 1;
            let (slot, next_i) = if i < lines.len() && lines[i].0 > expected_indent {
                let child_indent = lines[i].0;
                parse_nodes(lines, i, child_indent)
            } else {
                (vec![], i)
            };
            i = next_i;
            inc.slot = slot;
            nodes.push(Node::Include(inc));
            continue;
        }

        // $: let declaration
        if let Some(decl) = try_parse_let_decl(line) {
            nodes.push(Node::LetDecl(decl));
            i += 1;
            continue;
        }

        // match block
        if let Some(expr) = try_parse_match(line) {
            i += 1;
            let (arms, next_i) = parse_match_arms(lines, i, expected_indent);
            i = next_i;
            nodes.push(Node::Match(MatchBlock { expr, arms }));
            continue;
        }

        // if block
        if try_parse_if(line).is_some() {
            let (node, next_i) = parse_if_node(lines, i, expected_indent);
            i = next_i;
            nodes.push(node);
            continue;
        }

        i += 1;

        // Children: lines with strictly greater indent
        let (children, next_i) = if i < lines.len() && lines[i].0 > expected_indent {
            let child_indent = lines[i].0;
            parse_nodes(lines, i, child_indent)
        } else {
            (vec![], i)
        };
        i = next_i;

        if let Some((pattern, iterator)) = try_parse_for(line) {
            nodes.push(Node::For(ForBlock {
                pattern,
                iterator,
                body: children,
            }));
        } else if line.starts_with('"') {
            let parts = parse_text_template(line);
            nodes.push(Node::Text(parts));
        } else if is_raw_html_expr(line) {
            let inner = &line[2..line.len() - 1];
            nodes.push(Node::RawHtml(inner.trim().to_string()));
        } else if is_raw_expr(line) {
            // Raw expressions — rendered as evaluated text
            nodes.push(Node::RawText(line[1..line.len() - 1].trim().to_string()));
        } else {
            let element = merge_attr_only_children(parse_element_line(line, children));
            nodes.push(Node::Element(element));
        }
    }

    (nodes, i)
}

fn merge_attr_only_children(mut el: Element) -> Element {
    let mut kept = Vec::new();
    for child in std::mem::take(&mut el.children) {
        if let Node::Element(c) = child {
            if is_attr_only_element(&c) {
                hoist_attr_element(&mut el, &c);
                continue;
            }
            kept.push(Node::Element(merge_attr_only_children(c)));
        } else {
            kept.push(child);
        }
    }
    el.children = kept;
    el
}

fn is_attr_only_element(el: &Element) -> bool {
    el.tag.contains('=')
        && el.classes.is_empty()
        && el.children.is_empty()
        && el.bindings.is_empty()
}

fn hoist_attr_element(parent: &mut Element, attr_el: &Element) {
    let fake = parse_element_line(&format!("div {}", attr_el.tag), vec![]);
    parent.bindings.extend(fake.bindings);
    if let Some(id) = fake.id {
        parent.id = Some(id);
    }
    parent.classes.extend(fake.classes);
}

fn parse_if_node(lines: &[(usize, String)], i: usize, expected_indent: usize) -> (Node, usize) {
    let line = &lines[i].1;
    let condition = try_parse_if(line).unwrap_or_default();
    let mut i = i + 1;

    let (then_children, next_i) = if i < lines.len() && lines[i].0 > expected_indent {
        let child_indent = lines[i].0;
        parse_nodes(lines, i, child_indent)
    } else {
        (vec![], i)
    };
    i = next_i;

    let else_children = if i < lines.len() && lines[i].0 == expected_indent {
        let else_line = &lines[i].1;
        if else_line == "else" {
            i += 1;
            if i < lines.len() && lines[i].0 > expected_indent {
                let else_indent = lines[i].0;
                let (else_nodes, next_i) = parse_nodes(lines, i, else_indent);
                i = next_i;
                Some(else_nodes)
            } else {
                Some(vec![])
            }
        } else if else_line.starts_with("else if ") {
            let rewritten = else_line
                .strip_prefix("else ")
                .unwrap_or(else_line)
                .to_string();
            let mut patched = lines.to_vec();
            patched[i].1 = rewritten;
            let (else_if_node, next_i) = parse_if_node(&patched, i, expected_indent);
            i = next_i;
            Some(vec![else_if_node])
        } else {
            None
        }
    } else {
        None
    };

    (
        Node::If(IfBlock {
            condition,
            then_children,
            else_children,
        }),
        i,
    )
}

fn parse_match_arms(
    lines: &[(usize, String)],
    start: usize,
    expected_indent: usize,
) -> (Vec<MatchArm>, usize) {
    let mut arms = Vec::new();
    let mut i = start;

    while i < lines.len() {
        let (indent, line) = &lines[i];
        if *indent < expected_indent {
            break;
        }
        if *indent > expected_indent {
            i += 1;
            continue;
        }

        if let Some(pattern) = try_parse_match_arm(line) {
            i += 1;
            let (body, next_i) = if i < lines.len() && lines[i].0 > expected_indent {
                let body_indent = lines[i].0;
                parse_nodes(lines, i, body_indent)
            } else {
                (vec![], i)
            };
            i = next_i;
            arms.push(MatchArm { pattern, body });
        } else {
            break;
        }
    }

    (arms, i)
}

// ── Include parsing ───────────────────────────────────────────────────────────

fn try_parse_include(line: &str) -> Option<IncludeNode> {
    let rest = line.strip_prefix("include ")?;
    // First token is the path (no spaces in path), rest are props
    let (path, props_str) = match rest.find(' ') {
        Some(pos) => (rest[..pos].trim().to_string(), rest[pos + 1..].trim()),
        None => (rest.trim().to_string(), ""),
    };
    if path.is_empty() {
        return None;
    }
    let props = parse_props(props_str);
    Some(IncludeNode {
        path,
        props,
        slot: vec![],
    })
}

fn try_parse_embed(line: &str) -> Option<EmbedNode> {
    let rest = line.strip_prefix("embed ")?;
    let (src, props_str) = match rest.find(' ') {
        Some(pos) => (rest[..pos].trim().to_string(), rest[pos + 1..].trim()),
        None => (rest.trim().to_string(), ""),
    };
    if src.is_empty() {
        return None;
    }
    let mut props = parse_props(props_str);
    let adapter = take_literal_prop(&mut props, "adapter");
    Some(EmbedNode {
        src,
        adapter,
        props,
    })
}

fn take_literal_prop(props: &mut Vec<(String, String)>, key: &str) -> Option<String> {
    let pos = props.iter().position(|(k, _)| k == key)?;
    let (_, value) = props.remove(pos);
    Some(unquote_expr_string(&value).unwrap_or(value))
}

fn unquote_expr_string(value: &str) -> Option<String> {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        Some(value[1..value.len() - 1].replace("\\\"", "\""))
    } else {
        None
    }
}

fn parse_props(s: &str) -> Vec<(String, String)> {
    let mut props = Vec::new();
    let mut remaining = s.trim();

    while !remaining.is_empty() {
        // Find key= (key is an identifier, no spaces)
        let eq_pos = match remaining.find('=') {
            Some(p) => p,
            None => break,
        };
        let key = remaining[..eq_pos].trim().to_string();
        if key.is_empty() || key.contains(' ') {
            break;
        }
        remaining = remaining[eq_pos + 1..].trim_start();

        // Extract value
        let (expr_str, rest) = extract_prop_value(remaining);
        props.push((key, expr_str));
        remaining = rest.trim_start();
    }

    props
}

/// Extract a prop value token from the start of `s`.
/// Returns `(expr_string, remaining)`.
/// - `"quoted"` → returns the string content wrapped in quotes for the evaluator
/// - `{expr}` → returns the inner expr string
/// - `bare_token` → returns the token as-is (treated as a variable name / literal)
fn extract_prop_value(s: &str) -> (String, &str) {
    if s.is_empty() {
        return (String::new(), s);
    }

    if s.starts_with('"') || s.starts_with('\'') {
        let quote = s.as_bytes()[0];
        let mut i = 1;
        let mut escaped = false;
        while i < s.len() {
            let byte = s.as_bytes()[i];
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == quote {
                let content = &s[1..i];
                let escaped_content = content.replace('\\', "\\\\").replace('"', "\\\"");
                let expr = format!("\"{}\"", escaped_content);
                let rest = if i < s.len() { &s[i + 1..] } else { "" };
                return (expr, rest);
            }
            i += 1;
        }

        let content = &s[1..];
        let escaped_content = content.replace('\\', "\\\\").replace('"', "\\\"");
        let expr = format!("\"{}\"", escaped_content);
        return (expr, "");
    }

    if s.starts_with('{') {
        let mut depth = 0usize;
        for (i, c) in s.char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        let expr = s[1..i].trim().to_string();
                        return (expr, &s[i + 1..]);
                    }
                }
                _ => {}
            }
        }
        return (s.to_string(), "");
    }

    // Bare token: ends at next space
    let end = s.find(' ').unwrap_or(s.len());
    (s[..end].to_string(), &s[end..])
}

// ── Other parsers ─────────────────────────────────────────────────────────────

fn try_parse_if(line: &str) -> Option<String> {
    let rest = line.strip_prefix("if ")?;
    Some(extract_braced(rest.trim()).unwrap_or_else(|| rest.trim().to_string()))
}

fn try_parse_for(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix("for ")?;
    let in_pos = rest.find(" in ")?;
    let pattern = rest[..in_pos].trim().to_string();
    let after_in = rest[in_pos + 4..].trim();
    let iterator = extract_braced(after_in).unwrap_or_else(|| after_in.to_string());
    Some((pattern, iterator))
}

fn try_parse_match(line: &str) -> Option<String> {
    let rest = line.strip_prefix("match ")?;
    Some(extract_braced(rest.trim()).unwrap_or_else(|| rest.trim().to_string()))
}

fn try_parse_match_arm(line: &str) -> Option<String> {
    let pattern = line.strip_suffix(" =>")?;
    let pattern = pattern.trim();
    if pattern.starts_with('{') && pattern.ends_with('}') {
        Some(pattern[1..pattern.len() - 1].trim().to_string())
    } else {
        Some(pattern.to_string())
    }
}

fn try_parse_let_decl(line: &str) -> Option<LetDecl> {
    let (rest, is_default) = if let Some(r) = line.strip_prefix("$: default ") {
        (r, true)
    } else if let Some(r) = line.strip_prefix("$: let ") {
        (r, false)
    } else {
        return None;
    };
    let eq_pos = rest.find('=')?;
    let name = rest[..eq_pos].trim().to_string();
    let expr_str = rest[eq_pos + 1..].trim();
    let expr = extract_braced(expr_str).unwrap_or_else(|| expr_str.to_string());
    Some(LetDecl {
        name,
        expr,
        is_default,
    })
}

fn is_raw_expr(line: &str) -> bool {
    line.starts_with('{') && line.ends_with('}') && {
        let inner = &line[1..line.len() - 1];
        !inner.starts_with('=') && !inner.contains('"')
    }
}

fn is_raw_html_expr(line: &str) -> bool {
    line.starts_with("{=") && line.ends_with('}') && {
        let inner = &line[2..line.len() - 1];
        !inner.contains('"')
    }
}

fn extract_braced(s: &str) -> Option<String> {
    if !s.starts_with('{') {
        return None;
    }
    let mut depth = 0usize;
    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(s[1..i].trim().to_string());
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_element_line(line: &str, children: Vec<Node>) -> Element {
    let tokens = tokenize_line(line);
    if tokens.is_empty() {
        return Element {
            tag: "div".to_string(),
            id: None,
            classes: vec![],
            conditional_classes: vec![],
            event_handlers: vec![],
            bindings: vec![],
            animations: vec![],
            children,
        };
    }

    let tag = tokens[0].clone();
    let mut children = children;
    let inline_text = tokens
        .last()
        .filter(|token| is_inline_text_token(token))
        .cloned();
    let parse_limit = if inline_text.is_some() {
        tokens.len().saturating_sub(1)
    } else {
        tokens.len()
    };
    if let Some(text) = inline_text {
        children.insert(0, Node::Text(parse_text_template(&text)));
    }

    let mut id = None;
    let mut classes = Vec::new();
    let mut conditional_classes = Vec::new();
    let mut event_handlers = Vec::new();
    let mut bindings = Vec::new();
    let mut animations = Vec::new();

    for token in &tokens[1..parse_limit] {
        if let Some(rest) = token.strip_prefix('@') {
            if let Some(eq_pos) = rest.find('=') {
                let event_part = &rest[..eq_pos];
                let handler = strip_optional_quotes(&rest[eq_pos + 1..]).to_string();
                let event = event_part.split('|').next().unwrap_or("").to_string();
                let modifiers: Vec<String> = event_part
                    .split('|')
                    .skip(1)
                    .map(|s| s.to_string())
                    .collect();
                event_handlers.push(EventHandler {
                    event,
                    modifiers,
                    handler,
                });
            }
        } else if let Some(rest) = token.strip_prefix("when:") {
            if let Some((condition, raw_classes)) = parse_when_attribute_suffix(rest) {
                let classes_src = strip_optional_quotes(raw_classes.trim());
                for class in classes_src.split_whitespace() {
                    if class.is_empty() {
                        continue;
                    }
                    conditional_classes.push(ConditionalClass {
                        class: class.to_string(),
                        condition: condition.clone(),
                    });
                }
            }
        } else if let Some(rest) = token.strip_prefix("class:") {
            if let Some(eq_pos) = rest.find('=') {
                let class = rest[..eq_pos].to_string();
                let cond_str = rest[eq_pos + 1..].trim();
                let condition = if cond_str.starts_with('{') && cond_str.ends_with('}') {
                    cond_str[1..cond_str.len() - 1].trim().to_string()
                } else {
                    cond_str.to_string()
                };
                conditional_classes.push(ConditionalClass { class, condition });
            }
        } else if let Some(rest) = token.strip_prefix("bind:") {
            if let Some(eq_pos) = rest.find('=') {
                let prop = rest[..eq_pos].to_string();
                let value = rest[eq_pos + 1..]
                    .trim_matches(|c| c == '{' || c == '}')
                    .to_string();
                bindings.push(Binding { prop, value });
            }
        } else if let Some(rest) = token.strip_prefix("animate:") {
            // animate:property={duration easing} or animate:property={duration easing repeat}
            if let Some(eq_pos) = rest.find('=') {
                let property = rest[..eq_pos].to_string();
                let value_str = rest[eq_pos + 1..]
                    .trim_matches(|c| c == '{' || c == '}')
                    .trim()
                    .to_string();
                let parts: Vec<&str> = value_str.split_whitespace().collect();
                let duration_expr = parts.first().unwrap_or(&"300ms").to_string();
                let easing = parts.get(1).unwrap_or(&"linear").to_string();
                let repeat = parts.get(2).map(|s| *s == "repeat").unwrap_or(false);
                animations.push(AnimationSpec {
                    property,
                    duration_expr,
                    easing,
                    repeat,
                });
            }
        } else if let Some(rest) = token.strip_prefix('#') {
            if !rest.is_empty() {
                id = Some(rest.to_string());
            }
        } else if token.contains('=') {
            // HTML attribute: class="foo bar", type="button", data-action="x", key={expr}
            let eq_pos = token.find('=').unwrap();
            let key = &token[..eq_pos];
            let valid_key = !key.is_empty()
                && key
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_');
            if valid_key {
                let raw = token[eq_pos + 1..].trim();
                let unquoted = if raw.len() >= 2
                    && ((raw.starts_with('"') && raw.ends_with('"'))
                        || (raw.starts_with('\'') && raw.ends_with('\'')))
                {
                    &raw[1..raw.len() - 1]
                } else {
                    raw
                };
                if key == "class" {
                    // class="foo bar" → individual class tokens
                    for cls in unquoted.split_whitespace() {
                        classes.push(cls.to_string());
                    }
                } else if key == "id" {
                    id = Some(unquoted.to_string());
                } else {
                    let expr = if raw.starts_with('{') && raw.ends_with('}') {
                        raw[1..raw.len() - 1].trim().to_string()
                    } else {
                        format!("\"{}\"", unquoted)
                    };
                    bindings.push(Binding {
                        prop: key.to_string(),
                        value: expr,
                    });
                }
            } else {
                classes.push(token.clone());
            }
        } else if matches!(
            token.as_str(),
            "checked"
                | "disabled"
                | "hidden"
                | "required"
                | "readonly"
                | "multiple"
                | "selected"
                | "autofocus"
                | "open"
        ) {
            // Boolean HTML attributes
            bindings.push(Binding {
                prop: token.clone(),
                value: "\"\"".to_string(),
            });
        } else {
            classes.push(token.clone());
        }
    }

    Element {
        tag,
        id,
        classes,
        conditional_classes,
        event_handlers,
        bindings,
        animations,
        children,
    }
}

fn is_inline_text_token(token: &str) -> bool {
    token.len() >= 2 && token.starts_with('"') && token.ends_with('"')
}

fn strip_optional_quotes(s: &str) -> &str {
    if s.len() >= 2
        && ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')))
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Parses the right-hand side of a `when:` attribute (everything after the `when:` prefix).
///
/// Accepts:
/// - `{expr}=quoted-or-bare-classes` — expression may contain `=` (e.g. `{a == b}="x y"`)
/// - `ident=classes` — simple condition (variable name)
///
/// Returns `(condition_source, raw_value)`; the caller should strip optional surrounding
/// quotes from `raw_value` (matching the parser’s `when:` value rules) and split on whitespace
/// for Tailwind tokens.
pub fn parse_when_attribute_suffix(src: &str) -> Option<(String, String)> {
    let s = src.trim();
    if s.is_empty() {
        return None;
    }
    if s.starts_with('{') {
        let mut depth = 0usize;
        for (i, c) in s.char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        let cond = s[1..i].trim().to_string();
                        let mut tail = s[i + 1..].trim_start();
                        tail = tail.strip_prefix('=')?;
                        return Some((cond, tail.trim().to_string()));
                    }
                }
                _ => {}
            }
        }
        return None;
    }
    let eq_pos = s.find('=')?;
    let cond = s[..eq_pos].trim().to_string();
    if cond.is_empty() {
        return None;
    }
    Some((cond, s[eq_pos + 1..].trim().to_string()))
}

fn tokenize_line(line: &str) -> Vec<String> {
    let line = normalize_fullwidth_braces(line);
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut bracket_depth: usize = 0;
    let mut brace_depth: usize = 0;
    let mut in_string = false;
    let mut string_char = ' ';

    for ch in line.chars() {
        match ch {
            '[' if !in_string && brace_depth == 0 => {
                bracket_depth += 1;
                current.push(ch);
            }
            ']' if !in_string && brace_depth == 0 => {
                bracket_depth = bracket_depth.saturating_sub(1);
                current.push(ch);
            }
            '{' if !in_string && bracket_depth == 0 => {
                brace_depth += 1;
                current.push(ch);
            }
            '}' if !in_string && bracket_depth == 0 => {
                brace_depth = brace_depth.saturating_sub(1);
                current.push(ch);
            }
            '\'' | '"' => {
                if in_string && ch == string_char {
                    in_string = false;
                } else if !in_string {
                    in_string = true;
                    string_char = ch;
                }
                current.push(ch);
            }
            ' ' | '\t' if bracket_depth == 0 && brace_depth == 0 && !in_string => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Unescape `\n`, `\r`, `\t`, `\\`, `\"`, and `\'` inside a `.crepus` quoted text segment.
///
/// Unknown escapes keep the backslash (e.g. `\x` → `\x`).
pub fn unescape_crepus_text_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

fn parse_text_template(line: &str) -> Vec<TextPart> {
    let content = if line.starts_with('"') && line.ends_with('"') && line.len() >= 2 {
        &line[1..line.len() - 1]
    } else {
        line
    };

    let mut parts = Vec::new();
    let mut literal = String::new();
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            if !literal.is_empty() {
                parts.push(TextPart::Literal(unescape_crepus_text_literal(&literal)));
                literal.clear();
            }
            let mut expr = String::new();
            let mut depth = 1usize;
            for ec in chars.by_ref() {
                match ec {
                    '{' => {
                        depth += 1;
                        expr.push(ec);
                    }
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        expr.push(ec);
                    }
                    _ => expr.push(ec),
                }
            }
            parts.push(TextPart::Expr(expr.trim().to_string()));
        } else {
            literal.push(ch);
        }
    }

    if !literal.is_empty() {
        parts.push(TextPart::Literal(unescape_crepus_text_literal(&literal)));
    }

    parts
}

// ── JSX / HTML tag syntax parser ──────────────────────────────────────────────
//
// Activated automatically when parse_template() detects JSX mode (first content
// line starts with `<`). Produces the same Node/Element AST as the indentation
// parser, so every backend — GPUI, web, webext — works unchanged.
//
// Supported syntax:
//   <div class="text-white w-full">children</div>   — HTML element
//   <div className="text-white">children</div>       — JSX className alias
//   <img src={url} />                               — self-closing
//   <if condition={score > 50}>...
//     <else>...</else>
//   </if>                                            — conditional
//   <else-if condition={...}>...<else-if>            — else-if chain
//   <for let="item" in={list}>...</for>              — loop
//   <match on={status}>
//     <case pattern="ok"><div>Good</div></case>
//   </match>                                         — match / switch
//   <include src="file.crepus#Card" title={t} />    — include (self-closing = no slot)
//   <include src="file.crepus">...</include>         — include with slot
//   <let name="x" value={42} />                     — let declaration
//   <let-default name="x" value={42} />             — default let
//   $: let x = 42                                   — also works in JSX files

// ── Internal attribute types (private to this module) ─────────────────────────

struct JsxAttr {
    key: String,
    value: JsxAttrValue,
}

enum JsxAttrValue {
    Bool(bool),
    Str(String),
    Expr(String),
}

impl JsxAttr {
    fn as_str(&self) -> Option<&str> {
        if let JsxAttrValue::Str(s) = &self.value {
            Some(s)
        } else {
            None
        }
    }

    /// Returns an evaluable expression string for the attribute value.
    fn as_expr(&self) -> Option<String> {
        match &self.value {
            JsxAttrValue::Expr(e) => Some(e.clone()),
            JsxAttrValue::Str(s) => Some(format!("\"{}\"", s.replace('"', "\\\""))),
            JsxAttrValue::Bool(b) => Some(b.to_string()),
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn normalize_jsx_mapped(s: &str) -> (String, Vec<usize>) {
    let mut norm = String::with_capacity(s.len());
    let mut map: Vec<usize> = Vec::with_capacity(s.len());
    for (orig_b, ch) in s.char_indices() {
        match ch {
            '\u{FF5B}' => {
                norm.push('{');
                map.push(orig_b);
            }
            '\u{FF5D}' => {
                norm.push('}');
                map.push(orig_b);
            }
            c => {
                let mut buf = [0u8; 4];
                let enc = c.encode_utf8(&mut buf);
                for _ in 0..enc.len() {
                    map.push(orig_b);
                }
                norm.push_str(enc);
            }
        }
    }
    debug_assert_eq!(norm.len(), map.len());
    (norm, map)
}

fn map_jsx_offset(map: &[usize], off: usize) -> usize {
    map.get(off).copied().unwrap_or(off)
}

#[inline]
fn jsx_err(norm_root: &str, at: &str, message: impl Into<String>) -> RawParseError {
    RawParseError {
        message: message.into(),
        byte_offset: Some(subslice_byte_offset(norm_root, at)),
    }
}

fn parse_jsx_template(src: &str) -> Result<Vec<Node>, RawParseError> {
    let (norm, map) = normalize_jsx_mapped(src);
    let root = norm.as_str();
    match parse_jsx_nodes(root, root) {
        Ok((nodes, _)) => Ok(nodes),
        Err(mut err) => {
            if let Some(off) = err.byte_offset.take() {
                err.byte_offset = Some(map_jsx_offset(&map, off));
            }
            Err(err)
        }
    }
}

fn parse_jsx_nodes<'a>(
    norm_root: &'a str,
    src: &'a str,
) -> Result<(Vec<Node>, &'a str), RawParseError> {
    let mut nodes = Vec::new();
    let mut rest = src;

    loop {
        let t = rest.trim_start();

        if t.is_empty() {
            rest = t;
            break;
        }
        if t.starts_with("</") || t.starts_with("<else") {
            rest = t;
            break;
        }
        if t.starts_with("$:") {
            let end = t.find('\n').unwrap_or(t.len());
            let line = t[..end].trim();
            rest = &t[end..];
            if let Some(decl) = try_parse_let_decl(line) {
                nodes.push(Node::LetDecl(decl));
            }
            continue;
        }
        if t.starts_with('<') {
            rest = t;
            let (node, next) = parse_jsx_tag(norm_root, rest)?;
            nodes.push(node);
            rest = next;
            continue;
        }
        if t.starts_with("{=") {
            rest = t;
            let (mut expr, next) = jsx_brace_expr(norm_root, rest)?;
            if expr.starts_with('=') {
                expr = expr[1..].trim().to_string();
            }
            nodes.push(Node::RawHtml(expr));
            rest = next;
            continue;
        }
        if t.starts_with('{') {
            rest = t;
            let (expr, next) = jsx_brace_expr(norm_root, rest)?;
            nodes.push(Node::RawText(expr));
            rest = next;
            continue;
        }
        let prev_len = rest.len();
        let (node_opt, next) = jsx_text_node(rest);
        if let Some(node) = node_opt {
            nodes.push(node);
        }
        rest = next;
        if rest.len() == prev_len {
            let skip = rest
                .char_indices()
                .nth(1)
                .map(|(i, _)| i)
                .unwrap_or(rest.len());
            rest = &rest[skip..];
        }
    }

    Ok((nodes, rest))
}

fn parse_jsx_tag<'a>(norm_root: &'a str, src: &'a str) -> Result<(Node, &'a str), RawParseError> {
    let src = src.trim_start();
    let after_lt = &src[1..];
    let name_end = after_lt
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .unwrap_or(after_lt.len());
    let tag = &after_lt[..name_end];
    let rest = after_lt[name_end..].trim_start();

    let (attrs, after_gt, self_closing) = jsx_parse_attrs(norm_root, rest)?;

    match tag {
        "if" => parse_jsx_if(norm_root, attrs, after_gt),
        "else" | "else-if" => Err(jsx_err(
            norm_root,
            src,
            format!("<{tag}> encountered outside <if>"),
        )),
        "for" => parse_jsx_for(norm_root, attrs, after_gt),
        "match" => parse_jsx_match(norm_root, attrs, after_gt),
        "island" | "crepus-island" if self_closing => Ok((jsx_build_embed(attrs), after_gt)),
        "include" if self_closing => Ok((jsx_build_include(attrs, vec![]), after_gt)),
        "include" => {
            let (slot, rest) = parse_jsx_nodes(norm_root, after_gt)?;
            let rest = jsx_close(norm_root, rest, "include")?;
            Ok((jsx_build_include(attrs, slot), rest))
        }
        "let" => Ok((Node::LetDecl(jsx_build_let(attrs, false)), after_gt)),
        "let-default" => Ok((Node::LetDecl(jsx_build_let(attrs, true)), after_gt)),
        _ if self_closing => Ok((
            Node::Element(jsx_build_element(tag, attrs, vec![])),
            after_gt,
        )),
        _ => {
            let (children, rest) = parse_jsx_nodes(norm_root, after_gt)?;
            let rest = jsx_close(norm_root, rest, tag)?;
            Ok((Node::Element(jsx_build_element(tag, attrs, children)), rest))
        }
    }
}

fn parse_jsx_if<'a>(
    norm_root: &'a str,
    attrs: Vec<JsxAttr>,
    children_src: &'a str,
) -> Result<(Node, &'a str), RawParseError> {
    let condition = attrs
        .iter()
        .find(|a| matches!(a.key.as_str(), "condition" | "test" | "cond"))
        .and_then(|a| a.as_expr())
        .unwrap_or_default();

    let (then_children, rest) = parse_jsx_nodes(norm_root, children_src)?;
    let rest = rest.trim_start();

    let (else_children, rest) = if rest.starts_with("<else-if") {
        let after_name = rest.strip_prefix("<else-if").unwrap_or("").trim_start();
        let (ei_attrs, ei_body, _) = jsx_parse_attrs(norm_root, after_name)?;
        let (nested, next) = parse_jsx_if(norm_root, ei_attrs, ei_body)?;
        (Some(vec![nested]), next)
    } else if rest.starts_with("<else") {
        let after_name = rest.strip_prefix("<else").unwrap_or("").trim_start();
        let (_, else_body, self_closing) = jsx_parse_attrs(norm_root, after_name)?;
        if self_closing {
            (Some(vec![]), else_body)
        } else {
            let (else_nodes, after_nodes) = parse_jsx_nodes(norm_root, else_body)?;
            let after_close = jsx_close(norm_root, after_nodes, "else")?;
            (Some(else_nodes), after_close)
        }
    } else {
        (None, rest)
    };

    let rest = jsx_close(norm_root, rest, "if")?;
    Ok((
        Node::If(IfBlock {
            condition,
            then_children,
            else_children,
        }),
        rest,
    ))
}

fn parse_jsx_for<'a>(
    norm_root: &'a str,
    attrs: Vec<JsxAttr>,
    children_src: &'a str,
) -> Result<(Node, &'a str), RawParseError> {
    let pattern = attrs
        .iter()
        .find(|a| matches!(a.key.as_str(), "let" | "var"))
        .and_then(|a| a.as_str())
        .unwrap_or("")
        .to_string();
    let iterator = attrs
        .iter()
        .find(|a| a.key == "in")
        .and_then(|a| a.as_expr())
        .unwrap_or_default();

    let (body, rest) = parse_jsx_nodes(norm_root, children_src)?;
    let rest = jsx_close(norm_root, rest, "for")?;
    Ok((
        Node::For(ForBlock {
            pattern,
            iterator,
            body,
        }),
        rest,
    ))
}

fn parse_jsx_match<'a>(
    norm_root: &'a str,
    attrs: Vec<JsxAttr>,
    children_src: &'a str,
) -> Result<(Node, &'a str), RawParseError> {
    let expr = attrs
        .iter()
        .find(|a| matches!(a.key.as_str(), "on" | "value"))
        .and_then(|a| a.as_expr())
        .unwrap_or_default();

    let mut arms = Vec::new();
    let mut rest = children_src.trim_start();

    while rest.starts_with("<case") {
        let after_name = &rest["<case".len()..].trim_start();
        let (case_attrs, case_body, self_closing) = jsx_parse_attrs(norm_root, after_name)?;
        let pattern = case_attrs
            .iter()
            .find(|a| matches!(a.key.as_str(), "pattern" | "match" | "when"))
            .and_then(|a| match &a.value {
                JsxAttrValue::Str(s) => Some(s.clone()),
                JsxAttrValue::Expr(e) => Some(e.clone()),
                JsxAttrValue::Bool(_) => None,
            })
            .unwrap_or_else(|| "_".to_string());
        let (body, after_body): (Vec<Node>, &str) = if self_closing {
            (vec![], case_body)
        } else {
            let (b, r) = parse_jsx_nodes(norm_root, case_body)?;
            let r = jsx_close(norm_root, r, "case")?;
            (b, r)
        };
        arms.push(MatchArm { pattern, body });
        rest = after_body.trim_start();
    }

    let rest = jsx_close(norm_root, rest, "match")?;
    Ok((Node::Match(MatchBlock { expr, arms }), rest))
}

// ── Node builders ──────────────────────────────────────────────────────────────

fn jsx_build_element(tag: &str, attrs: Vec<JsxAttr>, children: Vec<Node>) -> Element {
    let mut id = None;
    let mut classes = Vec::new();
    let mut conditional_classes = Vec::new();
    let mut event_handlers = Vec::new();
    let mut bindings = Vec::new();
    let mut animations = Vec::new();

    for attr in attrs {
        let key = &attr.key;

        // class / className → split into individual class tokens
        if key == "class" || key == "className" {
            match &attr.value {
                JsxAttrValue::Str(s) => {
                    classes.extend(s.split_whitespace().map(|c| c.to_string()));
                }
                JsxAttrValue::Expr(e) => {
                    // Dynamic expression — keep as a single {expr} class token
                    classes.push(format!("{{{}}}", e));
                }
                JsxAttrValue::Bool(_) => {}
            }
            continue;
        }

        if key == "id" {
            if let Some(value) = attr.as_expr() {
                let trimmed = value.trim();
                if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
                    id = Some(trimmed[1..trimmed.len() - 1].to_string());
                }
            }
            continue;
        }

        // class:name={condition}
        if let Some(class_name) = key.strip_prefix("class:") {
            conditional_classes.push(ConditionalClass {
                class: class_name.to_string(),
                condition: attr.as_expr().unwrap_or_default(),
            });
            continue;
        }

        // when:{condition}="class1 class2 …"
        if let Some(cond_src) = key.strip_prefix("when:") {
            let condition = if cond_src.starts_with('{') && cond_src.ends_with('}') {
                cond_src[1..cond_src.len() - 1].trim().to_string()
            } else {
                cond_src.trim().to_string()
            };
            if condition.is_empty() {
                continue;
            }
            match &attr.value {
                JsxAttrValue::Str(s) => {
                    for class in s.split_whitespace() {
                        if class.is_empty() {
                            continue;
                        }
                        conditional_classes.push(ConditionalClass {
                            class: class.to_string(),
                            condition: condition.clone(),
                        });
                    }
                }
                JsxAttrValue::Expr(_) | JsxAttrValue::Bool(_) => {}
            }
            continue;
        }

        // @event={handler}
        if let Some(event_part) = key.strip_prefix('@') {
            let event = event_part.split('|').next().unwrap_or("").to_string();
            let modifiers = event_part
                .split('|')
                .skip(1)
                .map(|s| s.to_string())
                .collect();
            event_handlers.push(EventHandler {
                event,
                modifiers,
                handler: attr.as_expr().unwrap_or_default(),
            });
            continue;
        }

        // onEvent={handler} — React-style camelCase
        if key.starts_with("on") && key.len() > 2 {
            let rest = &key[2..];
            if rest.starts_with(|c: char| c.is_ascii_uppercase()) {
                let first = rest.chars().next().unwrap();
                let event = format!(
                    "{}{}",
                    first.to_ascii_lowercase(),
                    &rest[first.len_utf8()..]
                );
                event_handlers.push(EventHandler {
                    event,
                    modifiers: vec![],
                    handler: attr.as_expr().unwrap_or_default(),
                });
                continue;
            }
        }

        // animate:property={duration easing}
        if let Some(prop) = key.strip_prefix("animate:") {
            let val = attr.as_expr().unwrap_or_default();
            let parts: Vec<&str> = val.split_whitespace().collect();
            animations.push(AnimationSpec {
                property: prop.to_string(),
                duration_expr: parts.first().unwrap_or(&"300ms").to_string(),
                easing: parts.get(1).unwrap_or(&"linear").to_string(),
                repeat: parts.get(2).map(|s| *s == "repeat").unwrap_or(false),
            });
            continue;
        }

        // bind:prop={expr}
        if let Some(prop) = key.strip_prefix("bind:") {
            bindings.push(Binding {
                prop: prop.to_string(),
                value: attr.as_expr().unwrap_or_default(),
            });
            continue;
        }

        // All other attributes with values → binding
        if let Some(value) = attr.as_expr() {
            bindings.push(Binding {
                prop: key.clone(),
                value,
            });
        }
    }

    Element {
        tag: tag.to_string(),
        id,
        classes,
        conditional_classes,
        event_handlers,
        bindings,
        animations,
        children,
    }
}

fn jsx_build_include(attrs: Vec<JsxAttr>, slot: Vec<Node>) -> Node {
    let path = attrs
        .iter()
        .find(|a| matches!(a.key.as_str(), "src" | "path"))
        .and_then(|a| a.as_str())
        .unwrap_or("")
        .to_string();
    let props = attrs
        .iter()
        .filter(|a| !matches!(a.key.as_str(), "src" | "path"))
        .filter_map(|a| a.as_expr().map(|v| (a.key.clone(), v)))
        .collect();
    Node::Include(IncludeNode { path, props, slot })
}

fn jsx_build_embed(attrs: Vec<JsxAttr>) -> Node {
    let src = attrs
        .iter()
        .find(|a| matches!(a.key.as_str(), "src" | "path"))
        .and_then(|a| a.as_str())
        .unwrap_or("")
        .to_string();
    let adapter = attrs
        .iter()
        .find(|a| a.key == "adapter")
        .and_then(|a| a.as_str())
        .map(|s| s.to_string());
    let props = attrs
        .iter()
        .filter(|a| !matches!(a.key.as_str(), "src" | "path" | "adapter"))
        .filter_map(|a| a.as_expr().map(|v| (a.key.clone(), v)))
        .collect();
    Node::Embed(EmbedNode {
        src,
        adapter,
        props,
    })
}

fn jsx_build_let(attrs: Vec<JsxAttr>, is_default: bool) -> LetDecl {
    let name = attrs
        .iter()
        .find(|a| a.key == "name")
        .and_then(|a| a.as_str())
        .unwrap_or("")
        .to_string();
    let expr = attrs
        .iter()
        .find(|a| a.key == "value")
        .and_then(|a| a.as_expr())
        .unwrap_or_default();
    LetDecl {
        name,
        expr,
        is_default,
    }
}

// ── Low-level helpers ──────────────────────────────────────────────────────────

fn jsx_parse_attrs<'a>(
    norm_root: &'a str,
    src: &'a str,
) -> Result<(Vec<JsxAttr>, &'a str, bool), RawParseError> {
    let mut attrs = Vec::new();
    let mut rest = src.trim_start();
    let mut self_closing = false;

    loop {
        rest = rest.trim_start();
        if rest.is_empty() {
            return Err(jsx_err(norm_root, rest, "unclosed JSX tag"));
        }
        if rest.starts_with("/>") {
            self_closing = true;
            rest = &rest[2..];
            break;
        }
        if rest.starts_with('>') {
            rest = &rest[1..];
            break;
        }

        let key_end = rest
            .find(|c: char| c.is_whitespace() || c == '=' || c == '>' || c == '/')
            .unwrap_or(rest.len());
        if key_end == 0 {
            rest = &rest[1..];
            continue;
        }
        let key = rest[..key_end].to_string();
        rest = rest[key_end..].trim_start();

        if rest.starts_with('=') {
            rest = rest[1..].trim_start();
            let (value, next) = jsx_attr_value(norm_root, rest)?;
            attrs.push(JsxAttr { key, value });
            rest = next;
        } else {
            attrs.push(JsxAttr {
                key,
                value: JsxAttrValue::Bool(true),
            });
        }
    }

    Ok((attrs, rest, self_closing))
}

fn jsx_attr_value<'a>(
    norm_root: &'a str,
    src: &'a str,
) -> Result<(JsxAttrValue, &'a str), RawParseError> {
    if src.starts_with('"') {
        let mut i = 1;
        let bytes = src.as_bytes();
        while i < bytes.len() {
            match bytes[i] {
                b'\\' => i += 2,
                b'"' => {
                    let content = src[1..i].replace("\\\"", "\"");
                    return Ok((JsxAttrValue::Str(content), &src[i + 1..]));
                }
                _ => i += 1,
            }
        }
        let inner = src.strip_prefix('"').unwrap_or("");
        Ok((JsxAttrValue::Str(inner.replace("\\\"", "\"")), ""))
    } else if src.starts_with('\'') {
        let inner = src.strip_prefix('\'').unwrap_or(src);
        let end = inner.find('\'').unwrap_or(inner.len());
        Ok((
            JsxAttrValue::Str(inner[..end].to_string()),
            &inner[end + 1..],
        ))
    } else if src.starts_with('{') {
        let (expr, rest) = jsx_brace_expr(norm_root, src)?;
        Ok((JsxAttrValue::Expr(expr), rest))
    } else {
        let end = src
            .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
            .unwrap_or(src.len());
        let val = &src[..end];
        let value = match val {
            "true" => JsxAttrValue::Bool(true),
            "false" => JsxAttrValue::Bool(false),
            other => JsxAttrValue::Str(other.to_string()),
        };
        Ok((value, &src[end..]))
    }
}

fn jsx_brace_expr<'a>(
    norm_root: &'a str,
    src: &'a str,
) -> Result<(String, &'a str), RawParseError> {
    let src = src.trim_start();
    if !src.starts_with('{') {
        return Err(jsx_err(
            norm_root,
            src,
            format!("expected '{{', got: {}", &src[..src.len().min(10)]),
        ));
    }
    let mut depth = 0usize;
    for (i, c) in src.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let expr = src[1..i].trim().to_string();
                    return Ok((expr, &src[i + 1..]));
                }
            }
            _ => {}
        }
    }
    Err(jsx_err(norm_root, src, "unclosed '{' in JSX expression"))
}

/// Consume text content up to the next `<` tag, parsing `{expr}` interpolations.
fn jsx_text_node(src: &str) -> (Option<Node>, &str) {
    let end = src.find('<').unwrap_or(src.len());
    if end == 0 {
        return (None, src);
    }
    let text = &src[..end];
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return (None, &src[end..]);
    }
    let parts = parse_text_template(trimmed);
    (Some(Node::Text(parts)), &src[end..])
}

fn jsx_close<'a>(norm_root: &str, src: &'a str, tag: &str) -> Result<&'a str, RawParseError> {
    let src = src.trim_start();
    let prefix = format!("</{}", tag);
    if let Some(rest) = src.strip_prefix(&prefix) {
        let rest = rest.trim_start();
        if let Some(rest) = rest.strip_prefix('>') {
            return Ok(rest);
        }
    }
    Err(jsx_err(
        norm_root,
        src,
        format!("expected </{}>, got: {}", tag, &src[..src.len().min(40)]),
    ))
}

fn normalize_fullwidth_braces(s: &str) -> String {
    s.replace('\u{FF5B}', "{").replace('\u{FF5D}', "}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_structural_indent_basic() {
        assert_eq!(strip_structural_indent("    hello", 4), "hello");
        assert_eq!(strip_structural_indent("    hello", 2), "  hello");
        assert_eq!(strip_structural_indent("    hello", 0), "    hello");
        assert_eq!(strip_structural_indent("hi", 4), "hi");
        assert_eq!(strip_structural_indent("", 4), "");
    }

    fn text_from_node(node: &Node) -> String {
        let Node::Text(parts) = node else {
            panic!("expected Text node, got: {node:?}")
        };
        parts
            .iter()
            .map(|p| match p {
                crate::ast::TextPart::Literal(s) => s.clone(),
                crate::ast::TextPart::Expr(e) => format!("{{{e}}}"),
            })
            .collect()
    }

    #[test]
    fn parse_when_attribute_suffix_braced_condition_with_equals() {
        let (c, v) = parse_when_attribute_suffix(r#"{a == b}="x y z""#).unwrap();
        assert_eq!(c, "a == b");
        assert_eq!(v, r#""x y z""#);
    }

    #[test]
    fn parse_when_attribute_suffix_simple_ident() {
        let (c, v) = parse_when_attribute_suffix(r#"active=bg-red-500"#).unwrap();
        assert_eq!(c, "active");
        assert_eq!(v, "bg-red-500");
    }

    #[test]
    fn when_attribute_indent_expands_to_conditional_classes() {
        let nodes = parse_template(
            r#"div base when:{flag}="a b" when:{!flag}="c"
  "hi""#,
        )
        .unwrap();
        let Node::Element(el) = &nodes[0] else {
            panic!("expected element");
        };
        assert_eq!(el.classes, vec!["base".to_string()]);
        assert_eq!(el.conditional_classes.len(), 3);
        assert_eq!(el.conditional_classes[0].class, "a");
        assert_eq!(el.conditional_classes[0].condition, "flag");
        assert_eq!(el.conditional_classes[1].class, "b");
        assert_eq!(el.conditional_classes[2].class, "c");
        assert_eq!(el.conditional_classes[2].condition, "!flag");
    }

    #[test]
    fn when_attribute_jsx_expands_to_conditional_classes() {
        let nodes =
            parse_template(r#"<div class="base" when:{active}="font-bold text-white"></div>"#)
                .unwrap();
        let Node::Element(el) = &nodes[0] else {
            panic!("expected element");
        };
        assert_eq!(el.classes, vec!["base".to_string()]);
        assert_eq!(el.conditional_classes.len(), 2);
        assert_eq!(el.conditional_classes[0].class, "font-bold");
        assert_eq!(el.conditional_classes[0].condition, "active");
        assert_eq!(el.conditional_classes[1].class, "text-white");
    }

    #[test]
    fn jsx_class_and_class_name_are_class_tokens() {
        let nodes =
            parse_template(r#"<div class="flex gap-4" className="text-white bg-zinc-950"></div>"#)
                .unwrap();
        let Node::Element(el) = &nodes[0] else {
            panic!("expected element");
        };
        assert_eq!(
            el.classes,
            vec![
                "flex".to_string(),
                "gap-4".to_string(),
                "text-white".to_string(),
                "bg-zinc-950".to_string()
            ]
        );
    }

    #[test]
    fn embed_indent_parses_src_adapter_and_props() {
        let nodes =
            parse_template(r#"embed ./islands/wave.ts adapter="module" title="Wave" count={n}"#)
                .unwrap();
        let Node::Embed(embed) = &nodes[0] else {
            panic!("expected embed");
        };
        assert_eq!(embed.src, "./islands/wave.ts");
        assert_eq!(embed.adapter.as_deref(), Some("module"));
        assert_eq!(embed.props.len(), 2);
        assert_eq!(
            embed.props[0],
            ("title".to_string(), "\"Wave\"".to_string())
        );
        assert_eq!(embed.props[1], ("count".to_string(), "n".to_string()));
    }

    #[test]
    fn island_jsx_parses_src_adapter_and_props() {
        let nodes = parse_template(
            r#"<island src="./islands/wave.ts" adapter="module" title="Wave" count={n} />"#,
        )
        .unwrap();
        let Node::Embed(embed) = &nodes[0] else {
            panic!("expected embed");
        };
        assert_eq!(embed.src, "./islands/wave.ts");
        assert_eq!(embed.adapter.as_deref(), Some("module"));
        assert_eq!(embed.props.len(), 2);
        assert_eq!(
            embed.props[0],
            ("title".to_string(), "\"Wave\"".to_string())
        );
        assert_eq!(embed.props[1], ("count".to_string(), "n".to_string()));
    }

    #[test]
    fn multiline_string_preserves_inner_indent() {
        // `"` line is at indent=4 (4 leading spaces). Continuation "  indented line"
        // has 2 spaces → after stripping 4 structural spaces it becomes "indented line"
        // (only 4 available, so strip up to the string length → "indented line").
        // Actually strip min(4, available) → strips 2 available → "indented line".
        let template = "pre\n  \"line one\n  indented line\n    more indented\"";
        let nodes = parse_template(template).unwrap();
        let Node::Element(el) = &nodes[0] else {
            panic!("expected element")
        };
        let text = text_from_node(&el.children[0]);
        assert!(text.contains("indented line"), "got: {text:?}");
        // "  indented line" with structural indent 2 → strips 2 → "indented line" (0 extra spaces)
        let cont_indent = text
            .lines()
            .nth(1)
            .map(|l| l.len() - l.trim_start().len())
            .unwrap_or(0);
        assert_eq!(
            cont_indent, 0,
            "continuation at same level should have 0 leading spaces, got: {text:?}"
        );
        // "    more indented" with structural indent 2 → strips 2 → "  more indented" (2 extra)
        let deep_indent = text
            .lines()
            .nth(2)
            .map(|l| l.len() - l.trim_start().len())
            .unwrap_or(0);
        assert_eq!(
            deep_indent, 2,
            "deeper line should have 2 preserved leading spaces, got: {text:?}"
        );
    }

    #[test]
    fn multiline_string_preserves_extra_indent() {
        // structural indent of the `"` line is 2; continuation "    extra" has 4 spaces.
        // After stripping 2 structural spaces: "  extra" (2 extra spaces remain).
        let template = "pre\n  \"first\n    extra\"";
        let nodes = parse_template(template).unwrap();
        let Node::Element(el) = &nodes[0] else {
            panic!("expected element")
        };
        let text = text_from_node(&el.children[0]);
        assert!(
            text.contains("  extra"),
            "extra indent should be preserved, got: {text:?}"
        );
    }

    #[test]
    fn quoted_line_unescapes_backslash_sequences() {
        let nodes = parse_template("pre\n  \"a\\nb\\tc\\\\d\"").unwrap();
        let Node::Element(el) = &nodes[0] else {
            panic!("expected element");
        };
        let text = text_from_node(&el.children[0]);
        assert_eq!(text, "a\nb\tc\\d");
    }

    #[test]
    fn unescape_crepus_text_literal_accepts_common_escapes() {
        assert_eq!(
            unescape_crepus_text_literal(r#"line\n\t\"quote""#),
            "line\n\t\"quote\""
        );
    }

    #[test]
    fn parse_component_file_frontmatter_with_dotted_defaults() {
        let source = r#"+++
[Panel.defaults]
warning = "sandboxed"
show_source = true
+++

--- Panel
div
  "{warning}""#;
        let file = parse_component_file(source).expect("frontmatter should parse");
        let panel = file.components.get("Panel").expect("Panel component");
        assert_eq!(
            panel.meta.defaults.get("warning").map(|e| e.to_string()),
            Some("\"sandboxed\"".to_string())
        );
    }
}
