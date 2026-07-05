//! HTML void elements — must not emit a closing tag (e.g. `<br></br>` doubles line breaks).

const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

pub fn is_void_html_tag(tag: &str) -> bool {
    VOID_TAGS.contains(&tag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn br_is_void() {
        assert!(is_void_html_tag("br"));
        assert!(!is_void_html_tag("pre"));
    }
}
