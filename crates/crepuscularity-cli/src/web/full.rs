use console::style;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::ui;
use crepuscularity_core::TemplateContext;
use crepuscularity_web::par_render_from_files;

// ── build-full ───────────────────────────────────────────────────────────────

pub(crate) struct BuildFullArgs {
    pub(crate) site_dir: PathBuf,
    pub(crate) wasm: bool,
    pub(crate) server: bool,
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
                        crepus_files.insert(key, super::normalize_fullwidth_braces(&content));
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

pub(crate) fn run_build_full(args: &BuildFullArgs) {
    let t0 = Instant::now();
    eprintln!("{}", style("crepus web build-full").dim());
    eprintln!();

    // L1 Rayon: collect all .crepus files in the site dir.
    let (crepus_files, entries) = collect_crepus_files(&args.site_dir);

    let ctx = TemplateContext::new();
    let entry_refs: Vec<&str> = entries.iter().map(|s| s.as_str()).collect();
    let results = par_render_from_files(&crepus_files, &entry_refs, &ctx);

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
