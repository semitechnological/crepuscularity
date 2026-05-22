//! `crepus web` — `.crepus`-first static sites (WASM runtime) + dev server.
//!
//! Production builds mirror `crepus webext`: compile the site `runtime/` crate to
//! `wasm32-unknown-unknown`, run `wasm-bindgen`, ship `crepus-bundle.json` + a thin HTML shell.
//! The default WASM entrypoint calls `crepuscularity_web::render_bundle`; sites that need Rust
//! context can replace that with `render_from_files` + a hand-built `TemplateContext`.

use console::style;
use crepuscularity_core::preprocess::{
    extract_head_block, google_fonts_head_markup, merge_unique_font_families,
    strip_indent_decorators,
};
use crepuscularity_core::{DriverCache, Fingerprint};
use serde::Deserialize;
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
            build_site_wasm(&b);
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

fn parse_target_and_manifest(args: &[String]) -> (Option<String>, Option<PathBuf>) {
    let mut target = None;
    let mut manifest = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--target" | "-t" => {
                if let Some(x) = args.get(i + 1) {
                    target = Some(x.clone());
                    i += 1;
                }
            }
            "--manifest" => {
                if let Some(x) = args.get(i + 1) {
                    manifest = Some(PathBuf::from(x));
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    (target, manifest)
}

fn parse_serve_args(args: &[String]) -> ServeOptions {
    let (target_id, manifest) = parse_target_and_manifest(args);

    let mut site_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut port: u16 = 4000;
    let mut entry = "index.crepus".to_string();
    let mut explicit_site = false;
    let mut entry_from_args = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--site" => {
                if let Some(p) = args.get(i + 1) {
                    site_dir = PathBuf::from(p);
                    explicit_site = true;
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
                    entry_from_args = true;
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    if !explicit_site {
        if let Some(targets) = crate::crepus_toml::load_web_targets(manifest) {
            let picked = crate::crepus_toml::resolve_pick(&targets, target_id.as_deref())
                .unwrap_or_else(|m| ui::error(&m));
            site_dir = picked.site_dir;
            if !entry_from_args {
                entry = picked.entry;
            }
        }
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
    std::env::current_dir().unwrap_or_else(|e| {
        ui::error(&format!("cannot determine current directory: {e}"));
    })
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
    eprintln!(
        "  {}  {}",
        style("--target, -t ID        ").green(),
        style("web target id from crepus.toml (when multiple)").dim()
    );
    eprintln!(
        "  {}  {}",
        style("--manifest FILE        ").green(),
        style("path to crepus.toml (skip walk-up)").dim()
    );
    eprintln!();
    eprintln!("{}", style("BUILD ARGS").dim());
    eprintln!(
        "  {}  {}",
        style("--site DIR              ").green(),
        style("site root (default: cwd, or crepus.toml target)").dim()
    );
    eprintln!(
        "  {}  {}",
        style("--out-dir, -o DIR       ").green(),
        style("output directory (default: SITE/dist or target out)").dim()
    );
    eprintln!(
        "  {}  {}",
        style("--entry FILE            ").green(),
        style("entry .crepus path relative to site (default: index.crepus)").dim()
    );
    eprintln!(
        "  {}  {}",
        style("--target, -t ID         ").green(),
        style("pick [[targets]] type=web entry (multiple sites)").dim()
    );
    eprintln!(
        "  {}  {}",
        style("--manifest FILE         ").green(),
        style("crepus.toml path").dim()
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
    std::fs::create_dir_all(base.join("runtime/src")).unwrap_or_else(|e| {
        ui::error(&format!("create runtime/src dir: {e}"));
    });
    std::fs::create_dir_all(base.join("dist")).unwrap_or_else(|e| {
        ui::error(&format!("create dist dir: {e}"));
    });

    let web_toml = format!(
        r#"[site]
name = "{name}"
version = "0.1.0"
description = "Crepus static site (.crepus + WASM)"

# Dev:  crepus web serve --site .
# Ship: crepus web build --site .
"#
    );
    std::fs::write(base.join("web.toml"), web_toml).unwrap_or_else(|e| {
        ui::error(&format!("write web.toml: {e}"));
    });

    let index_crepus = r#"div w-full min-h-screen bg-zinc-950 text-zinc-50 p-8 flex flex-col gap-4
 div text-3xl font-bold
  "Hello from .crepus"
 div text-zinc-400 max-w-xl
  "This page is rendered in the browser by the same pipeline as crepus web serve — wasm32 + crepus-bundle.json."
"#;
    std::fs::write(base.join("index.crepus"), index_crepus).unwrap_or_else(|e| {
        ui::error(&format!("write index.crepus: {e}"));
    });

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
    std::fs::write(base.join("runtime/Cargo.toml"), cargo_toml).unwrap_or_else(|e| {
        ui::error(&format!("write runtime/Cargo.toml: {e}"));
    });

    let lib_rs = r#"use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn crepus_render(bundle_json: &str) -> Result<String, JsValue> {
    crepuscularity_web::render_bundle(bundle_json).map_err(|e| JsValue::from_str(&e))
}
"#;
    std::fs::write(base.join("runtime/src/lib.rs"), lib_rs).unwrap_or_else(|e| {
        ui::error(&format!("write runtime/src/lib.rs: {e}"));
    });

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
    out_dir: Option<PathBuf>,
    entry: Option<String>,
    target_id: Option<String>,
    manifest: Option<PathBuf>,
}

fn parse_build_args(args: &[String]) -> WebBuildArgs {
    if args.iter().any(|a| a == "--legacy-site-json") {
        ui::error(
            "crepus web build no longer supports --legacy-site-json; use `crepus web new` and `crepus web build` with `.crepus` + WASM (see docs/cli.md).",
        );
    }
    if args.iter().any(|a| a == "--json") {
        ui::error("crepus web build no longer supports --json for site.json paths.");
    }

    let (target_id, manifest) = parse_target_and_manifest(args);

    let mut site_dir = None;
    let mut out_dir = None;
    let mut entry = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--site" => {
                if let Some(p) = args.get(i + 1) {
                    site_dir = Some(PathBuf::from(p));
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
            "-o" | "--output" | "--out" => {
                if let Some(p) = args.get(i + 1) {
                    out_dir = Some(PathBuf::from(p));
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    WebBuildArgs {
        site_dir,
        out_dir,
        entry,
        target_id,
        manifest,
    }
}

fn resolve_wasm_build_args(args: &WebBuildArgs) -> WasmBuildArgs {
    if let Some(site_dir) = &args.site_dir {
        let out_dir = args
            .out_dir
            .clone()
            .unwrap_or_else(|| site_dir.join("dist"));
        let entry = args.entry.clone().unwrap_or_else(|| "index.crepus".into());
        return WasmBuildArgs {
            site_dir: site_dir.clone(),
            out_dir,
            entry,
        };
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Some(targets) = crate::crepus_toml::load_web_targets(args.manifest.clone()) {
        let picked = crate::crepus_toml::resolve_pick(&targets, args.target_id.as_deref())
            .unwrap_or_else(|m| ui::error(&m));
        let out_dir = args.out_dir.clone().unwrap_or(picked.out_dir);
        let entry = args.entry.clone().unwrap_or(picked.entry);
        return WasmBuildArgs {
            site_dir: picked.site_dir,
            out_dir,
            entry,
        };
    }

    let site_dir = cwd;
    let out_dir = args
        .out_dir
        .clone()
        .unwrap_or_else(|| site_dir.join("dist"));
    let entry = args.entry.clone().unwrap_or_else(|| "index.crepus".into());

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

    // ── head block extraction ──────────────────────────────────────────────
    let mut template_head_html = String::new();
    if let Some(entry_content) = files.get(&b.entry).cloned() {
        let (head_raw, body_raw) = extract_head_block(&entry_content);
        if let Some(head_src) = head_raw {
            template_head_html = render_head_raw(&head_src);
            files.insert(b.entry.clone(), body_raw);
        }
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
    let inline_css = merged_site_inline_css(&files);
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
    let index_html = render_index_html(&head, &google_fonts, &inline_css, &template_head_html);
    std::fs::write(b.out_dir.join(".nojekyll"), b"")
        .unwrap_or_else(|e| ui::error(&format!("write .nojekyll: {e}")));
    std::fs::write(b.out_dir.join("index.html"), index_html).unwrap_or_else(|e| {
        ui::error(&format!("write index.html: {e}"));
    });
    std::fs::write(b.out_dir.join("app.js"), WEB_APP_JS).unwrap_or_else(|e| {
        ui::error(&format!("write app.js: {e}"));
    });

    if let Some(repo_root) = b.site_dir.parent() {
        let docs_path = repo_root.join("docs");
        if docs_path.is_dir() {
            let theme = crate::web_docs::DocsSiteTheme {
                accent: head.theme.accent.clone(),
                accent_soft: head.theme.accent_soft.clone(),
                surface: head.theme.surface.clone(),
                text: head.theme.text.clone(),
                muted: head.theme.muted.clone(),
                border: head.theme.border.clone(),
            };
            if let Err(e) = crate::web_docs::emit_markdown_docs(
                &docs_path,
                &b.out_dir.join("docs"),
                &theme,
                &head.page_title,
            ) {
                ui::error(&format!("emit docs HTML: {e}"));
            }
        }
    }

    let static_src = b.site_dir.join("static");
    if static_src.is_dir() {
        copy_dir_recursive(&static_src, &b.out_dir.join("static")).unwrap_or_else(|e| {
            ui::error(&format!("copy static/: {e}"));
        });
    }

    if let Err(e) = crate::web_islands::build_web_islands(&b.site_dir, &b.out_dir, &files) {
        ui::error(&e);
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
                let normalized = normalize_fullwidth_braces(&content);
                map.insert(key, normalized);
            }
        }
    }
}

fn normalize_fullwidth_braces(s: &str) -> String {
    s.replace('\u{FF5B}', "{").replace('\u{FF5D}', "}")
}

fn relative_key(root: &Path, abs: &Path) -> String {
    abs.strip_prefix(root)
        .unwrap_or(abs)
        .to_string_lossy()
        .replace('\\', "/")
}

#[derive(Debug, Clone)]
pub(crate) struct SiteHead {
    pub(crate) page_title: String,
    pub(crate) description: String,
    pub(crate) og_image: Option<String>,
    pub(crate) extra_head_html: String,
    pub(crate) theme: ThemeCss,
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
pub(crate) struct ThemeCss {
    pub(crate) accent: String,
    pub(crate) accent_soft: String,
    pub(crate) surface: String,
    pub(crate) text: String,
    pub(crate) muted: String,
    pub(crate) border: String,
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

pub(crate) fn merged_site_inline_css(files: &HashMap<String, String>) -> String {
    let mut blocks: Vec<String> = Vec::new();
    for content in files.values() {
        let css = strip_indent_decorators(content).inline_css;
        if !css.trim().is_empty() {
            blocks.push(css.trim().to_string());
        }
    }
    blocks.join("\n\n")
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

pub(crate) fn load_site_head(site_dir: &Path) -> SiteHead {
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

pub(crate) fn render_index_html(
    head: &SiteHead,
    google_fonts: &[String],
    inline_css: &str,
    template_head: &str,
) -> String {
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
    let inline_css_tag = if inline_css.trim().is_empty() {
        String::new()
    } else {
        format!("<style>\n{}\n</style>", inline_css)
    };
    let extra_head = {
        let mut parts: Vec<&str> = Vec::new();
        if !template_head.trim().is_empty() {
            parts.push(template_head.trim());
        }
        if !head.extra_head_html.trim().is_empty() {
            parts.push(head.extra_head_html.trim());
        }
        if !inline_css_tag.trim().is_empty() {
            parts.push(inline_css_tag.trim());
        }
        parts.join("\n")
    };

    WEB_INDEX_HTML
        .replace("__CREPUS_TITLE__", &escape_html_attr(&head.page_title))
        .replace("__CREPUS_DESC__", &escape_html_attr(&head.description))
        .replace("__CREPUS_OG__", &og)
        .replace("__CREPUS_GOOGLE_FONTS__", &font_markup)
        .replace("__CREPUS_EXTRA_HEAD__", &extra_head)
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

fn render_head_raw(head_raw: &str) -> String {
    let lines: Vec<&str> = head_raw.lines().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start().len();
        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        if let Some(tail) = trimmed.strip_prefix("title ") {
            let text = tail.trim().trim_matches('"');
            out.push_str(&format!("<title>{}</title>\n", text));
            i += 1;
        } else if let Some(tail) = trimmed.strip_prefix("meta ") {
            out.push_str(&format!("<meta {}>\n", tail.trim()));
            i += 1;
        } else if let Some(tail) = trimmed.strip_prefix("link ") {
            out.push_str(&format!("<link {}>\n", tail.trim()));
            i += 1;
        } else if let Some(tail) = trimmed.strip_prefix("script ") {
            out.push_str(&format!("<script {}></script>\n", tail.trim()));
            i += 1;
        } else if trimmed == "style" {
            let mut css = String::new();
            i += 1;
            while i < lines.len() {
                let sub = lines[i];
                if sub.trim().is_empty() {
                    css.push('\n');
                    i += 1;
                    continue;
                }
                let sub_indent = sub.len() - sub.trim_start().len();
                if sub_indent > indent {
                    let dedented = &sub[indent + 2..];
                    css.push_str(dedented);
                    css.push('\n');
                    i += 1;
                } else {
                    break;
                }
            }
            out.push_str(&format!("<style>\n{}</style>\n", css.trim()));
        } else if trimmed.starts_with("google-fonts:") {
            i += 1;
        } else {
            // unknown — treat title-like: if it has a quoted string child
            i += 1;
        }
    }
    out.trim().to_string()
}

fn copy_unocss(vendor_dir: &Path) {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../crepuscularity-webext/assets/vendor/unocss.js");
    let dst = vendor_dir.join("unocss.js");
    if src.is_file() {
        let _ = std::fs::copy(&src, &dst);
    }
}

/// Directory under a site root where `crepus web serve` caches UnoCSS, `app.js`, and wasm-bindgen output.
pub(crate) const WEB_DEV_ARTIFACT_DIR: &str = ".crepus-dev";

/// Build site `runtime/` to WASM once and populate `.crepus-dev/` with the same assets as a `crepus web build` dist folder.
pub(crate) fn ensure_web_dev_artifacts(site_dir: &Path) -> Result<(), String> {
    let runtime_dir = site_dir.join("runtime");
    if !runtime_dir.join("Cargo.toml").is_file() {
        return Err(format!(
            "no runtime/Cargo.toml under {} — run `crepus web new <name>` or copy examples/web-site",
            site_dir.display()
        ));
    }

    let dev = site_dir.join(WEB_DEV_ARTIFACT_DIR);
    let vendor_dir = dev.join("vendor");
    let pkg_dir = dev.join("pkg");
    std::fs::create_dir_all(&vendor_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&pkg_dir).map_err(|e| e.to_string())?;

    copy_unocss(&vendor_dir);
    std::fs::write(dev.join("app.js"), WEB_APP_JS).map_err(|e| e.to_string())?;

    let mut files: HashMap<String, String> = HashMap::new();
    load_all_crepus(site_dir, site_dir, &mut files);
    crate::web_islands::build_web_islands(site_dir, &dev, &files)?;

    cargo_build_wasm32(&runtime_dir)?;
    let (workspace_target, local_target) = wasm_release_dirs(site_dir, &runtime_dir);
    let wasm_path = find_wasm_file(&workspace_target)
        .or_else(|| find_wasm_file(&local_target))
        .ok_or_else(|| {
            "built .wasm not found under target/wasm32-unknown-unknown/release/ (install wasm32 target and fix runtime errors)"
                .to_string()
        })?;
    run_wasm_bindgen(&wasm_path, &pkg_dir, "runtime")?;
    Ok(())
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

// ── build-full ───────────────────────────────────────────────────────────────

struct BuildFullArgs {
    site_dir: PathBuf,
    wasm: bool,
    server: bool,
}

fn parse_build_full_args(args: &[String]) -> BuildFullArgs {
    let mut site_dir = std::env::current_dir().unwrap_or_else(|e| {
        ui::error(&format!("cannot determine current directory: {e}"));
    });
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
                        crepus_files.insert(key, normalize_fullwidth_braces(&content));
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
