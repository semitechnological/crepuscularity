//! Emit static HTML from repository `docs/*.md` when running `crepus web build`.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use pulldown_cmark::{html, Options, Parser};
use serde::Deserialize;
use serde_json::json;

const DOCS_SEARCH_JS: &str = include_str!("docs_search.js");
const DOC_SHELL_CSS: &str = r#"
    * { box-sizing: border-box; }
    body {
      margin: 0;
      min-height: 100vh;
      overflow-x: hidden;
      background: var(--surface);
      color: var(--text);
      font-family: Inter, system-ui, sans-serif;
      -webkit-font-smoothing: antialiased;
      line-height: 1.6;
    }
    a { color: color-mix(in srgb, var(--text) 88%, transparent); text-decoration: none; }
    a:hover { color: var(--text); text-decoration: underline; text-underline-offset: 3px; }
    .doc-shell {
      display: grid;
      grid-template-columns: minmax(220px, 280px) 1fr;
      min-height: 100vh;
      align-items: stretch;
    }
    .doc-shell > * {
      min-width: 0;
    }
    aside {
      position: sticky;
      top: 0;
      align-self: stretch;
      height: 100vh;
      min-height: 100vh;
      overflow-y: auto;
      padding: 1.5rem 1.25rem 1.35rem;
      border-right: 1px solid var(--border);
      background: color-mix(in srgb, var(--surface) 92%, white 8%);
      display: flex;
      flex-direction: column;
      gap: 1rem;
      min-width: 0;
    }
    .brand {
      font-weight: 700;
      font-size: 0.95rem;
      letter-spacing: -0.02em;
      margin-bottom: 0.25rem;
      display: block;
      color: var(--text);
    }
    .brand:hover { text-decoration: none; color: var(--text); opacity: 0.85; }
    .doc-search-trigger {
      width: 100%;
      text-align: left;
      font: inherit;
      font-size: 0.8125rem;
      padding: 0.55rem 0.65rem;
      border-radius: 8px;
      border: 1px solid var(--border);
      background: color-mix(in srgb, var(--surface) 75%, white 25%);
      color: var(--muted);
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 0.5rem;
    }
    .doc-search-trigger:hover {
      border-color: color-mix(in srgb, var(--text) 25%, var(--border));
      color: var(--text);
    }
    .doc-search-trigger kbd {
      font-family: ui-monospace, monospace;
      font-size: 0.7rem;
      padding: 0.12rem 0.35rem;
      border-radius: 4px;
      border: 1px solid var(--border);
      background: var(--surface);
      color: var(--muted);
    }
    .doc-nav {
      list-style: none;
      padding: 0;
      margin: 0;
      font-size: 0.875rem;
    }
    .doc-nav li { margin: 0.35rem 0; }
    .doc-nav a {
      color: var(--muted);
      display: block;
      width: 100%;
      overflow-wrap: anywhere;
      word-break: break-word;
    }
    .doc-nav a:hover { color: var(--text); }
    .doc-nav a.active { color: var(--text); font-weight: 600; }
    .doc-toc {
      margin: 0.25rem 0 0;
      padding-top: 0.75rem;
      border-top: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
    }
    .doc-toc-title {
      font-size: 0.68rem;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: var(--muted);
      margin: 0 0 0.45rem;
    }
    .doc-toc ul {
      list-style: none;
      padding: 0;
      margin: 0;
      font-size: 0.8125rem;
      line-height: 1.35;
    }
    .doc-toc li { margin: 0.3rem 0; }
    .doc-toc a {
      color: var(--muted);
      display: block;
      width: 100%;
      text-decoration: none;
      overflow-wrap: anywhere;
      word-break: break-word;
    }
    .doc-toc a:hover { color: var(--text); text-decoration: underline; text-underline-offset: 2px; }
    .doc-toc li.doc-toc-h3 {
      padding-left: 0.75rem;
      font-size: 0.78rem;
      opacity: 0.95;
    }
    .doc-main {
      padding: clamp(1.5rem, 3vw, 2.5rem) clamp(1rem, 3vw, 2.75rem) clamp(3.25rem, 5vw, 5rem);
      max-width: 52rem;
      min-width: 0;
    }
    .doc-main.doc-main--wide {
      max-width: 74rem;
    }
    .prose h1 { font-size: 2rem; font-weight: 700; margin: 0 0 1rem; letter-spacing: -0.03em; line-height: 1.2; scroll-margin-top: 1.25rem; }
    .prose h2 { font-size: 1.35rem; font-weight: 600; margin: 2rem 0 0.75rem; letter-spacing: -0.02em; scroll-margin-top: 1.25rem; }
    .prose h3 { font-size: 1.05rem; font-weight: 600; margin: 1.5rem 0 0.5rem; scroll-margin-top: 1.25rem; }
    .prose p { margin: 0.75rem 0; color: color-mix(in srgb, var(--text) 88%, var(--muted)); }
    .prose ul, .prose ol { margin: 0.75rem 0; padding-left: 1.25rem; color: color-mix(in srgb, var(--text) 88%, var(--muted)); }
    .prose li { margin: 0.25rem 0; }
    .prose blockquote {
      margin: 1rem 0;
      padding-left: 1rem;
      border-left: 3px solid var(--accent);
      color: var(--muted);
    }
    .prose code {
      font-family: "JetBrains Mono", ui-monospace, monospace;
      font-size: 0.88em;
      background: color-mix(in srgb, var(--surface) 70%, var(--border));
      padding: 0.12em 0.35em;
      border-radius: 4px;
    }
    .prose pre {
      background: #0a0a0a;
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 0.95rem 1rem;
      overflow-x: auto;
      margin: 1rem 0;
    }
    .prose pre code {
      background: none;
      padding: 0;
      font-size: 0.8rem;
      line-height: 1.5;
    }
    .prose table {
      width: 100%;
      border-collapse: collapse;
      margin: 1rem 0;
      font-size: 0.9rem;
    }
    .prose th, .prose td {
      border: 1px solid var(--border);
      padding: 0.5rem 0.65rem;
      text-align: left;
    }
    .prose th {
      background: color-mix(in srgb, var(--surface) 80%, var(--border));
      font-weight: 600;
    }
    .prose hr { border: none; border-top: 1px solid var(--border); margin: 2rem 0; }
    .prose a { color: color-mix(in srgb, var(--text) 90%, var(--muted)); border-bottom: 1px solid color-mix(in srgb, var(--border) 70%, var(--muted)); }
    .prose a:hover { color: var(--text); border-bottom-color: var(--text); }
    .docs-landing .lede {
      font-size: 1rem;
      max-width: 48rem;
      margin: 0 0 2rem;
      color: color-mix(in srgb, var(--text) 85%, var(--muted));
    }
    .docs-landing .lede code {
      font-family: "JetBrains Mono", monospace;
      font-size: 0.9em;
    }
    .docs-landing .lede a {
      color: color-mix(in srgb, var(--text) 90%, var(--muted));
      border-bottom: 1px solid color-mix(in srgb, var(--border) 70%, var(--muted));
    }
    .docs-landing .lede a:hover {
      color: var(--text);
      border-bottom-color: var(--text);
    }
    .doc-grid {
      display: grid;
      gap: 1rem;
      grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    }
    .doc-card {
      display: block;
      padding: 1.15rem 1.2rem;
      border: 1px solid var(--border);
      border-radius: 12px;
      background: color-mix(in srgb, var(--surface) 92%, white 8%);
      transition: border-color 0.15s ease, background 0.15s ease;
    }
    .doc-card:hover {
      border-color: color-mix(in srgb, var(--text) 35%, var(--border));
      background: color-mix(in srgb, var(--surface) 88%, white 12%);
      text-decoration: none;
    }
    .doc-card h2 {
      margin: 0 0 0.35rem;
      font-size: 1.05rem;
      font-weight: 600;
      color: var(--text);
    }
    .doc-card p {
      margin: 0;
      font-size: 0.875rem;
      color: var(--muted);
      line-height: 1.5;
    }
    .footnote {
      margin-top: 2rem;
      font-size: 0.875rem;
      color: var(--muted);
    }
    .doc-footer {
      margin-top: auto;
      padding-top: 1rem;
      font-size: 0.75rem;
      color: var(--muted);
      line-height: 1.45;
    }
    .doc-footer strong { color: color-mix(in srgb, var(--text) 70%, var(--muted)); }
    .doc-search-overlay {
      position: fixed;
      inset: 0;
      z-index: 200;
      background: rgba(0,0,0,0.55);
      display: flex;
      align-items: flex-start;
      justify-content: center;
      padding: min(8vh, 4rem) 0.75rem 1rem;
    }
    .doc-search-overlay--hidden {
      display: none !important;
    }
    .doc-search-error,
    .doc-search-empty {
      padding: 0.75rem 0.65rem;
      font-size: 0.85rem;
      color: var(--muted);
      list-style: none;
    }
    .doc-search-error { color: #f87171; }
    .doc-search-dialog {
      width: min(560px, 100%);
      background: color-mix(in srgb, var(--surface) 94%, white 6%);
      border: 1px solid var(--border);
      border-radius: 12px;
      box-shadow: 0 24px 80px rgba(0,0,0,0.45);
      overflow: hidden;
    }
    .doc-search-dialog input {
      width: 100%;
      box-sizing: border-box;
      border: none;
      border-bottom: 1px solid var(--border);
      padding: 1rem 1.1rem;
      font: inherit;
      font-size: 1rem;
      background: transparent;
      color: var(--text);
    }
    .doc-search-dialog input:focus { outline: none; }
    .doc-search-dialog ul {
      list-style: none;
      margin: 0;
      padding: 0.5rem;
      max-height: min(340px, 48vh);
      overflow-y: auto;
    }
    .doc-search-dialog li {
      border-radius: 8px;
    }
    .doc-search-dialog li a {
      display: block;
      padding: 0.55rem 0.65rem;
      font-weight: 600;
      font-size: 0.9rem;
      color: var(--text);
    }
    .doc-search-dialog li:hover { background: color-mix(in srgb, var(--surface) 80%, white 20%); }
    .doc-search-snippet {
      display: block;
      padding: 0 0.65rem 0.55rem;
      font-size: 0.75rem;
      color: var(--muted);
      line-height: 1.35;
    }
    @media (max-width: 640px) {
      .doc-shell {
        min-height: auto;
      }
      aside {
        padding: 0.95rem 0.9rem 1rem;
        gap: 0.75rem;
      }
      .brand {
        font-size: 0.9rem;
      }
      .doc-search-trigger {
        padding: 0.6rem 0.7rem;
        font-size: 0.8rem;
      }
      .doc-search-trigger kbd {
        display: none;
      }
      .doc-nav {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 0.35rem 0.75rem;
        font-size: 0.82rem;
      }
      .doc-nav li {
        margin: 0;
      }
      .doc-toc {
        padding-top: 0.65rem;
      }
      .doc-toc ul {
        font-size: 0.78rem;
      }
      .doc-main {
        padding: 1.1rem 0.9rem 2.5rem;
      }
      .prose h1 {
        font-size: 1.65rem;
        margin-bottom: 0.85rem;
      }
      .prose h2 {
        font-size: 1.15rem;
        margin-top: 1.5rem;
      }
      .prose h3 {
        font-size: 0.98rem;
      }
      .prose p,
      .prose ul,
      .prose ol {
        margin-top: 0.65rem;
        margin-bottom: 0.65rem;
      }
      .prose pre {
        padding: 0.8rem 0.85rem;
        border-radius: 10px;
      }
      .prose pre code {
        font-size: 0.75rem;
      }
      .prose table {
        display: block;
        overflow-x: auto;
        -webkit-overflow-scrolling: touch;
      }
      .docs-landing .lede {
        font-size: 0.95rem;
        margin-bottom: 1.5rem;
      }
      .doc-grid {
        grid-template-columns: 1fr;
      }
      .doc-card {
        padding: 1rem 1.05rem;
      }
      .footnote {
        margin-top: 1.5rem;
      }
      .doc-search-overlay {
        padding-top: 0.75rem;
      }
      .doc-search-dialog {
        width: 100%;
      }
      .doc-search-dialog input {
        padding: 0.9rem 1rem;
      }
    }
    .doc-sidebar-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 0.75rem;
    }
    .doc-nav-toggle {
      appearance: none;
      border: 0;
      background: transparent;
      color: var(--text);
      border-radius: 999px;
      width: 2.25rem;
      height: 2.25rem;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      cursor: pointer;
      flex: 0 0 auto;
      padding: 0;
      line-height: 1;
      font-size: 1.45rem;
    }
    .doc-nav-toggle:hover {
      color: var(--accent);
    }
    .doc-nav-toggle--main { display: none; }
    .doc-main { min-width: 0; }
    .doc-search-trigger { min-width: 0; }
    @media (max-width: 860px) {
      .doc-shell { grid-template-columns: 1fr; }
      .doc-nav-toggle--main {
        display: inline-flex;
        position: fixed;
        top: 0.9rem;
        left: 0.9rem;
        z-index: 180;
      }
      aside.mobile-expanded + .doc-main .doc-nav-toggle--main {
        left: min(calc(84vw + 0.65rem), 328px);
      }
      aside {
        position: fixed;
        inset: 0 auto 0 0;
        width: min(84vw, 320px);
        transform: translateX(-105%);
        transition: transform 0.2s ease;
        z-index: 100;
        box-shadow: 2px 0 16px rgba(0, 0, 0, 0.18);
        height: 100vh;
      }
      aside.mobile-expanded { transform: translateX(0); }
      aside .doc-nav,
      aside .doc-toc,
      aside .doc-search-trigger,
      aside .doc-footer { display: none; }
      aside.mobile-expanded .doc-nav,
      aside.mobile-expanded .doc-toc,
      aside.mobile-expanded .doc-search-trigger,
      aside.mobile-expanded .doc-footer { display: block; }
      aside.mobile-expanded .doc-search-trigger { display: flex; }
      .doc-search-trigger { width: 100%; }
      .doc-main { padding: 4.25rem 1rem 3rem; }
    }
