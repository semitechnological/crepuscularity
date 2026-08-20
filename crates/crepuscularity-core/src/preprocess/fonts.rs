use std::collections::HashSet;

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

pub(crate) fn strip_google_font_pragmas(lines: &[&str]) -> (usize, Vec<String>) {
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
    (i, google_fonts)
}

fn parse_google_font_pragma(line: &str) -> Option<Vec<String>> {
    let t = line.trim();
    let (plural, after_kw) = t
        .strip_prefix("google-fonts")
        .map(|r| (true, r.trim_start()))
        .or_else(|| {
            t.strip_prefix("google-font")
                .map(|r| (false, r.trim_start()))
        })?;

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
        return None;
    }

    Some(vec![rest.to_string()])
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_unique_font_families() {
        let cases: Vec<(Vec<&str>, Vec<&str>)> = vec![
            (vec!["Inter", "Roboto", "Inter"], vec!["Inter", "Roboto"]),
            (
                vec!["INTER", "roboto", "inter", "ROBOTO"],
                vec!["INTER", "roboto"],
            ),
            (
                vec!["  Inter  ", " Roboto ", "Inter"],
                vec!["Inter", "Roboto"],
            ),
            (vec!["", "   ", "Inter"], vec!["Inter"]),
            (vec![], vec![]),
        ];

        for (input, expected) in cases {
            let input_strings: Vec<String> = input.into_iter().map(String::from).collect();
            let expected_strings: Vec<String> = expected.into_iter().map(String::from).collect();
            assert_eq!(merge_unique_font_families(input_strings), expected_strings);
        }
    }
}
