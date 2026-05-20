//! Emit static HTML from repository `docs/*.md` when running `crepus web build`.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use pulldown_cmark::{html, Options, Parser};
use serde_json::json;

const DOCS_SEARCH_JS: &str = include_str!("../assets/web/docs_search.js");

/// Theme variables mirrored from `site.json` for doc shells.
#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
struct OutlineItem {
    level: u8,
    text: String,
    id: String,
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

    let mut items: Vec<DocNavItem> = Vec::with_capacity(paths.len());
    for path in &paths {
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

    let index_body = render_docs_landing_body(site_name, &items);
    let index_html = render_doc_shell(
        site_name,
        "Documentation",
        theme,
        &render_nav_list(&items, Some("index.html")),
        "",
        &index_body,
        true,
    );
    fs::write(out_docs_dir.join("index.html"), index_html)?;

    let mut search_entries = vec![json!({
        "title": "Documentation — overview",
        "href": "index.html",
        "text": "Guides and references for the .crepus DSL, crepus CLI, GPUI, WASM sites, and web extensions."
    })];

    for (path, item) in paths.iter().zip(&items) {
        let raw = fs::read_to_string(path)?;
        let plain = strip_markdown_plain(&raw);
        let text: String = plain.chars().take(480).collect();
        search_entries.push(json!({
            "title": &item.title,
            "href": &item.href,
            "text": text,
        }));

        let outline = extract_outline(&raw);
        let body_html = markdown_to_html(&raw);
        let body_html = inject_heading_ids(&body_html, &outline);
        let toc = render_toc_nav(&outline);
        let nav = render_nav_list(&items, Some(&item.href));
        let doc_html = render_doc_shell(
            site_name,
            &item.title,
            theme,
            &nav,
            &toc,
            &format!(r#"<article class="prose">{body_html}</article>"#),
            false,
        );
        fs::write(out_docs_dir.join(&item.href), doc_html)?;
    }

    let search_json = json!({ "entries": search_entries }).to_string();
    fs::write(
        out_docs_dir.join("docs-search-index.json"),
        search_json.as_bytes(),
    )?;

    Ok(())
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
    escape_html(match stem {
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
    })
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
        let n = seen.entry(base.clone()).or_insert(0);
        *n += 1;
        let id = if *n == 1 {
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
    page_title: &str,
    theme: &DocsSiteTheme,
    nav: &str,
    page_toc: &str,
    main_inner: &str,
    wide: bool,
) -> String {
    let esc_site = escape_html(site_name);
    let esc_page = escape_html(page_title);
    let main_cls = if wide {
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
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      min-height: 100vh;
      overflow-x: hidden;
      background: var(--surface);
      color: var(--text);
      font-family: Inter, system-ui, sans-serif;
      -webkit-font-smoothing: antialiased;
      line-height: 1.6;
    }}
    a {{ color: color-mix(in srgb, var(--text) 88%, transparent); text-decoration: none; }}
    a:hover {{ color: var(--text); text-decoration: underline; text-underline-offset: 3px; }}
    .doc-shell {{
      display: grid;
      grid-template-columns: minmax(220px, 280px) 1fr;
      min-height: 100vh;
      align-items: stretch;
    }}
    .doc-shell > * {{
      min-width: 0;
    }}
    aside {{
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
    }}
    .brand {{
      font-weight: 700;
      font-size: 0.95rem;
      letter-spacing: -0.02em;
      margin-bottom: 0.25rem;
      display: block;
      color: var(--text);
    }}
    .brand:hover {{ text-decoration: none; color: var(--text); opacity: 0.85; }}
    .doc-search-trigger {{
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
    }}
    .doc-search-trigger:hover {{
      border-color: color-mix(in srgb, var(--text) 25%, var(--border));
      color: var(--text);
    }}
    .doc-search-trigger kbd {{
      font-family: ui-monospace, monospace;
      font-size: 0.7rem;
      padding: 0.12rem 0.35rem;
      border-radius: 4px;
      border: 1px solid var(--border);
      background: var(--surface);
      color: var(--muted);
    }}
    .doc-nav {{
      list-style: none;
      padding: 0;
      margin: 0;
      font-size: 0.875rem;
    }}
    .doc-nav li {{ margin: 0.35rem 0; }}
    .doc-nav a {{
      color: var(--muted);
      display: block;
      width: 100%;
      overflow-wrap: anywhere;
      word-break: break-word;
    }}
    .doc-nav a:hover {{ color: var(--text); }}
    .doc-nav a.active {{ color: var(--text); font-weight: 600; }}
    .doc-toc {{
      margin: 0.25rem 0 0;
      padding-top: 0.75rem;
      border-top: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
    }}
    .doc-toc-title {{
      font-size: 0.68rem;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: var(--muted);
      margin: 0 0 0.45rem;
    }}
    .doc-toc ul {{
      list-style: none;
      padding: 0;
      margin: 0;
      font-size: 0.8125rem;
      line-height: 1.35;
    }}
    .doc-toc li {{ margin: 0.3rem 0; }}
    .doc-toc a {{
      color: var(--muted);
      display: block;
      width: 100%;
      text-decoration: none;
      overflow-wrap: anywhere;
      word-break: break-word;
    }}
    .doc-toc a:hover {{ color: var(--text); text-decoration: underline; text-underline-offset: 2px; }}
    .doc-toc li.doc-toc-h3 {{
      padding-left: 0.75rem;
      font-size: 0.78rem;
      opacity: 0.95;
    }}
    .doc-main {{
      padding: clamp(1.5rem, 3vw, 2.5rem) clamp(1rem, 3vw, 2.75rem) clamp(3.25rem, 5vw, 5rem);
      max-width: 52rem;
      min-width: 0;
    }}
    .doc-main.doc-main--wide {{
      max-width: 74rem;
    }}
    .prose h1 {{ font-size: 2rem; font-weight: 700; margin: 0 0 1rem; letter-spacing: -0.03em; line-height: 1.2; scroll-margin-top: 1.25rem; }}
    .prose h2 {{ font-size: 1.35rem; font-weight: 600; margin: 2rem 0 0.75rem; letter-spacing: -0.02em; scroll-margin-top: 1.25rem; }}
    .prose h3 {{ font-size: 1.05rem; font-weight: 600; margin: 1.5rem 0 0.5rem; scroll-margin-top: 1.25rem; }}
    .prose p {{ margin: 0.75rem 0; color: color-mix(in srgb, var(--text) 88%, var(--muted)); }}
    .prose ul, .prose ol {{ margin: 0.75rem 0; padding-left: 1.25rem; color: color-mix(in srgb, var(--text) 88%, var(--muted)); }}
    .prose li {{ margin: 0.25rem 0; }}
    .prose blockquote {{
      margin: 1rem 0;
      padding-left: 1rem;
      border-left: 3px solid var(--accent);
      color: var(--muted);
    }}
    .prose code {{
      font-family: "JetBrains Mono", ui-monospace, monospace;
      font-size: 0.88em;
      background: color-mix(in srgb, var(--surface) 70%, var(--border));
      padding: 0.12em 0.35em;
      border-radius: 4px;
    }}
    .prose pre {{
      background: #0a0a0a;
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 0.95rem 1rem;
      overflow-x: auto;
      margin: 1rem 0;
    }}
    .prose pre code {{
      background: none;
      padding: 0;
      font-size: 0.8rem;
      line-height: 1.5;
    }}
    .prose table {{
      width: 100%;
      border-collapse: collapse;
      margin: 1rem 0;
      font-size: 0.9rem;
    }}
    .prose th, .prose td {{
      border: 1px solid var(--border);
      padding: 0.5rem 0.65rem;
      text-align: left;
    }}
    .prose th {{
      background: color-mix(in srgb, var(--surface) 80%, var(--border));
      font-weight: 600;
    }}
    .prose hr {{ border: none; border-top: 1px solid var(--border); margin: 2rem 0; }}
    .prose a {{ color: color-mix(in srgb, var(--text) 90%, var(--muted)); border-bottom: 1px solid color-mix(in srgb, var(--border) 70%, var(--muted)); }}
    .prose a:hover {{ color: var(--text); border-bottom-color: var(--text); }}
    .docs-landing .lede {{
      font-size: 1rem;
      max-width: 48rem;
      margin: 0 0 2rem;
      color: color-mix(in srgb, var(--text) 85%, var(--muted));
    }}
    .docs-landing .lede code {{
      font-family: "JetBrains Mono", monospace;
      font-size: 0.9em;
    }}
    .docs-landing .lede a {{
      color: color-mix(in srgb, var(--text) 90%, var(--muted));
      border-bottom: 1px solid color-mix(in srgb, var(--border) 70%, var(--muted));
    }}
    .docs-landing .lede a:hover {{
      color: var(--text);
      border-bottom-color: var(--text);
    }}
    .doc-grid {{
      display: grid;
      gap: 1rem;
      grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    }}
    .doc-card {{
      display: block;
      padding: 1.15rem 1.2rem;
      border: 1px solid var(--border);
      border-radius: 12px;
      background: color-mix(in srgb, var(--surface) 92%, white 8%);
      transition: border-color 0.15s ease, background 0.15s ease;
    }}
    .doc-card:hover {{
      border-color: color-mix(in srgb, var(--text) 35%, var(--border));
      background: color-mix(in srgb, var(--surface) 88%, white 12%);
      text-decoration: none;
    }}
    .doc-card h2 {{
      margin: 0 0 0.35rem;
      font-size: 1.05rem;
      font-weight: 600;
      color: var(--text);
    }}
    .doc-card p {{
      margin: 0;
      font-size: 0.875rem;
      color: var(--muted);
      line-height: 1.5;
    }}
    .footnote {{
      margin-top: 2rem;
      font-size: 0.875rem;
      color: var(--muted);
    }}
    .doc-footer {{
      margin-top: auto;
      padding-top: 1rem;
      font-size: 0.75rem;
      color: var(--muted);
      line-height: 1.45;
    }}
    .doc-footer strong {{ color: color-mix(in srgb, var(--text) 70%, var(--muted)); }}
    .doc-search-overlay {{
      position: fixed;
      inset: 0;
      z-index: 200;
      background: rgba(0,0,0,0.55);
      display: flex;
      align-items: flex-start;
      justify-content: center;
      padding: min(8vh, 4rem) 0.75rem 1rem;
    }}
    .doc-search-overlay--hidden {{
      display: none !important;
    }}
    .doc-search-error,
    .doc-search-empty {{
      padding: 0.75rem 0.65rem;
      font-size: 0.85rem;
      color: var(--muted);
      list-style: none;
    }}
    .doc-search-error {{ color: #f87171; }}
    .doc-search-dialog {{
      width: min(560px, 100%);
      background: color-mix(in srgb, var(--surface) 94%, white 6%);
      border: 1px solid var(--border);
      border-radius: 12px;
      box-shadow: 0 24px 80px rgba(0,0,0,0.45);
      overflow: hidden;
    }}
    .doc-search-dialog input {{
      width: 100%;
      box-sizing: border-box;
      border: none;
      border-bottom: 1px solid var(--border);
      padding: 1rem 1.1rem;
      font: inherit;
      font-size: 1rem;
      background: transparent;
      color: var(--text);
    }}
    .doc-search-dialog input:focus {{ outline: none; }}
    .doc-search-dialog ul {{
      list-style: none;
      margin: 0;
      padding: 0.5rem;
      max-height: min(340px, 48vh);
      overflow-y: auto;
    }}
    .doc-search-dialog li {{
      border-radius: 8px;
    }}
    .doc-search-dialog li a {{
      display: block;
      padding: 0.55rem 0.65rem;
      font-weight: 600;
      font-size: 0.9rem;
      color: var(--text);
    }}
    .doc-search-dialog li:hover {{ background: color-mix(in srgb, var(--surface) 80%, white 20%); }}
    .doc-search-snippet {{
      display: block;
      padding: 0 0.65rem 0.55rem;
      font-size: 0.75rem;
      color: var(--muted);
      line-height: 1.35;
    }}
    @media (max-width: 640px) {{
      .doc-shell {{
        min-height: auto;
      }}
      aside {{
        padding: 0.95rem 0.9rem 1rem;
        gap: 0.75rem;
      }}
      .brand {{
        font-size: 0.9rem;
      }}
      .doc-search-trigger {{
        padding: 0.6rem 0.7rem;
        font-size: 0.8rem;
      }}
      .doc-search-trigger kbd {{
        display: none;
      }}
      .doc-nav {{
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 0.35rem 0.75rem;
        font-size: 0.82rem;
      }}
      .doc-nav li {{
        margin: 0;
      }}
      .doc-toc {{
        padding-top: 0.65rem;
      }}
      .doc-toc ul {{
        font-size: 0.78rem;
      }}
      .doc-main {{
        padding: 1.1rem 0.9rem 2.5rem;
      }}
      .prose h1 {{
        font-size: 1.65rem;
        margin-bottom: 0.85rem;
      }}
      .prose h2 {{
        font-size: 1.15rem;
        margin-top: 1.5rem;
      }}
      .prose h3 {{
        font-size: 0.98rem;
      }}
      .prose p,
      .prose ul,
      .prose ol {{
        margin-top: 0.65rem;
        margin-bottom: 0.65rem;
      }}
      .prose pre {{
        padding: 0.8rem 0.85rem;
        border-radius: 10px;
      }}
      .prose pre code {{
        font-size: 0.75rem;
      }}
      .prose table {{
        display: block;
        overflow-x: auto;
        -webkit-overflow-scrolling: touch;
      }}
      .docs-landing .lede {{
        font-size: 0.95rem;
        margin-bottom: 1.5rem;
      }}
      .doc-grid {{
        grid-template-columns: 1fr;
      }}
      .doc-card {{
        padding: 1rem 1.05rem;
      }}
      .footnote {{
        margin-top: 1.5rem;
      }}
      .doc-search-overlay {{
        padding-top: 0.75rem;
      }}
      .doc-search-dialog {{
        width: 100%;
      }}
      .doc-search-dialog input {{
        padding: 0.9rem 1rem;
      }}
    }}
    .doc-sidebar-header {{
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 0.75rem;
    }}
    .doc-nav-toggle {{
      appearance: none;
      border: 1px solid var(--border);
      background: color-mix(in srgb, var(--surface) 78%, white 22%);
      color: var(--text);
      border-radius: 8px;
      width: 2.25rem;
      height: 2.25rem;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      cursor: pointer;
      flex: 0 0 auto;
      padding: 0;
      line-height: 1;
    }}
    .doc-nav-toggle:hover {{
      border-color: color-mix(in srgb, var(--text) 25%, var(--border));
    }}
    .doc-nav-toggle--main {{ display: none; }}
    .doc-nav-toggle--sidebar {{ display: inline-flex; }}
    aside.desktop-collapsed {{
      width: 72px;
      padding-inline: 0.75rem;
    }}
    aside.desktop-collapsed .doc-nav,
    aside.desktop-collapsed .doc-toc,
    aside.desktop-collapsed .doc-search-trigger,
    aside.desktop-collapsed .doc-footer {{
      display: none;
    }}
    aside.desktop-collapsed .doc-sidebar-header {{
      justify-content: center;
    }}
    .doc-main {{ min-width: 0; }}
    .doc-search-trigger {{ min-width: 0; }}
    @media (max-width: 860px) {{
      .doc-shell {{ grid-template-columns: 1fr; }}
      .doc-nav-toggle--main {{
        display: inline-flex;
        position: fixed;
        top: 0.9rem;
        left: 0.9rem;
        z-index: 101;
      }}
      aside {{
        position: fixed;
        inset: 0 auto 0 0;
        width: min(84vw, 320px);
        transform: translateX(-105%);
        transition: transform 0.2s ease;
        z-index: 100;
        box-shadow: 2px 0 16px rgba(0, 0, 0, 0.18);
        height: 100vh;
      }}
      aside.mobile-expanded {{ transform: translateX(0); }}
      aside .doc-nav,
      aside .doc-toc,
      aside .doc-search-trigger,
      aside .doc-footer {{ display: none; }}
      aside.mobile-expanded .doc-nav,
      aside.mobile-expanded .doc-toc,
      aside.mobile-expanded .doc-search-trigger,
      aside.mobile-expanded .doc-footer {{ display: block; }}
      aside.mobile-expanded .doc-search-trigger {{ display: flex; }}
      .doc-nav-toggle--sidebar {{ display: inline-flex; }}
      .doc-search-trigger {{ width: 100%; }}
      .doc-main {{ padding: 1.25rem 1rem 3rem; }}
    }}
  </style>
</head>
<body>
  <div class="doc-shell">
    <aside>
      <div class="doc-sidebar-header">
        <a class="brand" href="../index.html">{esc_site}</a>
        <button class="doc-nav-toggle doc-nav-toggle--sidebar" type="button" aria-label="Toggle navigation" onclick="toggleDocNav()">☰</button>
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
        <strong>crepuscularity-web</strong> renders these pages from Markdown.<br>
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
      if (window.innerWidth <= 860) {{
        aside.classList.toggle('mobile-expanded');
      }} else {{
        aside.classList.toggle('desktop-collapsed');
      }}
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