"#;

const SITE_BASE_URL: &str = "https://crepuscularity.undivisible.dev";
const SITE_IMAGE_URL: &str = "https://crepuscularity.undivisible.dev/static/og.png";

/// Theme variables mirrored from `site.json` for doc shells.
#[derive(Clone, Debug, Deserialize)]
pub struct DocsSiteTheme {
    pub accent: String,
    pub accent_soft: String,
    pub surface: String,
    pub text: String,
    pub muted: String,
    pub border: String,
}

struct DocNavItem {
    stem: String,
    title: String,
    href: String,
}

struct DocShellMeta<'a> {
    page_title: &'a str,
    description: &'a str,
    canonical_url: &'a str,
    article: bool,
    wide: bool,
}

#[derive(Clone, Debug)]
struct OutlineItem {
    level: u8,
    text: String,
    id: String,
}

fn main() {
    let mut docs_src = None;
    let mut out_dir = None;
    let mut site_name = None;
    let mut theme_json = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--docs-src" => docs_src = args.next().map(PathBuf::from),
            "--out-dir" => out_dir = args.next().map(PathBuf::from),
            "--site-name" => site_name = args.next(),
            "--theme-json" => theme_json = args.next(),
            _ => {}
        }
    }

    let docs_src = docs_src.unwrap_or_else(|| usage());
    let out_dir = out_dir.unwrap_or_else(|| usage());
    let site_name = site_name.unwrap_or_else(|| usage());
    let theme_json = theme_json.unwrap_or_else(|| usage());
    let theme: DocsSiteTheme = serde_json::from_str(&theme_json).unwrap_or_else(|e| {
        eprintln!("parse --theme-json: {e}");
        std::process::exit(1);
    });

    if let Err(e) = emit_markdown_docs(&docs_src, &out_dir, &theme, &site_name) {
        eprintln!("emit docs HTML: {e}");
        std::process::exit(1);
    }
}

