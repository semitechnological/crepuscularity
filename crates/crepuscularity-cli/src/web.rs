//! `crepus web` — `.crepus`-first static sites (WASM runtime) + dev server.
//!
//! Production builds mirror `crepus webext`: compile the site `runtime/` crate to
//! `wasm32-unknown-unknown`, run `wasm-bindgen`, ship `crepus-bundle.json` + a thin HTML shell.
//! The default WASM entrypoint calls `crepuscularity_web::render_bundle`; sites that need Rust
//! context can replace that with `render_from_files` + a hand-built `TemplateContext`.

use console::style;
use crepuscularity_core::preprocess::{
    google_fonts_head_markup, merge_unique_font_families, strip_indent_decorators,
};
use crepuscularity_core::{DriverCache, Fingerprint};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::ui;
use crate::wasm_bundle::{cargo_build_wasm32, find_wasm_file, run_wasm_bindgen, wasm_release_dirs};
use crate::web_serve::ServeOptions;

const WEB_INDEX_HTML: &str = include_str!("../assets/web/index.html");
const WEB_APP_JS: &str = include_str!("../assets/web/app.js");

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
            if b.legacy_site_json {
                build_site_legacy(&b);
            } else {
                build_site_wasm(&b);
            }
        }

        Some("site-json") => {
            let site_dir = parse_site_dir(&args[1..]);
            print_site_json(&site_dir);
        }

        Some("serve") => {
            let opts = parse_serve_args(&args[1..]);
            crate::web_serve::run(opts);
        }

        Some("build-full") => {
            let b = parse_build_full_args(&args[1..]);
            run_build_full(&b);
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
    eprintln!(
        "{}",
        style("crepus web site-json: deprecated — use .crepus + optional site.json for SEO only.")
            .yellow()
    );
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
    eprintln!(
        "{}",
        style(".crepus templates + WASM runtime (same model as crepus webext)").dim()
    );
    eprintln!();
    eprintln!("{}", style("COMMANDS").dim());
    eprintln!(
        "  {}  {}",
        style("new <name>                     ").green(),
        style("scaffold index.crepus + runtime/ + web.toml").dim()
    );
    eprintln!(
        "  {}  {}",
        style("build [--site DIR] ...         ").green(),
        style("emit dist/ (HTML shell + WASM + crepus-bundle.json)").dim()
    );
    eprintln!(
        "  {}  {}",
        style("site-json [--site DIR]         ").green(),
        style("pretty-print site.json (deprecated)").dim()
    );
    eprintln!(
        "  {}  {}",
        style("serve [--site DIR] [--port N]  ").green(),
        style("live-reload dev server for .crepus files").dim()
    );
    eprintln!(
        "  {}  {}",
        style("build-full [OPTIONS]           ").green(),
        style("parallel .crepus render + optional wasm/server cargo builds").dim()
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
        style("site root (default: cwd)").dim()
    );
    eprintln!(
        "  {}  {}",
        style("--out-dir, -o DIR       ").green(),
        style("output directory (default: SITE/dist)").dim()
    );
    eprintln!(
        "  {}  {}",
        style("--entry FILE            ").green(),
        style("entry .crepus path relative to site (default: index.crepus)").dim()
    );
    eprintln!(
        "  {}  {}",
        style("--legacy-site-json      ").green(),
        style("old pipeline: HTML from structured site.json only").dim()
    );
    eprintln!(
        "  {}  {}",
        style("--json FILE             ").green(),
        style("(legacy) explicit site.json path").dim()
    );
    eprintln!(
        "  {}  {}",
        style("[FILE | -]              ").green(),
        style("(legacy) site.json path or stdin").dim()
    );
    eprintln!(
        "  {}  {}",
        style("-o FILE.html            ").green(),
        style("(legacy) single HTML output with --legacy-site-json").dim()
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

    let rt_name = slug.replace('-', "_");
    std::fs::create_dir_all(base.join("runtime/src")).unwrap();
    std::fs::create_dir_all(base.join("dist")).unwrap();

    let web_toml = format!(
        r#"[site]
name = "{name}"
version = "0.1.0"
description = "Crepus static site (.crepus + WASM)"

# Dev:  crepus web serve --site .
# Ship: crepus web build --site .
"#
    );
    std::fs::write(base.join("web.toml"), web_toml).unwrap();

    let index_crepus = r#"div w-full min-h-screen bg-zinc-950 text-zinc-50 p-8 flex flex-col gap-4
 div text-3xl font-bold
  "Hello from .crepus"
 div text-zinc-400 max-w-xl
  "This page is rendered in the browser by the same pipeline as crepus web serve — wasm32 + crepus-bundle.json."
"#;
    std::fs::write(base.join("index.crepus"), index_crepus).unwrap();

    let cargo_toml = format!(
        r#"[package]
name = "{rt_name}_runtime"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
crepuscularity-web = "0.3"
wasm-bindgen = "0.2"

[workspace]
"#
    );
    std::fs::write(base.join("runtime/Cargo.toml"), cargo_toml).unwrap();

    let lib_rs = r#"use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn crepus_render(bundle_json: &str) -> Result<String, JsValue> {
    crepuscularity_web::render_bundle(bundle_json).map_err(|e| JsValue::from_str(&e))
}
"#;
    std::fs::write(base.join("runtime/src/lib.rs"), lib_rs).unwrap();

    eprintln!(
        "\n{} created {}",
        ui::ok(),
        style(format!("{slug}/")).cyan().bold()
    );
    eprintln!();
    eprintln!("{}", style("Next steps:").dim());
    eprintln!("  cd {slug}");
    eprintln!("  crepus web serve --site .");
    eprintln!("  crepus web build --site .");
    eprintln!(
        "  {}",
        style("# Optional: site.json for <title> / meta only (not page structure)").dim()
    );
    ui::done_in(t0.elapsed());
}

