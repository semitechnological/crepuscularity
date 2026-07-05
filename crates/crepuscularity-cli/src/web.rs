//! `crepus web` — `.crepus`-first static sites (WASM runtime) + dev server.
//!
//! Commands: new, build, dev (alias: serve), build-full.
//!
//! Production builds mirror `crepus webext`: compile the site `runtime/` crate to
//! `wasm32-unknown-unknown`, run `wasm-bindgen`, ship `crepus-bundle.json` + a thin HTML shell.
//! The default WASM entrypoint calls `crepuscularity_web::render_bundle`; sites that need Rust
//! context can replace that with `render_from_files` + a hand-built `TemplateContext`.

use console::style;
use crepuscularity_core::preprocess::{
    extract_head_block, google_font_css_family_name, google_fonts_head_markup,
    merge_unique_font_families, strip_indent_decorators,
};
use crepuscularity_core::{DriverCache, Fingerprint};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::build_options::BuildOptions;
use crate::cli::WebCommands;
use crate::docs_generator;
use crate::ui;
use crate::wasm_bundle::{
    cargo_build_wasm32, find_wasm_file, run_wasm_bindgen, run_wasm_opt, wasm_profile_dirs,
    WasmOptStatus,
};
use crepuscularity_web::render_bundle_with_ssr;

const WEB_INDEX_HTML: &str = include_str!("../assets/web/index.html");
const WEB_APP_JS: &str = include_str!("../assets/web/app.js");
const UNOCSS_JS: &[u8] = include_bytes!("../assets/vendor/unocss.js");

// ── Entry point ──────────────────────────────────────────────────────────────

pub fn execute(cmd: WebCommands) {
    match cmd {
        WebCommands::New { name } => scaffold_site(&name),
        WebCommands::Build {
            build,
            site,
            out_dir,
            output,
            entry,
            target_id,
            manifest,
        } => {
            let b = WebBuildArgs {
                site_dir: site,
                out_dir: out_dir.or(output),
                entry,
                target_id,
                manifest,
                meta: None,
                options: build.into_options_or_exit(),
            };
            build_site_wasm(&b);
        }
        WebCommands::Dev {
            site,
            port,
            entry,
            target_id,
            manifest,
            axum,
        } => {
            let opts = resolve_dev_options(site, port, entry, target_id, manifest, axum);
            crate::web_serve::run(opts);
        }
        WebCommands::BuildFull { site, wasm, server } => {
            let site_dir = site.unwrap_or_else(|| {
                std::env::current_dir().unwrap_or_else(|e| {
                    ui::error(&format!("cannot determine current directory: {e}"));
                })
            });
            run_build_full(&BuildFullArgs {
                site_dir,
                wasm,
                server,
            });
        }
    }
}

fn resolve_dev_options(
    site: Option<PathBuf>,
    port: u16,
    entry: String,
    target_id: Option<String>,
    manifest: Option<PathBuf>,
    axum: bool,
) -> crate::web_serve::ServeOptions {
    let explicit_site = site.is_some();
    let mut site_dir =
        site.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let entry_from_args = entry != "index.crepus";
    let mut entry = entry;
    let mut meta = None;

    if explicit_site {
        if let Some(targets) =
            crate::crepus_toml::load_web_targets(Some(site_dir.join("crepus.toml")))
        {
            let picked = crate::crepus_toml::resolve_pick(&targets, target_id.as_deref())
                .unwrap_or_else(|m| ui::error(&m));
            if !entry_from_args {
                entry = picked.entry;
            }
            meta = Some(picked.meta);
        }
    } else if let Some(targets) = crate::crepus_toml::load_web_targets(manifest) {
        let picked = crate::crepus_toml::resolve_pick(&targets, target_id.as_deref())
            .unwrap_or_else(|m| ui::error(&m));
        site_dir = picked.site_dir;
        if !entry_from_args {
            entry = picked.entry;
        }
        meta = Some(picked.meta);
    }

    crate::web_serve::ServeOptions {
        site_dir,
        port,
        entry,
        meta,
        axum,
    }
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

    let crepus_toml = format!(
        r#"[[targets]]
type = "web"
id = "site"
site = "."
out = "dist"
entry = "index.crepus"
name = "{name}"
description = "Crepus static site (.crepus + WASM)"

[targets.seo]
title = "{name}"
description = "Crepus static site (.crepus + WASM)"
robots = "index,follow"
twitter_card = "summary"
"#
    );
    std::fs::write(base.join("crepus.toml"), crepus_toml).unwrap_or_else(|e| {
        ui::error(&format!("write crepus.toml: {e}"));
    });

    let index_crepus = r#"div w-full min-h-screen bg-zinc-950 text-zinc-50 p-8 flex flex-col gap-4
 div text-3xl font-bold
  "Hello from .crepus"
 div text-zinc-400 max-w-xl
  "This page is rendered in the browser by the same pipeline as crepus web dev — wasm32 + crepus-bundle.json."
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
crepuscularity-web = "0.4.3"
wasm-bindgen = "0.2"

[profile.release]
lto = true
codegen-units = 1
opt-level = "z"

[workspace]
"#
    );
    std::fs::write(base.join("runtime/Cargo.toml"), cargo_toml).unwrap_or_else(|e| {
        ui::error(&format!("write runtime/Cargo.toml: {e}"));
    });

    let lib_rs = r#"use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn crepus_render(bundle_json: &str) -> Result<String, JsValue> {
    crepuscularity_web::render_bundle(bundle_json).map_err(|e| JsValue::from_str(&e.to_string()))
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
    eprintln!("  crepus web dev --site .");
    eprintln!("  crepus build");
    ui::done_in(t0.elapsed());
}