fn usage() -> ! {
    eprintln!(
        "usage: docs-site-renderer --docs-src DIR --out-dir DIR --site-name NAME --theme-json JSON"
    );
    std::process::exit(2);
}

/// Writes `out_docs_dir/*.html` plus `out_docs_dir/index.html`. No-op if `docs_src` is missing.
pub fn emit_markdown_docs(
    docs_src: &Path,
    out_docs_dir: &Path,
    theme: &DocsSiteTheme,
    site_name: &str,
) -> std::io::Result<()> {
    if !docs_src.is_dir() {
        return Ok(());
    }

    fs::create_dir_all(out_docs_dir)?;

    let paths = gather_markdown_paths(docs_src)?;
    let items = extract_doc_items(&paths)?;

    write_index_page(out_docs_dir, theme, site_name, &items)?;
    render_pages_and_search_index(&paths, out_docs_dir, theme, site_name, &items)?;

    Ok(())
}

fn gather_markdown_paths(docs_src: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut paths: Vec<std::path::PathBuf> = fs::read_dir(docs_src)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("md"))
        .collect();
    paths.sort();
    paths.retain(|p| {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        !name.eq_ignore_ascii_case("README.md")
            && !name.eq_ignore_ascii_case("CREPUS_WEB_IMPLEMENTATION_SPEC.md")
    });
    Ok(paths)
}

