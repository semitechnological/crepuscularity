//! Emit static HTML from repository `docs/*.md` when running `crepus web build`.

use std::fs;
use std::path::Path;

use pulldown_cmark::{html, Options, Parser};

/// Theme variables mirrored from `site.json` for doc shells.
#[derive(Clone)]
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
        !p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.eq_ignore_ascii_case("README.md"))
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
        &index_body,
        true,
    );
    fs::write(out_docs_dir.join("index.html"), index_html)?;

    for (path, item) in paths.iter().zip(&items) {
        let raw = fs::read_to_string(path)?;
        let body_html = markdown_to_html(&raw);
        let nav = render_nav_list(&items, Some(&item.href));
        let doc_html = render_doc_shell(
            site_name,
            &item.title,
            theme,
            &nav,
            &format!(r#"<article class="prose">{body_html}</article>"#),
            false,
        );
        fs::write(out_docs_dir.join(&item.href), doc_html)?;
    }

    Ok(())
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
  <p class="lede">Guides and references for the <strong>.crepus</strong> DSL, native and WASM renderers, and the <code>crepus</code> CLI. Same Markdown sources as the <a href="https://github.com/semitechnological/crepuscularity/tree/main/docs">repository <code>docs/</code> folder</a>.</p>
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
        "webext" => "Manifest V3 extensions from .crepus and Rust.",
        "WEB_BUILD_MIGRATION" => "Notes for migrating WASM static site builds.",
        "CREPUS_WEB_IMPLEMENTATION_SPEC" => {
            "Canonical web, WASM, and extension implementation spec."
        }
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
    format!(
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
      background: var(--surface);
      color: var(--text);
      font-family: Inter, system-ui, sans-serif;
      -webkit-font-smoothing: antialiased;
      line-height: 1.6;
    }}
    a {{ color: var(--accent-soft); text-decoration: none; }}
    a:hover {{ text-decoration: underline; }}
    .doc-shell {{
      display: grid;
      grid-template-columns: minmax(200px, 260px) 1fr;
      min-height: 100vh;
    }}
    @media (max-width: 860px) {{
      .doc-shell {{ grid-template-columns: 1fr; }}
      aside {{ border-bottom: 1px solid var(--border); border-right: none; }}
    }}
    aside {{
      padding: 1.5rem 1.25rem;
      border-right: 1px solid var(--border);
      background: color-mix(in srgb, var(--surface) 92%, #1a1a1e);
    }}
    .brand {{
      font-weight: 700;
      font-size: 0.95rem;
      letter-spacing: -0.02em;
      margin-bottom: 1rem;
      display: block;
      color: var(--text);
    }}
    .brand:hover {{ text-decoration: none; color: var(--accent-soft); }}
    .doc-nav {{
      list-style: none;
      padding: 0;
      margin: 0;
      font-size: 0.875rem;
    }}
    .doc-nav li {{ margin: 0.35rem 0; }}
    .doc-nav a {{ color: var(--muted); display: inline-block; }}
    .doc-nav a:hover {{ color: var(--text); }}
    .doc-nav a.active {{ color: var(--accent-soft); font-weight: 600; }}
    .doc-main {{
      padding: 2rem 2.5rem 4rem;
      max-width: 52rem;
    }}
    .doc-main.doc-main--wide {{
      max-width: 72rem;
    }}
    .prose h1 {{ font-size: 2rem; font-weight: 700; margin: 0 0 1rem; letter-spacing: -0.03em; line-height: 1.2; }}
    .prose h2 {{ font-size: 1.35rem; font-weight: 600; margin: 2rem 0 0.75rem; letter-spacing: -0.02em; }}
    .prose h3 {{ font-size: 1.05rem; font-weight: 600; margin: 1.5rem 0 0.5rem; }}
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
      background: #18181b;
      border: 1px solid var(--border);
      border-radius: 10px;
      padding: 1rem 1.1rem;
      overflow-x: auto;
      margin: 1rem 0;
    }}
    .prose pre code {{
      background: none;
      padding: 0;
      font-size: 0.82rem;
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
    .prose a {{ color: var(--accent-soft); }}
    .docs-landing .lede {{
      font-size: 1.05rem;
      max-width: 48rem;
      margin: 0 0 2rem;
      color: color-mix(in srgb, var(--text) 85%, var(--muted));
    }}
    .docs-landing .lede code {{
      font-family: "JetBrains Mono", monospace;
      font-size: 0.9em;
    }}
    .doc-grid {{
      display: grid;
      gap: 1rem;
      grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    }}
    .doc-card {{
      display: block;
      padding: 1.25rem 1.35rem;
      border: 1px solid var(--border);
      border-radius: 12px;
      background: color-mix(in srgb, var(--surface) 88%, #1f1f23);
      transition: border-color 0.15s ease, background 0.15s ease;
    }}
    .doc-card:hover {{
      border-color: color-mix(in srgb, var(--accent) 45%, var(--border));
      background: color-mix(in srgb, var(--surface) 82%, #252529);
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
      margin-top: 2.5rem;
      font-size: 0.875rem;
      color: var(--muted);
    }}
  </style>
</head>
<body>
  <div class="doc-shell">
    <aside>
      <a class="brand" href="../index.html">{esc_site}</a>
      <nav aria-label="Documentation">
        {nav}
      </nav>
    </aside>
    <main class="{main_cls}">
      {main_inner}
    </main>
  </div>
</body>
</html>"#,
        esc_site = esc_site,
        esc_page = esc_page,
        main_cls = main_cls,
        main_inner = main_inner,
        nav = nav,
        a = theme.accent,
        as = theme.accent_soft,
        s = theme.surface,
        t = theme.text,
        m = theme.muted,
        b = theme.border,
    )
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
