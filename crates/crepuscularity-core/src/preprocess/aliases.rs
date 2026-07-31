use std::collections::HashMap;

use crate::ast::{ConditionalClass, Node};

pub(crate) fn strip_class_aliases(
    lines: &[&str],
    start: usize,
    mut end: usize,
) -> (usize, HashMap<String, String>) {
    let mut alias_lines: Vec<(String, String)> = Vec::new();
    loop {
        if end <= start {
            break;
        }
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
        if end >= 2 {
            let name_line = lines[end - 2].trim();
            let exp_line = lines[end - 1];
            if name_line.starts_with('.') && !name_line.contains(' ') {
                let name_indent = lines[end - 2].len() - lines[end - 2].trim_start().len();
                let exp_indent = exp_line.len() - exp_line.trim_start().len();
                if exp_indent > name_indent && !exp_line.trim().is_empty() {
                    let alias_name = name_line
                        .strip_prefix('.')
                        .unwrap_or(name_line)
                        .trim()
                        .to_string();
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

    (end, class_aliases)
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
/// Returns owned strings; in the common no-alias case the input is moved
/// into the result without re-allocation.
pub fn expand_class_token(token: &str, aliases: &HashMap<String, String>) -> Vec<String> {
    if let Some(exp) = aliases.get(token) {
        return exp.split_whitespace().map(|s| s.to_string()).collect();
    }
    vec![token.to_string()]
}

/// Like `expand_class_token` but takes ownership of `token` to avoid
/// re-allocating in the common no-alias case.
pub fn expand_class_token_owned(token: String, aliases: &HashMap<String, String>) -> Vec<String> {
    if let Some(exp) = aliases.get(token.as_str()) {
        return exp.split_whitespace().map(|s| s.to_string()).collect();
    }
    vec![token]
}

/// Recursively expand class shortcuts on every element.
pub fn expand_class_aliases_in_nodes(nodes: &mut [Node], aliases: &HashMap<String, String>) {
    if aliases.is_empty() {
        return;
    }
    for node in nodes.iter_mut() {
        match node {
            Node::Element(el) => {
                let old_classes = std::mem::take(&mut el.classes);
                el.classes.reserve(old_classes.len());
                for c in old_classes {
                    if let Some(exp) = aliases.get(c.as_str()) {
                        el.classes
                            .extend(exp.split_whitespace().map(|s| s.to_string()));
                    } else {
                        el.classes.push(c);
                    }
                }

                let old_cc = std::mem::take(&mut el.conditional_classes);
                let mut out_cc = Vec::with_capacity(old_cc.len());
                for cc in old_cc {
                    if let Some(exp) = aliases.get(cc.class.as_str()) {
                        let mut iter = exp.split_whitespace();
                        if let Some(first) = iter.next() {
                            let mut prev = first;
                            for c in iter {
                                out_cc.push(ConditionalClass {
                                    class: prev.to_string(),
                                    condition: cc.condition.clone(),
                                });
                                prev = c;
                            }
                            out_cc.push(ConditionalClass {
                                class: prev.to_string(),
                                condition: cc.condition,
                            });
                        }
                    } else {
                        out_cc.push(cc);
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
