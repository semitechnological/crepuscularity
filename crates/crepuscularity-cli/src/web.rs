//! Static site commands for `crepus web` — structured `site.json` to a single HTML page.

use console::style;
use crepuscularity_core::{DriverCache, Fingerprint};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::ui;
use crate::web_serve::ServeOptions;

// ── Entry point ──────────────────────────────────────────────────────────────

pub fn run(args: &[String]) {
    match args.first().map(|s| s.as_str()) {
        Some("new") => {
            let name = args.get(1).map(|s| s.as_str()).unwrap_or_else(|| {
                ui::error("Usage: crepus web new <name>");
            });
            scaffold_site(name);
        }

        Some("build") => {
            let b = parse_build_args(&args[1..]);
            build_site(&b);
        }

        Some("site-json") => {
            let site_dir = parse_site_dir(&args[1..]);
            print_site_json(&site_dir);
        }

        Some("serve") => {
            let opts = parse_serve_args(&args[1..]);
            crate::web_serve::run(opts);
        }

        _ => print_web_usage(),
    }
}

fn parse_serve_args(args: &[String]) -> ServeOptions {
    let mut site_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut port: u16 = 4000;
    let mut entry = "index.crepus".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--site" => {
                if let Some(p) = args.get(i + 1) {
                    site_dir = PathBuf::from(p);
                    i += 1;
                }
            }
            "--port" => {
                if let Some(p) = args.get(i + 1) {
                    port = p.parse().unwrap_or(4000);
                    i += 1;
                }
            }
            "--entry" => {
                if let Some(e) = args.get(i + 1) {
                    entry = e.clone();
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    ServeOptions {
        site_dir,
        port,
        entry,
    }
}

fn parse_site_dir(args: &[String]) -> PathBuf {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--site" {
            if let Some(p) = args.get(i + 1) {
                return PathBuf::from(p);
            }
        }
        i += 1;
    }
    std::env::current_dir().unwrap()
}

fn print_site_json(site_dir: &Path) {
    let path = site_dir.join("site.json");
    if !path.exists() {
        ui::error(&format!("site.json not found in {}", site_dir.display()));
    }
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        ui::error(&format!("read {}: {e}", path.display()));
    });
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap_or_else(|e| {
        ui::error(&format!("parse {}: {e}", path.display()));
    });
    let pretty = serde_json::to_string_pretty(&v).unwrap_or_else(|e| {
        ui::error(&format!("serialize: {e}"));
    });
    println!("{pretty}");
}

fn print_web_usage() {
    eprintln!("{}", style("crepus web").cyan().bold());
    eprintln!("{}", style("Static sites from structured site.json").dim());
    eprintln!();
    eprintln!("{}", style("COMMANDS").dim());
    eprintln!(
        "  {}  {}",
        style("new <name>                     ").green(),
        style("scaffold site.json project + web.toml").dim()
    );
    eprintln!(
        "  {}  {}",
        style("build [--site DIR] ...         ").green(),
        style("render HTML from site.json").dim()
    );
    eprintln!(
        "  {}  {}",
        style("site-json [--site DIR]         ").green(),
        style("print site.json (pretty)").dim()
    );
    eprintln!(
        "  {}  {}",
        style("serve [--site DIR] [--port N]  ").green(),
        style("live-reload dev server for .crepus files").dim()
    );
    eprintln!();
    eprintln!("{}", style("SERVE ARGS").dim());
    eprintln!(
        "  {}  {}",
        style("--site DIR             ").green(),
        style("directory of .crepus files (default: cwd)").dim()
    );
    eprintln!(
        "  {}  {}",
        style("--port N               ").green(),
        style("HTTP port (default: 4000)").dim()
    );
    eprintln!(
        "  {}  {}",
        style("--entry FILE           ").green(),
        style("entry template (default: index.crepus)").dim()
    );
    eprintln!();
    eprintln!("{}", style("BUILD ARGS").dim());
    eprintln!(
        "  {}  {}",
        style("--site DIR              ").green(),
        style("use DIR/site.json").dim()
    );
    eprintln!(
        "  {}  {}",
        style("--json FILE             ").green(),
        style("explicit site.json path").dim()
    );
    eprintln!(
        "  {}  {}",
        style("[FILE | -]              ").green(),
        style("input path or `-` for stdin").dim()
    );
    eprintln!(
        "  {}  {}",
        style("-o, --output, --out FILE").green(),
        style("write HTML (default: stdout)").dim()
    );
}

// ── scaffold ─────────────────────────────────────────────────────────────────

