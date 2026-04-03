//! Browser extension commands for crepu CLI.

use std::path::{Path, PathBuf};

pub fn run(args: &[String]) {
    match args.first().map(|s| s.as_str()) {
        Some("new") => {
            let name = args.get(1).map(|s| s.as_str()).unwrap_or_else(|| {
                eprintln!("Usage: crepu webext new <name>");
                std::process::exit(1);
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
    eprintln!("crepu webext — Browser extension commands");
    eprintln!();
    eprintln!("COMMANDS:");
    eprintln!("  new <name>             scaffold a new browser extension");
    eprintln!("  build [--app PATH]     build extension to dist/unpacked/");
    eprintln!("  manifest [--app PATH]  print generated manifest.json");
}

fn scaffold_extension(name: &str) {
    let slug = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>();

    let base = PathBuf::from(&slug);
    if base.exists() {
        eprintln!("Directory already exists: {}", slug);
        std::process::exit(1);
    }

    // Create directory structure
    std::fs::create_dir_all(base.join("runtime/src")).unwrap();
    std::fs::create_dir_all(base.join("views")).unwrap();

    // webext.toml
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

    // runtime/Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{slug}_runtime"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
crepuscularity-webext = {{ git = "https://github.com/semitechnological/crepuscularity" }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
wasm-bindgen = "0.2"
"#
    );
    std::fs::write(base.join("runtime/Cargo.toml"), cargo_toml).unwrap();

    // runtime/src/lib.rs
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

    // views/ui.crepus
    let ui_crepus = r#"--- Popup
div popup flex flex-col gap-4 p-4
  h1 text-xl font-bold
    "{title}"
  p text-gray-600
    "{description}"
"#;
    std::fs::write(base.join("views/ui.crepus"), ui_crepus).unwrap();

    eprintln!("\x1b[32m✓\x1b[0m Created extension: {}/", slug);
    eprintln!();
    eprintln!("Next steps:");
    eprintln!("  cd {}", slug);
    eprintln!("  crepu webext build");
    eprintln!("  # Load dist/unpacked/ in chrome://extensions");
}

fn build_extension(app_path: &Path) {
    let webext_toml = app_path.join("webext.toml");
    if !webext_toml.exists() {
        eprintln!("No webext.toml found in {:?}", app_path);
        std::process::exit(1);
    }

    eprintln!("Building extension from {:?}", app_path);

    // Parse manifest
    let manifest = match crepuscularity_webext::ExtensionManifest::load(&webext_toml) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to parse webext.toml: {e}");
            std::process::exit(1);
        }
    };

    // Create dist/unpacked
    let dist = app_path.join("dist/unpacked");
    std::fs::create_dir_all(&dist).unwrap();

    // Generate manifest.json
    let manifest_json = manifest.to_manifest_v3_json();
    std::fs::write(dist.join("manifest.json"), &manifest_json).unwrap();

    // Copy views
    let views_src = app_path.join("views");
    let views_dst = dist.join("views");
    if views_src.exists() {
        copy_dir_recursive(&views_src, &views_dst);
    }

    // Copy assets from crepuscularity-webext (would need to be embedded or installed)
    // For now, just note that they need to be copied
    eprintln!("  \x1b[33m!\x1b[0m Copy webext assets to dist/src/ manually");

    eprintln!("\x1b[32m✓\x1b[0m Extension built to {}", dist.display());
}

fn print_manifest(app_path: &Path) {
    let webext_toml = app_path.join("webext.toml");
    if !webext_toml.exists() {
        // Try crex format
        let crex_path = app_path.join("manifest.crex");
        if crex_path.exists() {
            match crepuscularity_webext::ExtensionManifest::load(&crex_path) {
                Ok(m) => {
                    println!("{}", m.to_manifest_v3_json());
                    return;
                }
                Err(e) => {
                    eprintln!("Failed to parse manifest.crex: {e}");
                    std::process::exit(1);
                }
            }
        }
        eprintln!("No webext.toml or manifest.crex found in {:?}", app_path);
        std::process::exit(1);
    }

    match crepuscularity_webext::ExtensionManifest::load(&webext_toml) {
        Ok(m) => println!("{}", m.to_manifest_v3_json()),
        Err(e) => {
            eprintln!("Failed to parse webext.toml: {e}");
            std::process::exit(1);
        }
    }
}

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
