//! Browser extension commands for crepus CLI.

use console::style;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::ui;
use crate::wasm_bundle::{cargo_build_wasm32, find_wasm_file, run_wasm_bindgen, wasm_release_dirs};

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
            build_extension(&app_path, false);
        }

        Some("dev") => {
            let app_path = parse_app_path(&args[1..]);
            build_extension(&app_path, true);
            watch_and_reload(&app_path);
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
        style("dev [--app PATH]      ").green(),
        style("build, watch, and hot-reload").dim()
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

    let webext_toml = scaffold_webext_toml(name);
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
pub fn popup_main() {}

#[wasm_bindgen]
pub fn options_main() {}

#[wasm_bindgen]
pub fn content_main() {}

#[wasm_bindgen]
pub fn background_main() {}

#[wasm_bindgen]
pub fn settings_seed() -> Result<(), JsValue> {
    Ok(())
}

#[wasm_bindgen]
pub fn handle_background_message(message: JsValue) -> Result<JsValue, JsValue> {
    let message_json: serde_json::Value =
        serde_wasm_bindgen::from_value(message).unwrap_or(serde_json::Value::Null);
    let result = serde_json::json!({
        "ok": true,
        "echo": message_json,
    });
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

fn scaffold_webext_toml(name: &str) -> String {
    format!(
        r#"[extension]
name = "{name}"
version = "0.1.0"
description = "A browser extension built with crepuscularity"

[capabilities]
storage = true
background-script = true
content-script = true
host-permissions = ["https://example.com/*"]
"#
    )
}

// ── auto-detect ──────────────────────────────────────────────────────────────

fn auto_detect_pages(app_path: &Path, manifest: &mut crepuscularity_webext::ExtensionManifest) {
    let pages_dir = app_path.join("pages");
    let opts = &mut manifest.options;

    // action_popup: look for popup.crepus or action.crepus
    if opts.action_popup.is_none() {
        for stem in ["popup", "action"] {
            let p = pages_dir.join(format!("{stem}.crepus"));
            if p.exists() {
                opts.action_popup = Some(format!("pages/{stem}.crepus"));
                break;
            }
        }
    }

    // options_ui: look for options.crepus
    if opts.options_ui.is_none() {
        let p = pages_dir.join("options.crepus");
        if p.exists() {
            opts.options_ui = Some(crepuscularity_webext::OptionsUiSpec {
                page: "pages/options.crepus".to_string(),
                browser_style: Some(false),
                open_in_tab: Some(true),
            });
        }
    }

    // chrome_url_overrides: look for new-tab.crepus
    if manifest.chrome_url_overrides.is_empty() {
        let p = pages_dir.join("new-tab.crepus");
        if p.exists() {
            manifest.chrome_url_overrides.insert(
                "newtab".to_string(),
                "pages/new-tab.crepus".to_string(),
            );
        }
    }
}

fn auto_detect_icons(app_path: &Path, manifest: &mut crepuscularity_webext::ExtensionManifest) {
    let icons_dir = app_path.join("icons");
    if !icons_dir.is_dir() { return; }

    if manifest.options.icons.is_empty() {
        if let Ok(entries) = std::fs::read_dir(&icons_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                // Only match icon{size}.png (not action_* icons)
                if name.starts_with("icon") && !name.contains("action") {
                    if let Some(size) = detect_icon_size(&name) {
                        manifest.options.icons.insert(size, format!("icons/{name}"));
                    }
                }
            }
        }
    }

    if manifest.options.action_icons.is_empty() {
        if let Ok(entries) = std::fs::read_dir(&icons_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                // Match action_disabled_* files
                if name.contains("action_disabled") {
                    if let Some(size) = detect_icon_size(&name) {
                        manifest.options.action_icons.insert(size, format!("icons/{name}"));
                    }
                }
            }
        }
    }
}

fn detect_icon_size(filename: &str) -> Option<String> {
    // Extract digits from filename: icon128.png → "128", action_disabled_16.png → "16"
    let digits: String = filename.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() { None } else { Some(digits) }
}

// ── build ─────────────────────────────────────────────────────────────────────