fn scaffold_site(name: &str) {
    let t0 = Instant::now();

    let slug = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>();

    let base = PathBuf::from(&slug);
    if base.exists() {
        ui::error(&format!("directory already exists: {slug}"));
    }

    std::fs::create_dir_all(base.join("dist")).unwrap();

    let web_toml = format!(
        r#"[site]
name = "{name}"
version = "0.1.0"
description = "Static site generated from site.json"

# Build with: crepus web build --site {slug} -o dist/index.html
"#
    );
    std::fs::write(base.join("web.toml"), web_toml).unwrap();

    let starter = starter_site_json(name);
    std::fs::write(
        base.join("site.json"),
        serde_json::to_string_pretty(&starter).unwrap(),
    )
    .unwrap();

    eprintln!(
        "\n{} created {}",
        ui::ok(),
        style(format!("{slug}/")).cyan().bold()
    );
    eprintln!();
    eprintln!("{}", style("Next steps:").dim());
    eprintln!("  cd {slug}");
    eprintln!("  crepus web build --site . -o dist/index.html");
    ui::done_in(t0.elapsed());
}

fn starter_site_json(business_name: &str) -> SiteRoot {
    SiteRoot {
        business_name: business_name.to_string(),
        title: None,
        domain: None,
        seo: Seo {
            title: business_name.to_string(),
            description: format!("Welcome to {business_name}"),
            og_image: None,
        },
        theme: Theme {
            accent: "#3b82f6".to_string(),
            accent_soft: "#60a5fa".to_string(),
            surface: "#09090b".to_string(),
            text: "#fafafa".to_string(),
            muted: "#a1a1aa".to_string(),
            border: "#27272a".to_string(),
        },
        elements: vec![SiteElementRaw {
            ty: "hero".to_string(),
            id: "hero-1".to_string(),
            props: serde_json::to_value(HeroProps {
                eyebrow: "Welcome".to_string(),
                headline: format!("Meet {business_name}"),
                subheadline: "Ship a polished landing page from structured data.".to_string(),
                primary: Cta {
                    label: "Get started".to_string(),
                    href: "#contact".to_string(),
                    external: None,
                },
                secondary: None,
                media: None,
            })
            .unwrap(),
        }],
        analytics: None,
        turnstile: None,
    }
}

// ── site.json model ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SiteRoot {
    business_name: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    domain: Option<String>,
    seo: Seo,
    theme: Theme,
    elements: Vec<SiteElementRaw>,
    #[serde(default)]
    analytics: Option<serde_json::Value>,
    #[serde(default)]
    turnstile: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Seo {
    title: String,
    description: String,
    #[serde(default)]
    og_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Theme {
    accent: String,
    accent_soft: String,
    surface: String,
    text: String,
    muted: String,
    border: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SiteElementRaw {
    #[serde(rename = "type")]
    ty: String,
    id: String,
    props: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HeroProps {
    eyebrow: String,
    headline: String,
    subheadline: String,
    primary: Cta,
    #[serde(default)]
    secondary: Option<Cta>,
    #[serde(default)]
    media: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Cta {
    label: String,
    href: String,
    #[serde(default)]
    external: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FeatureGridProps {
    eyebrow: String,
    title: String,
    items: Vec<FeatureItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FeatureItem {
    icon: String,
    title: String,
    description: String,
}

// ── build ────────────────────────────────────────────────────────────────────

struct WebBuildArgs {
    site_dir: Option<PathBuf>,
    json_path: Option<PathBuf>,
    /// First non-flag token (`-` means stdin).
    positional: Option<String>,
    out: Option<PathBuf>,
}

fn parse_build_args(args: &[String]) -> WebBuildArgs {
    let mut site_dir = None;
    let mut json_path = None;
    let mut out = None;
    let mut positional = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--site" => {
                if let Some(p) = args.get(i + 1) {
                    site_dir = Some(PathBuf::from(p));
                    i += 1;
                }
            }
            "--json" => {
                if let Some(p) = args.get(i + 1) {
                    json_path = Some(PathBuf::from(p));
                    i += 1;
                }
            }
            "-o" | "--output" | "--out" => {
                if let Some(p) = args.get(i + 1) {
                    out = Some(PathBuf::from(p));
                    i += 1;
                }
            }
            "-" if positional.is_none() => {
                positional = Some("-".to_string());
            }
            s if !s.starts_with('-') && positional.is_none() => {
                positional = Some(s.to_string());
            }
            _ => {}
        }
        i += 1;
    }

    WebBuildArgs {
        site_dir,
        json_path,
        positional,
        out,
    }
}

fn build_site(args: &WebBuildArgs) {
    let t0 = Instant::now();
    eprintln!("{} building static HTML", style("crepus web").dim());
    eprintln!();

    let json_str = if args.positional.as_deref() == Some("-") {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .unwrap_or_else(|e| ui::error(&format!("stdin: {e}")));
        buf
    } else if let Some(p) = &args.json_path {
        std::fs::read_to_string(p).unwrap_or_else(|e| {
            ui::error(&format!("read {}: {e}", p.display()));
        })
    } else if let Some(pos) = &args.positional {
        std::fs::read_to_string(pos).unwrap_or_else(|e| {
            ui::error(&format!("read {pos}: {e}"));
        })
    } else if let Some(dir) = &args.site_dir {
        let p = dir.join("site.json");
        std::fs::read_to_string(&p).unwrap_or_else(|e| {
            ui::error(&format!("read {}: {e}", p.display()));
        })
    } else if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join("site.json");
        if p.exists() {
            std::fs::read_to_string(&p).unwrap_or_else(|e| {
                ui::error(&format!("read {}: {e}", p.display()));
            })
        } else {
            ui::error(
                "no input: pass site.json path, `-` for stdin, or --site / --json (see crepus web)",
            );
        }
    } else {
        ui::error("cannot resolve current directory");
    };

    let site: SiteRoot = serde_json::from_str(&json_str).unwrap_or_else(|e| {
        ui::error(&format!("parse site.json: {e}"));
    });

    let html = render_site_html(&site);

    if let Some(out_path) = &args.out {
        // Phase 2: skip write when output is unchanged (Turbopack-style).
        let project_root = args
            .site_dir
            .clone()
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        let cache = DriverCache::open(&project_root);
        let fp = Fingerprint::new(&json_str, None, "render");

        // Only skip the write when the output file already exists on disk and
        // the cache confirms its content is unchanged.  If the file is missing
        // (e.g. cleaned up by a test or `git clean`) we always write.
        if out_path.exists() && cache.is_up_to_date(&fp, &html) {
            eprintln!(
                "\n{} {} (no change — skipped write)",
                ui::ok(),
                style(out_path.display().to_string()).dim()
            );
            ui::done_in(t0.elapsed());
            return;
        }

        if let Some(parent) = out_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).unwrap_or_else(|e| {
                    ui::error(&format!("create {}: {e}", parent.display()));
                });
            }
        }
        std::fs::write(out_path, &html).unwrap_or_else(|e| {
            ui::error(&format!("write {}: {e}", out_path.display()));
        });
        cache.record(&fp, &html);
        eprintln!(
            "\n{} wrote {}",
            ui::ok(),
            style(out_path.display().to_string()).cyan()
        );
    } else {
        print!("{html}");
    }

    ui::done_in(t0.elapsed());
}

