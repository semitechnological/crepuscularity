use crate::ast::{Node, TextPart};

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
