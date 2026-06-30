//! Built-in Markdown → HTML docs generator for `crepus web build`.
//!
//! Auto-detects `docs/` or `public/` relative to the site directory.
//! No external binary or config needed — framework handles it like Svelte/Next.js.

use std::io;
use std::path::Path;

use pulldown_cmark::{html, Options, Parser};

use crate::web::ThemeCss;

pub(crate) fn generate_docs(
    src_dir: &Path,
    out_dir: &Path,
    theme: &ThemeCss,
    site_name: &str,
) -> io::Result<()> {
    if !src_dir.is_dir() {
        return Ok(());
    }

    let mut pages: Vec<(String, String)> = Vec::new();

    for entry in std::fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(String::from)
            .unwrap_or_default();
        let content = std::fs::read_to_string(&path)?;
        let (title, body_html) = render_md(&content);
        pages.push((stem.clone(), title.clone()));
        let nav = render_nav(&pages, &stem);
        let html = render_shell(&body_html, &title, &nav, theme, site_name);
        std::fs::create_dir_all(out_dir)?;
        std::fs::write(out_dir.join(format!("{stem}.html")), html)?;
    }

    if pages.is_empty() {
        return Ok(());
    }

    // Search index
    let idx: Vec<serde_json::Value> = pages
        .iter()
        .map(|(stem, title)| serde_json::json!({"path": format!("{stem}.html"), "title": title}))
        .collect();
    std::fs::write(
        out_dir.join("docs-search-index.json"),
        serde_json::to_string_pretty(&serde_json::json!(idx))?,
    )?;

    Ok(())
}

fn render_md(md: &str) -> (String, String) {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    opts.insert(Options::ENABLE_GFM);

    let mut title = String::new();
    for line in md.lines() {
        if title.is_empty() && line.starts_with("# ") {
            title = line[2..].trim().to_string();
        }
    }
    let mut body = String::new();
    html::push_html(&mut body, Parser::new_ext(md, opts));
    if title.is_empty() {
        title = "Documentation".into();
    }
    (title, body)
}

fn render_nav(pages: &[(String, String)], current: &str) -> String {
    let mut items = String::new();
    for (stem, page_title) in pages {
        let active = if stem == current {
            " class=\"active\""
        } else {
            ""
        };
        items.push_str(&format!(
            "<li><a href=\"{stem}.html\"{active}>{}</a></li>",
            esc(page_title)
        ));
    }
    format!("<nav aria-label=\"Documentation\"><ul class=\"doc-nav\">{items}</ul></nav>")
}

fn render_shell(body: &str, title: &str, nav: &str, theme: &ThemeCss, site_name: &str) -> String {
    let ttl = esc(title);
    let site = esc(site_name);
    let a = esc(&theme.accent);
    let s = esc(&theme.surface);
    let t = esc(&theme.text);
    let m = esc(&theme.muted);
    let b = esc(&theme.border);

    format!("<!DOCTYPE html>
<html lang=\"en\">
<head>
<meta charset=\"utf-8\">
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
<title>{ttl} — {site}</title>
<style>
  *{{box-sizing:border-box}}
  body{{margin:0;min-height:100vh;background:{s};color:{t};font-family:Inter,system-ui,sans-serif;-webkit-font-smoothing:antialiased;line-height:1.6}}
  a{{color:color-mix(in srgb,{t} 88%,transparent);text-decoration:none}}
  a:hover{{color:{t};text-decoration:underline;text-underline-offset:3px}}
  .doc-shell{{display:grid;grid-template-columns:minmax(220px,280px) 1fr;min-height:100vh}}
  aside{{position:sticky;top:0;height:100vh;overflow-y:auto;padding:1.5rem;border-right:1px solid {b};background:color-mix(in srgb,{s} 92%,white 8%)}}
  .brand{{font-weight:700;font-size:.95rem;color:{t};display:block;margin-bottom:1rem}}
  .brand:hover{{opacity:.85;text-decoration:none}}
  .doc-nav{{list-style:none;padding:0;margin:0;font-size:.875rem}}
  .doc-nav li{{margin:.35rem 0}}
  .doc-nav a{{color:{m}}}
  .doc-nav a.active{{color:{a};font-weight:600}}
  .doc-nav a:hover{{color:{t}}}
  article{{max-width:45rem;margin:0 auto;padding:1.5rem 2rem 3rem}}
  article h1,h2,h3{{color:{t}}}
  article h1{{font-size:1.75rem}}
  article h2{{font-size:1.3rem;border-bottom:1px solid {b};padding-bottom:.35rem}}
  article p,li{{color:{m}}}
  article code{{font-family:monospace;font-size:.85em;background:color-mix(in srgb,{t} 10%,transparent);padding:.12rem .35rem;border-radius:4px}}
  article pre{{background:color-mix(in srgb,{t} 6%,{s});border:1px solid {b};border-radius:8px;padding:1rem;overflow-x:auto;font-size:.82rem}}
  article pre code{{background:none;padding:0}}
  article blockquote{{margin:1rem 0;padding:.5rem 1rem;border-left:3px solid {a};background:color-mix(in srgb,{a} 8%,transparent);border-radius:0 8px 8px 0}}
  article table{{width:100%;border-collapse:collapse;margin:1rem 0;font-size:.875rem}}
  article th,td{{padding:.5rem .75rem;border:1px solid {b}}}
  article th{{background:color-mix(in srgb,{t} 6%,transparent);color:{t}}}
  @media(max-width:640px){{.doc-shell{{grid-template-columns:1fr}}aside{{position:static;height:auto;border-right:none;border-bottom:1px solid {b}}}}}
</style>
</head>
<body>
<div class=\"doc-shell\">
<aside><a class=\"brand\" href=\"../index.html\">{site}</a>{nav}</aside>
<article>{body}</article>
</div>
</body>
</html>")
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