fn build_extension(app_path: &Path, dev: bool) {
    let t0 = Instant::now();

    let webext_toml = app_path.join("webext.toml");
    if !webext_toml.exists() {
        ui::error(&format!("no webext.toml found in {}", app_path.display()));
    }

    let mut manifest = match crepuscularity_webext::ExtensionManifest::load(&webext_toml) {
        Ok(m) => m,
        Err(e) => ui::error(&format!("failed to parse webext.toml: {e}")),
    };

    // Auto-detect options from project structure
    auto_detect_pages(app_path, &mut manifest);
    auto_detect_icons(app_path, &mut manifest);

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
        let mut json = manifest.to_manifest_v3_json();
        if dev {
            json = inject_dev_content_script(&json);
        }
        std::fs::write(dist.join("manifest.json"), &json).unwrap();
        ui::spinner_ok(&sp, "manifest.json");
    }

    // ── Step 2: runtime assets ───────────────────────────────────────────────
    {
        let sp = ui::spinner("writing runtime assets");
        std::fs::write(
            src_dir.join("popup.html"),
            crepuscularity_webext::extension_assets::POPUP_HTML,
        )
        .unwrap();
        std::fs::write(
            src_dir.join("popup.css"),
            crepuscularity_webext::extension_assets::POPUP_CSS,
        )
        .unwrap();
        std::fs::write(
            src_dir.join("popup.js"),
            crepuscularity_webext::extension_assets::POPUP_JS,
        )
        .unwrap();
        std::fs::write(
            src_dir.join("options.js"),
            crepuscularity_webext::extension_assets::OPTIONS_JS,
        )
        .unwrap();
        std::fs::write(
            src_dir.join("background.js"),
            crepuscularity_webext::extension_assets::BACKGROUND_JS,
        )
        .unwrap();
        let custom_bg = app_path.join("src/background.js");
        if custom_bg.exists() {
            std::fs::copy(&custom_bg, src_dir.join("background.js")).unwrap();
        }
        std::fs::write(
            src_dir.join("content.js"),
            crepuscularity_webext::extension_assets::CONTENT_JS,
        )
        .unwrap();
        std::fs::write(
            src_dir.join("content.css"),
            crepuscularity_webext::extension_assets::CONTENT_CSS,
        )
        .unwrap();
        std::fs::write(
            src_dir.join("browser-shim.js"),
            crepuscularity_webext::extension_assets::BROWSER_SHIM,
        )
        .unwrap();
        std::fs::write(
            src_dir.join("runtime-as-adapter.js"),
            crepuscularity_webext::extension_assets::RUNTIME_ADAPTER,
        )
        .unwrap();
        if dev {
            std::fs::write(
                src_dir.join("dev.js"),
                crepuscularity_webext::extension_assets::DEV_JS,
            )
            .unwrap();
        }
        std::fs::write(
            vendor_dir.join("unocss.js"),
            crepuscularity_webext::extension_assets::UNOCSS_JS,
        )
        .unwrap();
        copy_app_assets(app_path, &dist).unwrap();
        render_crepus_css_assets(app_path, &dist).unwrap_or_else(|e| {
            ui::error(&format!("render extension .css.crepus assets: {e}"));
        });
        render_crepus_pages(app_path, &dist, &manifest).unwrap_or_else(|e| {
            ui::error(&format!("render extension .crepus pages: {e}"));
        });
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
    {
        let sp = ui::spinner("compiling WASM runtime");
        match cargo_build_wasm32(runtime_dir) {
            Ok(()) => {
                ui::spinner_ok(&sp, "WASM compiled");
            }
            Err(stderr) => {
                sp.finish_and_clear();
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
                return;
            }
        }
    }

    let (workspace_target, local_target) = wasm_release_dirs(app_path, runtime_dir);
    let wasm_file = find_wasm_file(&workspace_target).or_else(|| find_wasm_file(&local_target));

    let Some(wasm_path) = wasm_file else {
        ui::warning("built .wasm not found in target/wasm32-unknown-unknown/release/");
        return;
    };

    {
        let sp = ui::spinner("running wasm-bindgen");
        match run_wasm_bindgen(&wasm_path, vendor_dir, "runtime") {
            Ok(()) => {
                ui::spinner_ok(&sp, "wasm-bindgen — vendor/runtime.js + runtime_bg.wasm");
            }
            Err(err) => {
                sp.finish_and_clear();
                if err.starts_with("wasm-bindgen:") {
                    let _ = std::fs::copy(&wasm_path, vendor_dir.join("runtime_bg.wasm"));
                    ui::warning("wasm-bindgen not found — copied raw .wasm");
                    ui::warning("install: cargo install wasm-bindgen-cli");
                } else {
                    eprintln!("  {} wasm-bindgen failed", ui::err());
                    for line in err.lines().take(10) {
                        eprintln!("    {}", style(line).dim());
                    }
                    ui::warning("install wasm-bindgen-cli: cargo install wasm-bindgen-cli");
                }
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

── Format (crepus + data only — no JavaScript) ─────────────
<ai-anywhere type=\"widget\" title=\"Widget Title\">
  <anywhere-ui lang=\"crepus\">
    <!-- crepus template here (see syntax below) -->
  </anywhere-ui>
  <anywhere-data>{\"key\": \"value\"}</anywhere-data>
</ai-anywhere>

ui langs: crepus (preferred) | html
types:    widget | chart | form | visualization | tool
Do NOT use <anywhere-script>. Put all UI in crepus. Put all data in anywhere-data JSON.
Crepus expressions support variables, if/for/match, and simple math — not arrow functions or .filter().
Precompute counts/aggregates in anywhere-data when needed.

── Crepus template syntax ───────────────────────────────────
Indent-based. Children are indented under their parent.

Elements (put ALL HTML attributes on the SAME line as the tag):
  button.primary type=\"button\" data-action=\"toggle\" data-id={id}
    \"Toggle\"
  div card
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
/// The popup opens instantly from the pre-rendered HTML; popup.js still loads
/// runtime WASM and requires `popup_main` for behavior wiring.
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
    let css = crepuscularity_webext::extension_assets::POPUP_CSS;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffold_webext_toml_uses_narrow_content_scope() {
        let toml = scaffold_webext_toml("My Extension");
        assert!(!toml.contains("<all_urls>"));

        let manifest: crepuscularity_webext::ExtensionManifest = toml::from_str(&toml).unwrap();
        assert!(manifest.capabilities.content_script);
        assert_eq!(
            manifest.capabilities.host_permissions,
            vec!["https://example.com/*"]
        );
    }

    #[test]
    fn copy_app_assets_overlays_app_owned_directories_only() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let app = tmp.path().join("app");
        let dist = tmp.path().join("dist/unpacked");

        std::fs::create_dir_all(app.join("src")).expect("app src");
        std::fs::create_dir_all(app.join("pages")).expect("app pages");
        std::fs::create_dir_all(app.join("icons")).expect("app icons");
        std::fs::create_dir_all(app.join("resources")).expect("app resources");
        std::fs::create_dir_all(dist.join("src")).expect("dist src");
        std::fs::create_dir_all(dist.join("vendor")).expect("dist vendor");

        std::fs::write(app.join("src/content.js"), "custom content").expect("write app src");
        std::fs::write(app.join("pages/options.html"), "options").expect("write page");
        std::fs::write(app.join("icons/icon16.png"), b"icon").expect("write icon");
        std::fs::write(app.join("resources/tlds.txt"), "com").expect("write resource");
        std::fs::write(dist.join("src/content.js"), "runtime content").expect("write runtime src");
        std::fs::write(dist.join("vendor/runtime_bg.wasm"), b"wasm").expect("write wasm");

        copy_app_assets(&app, &dist).expect("copy app assets");

        assert_eq!(
            std::fs::read_to_string(dist.join("src/content.js")).expect("read content"),
            "custom content"
        );
        assert_eq!(
            std::fs::read_to_string(dist.join("pages/options.html")).expect("read page"),
            "options"
        );
        assert_eq!(
            std::fs::read(dist.join("icons/icon16.png")).expect("read icon"),
            b"icon"
        );
        assert_eq!(
            std::fs::read_to_string(dist.join("resources/tlds.txt")).expect("read resource"),
            "com"
        );
        assert_eq!(
            std::fs::read(dist.join("vendor/runtime_bg.wasm")).expect("read wasm"),
            b"wasm"
        );
    }

    #[test]
    fn renders_extension_pages_from_crepus_sources() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let app = tmp.path().join("app");
        let dist = tmp.path().join("dist/unpacked");
        std::fs::create_dir_all(app.join("pages")).expect("pages");

        std::fs::write(
            app.join("pages/options.crepus"),
            "section #root\n  h1\n    \"Options\"\n<style>\n#root{color:#000}\n</style>",
        )
        .expect("write crepus");

        let manifest = crepuscularity_webext::ExtensionManifest {
            extension: crepuscularity_webext::ExtensionInfo {
                name: "Test Extension".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                author: None,
                homepage: None,
                minimum_chrome_version: None,
            },
            capabilities: Default::default(),
            content_scripts: Vec::new(),
            plugins: HashMap::new(),
            options: Default::default(),
            web_accessible_resources: Default::default(),
            commands: Default::default(),
            chrome_url_overrides: Default::default(),
        };

        render_crepus_pages(&app, &dist, &manifest).expect("render pages");
        let html = std::fs::read_to_string(dist.join("pages/options.html")).expect("read html");
        assert!(html.contains("<title>Test Extension Options</title>"));
        assert!(html.contains("<section id=\"root\">"));
        assert!(html.contains("#root{color:#000}"));
        assert!(html.contains("../src/options.js"));
    }

    #[test]
    fn renders_extension_css_from_crepus_sources() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let app = tmp.path().join("app");
        let dist = tmp.path().join("dist/unpacked");
        std::fs::create_dir_all(app.join("src")).expect("src");

        std::fs::write(
            app.join("src/content.css.crepus"),
            "div\n<style>\n.vc-find{color:#000}\n</style>",
        )
        .expect("write crepus css");

        render_crepus_css_assets(&app, &dist).expect("render css");
        assert_eq!(
            std::fs::read_to_string(dist.join("src/content.css")).expect("read css"),
            ".vc-find{color:#000}"
        );
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn copy_app_assets(app_path: &Path, dist: &Path) -> std::io::Result<()> {
    for dir in ["src", "pages", "icons", "resources"] {
        let source = app_path.join(dir);
        if source.is_dir() {
            copy_dir_contents(&source, &dist.join(dir))?;
        }
    }
    Ok(())
}

fn copy_dir_contents(source: &Path, destination: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let target = destination.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_contents(&entry.path(), &target)?;
        } else if file_type.is_file() {
            if entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext == "crepus")
            {
                continue;
            }
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn render_crepus_css_assets(app_path: &Path, dist: &Path) -> Result<(), String> {
    let src_dir = app_path.join("src");
    if !src_dir.is_dir() {
        return Ok(());
    }
    render_crepus_css_assets_in(&src_dir, &src_dir, &dist.join("src"))
}

fn render_crepus_css_assets_in(root: &Path, dir: &Path, out_root: &Path) -> Result<(), String> {
    use crepuscularity_core::preprocess::strip_indent_decorators;

    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            render_crepus_css_assets_in(root, &path, out_root)?;
            continue;
        }
        if !path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".css.crepus"))
        {
            continue;
        }
        let rel = path.strip_prefix(root).map_err(|e| e.to_string())?;
        let output_name = rel
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| "invalid css asset name".to_string())?
            .trim_end_matches(".crepus")
            .to_string();
        let output = out_root.join(rel).with_file_name(output_name);
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let source = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        std::fs::write(output, strip_indent_decorators(&source).inline_css)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn render_crepus_pages(
    app_path: &Path,
    dist: &Path,
    manifest: &crepuscularity_webext::ExtensionManifest,
) -> Result<(), String> {
    let pages_dir = app_path.join("pages");
    if !pages_dir.is_dir() {
        return Ok(());
    }

    let mut files = HashMap::new();
    let mut entries = Vec::new();
    collect_crepus_pages(&pages_dir, &pages_dir, &mut files, &mut entries)?;
    if entries.is_empty() {
        return Ok(());
    }

    let out_dir = dist.join("pages");
    let src_dir = dist.join("src");
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;

    for entry in &entries {
        let source = files
            .get(entry)
            .ok_or_else(|| format!("missing page source: {entry}"))?;
        let stem = Path::new(entry)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("page");
        // WASM exports use underscore, not hyphen
        let fn_name = stem.replace('-', "_");

        // Generate JS bootstrapper for every page stem
        let js = format!(
            r#"import init, * as runtime from "../vendor/runtime.js";
const wasmBytes = await fetch("../vendor/runtime_bg.wasm").then(r => r.arrayBuffer());
await init({{ module_or_path: wasmBytes }});
if (typeof runtime.{fn}_main === "function") runtime.{fn}_main();"#,
            fn = fn_name
        );
        std::fs::write(src_dir.join(format!("{stem}.js")), &js)
            .map_err(|e| format!("write {stem}.js: {e}"))?;

        let html = render_crepus_page_html(source, &files, entry, manifest)?;
        let output = out_dir
            .join(entry.trim_end_matches(".crepus"))
            .with_extension("html");
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(output, html).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn collect_crepus_pages(
    root: &Path,
    dir: &Path,
    files: &mut HashMap<String, String>,
    entries: &mut Vec<String>,
) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            collect_crepus_pages(root, &path, files, entries)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("crepus") {
            let rel = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            let source = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            files.insert(rel.clone(), source);
            entries.push(rel);
        }
    }
    entries.sort();
    Ok(())
}