fn extract_doc_items(paths: &[PathBuf]) -> std::io::Result<Vec<DocNavItem>> {
    let mut items: Vec<DocNavItem> = Vec::with_capacity(paths.len());
    for path in paths {
        let stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let raw = fs::read_to_string(path)?;
        let title = first_markdown_title(&raw).unwrap_or_else(|| prettify_stem(&stem));
        items.push(DocNavItem {
            stem,
            title,
            href: format!(
                "{}.html",
                path.file_stem().unwrap_or_default().to_string_lossy()
            ),
        });
    }
    Ok(items)
}

fn write_index_page(
    out_docs_dir: &Path,
    theme: &DocsSiteTheme,
    site_name: &str,
    items: &[DocNavItem],
) -> std::io::Result<()> {
    let index_body = render_docs_landing_body(site_name, items);
    let index_description =
        "Guides and references for the .crepus DSL, native and WASM renderers, and the crepus CLI.";
    let index_html = render_doc_shell(
        site_name,
        DocShellMeta {
            page_title: "Documentation",
            description: index_description,
            canonical_url: &format!("{SITE_BASE_URL}/docs/"),
            article: false,
            wide: true,
        },
        theme,
        &render_nav_list(items, Some("index.html")),
        "",
        &index_body,
    );
    fs::write(out_docs_dir.join("index.html"), index_html)
}