fn render_site_html(site: &SiteRoot) -> String {
    let page_title = site.title.clone().unwrap_or_else(|| site.seo.title.clone());

    let mut body = String::new();
    for el in &site.elements {
        body.push_str(&render_element(el));
    }

    let desc_escaped = escape_html(&site.seo.description);
    let og = site
        .seo
        .og_image
        .as_ref()
        .map(|u| {
            format!(
                r#"  <meta property="og:image" content="{}">"#,
                escape_html(u)
            )
        })
        .unwrap_or_default();

    let t = &site.theme;
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{page_title}</title>
  <meta name="description" content="{desc_escaped}">
{og}
  <style>
:root {{
  --accent: {accent};
  --accent-soft: {accent_soft};
  --surface: {surface};
  --text: {text};
  --muted: {muted};
  --border: {border};
}}
* {{ box-sizing: border-box; }}
body {{
  margin: 0;
  font-family: system-ui, -apple-system, Segoe UI, Roboto, Ubuntu, sans-serif;
  background: var(--surface);
  color: var(--text);
  line-height: 1.6;
}}
a {{ color: var(--accent-soft); text-decoration: none; }}
a:hover {{ text-decoration: underline; }}
.page {{
  max-width: 72rem;
  margin: 0 auto;
  padding: 2rem 1.25rem 4rem;
}}
.hero {{
  padding: 3rem 0 2rem;
  border-bottom: 1px solid var(--border);
  margin-bottom: 3rem;
}}
.hero__eyebrow {{
  text-transform: uppercase;
  letter-spacing: .08em;
  font-size: .75rem;
  color: var(--muted);
  margin: 0 0 .75rem;
}}
.hero__headline {{
  font-size: clamp(2rem, 4vw, 3rem);
  font-weight: 700;
  margin: 0 0 .75rem;
  line-height: 1.15;
}}
.hero__sub {{
  font-size: 1.125rem;
  color: var(--muted);
  max-width: 42rem;
  margin: 0 0 1.5rem;
}}
.hero__actions {{ display: flex; flex-wrap: wrap; gap: .75rem; }}
.btn {{
  display: inline-flex;
  align-items: center;
  padding: .6rem 1.1rem;
  border-radius: .5rem;
  font-weight: 600;
  font-size: .95rem;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text);
}}
.btn--primary {{
  background: var(--accent);
  color: #fff;
  border-color: transparent;
}}
.btn--primary:hover {{ filter: brightness(1.08); }}
.features {{ margin-top: 2rem; }}
.features__eyebrow {{
  text-transform: uppercase;
  letter-spacing: .08em;
  font-size: .75rem;
  color: var(--muted);
  margin: 0 0 .5rem;
}}
.features__title {{
  font-size: clamp(1.5rem, 3vw, 2.25rem);
  font-weight: 700;
  margin: 0 0 2rem;
}}
.feature-grid {{
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(16rem, 1fr));
  gap: 1.25rem;
}}
.feature-card {{
  border: 1px solid var(--border);
  border-radius: .75rem;
  padding: 1.25rem;
  background: color-mix(in srgb, var(--surface) 92%, var(--border));
}}
.feature-card__icon {{
  font-size: 1.5rem;
  margin-bottom: .5rem;
  opacity: .9;
}}
.feature-card__title {{ font-weight: 600; margin: 0 0 .35rem; }}
.feature-card__desc {{ margin: 0; font-size: .9rem; color: var(--muted); }}
  </style>