fn render_crepus_page_html(
    source: &str,
    files: &HashMap<String, String>,
    entry: &str,
    manifest: &crepuscularity_webext::ExtensionManifest,
) -> Result<String, String> {
    use crepuscularity_core::context::TemplateContext;
    use crepuscularity_core::preprocess::{google_fonts_head_markup, strip_indent_decorators};
    use crepuscularity_web::render_from_files;

    let decorators = strip_indent_decorators(source);
    let body = render_from_files(files, entry, &TemplateContext::new())?;
    let stem = Path::new(entry)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");
    let title_suffix = title_case(stem);
    let title = if title_suffix.is_empty() {
        manifest.extension.name.clone()
    } else {
        format!("{} {}", manifest.extension.name, title_suffix)
    };
    let script = format!(r#"<script type="module" src="../src/{stem}.js"></script>"#);
    let fonts = google_fonts_head_markup(&decorators.google_fonts);
    let style = if decorators.inline_css.is_empty() {
        String::new()
    } else {
        format!("<style>{}</style>", decorators.inline_css)
    };
    Ok(format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title}</title>
  {fonts}
  {style}
</head>
<body>
{body}
{script}
</body>
</html>"#
    ))
}

fn inject_dev_content_script(manifest_json: &str) -> String {
    if let Ok(mut v) = serde_json::from_str::<serde_json::Value>(manifest_json) {
        if let Some(scripts) = v.get_mut("content_scripts").and_then(|v| v.as_array_mut()) {
            for entry in scripts {
                if let Some(js) = entry.get_mut("js").and_then(|v| v.as_array_mut()) {
                    js.push(serde_json::Value::String("src/dev.js".to_string()));
                }
            }
        }
        return serde_json::to_string_pretty(&v).unwrap_or_else(|_| manifest_json.to_string());
    }
    manifest_json.to_string()
}