fn render_pages_and_search_index(
    paths: &[PathBuf],
    out_docs_dir: &Path,
    theme: &DocsSiteTheme,
    site_name: &str,
    items: &[DocNavItem],
) -> std::io::Result<()> {
    let mut search_entries = vec![json!({
        "title": "Documentation — overview",
        "href": "index.html",
        "text": "Guides and references for the .crepus DSL, crepus CLI, GPUI, WASM sites, and web extensions."
    })];

    for (path, item) in paths.iter().zip(items) {
        let raw = fs::read_to_string(path)?;
        let plain = strip_markdown_plain(&raw);
        let text: String = plain.chars().take(480).collect();
        let description = first_markdown_description(&raw, doc_blurb_plain(&item.stem));
        let canonical_url = format!("{SITE_BASE_URL}/docs/{}", item.href);
        search_entries.push(json!({
            "title": &item.title,
            "href": &item.href,
            "text": text,
        }));

        let outline = extract_outline(&raw);
        let body_html = markdown_to_html(&raw);
        let body_html = inject_heading_ids(&body_html, &outline);
        let toc = render_toc_nav(&outline);
        let nav = render_nav_list(items, Some(&item.href));
        let doc_html = render_doc_shell(
            site_name,
            DocShellMeta {
                page_title: &item.title,
                description: &description,
                canonical_url: &canonical_url,
                article: true,
                wide: false,
            },
            theme,
            &nav,
            &toc,
            &format!(r#"<article class="prose">{body_html}</article>"#),
        );
        fs::write(out_docs_dir.join(&item.href), doc_html)?;
    }

    let search_json = json!({ "entries": search_entries }).to_string();
    fs::write(
        out_docs_dir.join("docs-search-index.json"),
        search_json.as_bytes(),
    )
}

fn strip_markdown_plain(md: &str) -> String {
    let mut out = String::new();
    let mut in_fence = false;
    for line in md.lines() {
        let t = line.trim();
        if t.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence || t.is_empty() {
            continue;
        }
        let t = if let Some(rest) = t.strip_prefix('#') {
            rest.trim()
        } else {
            t
        };
        if !t.is_empty() {
            out.push_str(t);
            out.push(' ');
        }
    }
    out
}

fn render_docs_landing_body(site_name: &str, items: &[DocNavItem]) -> String {
    let esc = escape_html(site_name);
    let mut cards = String::new();
    for it in items {
        let blurb = doc_blurb(&it.stem);
        cards.push_str(&format!(
            r#"<a class="doc-card" href="{}"><h2>{}</h2><p>{blurb}</p></a>"#,
            escape_html(&it.href),
            escape_html(&it.title),
        ));
    }
    format!(
        r#"<div class="docs-landing">
  <p class="lede">Guides and references for the <strong>.crepus</strong> DSL, native and WASM renderers, and the <code>crepus</code> CLI. Same Markdown sources as the <a href="https://github.com/tschk/crepuscularity/tree/main/docs">repository <code>docs/</code> folder</a>.</p>
  <div class="doc-grid">{cards}</div>
  <p class="footnote"><a href="../index.html">← Back to {esc} home</a></p>
</div>"#
    )
}

fn doc_blurb(stem: &str) -> String {
    escape_html(doc_blurb_plain(stem))
}

fn doc_blurb_plain(stem: &str) -> &'static str {
    match stem {
        "dsl" => "Indent and JSX-style syntax, control flow, attributes, animations.",
        "components" => "include, slots, defaults, and multi-component files.",
        "cli" => "new, dev, build, web, webext, preview, and more.",
        "production" => "release gates, security boundaries, and performance checks.",
        "runtime" => "state model, update lifecycle, hydration, and Metal setup.",
        "webext" => "Manifest V3 extensions from .crepus and Rust.",
        "gpui" => "native desktop rendering with GPUI.",
        "native" => "SwiftUI and Jetpack Compose shells from View IR.",
        "tui" => "terminal UI rendering from .crepus templates.",
        _ => "Documentation page.",
    }
}

fn first_markdown_title(md: &str) -> Option<String> {
    for line in md.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("# ") {
            let s = rest.trim();
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }
    None
}

fn first_markdown_description(md: &str, fallback: &str) -> String {
    let mut paragraph = Vec::new();
    let mut in_fence = false;
    for line in md.lines() {
        let t = line.trim();
        if t.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if t.is_empty() {
            if !paragraph.is_empty() {
                break;
            }
            continue;
        }
        if t.starts_with('#') {
            continue;
        }
        let cleaned = clean_markdown_inline(t);
        if cleaned.starts_with("Also:") {
            continue;
        }
        paragraph.push(t);
    }
    let raw = if paragraph.is_empty() {
        fallback.to_string()
    } else {
        paragraph.join(" ")
    };
    clean_markdown_inline(&raw)
}

