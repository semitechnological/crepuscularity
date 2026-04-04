//! Browser extension commands for crepus CLI.

use console::style;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::ui;

// ── Embedded runtime assets ──────────────────────────────────────────────────

const ASSET_POPUP_HTML: &str = include_str!("../../crepuscularity-webext/assets/popup.html");
const ASSET_POPUP_JS: &str = include_str!("../../crepuscularity-webext/assets/popup.js");
const ASSET_POPUP_CSS: &str = include_str!("../../crepuscularity-webext/assets/popup.css");
const ASSET_BACKGROUND_JS: &str = include_str!("../../crepuscularity-webext/assets/background.js");
const ASSET_CONTENT_JS: &str = include_str!("../../crepuscularity-webext/assets/content.js");
const ASSET_CONTENT_CSS: &str = include_str!("../../crepuscularity-webext/assets/content.css");
const ASSET_BROWSER_SHIM: &str = include_str!("../../crepuscularity-webext/assets/browser-shim.js");
const ASSET_RUNTIME_ADAPTER: &str =
    include_str!("../../crepuscularity-webext/assets/runtime-as-adapter.js");
const ASSET_UNOCSS_JS: &[u8] =
    include_bytes!("../../crepuscularity-webext/assets/vendor/unocss.js");

// ── Entry point ──────────────────────────────────────────────────────────────

pub fn run(args: &[String]) {
    match args.first().map(|s| s.as_str()) {
        Some("new") => {
            let name = args.get(1).map(|s| s.as_str()).unwrap_or_else(|| {
                ui::error("Usage: crepus webext new <name>");
            });
            scaffold_extension(name);
        }

        Some("build") => {
            let app_path = parse_app_path(&args[1..]);
            build_extension(&app_path);
        }

        Some("manifest") => {
            let app_path = parse_app_path(&args[1..]);
            print_manifest(&app_path);
        }

        _ => print_webext_usage(),
    }
}

fn parse_app_path(args: &[String]) -> PathBuf {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--app" {
            if let Some(path) = args.get(i + 1) {
                return PathBuf::from(path);
            }
        }
        i += 1;
    }
    std::env::current_dir().unwrap()
}

fn print_webext_usage() {
    eprintln!("{}", style("crepus webext").cyan().bold());
    eprintln!("{}", style("Browser extension commands").dim());
    eprintln!();
    eprintln!("{}", style("COMMANDS").dim());
    eprintln!(
        "  {}  {}",
        style("new <name>            ").green(),
        style("scaffold a new browser extension").dim()
    );
    eprintln!(
        "  {}  {}",
        style("build [--app PATH]    ").green(),
        style("build extension to dist/unpacked/").dim()
    );
    eprintln!(
        "  {}  {}",
        style("manifest [--app PATH] ").green(),
        style("print generated manifest.json").dim()
    );
}

// ── scaffold ─────────────────────────────────────────────────────────────────

