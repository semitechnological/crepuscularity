//! Browser extension commands for crepus CLI.

use console::style;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::ui;

// ── Embedded runtime assets ──────────────────────────────────────────────────

const ASSET_POPUP_HTML: &str =
    include_str!("../../crepuscularity-webext/assets/popup.html");
const ASSET_POPUP_JS: &str =
    include_str!("../../crepuscularity-webext/assets/popup.js");
const ASSET_POPUP_CSS: &str =
    include_str!("../../crepuscularity-webext/assets/popup.css");
const ASSET_BACKGROUND_JS: &str =
    include_str!("../../crepuscularity-webext/assets/background.js");
const ASSET_CONTENT_JS: &str =
    include_str!("../../crepuscularity-webext/assets/content.js");
const ASSET_CONTENT_CSS: &str =
    include_str!("../../crepuscularity-webext/assets/content.css");
const ASSET_BROWSER_SHIM: &str =
    include_str!("../../crepuscularity-webext/assets/browser-shim.js");
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
    eprintln!("  {}", style("# Load dist/unpacked/ in chrome://extensions").dim());
    ui::done_in(t0.elapsed());
}

// ── build ─────────────────────────────────────────────────────────────────────

fn build_extension(app_path: &Path) {
    let t0 = Instant::now();

    let webext_toml = app_path.join("webext.toml");
    if !webext_toml.exists() {
        ui::error(&format!(
            "no webext.toml found in {}",
            app_path.display()
        ));
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

    // ── Step 2: copy views ───────────────────────────────────────────────────
    let views_src = app_path.join("views");
    if views_src.exists() {
        let sp = ui::spinner("copying views");
        copy_dir_recursive(&views_src, &dist.join("views"));
        ui::spinner_ok(&sp, "views/");
    }

    // ── Step 3: runtime assets ───────────────────────────────────────────────
    {
        let sp = ui::spinner("writing runtime assets");
        std::fs::write(dist.join("popup.html"), ASSET_POPUP_HTML).unwrap();
        std::fs::write(dist.join("popup.css"), ASSET_POPUP_CSS).unwrap();
        std::fs::write(src_dir.join("popup.js"), ASSET_POPUP_JS).unwrap();
        std::fs::write(src_dir.join("background.js"), ASSET_BACKGROUND_JS).unwrap();
        std::fs::write(src_dir.join("content.js"), ASSET_CONTENT_JS).unwrap();
        std::fs::write(src_dir.join("content.css"), ASSET_CONTENT_CSS).unwrap();
        std::fs::write(src_dir.join("browser-shim.js"), ASSET_BROWSER_SHIM).unwrap();
        std::fs::write(src_dir.join("runtime-as-adapter.js"), ASSET_RUNTIME_ADAPTER).unwrap();
        std::fs::write(vendor_dir.join("unocss.js"), ASSET_UNOCSS_JS).unwrap();
        ui::spinner_ok(&sp, "runtime assets");
    }

    // ── Step 4: WASM runtime ─────────────────────────────────────────────────
    let runtime_dir = app_path.join("runtime");
    if runtime_dir.exists() {
        build_wasm_runtime(&runtime_dir, &vendor_dir);
    } else {
        ui::warning("no runtime/ directory — skipping WASM compile");
        ui::warning("run `crepus webext new` to scaffold a full project");
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

fn build_wasm_runtime(runtime_dir: &Path, vendor_dir: &Path) {
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
                    eprintln!("    {}", style("... (run cargo build manually for full output)").dim());
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

    // Find the built .wasm file
    let wasm_glob_dir = runtime_dir.join("target/wasm32-unknown-unknown/release");
    let wasm_file = find_wasm_file(&wasm_glob_dir);

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

fn copy_dir_recursive(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    if let Ok(entries) = std::fs::read_dir(src) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap();
            let dst_path = dst.join(name);
            if path.is_dir() {
                copy_dir_recursive(&path, &dst_path);
            } else {
                std::fs::copy(&path, &dst_path).unwrap();
            }
        }
    }
}