</head>
<body>
  <div class="page">
{body}
  </div>
</body>
</html>
"#,
        page_title = escape_html(&page_title),
        accent = t.accent,
        accent_soft = t.accent_soft,
        surface = t.surface,
        text = t.text,
        muted = t.muted,
        border = t.border,
        desc_escaped = desc_escaped,
        og = og,
        body = body,
    )
}

fn render_element(el: &SiteElementRaw) -> String {
    match el.ty.as_str() {
        "hero" => match serde_json::from_value::<HeroProps>(el.props.clone()) {
            Ok(p) => render_hero(&p),
            Err(_) => String::new(),
        },
        "feature_grid" => match serde_json::from_value::<FeatureGridProps>(el.props.clone()) {
            Ok(p) => render_feature_grid(&p),
            Err(_) => String::new(),
        },
        _ => String::new(),
    }
}

fn render_hero(p: &HeroProps) -> String {
    let primary_ext = p.primary.external.unwrap_or(false);
    let primary_attrs = if primary_ext {
        format!(
            r#" class="btn btn--primary" href="{}" rel="noopener noreferrer" target="_blank""#,
            escape_html_attr(&p.primary.href)
        )
    } else {
        format!(
            r#" class="btn btn--primary" href="{}""#,
            escape_html_attr(&p.primary.href)
        )
    };

    let secondary_block = if let Some(sec) = &p.secondary {
        let ext = sec.external.unwrap_or(false);
        let attrs = if ext {
            format!(
                r#" class="btn" href="{}" rel="noopener noreferrer" target="_blank""#,
                escape_html_attr(&sec.href)
            )
        } else {
            format!(r#" class="btn" href="{}""#, escape_html_attr(&sec.href))
        };
        format!(r#"<a{attrs}>{label}</a>"#, label = escape_html(&sec.label))
    } else {
        String::new()
    };

    format!(
        r#"    <header class="hero">
      <p class="hero__eyebrow">{eyebrow}</p>
      <h1 class="hero__headline">{headline}</h1>
      <p class="hero__sub">{sub}</p>
      <div class="hero__actions">
        <a{primary_attrs}>{primary_label}</a>
        {secondary_block}
      </div>
    </header>
"#,
        eyebrow = escape_html(&p.eyebrow),
        headline = escape_html(&p.headline),
        sub = escape_html(&p.subheadline),
        primary_attrs = primary_attrs,
        primary_label = escape_html(&p.primary.label),
        secondary_block = secondary_block,
    )
}

fn render_feature_grid(p: &FeatureGridProps) -> String {
    let mut cards = String::new();
    for item in &p.items {
        let icon = feature_icon_char(&item.icon);
        cards.push_str(&format!(
            r#"        <article class="feature-card">
          <div class="feature-card__icon" aria-hidden="true">{icon}</div>
          <h3 class="feature-card__title">{title}</h3>
          <p class="feature-card__desc">{desc}</p>
        </article>
"#,
            icon = escape_html(&icon),
            title = escape_html(&item.title),
            desc = escape_html(&item.description),
        ));
    }

    format!(
        r#"    <section class="features">
      <p class="features__eyebrow">{eyebrow}</p>
      <h2 class="features__title">{title}</h2>
      <div class="feature-grid">
{cards}
      </div>
    </section>
"#,
        eyebrow = escape_html(&p.eyebrow),
        title = escape_html(&p.title),
        cards = cards,
    )
}

fn feature_icon_char(icon: &str) -> String {
    match icon {
        "terminal" => "⌘".to_string(),
        "code" => "</>".to_string(),
        "zap" => "⚡".to_string(),
        "globe" => "🌐".to_string(),
        "layout" => "▦".to_string(),
        "browser" => "🧭".to_string(),
        _ => "◆".to_string(),
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_html_attr(s: &str) -> String {
    escape_html(s)
}