// ── WASM build ───────────────────────────────────────────────────────────────

struct WasmBuildArgs {
    site_dir: PathBuf,
    out_dir: PathBuf,
    entry: String,
}

struct WebBuildArgs {
    site_dir: Option<PathBuf>,
    json_path: Option<PathBuf>,
    positional: Option<String>,
    out: Option<PathBuf>,
    out_dir: Option<PathBuf>,
    entry: Option<String>,
    legacy_site_json: bool,
}

fn parse_build_args(args: &[String]) -> WebBuildArgs {
    let legacy_site_json = args.iter().any(|a| a == "--legacy-site-json");

    let mut site_dir = None;
    let mut json_path = None;
    let mut out = None;
    let mut out_dir = None;
    let mut entry = None;
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
            "--out-dir" => {
                if let Some(p) = args.get(i + 1) {
                    out_dir = Some(PathBuf::from(p));
                    i += 1;
                }
            }
            "--entry" => {
                if let Some(p) = args.get(i + 1) {
                    entry = Some(p.clone());
                    i += 1;
                }
            }
            "--legacy-site-json" => {}
            "-o" | "--output" | "--out" => {
                if let Some(p) = args.get(i + 1) {
                    let pb = PathBuf::from(p);
                    if legacy_site_json {
                        if pb.extension().is_some_and(|e| e == "html") {
                            out = Some(pb);
                        } else {
                            out_dir = Some(pb);
                        }
                    } else {
                        out_dir = Some(pb);
                    }
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
        out_dir,
        entry,
        legacy_site_json,
    }
}

fn resolve_wasm_build_args(args: &WebBuildArgs) -> WasmBuildArgs {
    let site_dir = args
        .site_dir
        .clone()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    let out_dir = args
        .out_dir
        .clone()
        .or_else(|| args.out.clone())
        .unwrap_or_else(|| site_dir.join("dist"));

    let entry = args
        .entry
        .clone()
        .unwrap_or_else(|| "index.crepus".to_string());

    WasmBuildArgs {
        site_dir,
        out_dir,
        entry,
    }
}

fn build_site_wasm(cli: &WebBuildArgs) {
    let t0 = Instant::now();
    let b = resolve_wasm_build_args(cli);
    let runtime_dir = b.site_dir.join("runtime");
    if !runtime_dir.join("Cargo.toml").is_file() {
        ui::error(&format!(
            "no runtime/Cargo.toml under {} — run `crepus web new <name>` or copy examples/web-site",
            b.site_dir.display()
        ));
    }

    eprintln!(
        "{} building WASM site → {}",
        style("crepus web").dim(),
        style(b.out_dir.display().to_string()).cyan()
    );
    eprintln!();

    let mut files: HashMap<String, String> = HashMap::new();
    load_all_crepus(&b.site_dir, &b.site_dir, &mut files);
    if files.is_empty() {
        ui::error(&format!("no .crepus files under {}", b.site_dir.display()));
    }

    if !files.contains_key(&b.entry) {
        ui::error(&format!(
            "entry {:?} not found in virtual file map (keys: {:?})",
            b.entry,
            files.keys().take(5).collect::<Vec<_>>()
        ));
    }

    let bundle = json!({
        "entry": b.entry,
        "files": files,
    });
    let bundle_str = serde_json::to_string(&bundle).unwrap_or_else(|e| {
        ui::error(&format!("serialize bundle: {e}"));
    });

    let head = load_site_head(&b.site_dir);
    let google_fonts = merged_site_google_fonts(&b.site_dir, &files);
    let vendor_dir = b.out_dir.join("vendor");
    let pkg_dir = b.out_dir.join("pkg");
    std::fs::create_dir_all(&vendor_dir).unwrap_or_else(|e| {
        ui::error(&format!("mkdir vendor: {e}"));
    });
    std::fs::create_dir_all(&pkg_dir).unwrap_or_else(|e| {
        ui::error(&format!("mkdir pkg: {e}"));
    });

    let _ = std::fs::create_dir_all(b.site_dir.join(".crepus-cache"));
    let cache = DriverCache::open(&b.site_dir);
    let fp = Fingerprint::new(&bundle_str, None, "web-wasm-bundle");
    let bundle_path = b.out_dir.join("crepus-bundle.json");
    let skip_bundle_write = bundle_path.is_file() && cache.is_up_to_date(&fp, &bundle_str);
    if !skip_bundle_write {
        std::fs::write(&bundle_path, &bundle_str).unwrap_or_else(|e| {
            ui::error(&format!("write {}: {e}", bundle_path.display()));
        });
        cache.record(&fp, &bundle_str);
    }

    copy_unocss(&vendor_dir);
    let index_html = render_index_html(&head, &google_fonts);
    std::fs::write(b.out_dir.join("index.html"), index_html).unwrap_or_else(|e| {
        ui::error(&format!("write index.html: {e}"));
    });
    std::fs::write(b.out_dir.join("app.js"), WEB_APP_JS).unwrap_or_else(|e| {
        ui::error(&format!("write app.js: {e}"));
    });

    let static_src = b.site_dir.join("static");
    if static_src.is_dir() {
        copy_dir_recursive(&static_src, &b.out_dir.join("static")).unwrap_or_else(|e| {
            ui::error(&format!("copy static/: {e}"));
        });
    }

    {
        let sp = ui::spinner("compiling site WASM (wasm32-unknown-unknown)");
        match cargo_build_wasm32(&runtime_dir) {
            Ok(()) => ui::spinner_ok(&sp, "WASM compiled"),
            Err(stderr) => {
                sp.finish_and_clear();
                eprintln!("  {} WASM compile failed", ui::err());
                for line in stderr.lines().take(20) {
                    eprintln!("    {}", style(line).dim());
                }
                ui::error("fix runtime compile errors (see above)");
            }
        }
    }

    let (workspace_target, local_target) = wasm_release_dirs(&b.site_dir, &runtime_dir);
    let wasm_path = find_wasm_file(&workspace_target)
        .or_else(|| find_wasm_file(&local_target))
        .unwrap_or_else(|| {
            ui::error("built .wasm not found under target/wasm32-unknown-unknown/release/");
        });

    {
        let sp = ui::spinner("wasm-bindgen");
        match run_wasm_bindgen(&wasm_path, &pkg_dir, "runtime") {
            Ok(()) => ui::spinner_ok(&sp, "pkg/runtime.js + runtime_bg.wasm"),
            Err(err) => {
                sp.finish_and_clear();
                if err.starts_with("wasm-bindgen:") {
                    ui::error("wasm-bindgen not found — install: cargo install wasm-bindgen-cli");
                }
                ui::error(&format!("wasm-bindgen: {err}"));
            }
        }
    }

    eprintln!(
        "\n{} wrote {}",
        ui::ok(),
        style(b.out_dir.display().to_string()).cyan()
    );
    eprintln!(
        "  {} open {}/index.html via a static server (fetch + WASM modules need HTTP)",
        ui::dim("→"),
        b.out_dir.display()
    );
    ui::done_in(t0.elapsed());
}

fn load_all_crepus(root: &Path, dir: &Path, map: &mut HashMap<String, String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip output and cargo targets
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if matches!(name, "dist" | "target" | ".git" | "node_modules") {
                continue;
            }
            load_all_crepus(root, &path, map);
        } else if path.extension().is_some_and(|e| e == "crepus") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let key = relative_key(root, &path);
                map.insert(key, content);
            }
        }
    }
}