// ── WASM build ───────────────────────────────────────────────────────────────

struct WasmBuildArgs {
    site_dir: PathBuf,
    out_dir: PathBuf,
    entry: String,
    meta: Option<crate::crepus_toml::WebTargetMeta>,
    options: BuildOptions,
}

pub(crate) struct WebBuildArgs {
    pub(crate) site_dir: Option<PathBuf>,
    pub(crate) out_dir: Option<PathBuf>,
    pub(crate) entry: Option<String>,
    pub(crate) target_id: Option<String>,
    pub(crate) manifest: Option<PathBuf>,
    pub(crate) meta: Option<crate::crepus_toml::WebTargetMeta>,
    pub(crate) options: BuildOptions,
}

fn resolve_wasm_build_args(args: &WebBuildArgs) -> WasmBuildArgs {
    if let Some(site_dir) = &args.site_dir {
        let local_targets =
            crate::crepus_toml::load_web_targets(Some(site_dir.join("crepus.toml")));
        let picked = local_targets.as_ref().and_then(|targets| {
            crate::crepus_toml::resolve_pick(targets, args.target_id.as_deref()).ok()
        });
        let out_dir = args
            .out_dir
            .clone()
            .or_else(|| picked.as_ref().map(|target| target.out_dir.clone()))
            .unwrap_or_else(|| site_dir.join("dist"));
        let entry = args
            .entry
            .clone()
            .or_else(|| picked.as_ref().map(|target| target.entry.clone()))
            .unwrap_or_else(|| "index.crepus".into());
        let meta = args
            .meta
            .clone()
            .or_else(|| picked.as_ref().map(|target| target.meta.clone()));
        return WasmBuildArgs {
            site_dir: site_dir.clone(),
            out_dir,
            entry,
            meta,
            options: args.options,
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
            meta: Some(picked.meta),
            options: args.options,
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
        meta: args.meta.clone(),
        options: args.options,
    }
}

pub(crate) fn build_site_wasm(cli: &WebBuildArgs) {
    let t0 = Instant::now();
    let b = resolve_wasm_build_args(cli);
    let runtime_dir = b.site_dir.join("runtime");
    if !runtime_dir.join("Cargo.toml").is_file() {
        ui::error(&format!(
            "no runtime/Cargo.toml under {} — run `crepus web new <name>` or copy examples/web-site",
            b.site_dir.display()
        ));
    }

    let label = if b.options.release() {
        "crepus web release"
    } else {
        "crepus web debug"
    };
    eprintln!(
        "{} building WASM site → {}",
        style(label).dim(),
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

    let llms_site_text = files
        .get(&b.entry)
        .map(|source| render_crepus_readable_text(source));

    let bundle = json!({
        "entry": b.entry,
        "files": files,
    });
    let bundle_str = serde_json::to_string(&bundle).unwrap_or_else(|e| {
        ui::error(&format!("serialize bundle: {e}"));
    });

    let head = load_site_head(&b.site_dir);
    let head = merge_site_head_meta(head, b.meta.as_ref());
    let head = merge_llms_alternates(head, b.meta.as_ref());
    let google_fonts = merged_site_google_fonts(&b.site_dir, &files, b.meta.as_ref());
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
    let mut index_html = render_index_html(&head, &google_fonts, &inline_css, &template_head_html);
    // SSR + inline bundle: pre-render entry, embed bundle JSON to eliminate HTTP fetch
    if let Ok(ssr_html) = render_bundle_with_ssr(&bundle_str, true) {
        let needle = r#"<div id="crepus-root"></div>"#;
        if let Some(pos) = index_html.find(needle) {
            let bundle_escaped = ssr_escape_json(&bundle_str);
            let replacement = format!(
                r#"<div id="crepus-root">{}</div><script id="__crepus_bundle__" type="application/json">{}</script>"#,
                ssr_html, bundle_escaped
            );
            index_html.replace_range(pos..pos + needle.len(), &replacement);
        }
    }
    std::fs::write(b.out_dir.join(".nojekyll"), b"")
        .unwrap_or_else(|e| ui::error(&format!("write .nojekyll: {e}")));
    std::fs::write(b.out_dir.join("index.html"), index_html).unwrap_or_else(|e| {
        ui::error(&format!("write index.html: {e}"));
    });
    std::fs::write(b.out_dir.join("app.js"), WEB_APP_JS).unwrap_or_else(|e| {
        ui::error(&format!("write app.js: {e}"));
    });

    if let Some(docs) = b.meta.as_ref().and_then(|meta| meta.docs.as_ref()) {
        let theme = crate::web_docs_hook::DocsHookTheme {
            accent: head.theme.accent.clone(),
            accent_soft: head.theme.accent_soft.clone(),
            surface: head.theme.surface.clone(),
            text: head.theme.text.clone(),
            muted: head.theme.muted.clone(),
            border: head.theme.border.clone(),
        };
        if let Err(e) = crate::web_docs_hook::run_docs_hook(
            &b.site_dir,
            &b.out_dir.join("docs"),
            docs,
            &head.page_title,
            &theme,
        ) {
            ui::error(&format!("run docs hook: {e}"));
        }
    }

    let static_src = b.site_dir.join("static");
    if static_src.is_dir() {
        copy_dir_recursive(&static_src, &b.out_dir.join("static")).unwrap_or_else(|e| {
            ui::error(&format!("copy static/: {e}"));
        });
    }

    // Auto-detect docs/ directory (like Svelte/Next.js — just works)
    if let Some(src) = docs_src_candidate(&b.site_dir) {
        let has_md = std::fs::read_dir(&src).is_ok_and(|mut d| {
            d.any(|e| {
                e.ok()
                    .is_some_and(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
            })
        });
        if has_md && b.meta.as_ref().and_then(|m| m.docs.as_ref()).is_none() {
            if let Err(e) = docs_generator::generate_docs(
                &src,
                &b.out_dir.join("docs"),
                &head.theme,
                &head.page_title,
            ) {
                ui::warning(&format!("docs generation: {e}"));
            }
        }
    }

    if let Err(e) = crate::web_islands::build_web_islands(&b.site_dir, &b.out_dir, &files) {
        ui::error(&e);
    }

    write_seo_files(&b.out_dir, &head);
    write_llms_files(
        &b.site_dir,
        &b.out_dir,
        &head,
        b.meta.as_ref(),
        llms_site_text.as_deref(),
    );

    {
        let sp = ui::spinner("compiling site WASM (wasm32-unknown-unknown)");
        match cargo_build_wasm32(&runtime_dir, b.options) {
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

    let (workspace_target, local_target) = wasm_profile_dirs(&b.site_dir, &runtime_dir, b.options);
    let wasm_path = find_wasm_file(&workspace_target)
        .or_else(|| find_wasm_file(&local_target))
        .unwrap_or_else(|| {
            ui::error(&format!(
                "built .wasm not found under target/wasm32-unknown-unknown/{}/",
                b.options.cargo_profile()
            ));
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

    if b.options.optimize_artifacts() {
        let wasm = pkg_dir.join("runtime_bg.wasm");
        if wasm.is_file() {
            let sp = ui::spinner("optimizing WASM");
            match run_wasm_opt(&wasm, b.options.optimization) {
                Ok(WasmOptStatus::Optimized) => ui::spinner_ok(&sp, "WASM optimized"),
                Ok(WasmOptStatus::NotInstalled) => {
                    sp.finish_and_clear();
                    ui::warning("wasm-opt not found — install Binaryen to optimize WASM");
                }
                Err(err) => {
                    sp.finish_and_clear();
                    ui::warning(&format!("wasm-opt failed: {err}"));
                }
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
    pub(crate) seo: crate::crepus_toml::SeoConfig,
}

impl Default for SiteHead {
    fn default() -> Self {
        Self {
            page_title: "Crepus site".into(),
            description: "Built with Crepuscularity".into(),
            og_image: None,
            extra_head_html: String::new(),
            theme: ThemeCss::default(),
            seo: crate::crepus_toml::SeoConfig::default(),
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

/// Merge `crepus.toml` target fonts with every `google-font` pragma in bundled `.crepus` files.
pub(crate) fn merged_site_google_fonts(
    _site_dir: &Path,
    files: &HashMap<String, String>,
    meta: Option<&crate::crepus_toml::WebTargetMeta>,
) -> Vec<String> {
    let mut collected = Vec::new();
    if let Some(meta) = meta {
        collected.extend(meta.google_fonts.clone());
    }
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
                    head.seo.description = Some(d.clone());
                }
                head.og_image = seo.og_image.clone();
                head.seo.title = seo.title.clone();
                head.seo.image = seo.og_image.clone();
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

    head
}

pub(crate) fn merge_site_head_meta(
    mut head: SiteHead,
    meta: Option<&crate::crepus_toml::WebTargetMeta>,
) -> SiteHead {
    let Some(meta) = meta else {
        return head;
    };
    if let Some(name) = &meta.name {
        head.page_title = name.clone();
    }
    if let Some(description) = &meta.description {
        head.description = description.clone();
    }
    if let Some(head_html) = &meta.head_html {
        head.extra_head_html = head_html.clone();
    }
    if let Some(seo) = &meta.seo {
        head.seo = seo.clone();
        if let Some(title) = &seo.title {
            head.page_title = title.clone();
        }
        if let Some(description) = &seo.description {
            head.description = description.clone();
        }
        if let Some(image) = &seo.image {
            head.og_image = Some(image.clone());
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
    let seo = render_seo_head(head);
    let font_markup = google_fonts_head_markup(google_fonts);
    let body_font_css = google_fonts
        .first()
        .map(|n| {
            let q = google_font_css_family_name(n)
                .replace('\\', r"\\")
                .replace('"', r#"\""#);
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

    let html_title = head.seo.title.as_deref().unwrap_or(&head.page_title);
    let html_description = head.seo.description.as_deref().unwrap_or(&head.description);

    WEB_INDEX_HTML
        .replace("__CREPUS_TITLE__", &escape_html_attr(html_title))
        .replace("__CREPUS_DESC__", &escape_html_attr(html_description))
        .replace("__CREPUS_OG__", &seo)
        .replace("__CREPUS_GOOGLE_FONTS__", &font_markup)
        .replace("__CREPUS_EXTRA_HEAD__", &extra_head)
        .replace("__CREPUS_NOSCRIPT__", &render_llms_noscript(head))
        .replace("__CREPUS_BODY_FONT__", &body_font_css)
        .replace("__THEME_ACCENT__", &escape_html_attr(&t.accent))
        .replace("__THEME_ACCENT_SOFT__", &escape_html_attr(&t.accent_soft))
        .replace("__THEME_SURFACE__", &escape_html_attr(&t.surface))
        .replace("__THEME_TEXT__", &escape_html_attr(&t.text))
        .replace("__THEME_MUTED__", &escape_html_attr(&t.muted))
        .replace("__THEME_BORDER__", &escape_html_attr(&t.border))
}

/// Escape JSON string for safe embedding in HTML `<script type="application/json">`.
/// Replaces `</script>` with the Unicode escape sequence so the parser doesn't see a closing tag.
fn ssr_escape_json(json: &str) -> String {
    json.replace("</script>", "<\\/script>")
        .replace("</Script>", "<\\/Script>")
        .replace("</SCRIPT>", "<\\/SCRIPT>")
}

fn render_seo_head(head: &SiteHead) -> String {
    let seo = &head.seo;
    let title = seo.title.as_deref().unwrap_or(&head.page_title);
    let description = seo.description.as_deref().unwrap_or(&head.description);
    let og_type = seo.og_type.as_deref().unwrap_or("website");
    let image = seo.image.as_ref().or(head.og_image.as_ref());
    let twitter_card = seo.twitter_card.as_deref().unwrap_or_else(|| {
        if image.is_some() {
            "summary_large_image"
        } else {
            "summary"
        }
    });
    let mut lines = Vec::new();

    if let Some(canonical) = &seo.canonical {
        lines.push(format!(
            r#"  <link rel="canonical" href="{}">"#,
            escape_html_attr(canonical)
        ));
    }
    if !seo.keywords.is_empty() {
        lines.push(format!(
            r#"  <meta name="keywords" content="{}">"#,
            escape_html_attr(&seo.keywords.join(", "))
        ));
    }
    if let Some(author) = &seo.author {
        lines.push(format!(
            r#"  <meta name="author" content="{}">"#,
            escape_html_attr(author)
        ));
    }
    if let Some(robots) = &seo.robots {
        lines.push(format!(
            r#"  <meta name="robots" content="{}">"#,
            escape_html_attr(robots)
        ));
    }
    if let Some(theme_color) = &seo.theme_color {
        lines.push(format!(
            r#"  <meta name="theme-color" content="{}">"#,
            escape_html_attr(theme_color)
        ));
    }
    if let Some(application_name) = &seo.application_name {
        lines.push(format!(
            r#"  <meta name="application-name" content="{}">"#,
            escape_html_attr(application_name)
        ));
    }
    if let Some(generator) = &seo.generator {
        if !generator.trim().is_empty() {
            lines.push(format!(
                r#"  <meta name="generator" content="{}">"#,
                escape_html_attr(generator)
            ));
        }
    }

    lines.push(format!(
        r#"  <meta property="og:title" content="{}">"#,
        escape_html_attr(title)
    ));
    lines.push(format!(
        r#"  <meta property="og:description" content="{}">"#,
        escape_html_attr(description)
    ));
    lines.push(format!(
        r#"  <meta property="og:type" content="{}">"#,
        escape_html_attr(og_type)
    ));
    if let Some(canonical) = &seo.canonical {
        lines.push(format!(
            r#"  <meta property="og:url" content="{}">"#,
            escape_html_attr(canonical)
        ));
    }
    if let Some(site_name) = &seo.site_name {
        lines.push(format!(
            r#"  <meta property="og:site_name" content="{}">"#,
            escape_html_attr(site_name)
        ));
    }
    if let Some(locale) = &seo.locale {
        lines.push(format!(
            r#"  <meta property="og:locale" content="{}">"#,
            escape_html_attr(locale)
        ));
    }
    if let Some(image) = image {
        lines.push(format!(
            r#"  <meta property="og:image" content="{}">"#,
            escape_html_attr(image)
        ));
    }
    if let Some(image_alt) = &seo.image_alt {
        lines.push(format!(
            r#"  <meta property="og:image:alt" content="{}">"#,
            escape_html_attr(image_alt)
        ));
    }

    lines.push(format!(
        r#"  <meta name="twitter:card" content="{}">"#,
        escape_html_attr(twitter_card)
    ));
    lines.push(format!(
        r#"  <meta name="twitter:title" content="{}">"#,
        escape_html_attr(title)
    ));
    lines.push(format!(
        r#"  <meta name="twitter:description" content="{}">"#,
        escape_html_attr(description)
    ));
    if let Some(image) = image {
        lines.push(format!(
            r#"  <meta name="twitter:image" content="{}">"#,
            escape_html_attr(image)
        ));
    }
    if let Some(image_alt) = &seo.image_alt {
        lines.push(format!(
            r#"  <meta name="twitter:image:alt" content="{}">"#,
            escape_html_attr(image_alt)
        ));
    }
    if let Some(site) = &seo.twitter_site {
        lines.push(format!(
            r#"  <meta name="twitter:site" content="{}">"#,
            escape_html_attr(site)
        ));
    }
    if let Some(creator) = &seo.twitter_creator {
        lines.push(format!(
            r#"  <meta name="twitter:creator" content="{}">"#,
            escape_html_attr(creator)
        ));
    }

    for alt in &seo.alternates {
        let mut attrs = vec![
            r#"rel="alternate""#.to_string(),
            format!(r#"href="{}""#, escape_html_attr(&alt.href)),
        ];
        if let Some(hreflang) = &alt.hreflang {
            attrs.push(format!(r#"hreflang="{}""#, escape_html_attr(hreflang)));
        }
        if let Some(media) = &alt.media {
            attrs.push(format!(r#"media="{}""#, escape_html_attr(media)));
        }
        if let Some(title) = &alt.title {
            attrs.push(format!(r#"title="{}""#, escape_html_attr(title)));
        }
        if let Some(mime_type) = &alt.mime_type {
            attrs.push(format!(r#"type="{}""#, escape_html_attr(mime_type)));
        }
        lines.push(format!("  <link {}>", attrs.join(" ")));
    }

    for json in &seo.json_ld {
        if !json.trim().is_empty() {
            lines.push(format!(
                r#"  <script type="application/ld+json">{}</script>"#,
                json.trim()
            ));
        }
    }

    lines.join("\n")
}

fn merge_llms_alternates(
    mut head: SiteHead,
    meta: Option<&crate::crepus_toml::WebTargetMeta>,
) -> SiteHead {
    let Some(llms) = meta.and_then(|m| m.llms.as_ref()) else {
        return head;
    };
    if !llms.enabled {
        return head;
    }
    let base = llms_base_url(llms, &head);
    let alts = [
        ("llms.txt", "LLM index", "text/plain"),
        ("llms-full.txt", "Full LLM bundle", "text/plain"),
        ("agent.md", "Agent guide", "text/markdown"),
    ];
    for (path, title, mime_type) in alts {
        let href = format!("{base}/{path}");
        if head.seo.alternates.iter().any(|alt| alt.href == href) {
            continue;
        }
        head.seo.alternates.push(crate::crepus_toml::SeoAlternate {
            href,
            hreflang: None,
            media: None,
            title: Some(title.into()),
            mime_type: Some(mime_type.into()),
        });
    }
    head
}

fn write_seo_files(out_dir: &Path, head: &SiteHead) {
    let seo = &head.seo;
    if let Some(sitemap) = &seo.sitemap {
        if sitemap.enabled {
            let discovered_paths = discover_html_sitemap_paths(out_dir);
            if let Some(xml) = render_sitemap_xml(sitemap, &discovered_paths) {
                std::fs::write(out_dir.join("sitemap.xml"), xml)
                    .unwrap_or_else(|e| ui::error(&format!("write sitemap.xml: {e}")));
            }
        }
    }
    if let Some(robots) = &seo.robots_txt {
        if robots.enabled {
            let txt = render_robots_txt(robots, seo);
            std::fs::write(out_dir.join("robots.txt"), txt)
                .unwrap_or_else(|e| ui::error(&format!("write robots.txt: {e}")));
        }
    }
}

fn write_llms_files(
    site_dir: &Path,
    out_dir: &Path,
    head: &SiteHead,
    meta: Option<&crate::crepus_toml::WebTargetMeta>,
    site_text: Option<&str>,
) {
    let Some(llms) = meta.and_then(|m| m.llms.as_ref()) else {
        return;
    };
    if !llms.enabled {
        return;
    }
    let base = llms_base_url(llms, head);
    let title = llms
        .title
        .as_deref()
        .or(head.seo.title.as_deref())
        .unwrap_or(&head.page_title);
    let description = llms
        .description
        .as_deref()
        .or(head.seo.description.as_deref())
        .unwrap_or(&head.description);

    let mut index = format!(
        "# {title}\n\n> {description}\n\nStatic Crepus site. Agents should fetch the Markdown/text files below instead of scraping the rendered app shell.\n\n## Markdown\n\n- [Full bundle]({base}/llms-full.txt): One request with the agent guide and readable site text.\n- [Agent guide]({base}/agent.md): Fetch order and hosting notes.\n"
    );
    for source in &llms.sources {
        let label = source.title.as_deref().unwrap_or(&source.path);
        let href = source
            .href
            .clone()
            .unwrap_or_else(|| format!("{base}/{}", source.path.trim_start_matches('/')));
        index.push_str(&format!("- [{label}]({href}): Source Markdown.\n",));
    }

    let agent = format!(
        "# {title} — agent guide\n\n> Use direct static files. Do not scrape the HTML shell unless no Markdown file answers the task.\n\n## Best first request\n\n`curl -sL {base}/llms-full.txt`\n\n## Files\n\n- {base}/llms.txt — compact index\n- {base}/llms-full.txt — full bundle\n{}\n",
        llms.sources
            .iter()
            .map(|source| {
                source
                    .href
                    .clone()
                    .unwrap_or_else(|| format!("{base}/{}", source.path.trim_start_matches('/')))
            })
            .map(|href| format!("- {href}"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let mut full = format!("{index}\n---\n\n{agent}");
    if let Some(site_text) = site_text {
        if !site_text.trim().is_empty() {
            full.push_str(&format!(
                "\n---\n\n# Rendered site text\n\n{}\n",
                site_text.trim()
            ));
        }
    }
    for source in &llms.sources {
        let rel = source.path.trim_start_matches('/');
        let path = site_dir.join(rel);
        let body = std::fs::read_to_string(&path).unwrap_or_else(|_| String::new());
        let label = source.title.as_deref().unwrap_or(rel);
        full.push_str(&format!("\n---\n\n# {label}\n\n{body}\n"));
        if !body.is_empty() && !rel.split('/').any(|part| part == "..") {
            if let Some(parent) = Path::new(rel).parent() {
                std::fs::create_dir_all(out_dir.join(parent))
                    .unwrap_or_else(|e| ui::error(&format!("mkdir llms source dir: {e}")));
            }
            std::fs::write(out_dir.join(rel), body)
                .unwrap_or_else(|e| ui::error(&format!("write {rel}: {e}")));
        }
    }

    std::fs::write(out_dir.join("llms.txt"), index)
        .unwrap_or_else(|e| ui::error(&format!("write llms.txt: {e}")));
    std::fs::write(out_dir.join("llms-full.txt"), full)
        .unwrap_or_else(|e| ui::error(&format!("write llms-full.txt: {e}")));
    std::fs::write(out_dir.join("agent.md"), agent)
        .unwrap_or_else(|e| ui::error(&format!("write agent.md: {e}")));
}

fn render_llms_noscript(head: &SiteHead) -> String {
    let Some(href) = head
        .seo
        .alternates
        .iter()
        .find(|alt| alt.href.ends_with("/llms-full.txt"))
        .map(|alt| alt.href.as_str())
    else {
        return String::new();
    };
    format!(
        r#"<noscript><p><a href="{}">Agent-readable text version</a></p></noscript>"#,
        escape_html_attr(href)
    )
}

#[derive(Clone)]
struct ReadableElement {
    tag: String,
    href: Option<String>,
}

fn render_crepus_readable_text(source: &str) -> String {
    let mut stack: Vec<ReadableElement> = Vec::new();
    let mut out: Vec<String> = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('.') {
            continue;
        }
        let depth = (line.len() - line.trim_start().len()) / 2;
        stack.truncate(depth);
        if let Some(text) = quoted_crepus_text(trimmed) {
            push_readable_text(&mut out, &stack, text);
            continue;
        }
        if let Some(element) = readable_element(trimmed) {
            stack.push(element);
        }
    }
    out.join("\n\n")
}

fn push_readable_text(out: &mut Vec<String>, stack: &[ReadableElement], text: String) {
    if text.trim().is_empty() {
        return;
    }
    if let Some(tag) = stack
        .iter()
        .rev()
        .find_map(|element| match element.tag.as_str() {
            "h1" => Some("#"),
            "h2" => Some("##"),
            "h3" => Some("###"),
            _ => None,
        })
    {
        out.push(format!("{tag} {text}"));
        return;
    }
    if let Some(href) = stack
        .iter()
        .rev()
        .find_map(|element| element.href.as_deref())
    {
        let link = format!("[{text}]({href})");
        if let Some(last) = out.last_mut() {
            if last.ends_with(&format!("]({href})")) {
                let insert_at = last.find("](").unwrap_or(last.len());
                last.insert_str(insert_at, &text);
                return;
            }
        }
        out.push(link);
        return;
    }
    out.push(text);
}

fn readable_element(trimmed: &str) -> Option<ReadableElement> {
    let tag = trimmed.split_whitespace().next()?.to_string();
    if tag.starts_with('@') {
        return None;
    }
    Some(ReadableElement {
        tag,
        href: attr_value(trimmed, "href"),
    })
}

fn quoted_crepus_text(trimmed: &str) -> Option<String> {
    if !trimmed.starts_with('"') || !trimmed.ends_with('"') || trimmed.len() < 2 {
        return None;
    }
    Some(trimmed[1..trimmed.len() - 1].replace("\\\"", "\""))
}

fn attr_value(line: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=\"");
    let start = line.find(&needle)? + needle.len();
    let end = line[start..].find('"')? + start;
    Some(line[start..end].to_string())
}

fn llms_base_url(llms: &crate::crepus_toml::LlmsConfig, head: &SiteHead) -> String {
    llms.base_url
        .as_deref()
        .or(head.seo.canonical.as_deref())
        .unwrap_or("")
        .trim_end_matches('/')
        .to_string()
}

fn render_robots_txt(
    robots: &crate::crepus_toml::RobotsTxtConfig,
    seo: &crate::crepus_toml::SeoConfig,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "User-agent: {}",
        robots.user_agent.as_deref().unwrap_or("*")
    ));
    for allow in &robots.allow {
        lines.push(format!("Allow: {allow}"));
    }
    for disallow in &robots.disallow {
        lines.push(format!("Disallow: {disallow}"));
    }
    if let Some(sitemap) = &robots.sitemap {
        lines.push(format!("Sitemap: {sitemap}"));
    } else if let Some(sitemap) = &seo.sitemap {
        if sitemap.enabled {
            if let Some(base_url) = &sitemap.base_url {
                lines.push(format!(
                    "Sitemap: {}/sitemap.xml",
                    base_url.trim_end_matches('/')
                ));
            }
        }
    }
    for alt in &seo.alternates {
        if alt.href.ends_with("/llms.txt") || alt.href.ends_with("/llms-full.txt") {
            lines.push(format!("# {}", alt.href));
        }
    }
    lines.push(String::new());
    lines.join("\n")
}

fn render_sitemap_xml(
    sitemap: &crate::crepus_toml::SitemapConfig,
    discovered_paths: &[String],
) -> Option<String> {
    let base = sitemap.base_url.as_ref()?.trim_end_matches('/').to_string();
    let mut paths = if sitemap.paths.is_empty() {
        vec!["/".to_string()]
    } else {
        sitemap.paths.clone()
    };
    for path in discovered_paths {
        if !paths.iter().any(|known| known == path) {
            paths.push(path.clone());
        }
    }
    let mut out = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
"#,
    );
    for path in paths {
        let loc = if path.starts_with("http://") || path.starts_with("https://") {
            path
        } else {
            format!("{base}/{}", path.trim_start_matches('/'))
        };
        out.push_str("  <url>\n");
        out.push_str(&format!("    <loc>{}</loc>\n", escape_xml_text(&loc)));
        if let Some(changefreq) = &sitemap.changefreq {
            out.push_str(&format!(
                "    <changefreq>{}</changefreq>\n",
                escape_xml_text(changefreq)
            ));
        }
        if let Some(priority) = sitemap.priority {
            out.push_str(&format!("    <priority>{priority:.1}</priority>\n"));
        }
        out.push_str("  </url>\n");
    }
    out.push_str("</urlset>\n");
    Some(out)
}

fn discover_html_sitemap_paths(out_dir: &Path) -> Vec<String> {
    let mut paths = Vec::new();
    collect_html_sitemap_paths(out_dir, out_dir, &mut paths);
    paths.sort();
    paths
}

fn collect_html_sitemap_paths(base: &Path, dir: &Path, paths: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_html_sitemap_paths(base, &path, paths);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("html") {
            continue;
        }
        let Ok(rel) = path.strip_prefix(base) else {
            continue;
        };
        let rel = rel.to_string_lossy().replace('\\', "/");
        let route = if rel == "index.html" {
            "/".to_string()
        } else if let Some(prefix) = rel.strip_suffix("/index.html") {
            format!("/{prefix}/")
        } else {
            format!("/{rel}")
        };
        if !paths.iter().any(|known| known == &route) {
            paths.push(route);
        }
    }
}

fn escape_html_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_xml_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Find a docs/ or public/ directory relative to the site dir.
fn docs_src_candidate(site_dir: &Path) -> Option<PathBuf> {
    let dirs = [
        site_dir.join("../docs"),
        site_dir.join("docs"),
        site_dir.join("public"),
    ];
    for d in &dirs {
        if d.is_dir() {
            return Some(d.clone());
        }
    }
    if let Some(parent) = site_dir.parent() {
        let p = parent.join("docs");
        if p.is_dir() {
            return Some(p);
        }
    }
    None
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
    let dst = vendor_dir.join("unocss.js");
    std::fs::write(&dst, UNOCSS_JS).unwrap_or_else(|e| {
        ui::error(&format!("write {}: {e}", dst.display()));
    });
}

/// Directory under a site root where `crepus web dev` caches UnoCSS, `app.js`, and wasm-bindgen output.
pub(crate) const WEB_DEV_ARTIFACT_DIR: &str = ".crepus-dev";

/// Build site `runtime/` to WASM once and populate `.crepus-dev/` with the same assets as a `crepus web build` dist folder.
pub(crate) fn ensure_web_dev_artifacts(site_dir: &Path) -> Result<(), String> {
    let options = BuildOptions::debug();
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

    cargo_build_wasm32(&runtime_dir, options)?;
    let (workspace_target, local_target) = wasm_profile_dirs(site_dir, &runtime_dir, options);
    let wasm_path = find_wasm_file(&workspace_target)
        .or_else(|| find_wasm_file(&local_target))
        .ok_or_else(|| {
            format!(
                "built .wasm not found under target/wasm32-unknown-unknown/{}/ (install wasm32 target and fix runtime errors)",
                options.cargo_profile()
            )
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

fn collect_crepus_files(site_dir: &Path) -> (HashMap<String, String>, Vec<String>) {
    let mut crepus_files: HashMap<String, String> = HashMap::new();
    let mut entries: Vec<String> = Vec::new();

    match std::fs::read_dir(site_dir) {
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
            ui::error(&format!("read dir {}: {e}", site_dir.display()));
        }
    }

    (crepus_files, entries)
}

fn run_build_full(args: &BuildFullArgs) {
    let t0 = Instant::now();
    eprintln!("{}", style("crepus web build-full").dim());
    eprintln!();

    // L1 Rayon: collect all .crepus files in the site dir.
    let (crepus_files, entries) = collect_crepus_files(&args.site_dir);

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
        let mut wcmd = std::process::Command::new("cargo");
        wcmd.args(["build", "--target", "wasm32-unknown-unknown", "-p", &target])
            .env("CARGO_BUILD_INCREMENTAL", "true");
        match wcmd.spawn() {
            Ok(child) => wasm_child = Some(child),
            Err(e) => ui::warning(&format!("could not spawn wasm build: {e}")),
        }
    }

    if args.server {
        let target = format!("{site_name}-server");
        ui::step(&format!("spawning cargo build -p {target}"));
        let mut scmd = std::process::Command::new("cargo");
        scmd.args(["build", "-p", &target])
            .env("CARGO_BUILD_INCREMENTAL", "true");
        match scmd.spawn() {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn seo_head() -> SiteHead {
        let mut head = SiteHead {
            page_title: "Fallback title".into(),
            description: "Fallback description".into(),
            ..SiteHead::default()
        };
        head.seo = crate::crepus_toml::SeoConfig {
            title: Some("SEO title".into()),
            description: Some("SEO description".into()),
            canonical: Some("https://example.com/".into()),
            image: Some("https://example.com/og.png".into()),
            image_alt: Some("Preview".into()),
            site_name: Some("Example".into()),
            locale: Some("en_US".into()),
            og_type: Some("website".into()),
            twitter_card: Some("summary_large_image".into()),
            twitter_site: Some("@example".into()),
            twitter_creator: Some("@author".into()),
            keywords: vec!["rust".into(), "ui".into()],
            author: Some("AJ".into()),
            robots: Some("index,follow".into()),
            theme_color: Some("#101820".into()),
            application_name: Some("Example App".into()),
            generator: Some("crepuscularity".into()),
            alternates: vec![crate::crepus_toml::SeoAlternate {
                href: "https://example.com/fr/".into(),
                hreflang: Some("fr".into()),
                media: None,
                title: None,
                mime_type: None,
            }],
            json_ld: vec![r#"{"@context":"https://schema.org","@type":"WebSite"}"#.into()],
            robots_txt: None,
            sitemap: None,
        };
        head
    }

    #[test]
    fn render_index_html_emits_full_seo_head() {
        let html = render_index_html(&seo_head(), &[], "", "");
        assert!(html.contains(r#"<title>SEO title</title>"#));
        assert!(html.contains(r#"<meta name="description" content="SEO description">"#));
        assert!(html.contains(r#"<link rel="canonical" href="https://example.com/">"#));
        assert!(html.contains(r#"<meta name="keywords" content="rust, ui">"#));
        assert!(html.contains(r#"<meta name="author" content="AJ">"#));
        assert!(html.contains(r#"<meta name="robots" content="index,follow">"#));
        assert!(html.contains(r##"<meta name="theme-color" content="#101820">"##));
        assert!(html.contains(r#"<meta property="og:title" content="SEO title">"#));
        assert!(html.contains(r#"<meta property="og:image" content="https://example.com/og.png">"#));
        assert!(html.contains(r#"<meta name="twitter:creator" content="@author">"#));
        assert!(
            html.contains(r#"<link rel="alternate" href="https://example.com/fr/" hreflang="fr">"#)
        );
        assert!(html.contains(r#"<script type="application/ld+json">{"@context":"https://schema.org","@type":"WebSite"}</script>"#));
    }

    #[test]
    fn render_seo_auxiliary_files_from_config() {
        let mut head = seo_head();
        head.seo.robots_txt = Some(crate::crepus_toml::RobotsTxtConfig {
            enabled: true,
            user_agent: None,
            allow: vec!["/".into()],
            disallow: vec!["/private".into()],
            sitemap: None,
        });
        head.seo.sitemap = Some(crate::crepus_toml::SitemapConfig {
            enabled: true,
            base_url: Some("https://example.com".into()),
            paths: vec!["/".into(), "/docs/".into()],
            changefreq: Some("weekly".into()),
            priority: Some(0.8),
        });
        let tmp = tempfile::tempdir().expect("tempdir");
        write_seo_files(tmp.path(), &head);
        let robots = std::fs::read_to_string(tmp.path().join("robots.txt")).expect("robots");
        assert!(robots.contains("User-agent: *"));
        assert!(robots.contains("Allow: /"));
        assert!(robots.contains("Disallow: /private"));
        assert!(robots.contains("Sitemap: https://example.com/sitemap.xml"));
        let sitemap = std::fs::read_to_string(tmp.path().join("sitemap.xml")).expect("sitemap");
        assert!(sitemap.contains("<loc>https://example.com/</loc>"));
        assert!(sitemap.contains("<loc>https://example.com/docs/</loc>"));
        assert!(sitemap.contains("<changefreq>weekly</changefreq>"));
        assert!(sitemap.contains("<priority>0.8</priority>"));
    }

    #[test]
    fn llms_mode_emits_alternates_and_markdown_files() {
        let site = tempfile::tempdir().expect("site");
        let out = tempfile::tempdir().expect("out");
        std::fs::write(site.path().join("README.md"), "# Example\n\nReadable docs.")
            .expect("readme");
        let meta = crate::crepus_toml::WebTargetMeta {
            llms: Some(crate::crepus_toml::LlmsConfig {
                enabled: true,
                base_url: Some("https://example.com".into()),
                title: Some("Example".into()),
                description: Some("Agent-readable example.".into()),
                sources: vec![crate::crepus_toml::LlmsSource {
                    path: "README.md".into(),
                    href: None,
                    title: Some("README".into()),
                }],
            }),
            ..crate::crepus_toml::WebTargetMeta::default()
        };
        let head = merge_llms_alternates(seo_head(), Some(&meta));
        let html = render_index_html(&head, &[], "", "");
        assert!(html.contains(r#"<link rel="alternate" href="https://example.com/llms-full.txt" title="Full LLM bundle" type="text/plain">"#));
        assert!(html.contains(r#"<noscript><p><a href="https://example.com/llms-full.txt">Agent-readable text version</a></p></noscript>"#));
        let page_text = render_crepus_readable_text(
            r#"
h1
  "Main title"
section
  h2
    "Projects"
  a href="https://example.com/project"
    span
      "Project"
    span
      " — readable link text"
"#,
        );
        write_llms_files(
            site.path(),
            out.path(),
            &head,
            Some(&meta),
            Some(&page_text),
        );
        let llms = std::fs::read_to_string(out.path().join("llms.txt")).expect("llms");
        let full = std::fs::read_to_string(out.path().join("llms-full.txt")).expect("full");
        assert!(llms.contains("https://example.com/README.md"));
        assert!(full.contains("# Main title"));
        assert!(full.contains("## Projects"));
        assert!(full.contains("[Project — readable link text](https://example.com/project)"));
        assert!(full.contains("# README"));
        assert!(full.contains("Readable docs."));
        assert!(out.path().join("README.md").is_file());
    }

    #[test]
    fn sitemap_includes_discovered_html_when_enabled() {
        let mut head = seo_head();
        head.seo.sitemap = Some(crate::crepus_toml::SitemapConfig {
            enabled: true,
            base_url: Some("https://example.com".into()),
            paths: vec!["/".into()],
            changefreq: None,
            priority: None,
        });
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join("docs")).expect("docs dir");
        std::fs::write(tmp.path().join("docs/cli.html"), "").expect("cli");
        std::fs::write(tmp.path().join("docs/runtime.html"), "").expect("runtime");
        write_seo_files(tmp.path(), &head);
        let sitemap = std::fs::read_to_string(tmp.path().join("sitemap.xml")).expect("sitemap");
        assert!(sitemap.contains("<loc>https://example.com/</loc>"));
        assert!(sitemap.contains("<loc>https://example.com/docs/cli.html</loc>"));
        assert!(sitemap.contains("<loc>https://example.com/docs/runtime.html</loc>"));
    }
}