fn clean_markdown_inline(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '`' | '*' | '_' => {}
            '[' => {
                let mut text = String::new();
                for next in chars.by_ref() {
                    if next == ']' {
                        break;
                    }
                    text.push(next);
                }
                if chars.peek() == Some(&'(') {
                    for next in chars.by_ref() {
                        if next == ')' {
                            break;
                        }
                    }
                }
                out.push_str(&text);
            }
            _ => out.push(c),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn prettify_stem(stem: &str) -> String {
    stem.replace('_', " ")
}

fn markdown_to_html(md: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(md, opts);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    rewrite_local_md_links(&html_out)
}

fn rewrite_local_md_links(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    while let Some(idx) = rest.find("href=\"") {
        out.push_str(&rest[..idx + 6]);
        rest = &rest[idx + 6..];
        if let Some(end) = rest.find('"') {
            let url = &rest[..end];
            out.push_str(&fix_local_md_href(url));
            out.push('"');
            rest = &rest[end + 1..];
        } else {
            out.push_str(rest);
            return out;
        }
    }
    out.push_str(rest);
    out
}

fn fix_local_md_href(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("mailto:") {
        return url.to_string();
    }
    let (path, frag) = match url.split_once('#') {
        Some((p, f)) => (p, Some(f.to_string())),
        None => (url, None),
    };
    let new_path = if path.ends_with(".md") {
        format!("{}.html", path.trim_end_matches(".md"))
    } else {
        path.to_string()
    };
    match frag {
        Some(f) => format!("{new_path}#{f}"),
        None => new_path,
    }
}

fn slugify_heading_title(title: &str) -> String {
    let mut slug = String::new();
    let mut prev_hyphen = true;
    for c in title.chars() {
        let c = c.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            slug.push(c);
            prev_hyphen = false;
        } else if !prev_hyphen {
            slug.push('-');
            prev_hyphen = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "section".into()
    } else {
        slug
    }
}

fn extract_outline(md: &str) -> Vec<OutlineItem> {
    let mut out = Vec::new();
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut in_fence = false;
    for line in md.lines() {
        let t = line.trim_start();
        if t.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let t = t.trim();
        let (level, title_raw) = if t.starts_with("### ") && !t.starts_with("#### ") {
            (3u8, t[4..].trim())
        } else if t.starts_with("## ") && !t.starts_with("### ") {
            (2u8, t[3..].trim())
        } else {
            continue;
        };
        if title_raw.is_empty() {
            continue;
        }
        let text = title_raw.to_string();
        let base = slugify_heading_title(title_raw);
        let n = if let Some(n) = seen.get_mut(&base) {
            *n += 1;
            *n
        } else {
            seen.insert(base.clone(), 1);
            1
        };
        let id = if n == 1 {
            base
        } else {
            format!("{}-{}", base, n)
        };
        out.push(OutlineItem { level, text, id });
    }
    out
}

fn inject_heading_ids(html: &str, outline: &[OutlineItem]) -> String {
    let mut out = String::with_capacity(html.len().saturating_add(outline.len() * 24));
    let mut rest = html;
    for item in outline {
        let (open_pat, close_pat) = if item.level == 2 {
            ("<h2>", "</h2>")
        } else {
            ("<h3>", "</h3>")
        };
        let Some(pos) = rest.find(open_pat) else {
            out.push_str(rest);
            return out;
        };
        out.push_str(&rest[..pos]);
        rest = &rest[pos + open_pat.len()..];
        let Some(close_pos) = rest.find(close_pat) else {
            out.push_str(open_pat);
            out.push_str(rest);
            return out;
        };
        let inner = &rest[..close_pos];
        out.push_str(&format!(
            r#"<h{} id="{}">"#,
            item.level,
            escape_html_attr(&item.id)
        ));
        out.push_str(inner);
        out.push_str(close_pat);
        rest = &rest[close_pos + close_pat.len()..];
    }
    out.push_str(rest);
    out
}

fn render_toc_nav(outline: &[OutlineItem]) -> String {
    if outline.is_empty() {
        return String::new();
    }
    let mut s = String::from(
        r#"<nav class="doc-toc" aria-label="On this page"><div class="doc-toc-title">On this page</div><ul>"#,
    );
    for it in outline {
        let li_cls = if it.level == 3 {
            r#" class="doc-toc-h3""#
        } else {
            ""
        };
        s.push_str(&format!(
            r##"<li{li_cls}><a href="#{}">{}</a></li>"##,
            escape_html_attr(&it.id),
            escape_html(&it.text)
        ));
    }
    s.push_str("</ul></nav>");
    s
}

fn render_nav_list(items: &[DocNavItem], active_href: Option<&str>) -> String {
    let mut lis = String::new();
    let home_active = active_href == Some("index.html");
    lis.push_str(if home_active {
        r#"<li><a class="active" href="index.html">Overview</a></li>"#
    } else {
        r#"<li><a href="index.html">Overview</a></li>"#
    });

    for it in items {
        let cls = if active_href == Some(it.href.as_str()) {
            r#" class="active""#
        } else {
            ""
        };
        lis.push_str(&format!(
            r#"<li><a{cls} href="{}">{}</a></li>"#,
            escape_html(&it.href),
            escape_html(&it.title),
        ));
    }
    format!(r#"<ul class="doc-nav">{lis}</ul>"#)
}

fn render_doc_shell(
    site_name: &str,
    meta: DocShellMeta<'_>,
    theme: &DocsSiteTheme,
    nav: &str,
    page_toc: &str,
    main_inner: &str,
) -> String {
    let esc_site = escape_html(site_name);
    let esc_page = escape_html(meta.page_title);
    let full_title = format!("{} — {site_name}", meta.page_title);
    let esc_full_title = escape_html(&full_title);
    let esc_description = escape_html_attr(meta.description);
    let esc_canonical = escape_html_attr(meta.canonical_url);
    let schema_type = if meta.article {
        "TechArticle"
    } else {
        "CollectionPage"
    };
    let og_type = if meta.article { "article" } else { "website" };
    let json_ld = json!({
        "@context": "https://schema.org",
        "@type": schema_type,
        "headline": meta.page_title,
        "name": meta.page_title,
        "description": meta.description,
        "url": meta.canonical_url,
        "image": SITE_IMAGE_URL,
        "isPartOf": {
            "@type": "WebSite",
            "name": site_name,
            "url": SITE_BASE_URL
        }
    })
    .to_string();
    let main_cls = if meta.wide {
        "doc-main doc-main--wide"
    } else {
        "doc-main"
    };
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
	  <meta charset="utf-8">
	  <meta name="viewport" content="width=device-width, initial-scale=1">
	  <title>{esc_page} — {esc_site}</title>
	  <meta name="description" content="{esc_description}">
	  <link rel="canonical" href="{esc_canonical}">
	  <link rel="alternate" type="text/plain" href="{SITE_BASE_URL}/llms.txt" title="LLM index">
	  <link rel="alternate" type="text/plain" href="{SITE_BASE_URL}/llms-full.txt" title="Full LLM bundle">
	  <link rel="alternate" type="text/markdown" href="{SITE_BASE_URL}/agent.md" title="Agent guide">
	  <meta name="robots" content="index,follow">
	  <meta name="theme-color" content="{s}">
	  <meta property="og:title" content="{esc_full_title}">
	  <meta property="og:description" content="{esc_description}">
	  <meta property="og:type" content="{og_type}">
	  <meta property="og:url" content="{esc_canonical}">
	  <meta property="og:site_name" content="{esc_site}">
	  <meta property="og:image" content="{SITE_IMAGE_URL}">
	  <meta property="og:image:alt" content="Crepuscularity interface preview">
	  <meta name="twitter:card" content="summary_large_image">
	  <meta name="twitter:title" content="{esc_full_title}">
	  <meta name="twitter:description" content="{esc_description}">
	  <meta name="twitter:image" content="{SITE_IMAGE_URL}">
	  <script type="application/ld+json">{json_ld}</script>
	  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
  <style>
    :root {{
      --accent: {a};
      --accent-soft: {as};
      --surface: {s};
      --text: {t};
      --muted: {m};
      --border: {b};
    }}
    {DOC_SHELL_CSS}
  </style>
</head>
<body>
  <div class="doc-shell">
    <aside>
      <div class="doc-sidebar-header">
        <a class="brand" href="../index.html">{esc_site}</a>
      </div>
      <button type="button" class="doc-search-trigger" id="doc-search-open" aria-label="Open documentation search">
        <span>Search…</span>
        <kbd>⌘K</kbd>
      </button>
      <nav aria-label="Documentation">
        {nav}
      </nav>
      {page_toc}
      <footer class="doc-footer">
        <strong>docs-site</strong> renders these pages from Markdown.<br>
        Press <kbd style="font-size:0.65rem;padding:0.1rem 0.3rem;border:1px solid var(--border);border-radius:3px;">⌘K</kbd> or <kbd style="font-size:0.65rem;padding:0.1rem 0.3rem;border:1px solid var(--border);border-radius:3px;">Ctrl+K</kbd> to search.
      </footer>
    </aside>
    <main class="{main_cls}">
      <button class="doc-nav-toggle doc-nav-toggle--main" id="doc-nav-toggle" type="button" aria-label="Open navigation" onclick="toggleDocNav()">☰</button>
      {main_inner}
    </main>
  </div>
  <div id="doc-search-overlay" class="doc-search-overlay doc-search-overlay--hidden" aria-hidden="true">
    <div class="doc-search-dialog" role="dialog" aria-modal="true" aria-label="Search documentation">
      <input type="search" id="doc-search-input" autocomplete="off" placeholder="Fuzzy search titles and body text…">
      <ul id="doc-search-results"></ul>
    </div>
  </div>
  <script>__CREPUS_DOCS_SEARCH__</script>
  <script>
    function toggleDocNav() {{
      var aside = document.querySelector('aside');
      aside.classList.toggle('mobile-expanded');
    }}
  </script>
</body>
</html>"#,
        esc_site = esc_site,
        esc_page = esc_page,
        main_cls = main_cls,
        main_inner = main_inner,
        nav = nav,
        page_toc = page_toc,
        a = theme.accent,
        as = theme.accent_soft,
        s = theme.surface,
        t = theme.text,
        m = theme.muted,
        b = theme.border,
        DOC_SHELL_CSS = DOC_SHELL_CSS,
    );
    html.replace("__CREPUS_DOCS_SEARCH__", DOCS_SEARCH_JS)
}

fn escape_html(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

fn escape_html_attr(s: &str) -> String {
    escape_html(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme() -> DocsSiteTheme {
        DocsSiteTheme {
            accent: "#ffb070".to_string(),
            accent_soft: "#e8c4ff".to_string(),
            surface: "#0c0612".to_string(),
            text: "#f4ebff".to_string(),
            muted: "#9a8ab8".to_string(),
            border: "#4c4168".to_string(),
        }
    }

    #[test]
    fn emitted_docs_pages_include_page_specific_seo() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let docs_src = tmp.path().join("docs");
        let out_docs = tmp.path().join("dist/docs");
        fs::create_dir_all(&docs_src).expect("mkdir docs");
        fs::write(
            docs_src.join("cli.md"),
            "# CLI Guide\n\n**Also:** [Documentation home](README.md) · [DSL](dsl.md)\n\nBuild static WASM sites, browser extensions, native IR, and docs from one Crepuscularity manifest.\n\n## Usage\n\nRun `crepus web build`.",
        )
        .expect("write doc");

        emit_markdown_docs(&docs_src, &out_docs, &theme(), "Crepuscularity").expect("emit docs");

        let html = fs::read_to_string(out_docs.join("cli.html")).expect("cli html");
        assert!(html.contains(r#"<title>CLI Guide — Crepuscularity</title>"#));
        assert!(html.contains(r#"<meta name="description" content="Build static WASM sites, browser extensions, native IR, and docs from one Crepuscularity manifest.">"#));
        assert!(html.contains(
            r#"<link rel="canonical" href="https://crepuscularity.undivisible.dev/docs/cli.html">"#
        ));
        assert!(html.contains(r#"<meta property="og:title" content="CLI Guide — Crepuscularity">"#));
        assert!(html.contains(r#"<meta property="og:url" content="https://crepuscularity.undivisible.dev/docs/cli.html">"#));
        assert!(html.contains(r#"<meta name="twitter:card" content="summary_large_image">"#));
        assert!(html.contains(r#""@type":"TechArticle""#));
    }

    #[test]
    fn mobile_docs_nav_uses_single_borderless_overlay_toggle() {
        let html = render_doc_shell(
            "Crepuscularity",
            DocShellMeta {
                page_title: "Docs",
                description: "Documentation for Crepuscularity.",
                canonical_url: "https://crepuscularity.undivisible.dev/docs/",
                article: false,
                wide: false,
            },
            &theme(),
            "<ul class=\"doc-nav\"><li><a href=\"index.html\">Docs</a></li></ul>",
            "",
            "<article class=\"prose\"><h1>Docs</h1></article>",
        );

        assert!(html.contains(".doc-nav-toggle {\n      appearance: none;\n      border: 0;"));
        assert!(html.contains("background: transparent;"));
        assert!(html.contains("z-index: 180;"));
        assert!(html.contains(".doc-nav-toggle--main { display: none; }"));
        assert!(html.contains("@media (max-width: 860px)"));
        assert!(html.contains("aside.mobile-expanded + .doc-main .doc-nav-toggle--main"));
        assert!(html.contains(".doc-main { padding: 4.25rem 1rem 3rem; }"));
        assert!(!html.contains("doc-nav-toggle--sidebar"));
        assert!(!html.contains("desktop-collapsed"));
    }
}
