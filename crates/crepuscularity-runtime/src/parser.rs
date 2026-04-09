/// Runtime parser for the crepuscularity template DSL.
/// Mirrors the compile-time proc-macro parser but operates on strings at runtime.
use std::collections::HashMap;

use crepuscularity_core::parser::{parse_when_attribute_suffix, unescape_crepus_text_literal};

use crate::ast::*;

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
pub fn parse_component_file(content: &str) -> Result<ComponentFile, String> {
    let (frontmatter_str, body) = split_frontmatter(content);

    // Parse TOML for metadata / defaults.
    let mut meta_map: HashMap<String, ComponentMeta> = HashMap::new();
    if let Some(toml_str) = frontmatter_str {
        let value: toml::Value = toml_str
            .parse()
            .map_err(|e| format!("TOML parse error in frontmatter: {e}"))?;

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

    let sections = split_sections(body);
    let mut components = HashMap::new();

    for (name, section_content) in sections {
        let nodes = parse_template(&section_content)?;
        let meta = meta_map.remove(&name).unwrap_or_default();
        components.insert(name, ComponentDef { nodes, meta });
    }

    Ok(ComponentFile { components })
}

/// Split `+++...+++` TOML frontmatter from the rest of the file.
fn split_frontmatter(content: &str) -> (Option<&str>, &str) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("+++") {
        return (None, content);
    }
    let after_open = &trimmed[3..];
    if let Some(close_pos) = after_open.find("\n+++") {
        let toml_str = &after_open[..close_pos];
        let rest = &after_open[close_pos + 4..]; // skip "\n+++"
        (Some(toml_str.trim()), rest)
    } else {
        (None, content)
    }
}

/// Split body into named sections on `--- ComponentName` lines.
fn split_sections(body: &str) -> Vec<(String, String)> {
    let mut sections: Vec<(String, String)> = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_lines: Vec<&str> = Vec::new();

    for line in body.lines() {
        if let Some(name) = line.trim().strip_prefix("--- ") {
            if let Some(prev) = current_name.take() {
                sections.push((prev, current_lines.join("\n")));
                current_lines.clear();
            }
            current_name = Some(name.trim().to_string());
        } else if current_name.is_some() {
            current_lines.push(line);
        }
    }

    if let Some(name) = current_name {
        sections.push((name, current_lines.join("\n")));
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

pub fn parse_template(template: &str) -> Result<Vec<Node>, String> {
    let lines = collect_lines(template);
    let (nodes, _) = parse_nodes(&lines, 0, 0);
    Ok(nodes)
}

fn collect_lines(template: &str) -> Vec<(usize, String)> {
    let raw: Vec<(usize, String)> = template
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();
            (indent, trimmed.to_string())
        })
        .filter(|(_, line)| !line.is_empty() && !line.starts_with('#'))
        .collect();

    // Normalize indentation so root elements always start at column 0.
    let min_indent = raw.iter().map(|(i, _)| *i).min().unwrap_or(0);
    if min_indent == 0 {
        return raw;
    }
    raw.into_iter().map(|(i, l)| (i - min_indent, l)).collect()
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
        } else if is_raw_expr(line) {
            // Raw expressions — rendered as evaluated text
            nodes.push(Node::RawText(line[1..line.len() - 1].trim().to_string()));
        } else {
            let element = parse_element_line(line, children);
            nodes.push(Node::Element(element));
        }
    }

    (nodes, i)
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
                let escaped_content = content
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"");
                let expr = format!("\"{}\"", escaped_content);
                let rest = if i + 1 <= s.len() { &s[i + 1..] } else { "" };
                return (expr, rest);
            }
            i += 1;
        }

        let content = &s[1..];
        let escaped_content = content
            .replace('\\', "\\\\")
            .replace('"', "\\\"");
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

fn tokenize_line(line: &str) -> Vec<String> {
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
                if bracket_depth > 0 {
                    bracket_depth -= 1;
                }
                current.push(ch);
            }
            '{' if !in_string && bracket_depth == 0 => {
                brace_depth += 1;
                current.push(ch);
            }
            '}' if !in_string && bracket_depth == 0 => {
                if brace_depth > 0 {
                    brace_depth -= 1;
                }
                current.push(ch);
            }
            '\'' | '"' if bracket_depth > 0 || brace_depth > 0 => {
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
