use std::path::PathBuf;
use std::time::Instant;

use crate::ui;
use console::style;

// ── scaffold ─────────────────────────────────────────────────────────────────

pub(crate) fn scaffold_site(name: &str) {
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