fn relative_key(root: &Path, abs: &Path) -> String {
    abs.strip_prefix(root)
        .unwrap_or(abs)
        .to_string_lossy()
        .replace('\\', "/")
}

#[derive(Debug, Clone)]
struct SiteHead {
    page_title: String,
    description: String,
    og_image: Option<String>,
    extra_head_html: String,
    theme: ThemeCss,
}

impl Default for SiteHead {
    fn default() -> Self {
        Self {
            page_title: "Crepus site".into(),
            description: "Built with Crepuscularity".into(),
            og_image: None,
            extra_head_html: String::new(),
            theme: ThemeCss::default(),
        }
    }
}

#[derive(Debug, Clone)]
struct ThemeCss {
    accent: String,
    accent_soft: String,
    surface: String,
    text: String,
    muted: String,
    border: String,
}

impl Default for ThemeCss {
    fn default() -> Self {
        Self {
            accent: "#3b82f6".into(),
            accent_soft: "#60a5fa".into(),
            surface: "#09090b".into(),
            text: "#fafafa".into(),
            muted: "#a1a1aa".into(),
            border: "#27272a".into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SiteJsonPartial {
    business_name: Option<String>,
    title: Option<String>,
    seo: Option<SeoPartial>,
    theme: Option<ThemePartial>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeoPartial {
    title: Option<String>,
    description: Option<String>,
    og_image: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThemePartial {
    accent: Option<String>,
    accent_soft: Option<String>,
    surface: Option<String>,
    text: Option<String>,
    muted: Option<String>,
    border: Option<String>,
}

/// Merge `[site].google_fonts` / top-level `google_fonts` from `web.toml` with every `google-font` pragma in bundled `.crepus` files.
pub(crate) fn merged_site_google_fonts(
    site_dir: &Path,
    files: &HashMap<String, String>,
) -> Vec<String> {
    let mut collected = load_web_toml_google_fonts(site_dir);
    for content in files.values() {
        collected.extend(strip_indent_decorators(content).google_fonts);
    }
    merge_unique_font_families(collected)
}

fn load_web_toml_google_fonts(site_dir: &Path) -> Vec<String> {
    let path = site_dir.join("web.toml");
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(v) = raw.parse::<toml::Table>() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    if let Some(arr) = v.get("google_fonts").and_then(|x| x.as_array()) {
        for x in arr {
            if let Some(s) = x.as_str() {
                out.push(s.to_string());
            }
        }
    }
    if let Some(site) = v.get("site").and_then(|x| x.as_table()) {
        if let Some(arr) = site.get("google_fonts").and_then(|x| x.as_array()) {
            for x in arr {
                if let Some(s) = x.as_str() {
                    out.push(s.to_string());
                }
            }
        }
    }
    out
}

fn load_site_head(site_dir: &Path) -> SiteHead {
    let mut head = SiteHead::default();

    let path = site_dir.join("site.json");
    if let Ok(raw) = std::fs::read_to_string(&path) {
        if let Ok(partial) = serde_json::from_str::<SiteJsonPartial>(&raw) {
            let seo_title = partial.seo.as_ref().and_then(|s| s.title.clone());
            head.page_title = partial
                .title
                .clone()
                .or(seo_title)
                .or(partial.business_name.clone())
                .unwrap_or_else(|| head.page_title.clone());
            if let Some(seo) = &partial.seo {
                if let Some(d) = &seo.description {
                    head.description = d.clone();
                }
                head.og_image = seo.og_image.clone();
            }
            if let Some(t) = &partial.theme {
                let mut th = ThemeCss::default();
                if let Some(x) = &t.accent {
                    th.accent = x.clone();
                }
                if let Some(x) = &t.accent_soft {
                    th.accent_soft = x.clone();
                }
                if let Some(x) = &t.surface {
                    th.surface = x.clone();
                }
                if let Some(x) = &t.text {
                    th.text = x.clone();
                }
                if let Some(x) = &t.muted {
                    th.muted = x.clone();
                }
                if let Some(x) = &t.border {
                    th.border = x.clone();
                }
                head.theme = th;
            }
        }
    }

    let web_toml = site_dir.join("web.toml");
    if let Ok(raw) = std::fs::read_to_string(&web_toml) {
        if let Ok(v) = raw.parse::<toml::Table>() {
            if let Some(site) = v.get("site").and_then(|x| x.as_table()) {
                if let Some(name) = site.get("name").and_then(|x| x.as_str()) {
                    if head.page_title == "Crepus site" {
                        head.page_title = name.to_string();
                    }
                }
                if let Some(extra_head_html) = site.get("head_html").and_then(|x| x.as_str()) {
                    head.extra_head_html = extra_head_html.to_string();
                }
            }
        }
    }

    head
}

fn render_index_html(head: &SiteHead, google_fonts: &[String]) -> String {
    let og = head
        .og_image
        .as_ref()
        .map(|u| {
            format!(
                r#"  <meta property="og:image" content="{}">"#,
                escape_html_attr(u)
            )
        })
        .unwrap_or_default();
    let font_markup = google_fonts_head_markup(google_fonts);
    let body_font_css = google_fonts
        .first()
        .map(|n| {
            let q = n.trim().replace('\\', r"\\").replace('"', r#"\""#);
            format!(r#""{q}", system-ui, -apple-system, sans-serif"#)
        })
        .unwrap_or_else(|| "system-ui, -apple-system, sans-serif".to_string());
    let t = &head.theme;
    WEB_INDEX_HTML
        .replace("__CREPUS_TITLE__", &escape_html_attr(&head.page_title))
        .replace("__CREPUS_DESC__", &escape_html_attr(&head.description))
        .replace("__CREPUS_OG__", &og)
        .replace("__CREPUS_GOOGLE_FONTS__", &font_markup)
        .replace("__CREPUS_EXTRA_HEAD__", &head.extra_head_html)
        .replace("__CREPUS_BODY_FONT__", &body_font_css)
        .replace("__THEME_ACCENT__", &escape_html_attr(&t.accent))
        .replace("__THEME_ACCENT_SOFT__", &escape_html_attr(&t.accent_soft))
        .replace("__THEME_SURFACE__", &escape_html_attr(&t.surface))
        .replace("__THEME_TEXT__", &escape_html_attr(&t.text))
        .replace("__THEME_MUTED__", &escape_html_attr(&t.muted))
        .replace("__THEME_BORDER__", &escape_html_attr(&t.border))
}

fn escape_html_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn copy_unocss(vendor_dir: &Path) {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../crepuscularity-webext/assets/vendor/unocss.js");
    let dst = vendor_dir.join("unocss.js");
    if src.is_file() {
        let _ = std::fs::copy(&src, &dst);
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let p = entry.path();
        let name = entry.file_name();
        if p.is_dir() {
            copy_dir_recursive(&p, &dst.join(name))?;
        } else {
            let _ = std::fs::copy(&p, dst.join(name))?;
        }
    }
    Ok(())
}

// ── Legacy site.json build ────────────────────────────────────────────────────

fn build_site_legacy(cli: &WebBuildArgs) {
    let t0 = Instant::now();
    eprintln!(
        "{} {}",
        style("crepus web build").dim(),
        style("--legacy-site-json").yellow()
    );
    eprintln!();

    let json_str = if cli.positional.as_deref() == Some("-") {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .unwrap_or_else(|e| ui::error(&format!("stdin: {e}")));
        buf
    } else if let Some(p) = &cli.json_path {
        std::fs::read_to_string(p).unwrap_or_else(|e| {
            ui::error(&format!("read {}: {e}", p.display()));
        })
    } else if let Some(pos) = &cli.positional {
        std::fs::read_to_string(pos).unwrap_or_else(|e| {
            ui::error(&format!("read {pos}: {e}"));
        })
    } else if let Some(dir) = &cli.site_dir {
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
            ui::error("legacy build: pass site.json, `-`, or --site with site.json");
        }
    } else {
        ui::error("cannot resolve current directory");
    };

    let site: SiteRoot = serde_json::from_str(&json_str).unwrap_or_else(|e| {
        ui::error(&format!("parse site.json: {e}"));
    });

    let html = render_site_html(&site);

    let out_path = resolve_legacy_out(cli);
    if let Some(out_path) = out_path {
        let project_root = cli
            .site_dir
            .clone()
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        let cache = DriverCache::open(&project_root);
        let fp = Fingerprint::new(&json_str, None, "render");

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
        std::fs::write(&out_path, &html).unwrap_or_else(|e| {
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

fn resolve_legacy_out(cli: &WebBuildArgs) -> Option<PathBuf> {
    cli.out.clone().or_else(|| cli.out_dir.clone())
}

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

fn render_site_html(site: &SiteRoot) -> String {
    let page_title = site.title.clone().unwrap_or_else(|| site.seo.title.clone());

    let mut body = String::new();
    for el in &site.elements {
        body.push_str(&render_element(el));
    }

    let desc_escaped = escape_html_legacy(&site.seo.description);
    let og = site
        .seo
        .og_image
        .as_ref()
        .map(|u| {
            format!(
                r#"  <meta property="og:image" content="{}">"#,
                escape_html_legacy(u)
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
        page_title = escape_html_legacy(&page_title),
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
            escape_html_attr_legacy(&p.primary.href)
        )
    } else {
        format!(
            r#" class="btn btn--primary" href="{}""#,
            escape_html_attr_legacy(&p.primary.href)
        )
    };

    let secondary_block = if let Some(sec) = &p.secondary {
        let ext = sec.external.unwrap_or(false);
        let attrs = if ext {
            format!(
                r#" class="btn" href="{}" rel="noopener noreferrer" target="_blank""#,
                escape_html_attr_legacy(&sec.href)
            )
        } else {
            format!(
                r#" class="btn" href="{}""#,
                escape_html_attr_legacy(&sec.href)
            )
        };
        format!(
            r#"<a{attrs}>{label}</a>"#,
            label = escape_html_legacy(&sec.label)
        )
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
        eyebrow = escape_html_legacy(&p.eyebrow),
        headline = escape_html_legacy(&p.headline),
        sub = escape_html_legacy(&p.subheadline),
        primary_attrs = primary_attrs,
        primary_label = escape_html_legacy(&p.primary.label),
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
            icon = escape_html_legacy(&icon),
            title = escape_html_legacy(&item.title),
            desc = escape_html_legacy(&item.description),
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
        eyebrow = escape_html_legacy(&p.eyebrow),
        title = escape_html_legacy(&p.title),
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

// ── build-full ───────────────────────────────────────────────────────────────

struct BuildFullArgs {
    site_dir: PathBuf,
    wasm: bool,
    server: bool,
}

fn parse_build_full_args(args: &[String]) -> BuildFullArgs {
    let mut site_dir = std::env::current_dir().unwrap();
    let mut wasm = false;
    let mut server = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--site" => {
                if let Some(p) = args.get(i + 1) {
                    site_dir = PathBuf::from(p);
                    i += 1;
                }
            }
            "--wasm" => wasm = true,
            "--server" => server = true,
            _ => {}
        }
        i += 1;
    }
    BuildFullArgs {
        site_dir,
        wasm,
        server,
    }
}

fn run_build_full(args: &BuildFullArgs) {
    let t0 = Instant::now();
    eprintln!("{}", style("crepus web build-full").dim());
    eprintln!();

    // L1 Rayon: collect all .crepus files in the site dir.
    let mut crepus_files: HashMap<String, String> = HashMap::new();
    let mut entries: Vec<String> = Vec::new();

    match std::fs::read_dir(&args.site_dir) {
        Ok(dir) => {
            for entry in dir.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("crepus") {
                    let key = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        entries.push(key.clone());
                        crepus_files.insert(key, content);
                    }
                }
            }
        }
        Err(e) => {
            ui::error(&format!("read dir {}: {e}", args.site_dir.display()));
        }
    }

    let ctx = crepuscularity_core::TemplateContext::new();
    let entry_refs: Vec<&str> = entries.iter().map(|s| s.as_str()).collect();
    let results = crepuscularity_web::par_render_from_files(&crepus_files, &entry_refs, &ctx);

    for (entry, result) in &results {
        match result {
            Ok(_html) => ui::step(&format!("rendered {entry}")),
            Err(e) => ui::warning(&format!("error rendering {entry}: {e}")),
        }
    }

    // Derive site name from directory for cargo build targets.
    let site_name = args
        .site_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // L2 overlapped cargo builds — spawn both before waiting on either.
    let mut wasm_child = None;
    let mut server_child = None;

    if args.wasm {
        let target = format!("{site_name}-runtime");
        ui::step(&format!(
            "spawning cargo build --target wasm32-unknown-unknown -p {target}"
        ));
        match std::process::Command::new("cargo")
            .args(["build", "--target", "wasm32-unknown-unknown", "-p", &target])
            .spawn()
        {
            Ok(child) => wasm_child = Some(child),
            Err(e) => ui::warning(&format!("could not spawn wasm build: {e}")),
        }
    }

    if args.server {
        let target = format!("{site_name}-server");
        ui::step(&format!("spawning cargo build -p {target}"));
        match std::process::Command::new("cargo")
            .args(["build", "-p", &target])
            .spawn()
        {
            Ok(child) => server_child = Some(child),
            Err(e) => ui::warning(&format!("could not spawn server build: {e}")),
        }
    }

    // Wait on both (order doesn't matter; both were already spawned).
    if let Some(mut child) = wasm_child {
        match child.wait() {
            Ok(status) if status.success() => ui::step("wasm build complete"),
            Ok(status) => ui::warning(&format!("wasm build exited with {status}")),
            Err(e) => ui::warning(&format!("wasm build wait error: {e}")),
        }
    }
    if let Some(mut child) = server_child {
        match child.wait() {
            Ok(status) if status.success() => ui::step("server build complete"),
            Ok(status) => ui::warning(&format!("server build exited with {status}")),
            Err(e) => ui::warning(&format!("server build wait error: {e}")),
        }
    }

    ui::success(&format!("{} files rendered", results.len()));
    ui::done_in(t0.elapsed());
}

fn escape_html_legacy(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_html_attr_legacy(s: &str) -> String {
    escape_html_legacy(s)
}