fn scaffold_extension(name: &str) {
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

    std::fs::create_dir_all(base.join("runtime/src")).unwrap();
    std::fs::create_dir_all(base.join("views")).unwrap();

    let webext_toml = format!(
        r#"[extension]
name = "{name}"
version = "0.1.0"
description = "A browser extension built with crepuscularity"

[capabilities]
storage = true
background-script = true
content-script = true
host-permissions = ["<all_urls>"]
"#
    );
    std::fs::write(base.join("webext.toml"), webext_toml).unwrap();

    let cargo_toml = format!(
        r#"[package]
name = "{slug}_runtime"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
crepuscularity-webext = {{ version = "0.1" }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
serde-wasm-bindgen = "0.6"
wasm-bindgen = "0.2"
"#
    );
    std::fs::write(base.join("runtime/Cargo.toml"), cargo_toml).unwrap();

    let lib_rs = r##"use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn runtime_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn render_popup(_state: JsValue) -> Result<JsValue, JsValue> {
    let html = r#"<div class="popup">Hello from crepuscularity!</div>"#;
    let result = serde_json::json!({ "html": html });
    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
"##;
    std::fs::write(base.join("runtime/src/lib.rs"), lib_rs).unwrap();

    let ui_crepus = r#"+++
[Popup.defaults]
title = "Extension"
description = ""
+++

--- Popup
div flex flex-col gap-4 p-4
  div text-xl font-bold
    "{title}"
  div text-sm text-zinc-500
    "{description}"
"#;
    std::fs::write(base.join("views/ui.crepus"), ui_crepus).unwrap();

    eprintln!(
        "\n{} created {}",
        ui::ok(),
        style(format!("{slug}/")).cyan().bold()
    );
    eprintln!();
    eprintln!("{}", style("Next steps:").dim());
    eprintln!("  cd {slug}");
    eprintln!("  crepus webext build");
    eprintln!(
        "  {}",
        style("# Load dist/unpacked/ in chrome://extensions").dim()
    );
    ui::done_in(t0.elapsed());
}

// ── build ─────────────────────────────────────────────────────────────────────

fn build_extension(app_path: &Path) {
    let t0 = Instant::now();

    let webext_toml = app_path.join("webext.toml");
    if !webext_toml.exists() {
        ui::error(&format!("no webext.toml found in {}", app_path.display()));
    }

    let manifest = match crepuscularity_webext::ExtensionManifest::load(&webext_toml) {
        Ok(m) => m,
        Err(e) => ui::error(&format!("failed to parse webext.toml: {e}")),
    };

    let ext_name = style(manifest.extension.name.as_str()).cyan().bold();
    eprintln!("{} building {ext_name}", style("crepus webext").dim());
    eprintln!();

    let dist = app_path.join("dist/unpacked");
    let src_dir = dist.join("src");
    let vendor_dir = dist.join("vendor");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::create_dir_all(&vendor_dir).unwrap();

    // ── Step 1: manifest.json ────────────────────────────────────────────────
    {
        let sp = ui::spinner("generating manifest.json");
        let json = manifest.to_manifest_v3_json();
        std::fs::write(dist.join("manifest.json"), &json).unwrap();
        ui::spinner_ok(&sp, "manifest.json");
    }

    // ── Step 2: runtime assets ───────────────────────────────────────────────
    {
        let sp = ui::spinner("writing runtime assets");
        std::fs::write(src_dir.join("popup.html"), ASSET_POPUP_HTML).unwrap();
        std::fs::write(src_dir.join("popup.css"), ASSET_POPUP_CSS).unwrap();
        std::fs::write(src_dir.join("popup.js"), ASSET_POPUP_JS).unwrap();
        std::fs::write(src_dir.join("background.js"), ASSET_BACKGROUND_JS).unwrap();
        std::fs::write(src_dir.join("content.js"), ASSET_CONTENT_JS).unwrap();
        std::fs::write(src_dir.join("content.css"), ASSET_CONTENT_CSS).unwrap();
        std::fs::write(src_dir.join("browser-shim.js"), ASSET_BROWSER_SHIM).unwrap();
        std::fs::write(src_dir.join("runtime-as-adapter.js"), ASSET_RUNTIME_ADAPTER).unwrap();
        std::fs::write(vendor_dir.join("unocss.js"), ASSET_UNOCSS_JS).unwrap();
        ui::spinner_ok(&sp, "runtime assets");
    }

    // ── Step 3: WASM runtime ─────────────────────────────────────────────────
    let runtime_dir = app_path.join("runtime");
    if runtime_dir.exists() {
        build_wasm_runtime(app_path, &runtime_dir, &vendor_dir);
    } else {
        ui::warning("no runtime/ directory — skipping WASM compile");
        ui::warning("run `crepus webext new` to scaffold a full project");
    }

    // ── Step 4: pre-render popup HTML ────────────────────────────────────────
    // If views/popup.crepus exists, render it natively and write popup.html so
    // the popup opens instantly without loading WASM at runtime.
    let popup_template = app_path.join("views/popup.crepus");
    if popup_template.exists() {
        let sp = ui::spinner("pre-rendering popup.html");
        match prerender_popup_html(&popup_template, &manifest) {
            Ok(html) => {
                std::fs::write(src_dir.join("popup.html"), &html).unwrap();
                ui::spinner_ok(&sp, "popup.html (pre-rendered from popup.crepus)");
            }
            Err(e) => {
                sp.finish_and_clear();
                ui::warning(&format!("popup pre-render failed: {e}"));
            }
        }
    }

    // ── Capability check ────────────────────────────────────────────────────
    match crepuscularity_webext::check_project_capabilities(app_path) {
        Ok(missing) if !missing.is_empty() => {
            eprintln!();
            ui::warning("missing capabilities detected — add these to webext.toml:");
            for cap in &missing {
                eprintln!(
                    "  {} {}",
                    ui::dim("→"),
                    style(cap.to_permission_string()).yellow()
                );
            }
        }
        Err(e) => {
            ui::warning(&format!("capability scan failed: {e}"));
        }
        _ => {}
    }

    eprintln!(
        "\n{} built to {}",
        ui::ok(),
        style(dist.display().to_string()).cyan()
    );
    eprintln!(
        "  {} load {} in {}",
        ui::dim("→"),
        style(dist.display().to_string()).underlined(),
        style("chrome://extensions").cyan()
    );
    ui::done_in(t0.elapsed());
}

fn build_wasm_runtime(app_path: &Path, runtime_dir: &Path, vendor_dir: &Path) {
    // Check wasm32 target is available
    {
        let sp = ui::spinner("compiling WASM runtime");
        let result = std::process::Command::new("cargo")
            .args([
                "build",
                "--target",
                "wasm32-unknown-unknown",
                "--release",
                "--quiet",
            ])
            .current_dir(runtime_dir)
            .output();

        match result {
            Ok(out) if out.status.success() => {
                ui::spinner_ok(&sp, "WASM compiled");
            }
            Ok(out) => {
                sp.finish_and_clear();
                let stderr = String::from_utf8_lossy(&out.stderr);
                eprintln!("  {} WASM compile failed", ui::err());
                for line in stderr.lines().take(15) {
                    eprintln!("    {}", style(line).dim());
                }
                if stderr.lines().count() > 15 {
                    eprintln!(
                        "    {}",
                        style("... (run cargo build manually for full output)").dim()
                    );
                }
                // Non-fatal — extension is still partially built
                return;
            }
            Err(e) => {
                sp.finish_and_clear();
                ui::warning(&format!("cargo not found: {e}"));
                return;
            }
        }
    }

    // Find the built .wasm file. Cargo puts the target at the workspace root,
    // which may be app_path (when runtime/ is a workspace member) rather than
    // runtime_dir itself. Check both.
    let workspace_target = app_path.join("target/wasm32-unknown-unknown/release");
    let local_target = runtime_dir.join("target/wasm32-unknown-unknown/release");
    let wasm_file = find_wasm_file(&workspace_target).or_else(|| find_wasm_file(&local_target));

    let Some(wasm_path) = wasm_file else {
        ui::warning("built .wasm not found in target/wasm32-unknown-unknown/release/");
        return;
    };

    // Run wasm-bindgen
    {
        let sp = ui::spinner("running wasm-bindgen");
        let out_dir = vendor_dir.to_string_lossy().to_string();
        let result = std::process::Command::new("wasm-bindgen")
            .args([
                "--target",
                "web",
                "--out-dir",
                &out_dir,
                "--out-name",
                "runtime",
                wasm_path.to_str().unwrap(),
            ])
            .output();

        match result {
            Ok(out) if out.status.success() => {
                ui::spinner_ok(&sp, "wasm-bindgen — vendor/runtime.js + runtime_bg.wasm");
            }
            Ok(out) => {
                sp.finish_and_clear();
                let stderr = String::from_utf8_lossy(&out.stderr);
                eprintln!("  {} wasm-bindgen failed", ui::err());
                for line in stderr.lines().take(10) {
                    eprintln!("    {}", style(line).dim());
                }
                ui::warning("install wasm-bindgen-cli: cargo install wasm-bindgen-cli");
            }
            Err(_) => {
                sp.finish_and_clear();
                // Copy raw wasm as fallback
                std::fs::copy(&wasm_path, vendor_dir.join("runtime_bg.wasm")).ok();
                ui::warning("wasm-bindgen not found — copied raw .wasm");
                ui::warning("install: cargo install wasm-bindgen-cli");
            }
        }
    }
}

/// System prompt text passed as `{system_prompt}` context variable into the
/// popup template. Defined here so it can be kept in sync with the pre-render
/// step without duplicating it in the app's Rust source.
pub const SYSTEM_PROMPT: &str = "\
When creating charts, forms, interactive UI, or visualisations,
wrap output in <ai-anywhere> tags for the AI Anywhere browser extension.

── Format ───────────────────────────────────────────────────
<ai-anywhere type=\"widget\" title=\"Widget Title\">
  <anywhere-ui lang=\"crepus\">
    <!-- crepus template here (see syntax below) -->
  </anywhere-ui>
  <anywhere-data>{\"key\": \"value\"}</anywhere-data>
  <anywhere-script lang=\"js\">/* optional JS */</anywhere-script>
</ai-anywhere>

ui langs:     crepus (preferred) | html
script langs: js | mermaid | latex
types:        widget | chart | form | visualization | tool

── Crepus template syntax ───────────────────────────────────
Indent-based. Children are indented under their parent.

Elements:
  tag class1 class2 attr=\"value\" dynattr={expr}
    \"child text with {variable} interpolation\"

Conditionals:
  if {condition}
    span
      \"yes\"
  else
    span
      \"no\"

Loops:
  for item in {items}
    div row
      \"{item.name}: {item.value}\"

Match:
  match {status}
    \"ok\" =>
      span green
        \"OK\"
    _ =>
      span
        \"unknown\"

Variables:
  $: let total = {count * price}
  $: default title = {\"Untitled\"}

Actions (pass data-* as action payload):
  button primary type=\"button\" data-action=\"submit\" data-id={item.id}
    \"Submit\"

── Crepus example ───────────────────────────────────────────
<ai-anywhere type=\"chart\" title=\"Status\">
  <anywhere-ui lang=\"crepus\">
div dashboard
  h2 title
    \"{title}\"
  for item in {items}
    div row
      span label
        \"{item.label}\"
      span value
        \"{item.value}\"
  </anywhere-ui>
  <anywhere-data>{\"title\":\"Stats\",\"items\":[{\"label\":\"Users\",\"value\":\"42\"},{\"label\":\"Active\",\"value\":\"7\"}]}</anywhere-data>
</ai-anywhere>";

/// Render `views/popup.crepus` with three states (main / help / crepus-ref)
/// and embed all three as sibling `<div>` elements in the returned HTML.
/// The popup opens instantly from the pre-rendered HTML; popup.js only reads
/// storage to patch checkbox states and handles click events — no WASM needed.
fn prerender_popup_html(
    template_path: &Path,
    manifest: &crepuscularity_webext::ExtensionManifest,
) -> Result<String, String> {
    use crepuscularity_core::context::TemplateContext;
    use crepuscularity_web::render_from_files;

    let source = std::fs::read_to_string(template_path).map_err(|e| e.to_string())?;
    let mut files = HashMap::new();
    files.insert("popup.crepus".to_string(), source);

    let render = |show_help: bool, show_crepus: bool| -> Result<String, String> {
        let mut ctx = TemplateContext::new();
        ctx.set("enabled", true);
        ctx.set("auto_render", false);
        ctx.set("show_help", show_help);
        ctx.set("show_crepus", show_crepus);
        ctx.set("system_prompt", SYSTEM_PROMPT);
        render_from_files(&files, "popup.crepus", &ctx)
    };

    let main_html = render(false, false)?;
    let help_html = render(true, false)?;
    let crepus_html = render(false, true)?;

    let title = &manifest.extension.name;
    // Inline the CSS so the popup is a single self-contained file with zero
    // external fetches before it renders. UnoCSS is not needed here — the popup
    // uses BEM class names from popup.css, not utility classes.
    let css = ASSET_POPUP_CSS;
    Ok(format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title}</title>
  <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200">
  <style>{css}</style>
</head>
<body>
  <div id="view-main">{main_html}</div>
  <div id="view-help" hidden>{help_html}</div>
  <div id="view-crepus" hidden>{crepus_html}</div>
  <script type="module" src="./popup.js"></script>
</body>
</html>"#
    ))
}

fn find_wasm_file(dir: &Path) -> Option<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return None;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.extension().map(|e| e == "wasm").unwrap_or(false)
            && !p
                .file_name()
                .map(|n| n.to_string_lossy().ends_with(".d.wasm"))
                .unwrap_or(false)
        {
            return Some(p);
        }
    }
    None
}

// ── manifest ──────────────────────────────────────────────────────────────────

fn print_manifest(app_path: &Path) {
    let webext_toml = app_path.join("webext.toml");
    if !webext_toml.exists() {
        let crex_path = app_path.join("manifest.crex");
        if crex_path.exists() {
            match crepuscularity_webext::ExtensionManifest::load(&crex_path) {
                Ok(m) => {
                    println!("{}", m.to_manifest_v3_json());
                    return;
                }
                Err(e) => ui::error(&format!("failed to parse manifest.crex: {e}")),
            }
        }
        ui::error(&format!(
            "no webext.toml or manifest.crex found in {}",
            app_path.display()
        ));
    }

    match crepuscularity_webext::ExtensionManifest::load(&webext_toml) {
        Ok(m) => println!("{}", m.to_manifest_v3_json()),
        Err(e) => ui::error(&format!("failed to parse webext.toml: {e}")),
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────