fn watch_and_reload(app_path: &Path) {
    use console::style;
    use notify::Watcher;
    use std::sync::mpsc::channel;
    use std::sync::atomic::{AtomicU64, Ordering};

    eprintln!();
    eprintln!("  {}", style("watching for changes — Ctrl+C to stop").dim());

    let reload_id = AtomicU64::new(1);
    let src_dir = app_path.join("dist/unpacked/src");
    let _ = std::fs::write(src_dir.join(".reload-id"), "1");

    let (tx, rx) = channel();
    let mut watcher = match notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
        if let Ok(event) = res {
            if event.kind.is_modify() || matches!(event.kind, notify::EventKind::Create(_)) {
                let _ = tx.send(());
            }
        }
    }) {
        Ok(w) => w,
        Err(e) => {
            ui::warning(&format!("file watcher failed: {e}"));
            return;
        }
    };

    let dirs = ["runtime/src", "runtime/views", "src", "pages", "views", "icons", "resources"];
    for dir in &dirs {
        let d = app_path.join(dir);
        if d.exists() {
            let _ = watcher.watch(&d, notify::RecursiveMode::Recursive);
        }
    }
    let _ = watcher.watch(
        app_path.join("webext.toml").as_path(),
        notify::RecursiveMode::NonRecursive,
    );

    // Throttle: rebuild at most every 500ms
    loop {
        let _ = rx.recv();
        // drain pending events
        while rx.try_recv().is_ok() {}
        std::thread::sleep(std::time::Duration::from_millis(200));

        eprintln!();
        let sp = ui::spinner("rebuilding");
        let t0 = std::time::Instant::now();
        build_extension(app_path, true);
        sp.finish_and_clear();

        let id = reload_id.fetch_add(1, Ordering::SeqCst);
        let _ = std::fs::write(src_dir.join(".reload-id"), id.to_string());
        ui::done_in(t0.elapsed());
    }
}

fn title_case(value: &str) -> String {
    value
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            let mut out = first.to_uppercase().to_string();
            out.push_str(chars.as_str());
            out
        })
        .collect::<Vec<_>>()
        .join(" ")
}
