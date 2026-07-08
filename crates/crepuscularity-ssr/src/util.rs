//! Shared utilities for crepuscularity-ssr.
//!
//! # ponytail: extracted to avoid duplicate escape_html_error in handler.rs and router.rs.
//!   If this grows beyond 3 functions, move to crepuscularity-web.

/// Escape HTML special characters in error messages before injecting into HTML.
pub fn escape_html_error(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
