//! crepuss — the Crepuscularity CLI
//!
//! COMMANDS
//!   crepuss new NAME                              scaffold a new GPUI app
//!   crepuss dev [--bin NAME] [--release] [--emit-events]  watch → rebuild → relaunch
//!   crepuss build [--release]                     cargo build wrapper
//!   crepuss preview FILE                          live-preview a .crepuss template
//!   crepuss webext new NAME                       scaffold a browser extension
//!   crepuss webext build [--app PATH]             build browser extension
//!   crepuss webext manifest [--app PATH]          print manifest.json

mod builder;
mod dev;
pub mod events;
mod hud;
mod new;
mod webext;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("new") => {
            let name = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| {
                eprintln!("Usage: crepus new <name>");
                std::process::exit(1);
            });
            new::run(name);
        }

        Some("dev") => {
            let mut bin: Option<String> = None;
            let mut release = false;
            let mut emit_events = false;
            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--bin" => {
                        i += 1;
                        bin = args.get(i).cloned();
                    }
                    "--release" => release = true,
                    "--emit-events" => emit_events = true,
                    _ => {}
                }
                i += 1;
            }
            dev::run(bin, release, emit_events);
        }

        Some("build") => {
            let release = args.iter().any(|a| a == "--release");
            let cwd = std::env::current_dir().unwrap();
            let outcome = builder::cargo_build(&cwd, release, None);
            if outcome.success {
                eprintln!("\x1b[32m✓\x1b[0m Build succeeded");
                std::process::exit(0);
            } else {
                std::process::exit(1);
            }
        }

        Some("preview") => {
            let path = args
                .get(2)
                .map(|s| std::path::PathBuf::from(s))
                .unwrap_or_else(|| {
                    eprintln!("Usage: crepus preview <file.crepuss>");
                    std::process::exit(1);
                });
            if !path.exists() {
                eprintln!("File not found: {:?}", path);
                std::process::exit(1);
            }
            run_preview(path);
        }

        Some("webext") => {
            webext::run(&args[2..]);
        }

        _ => print_usage(),
    }
}

fn print_usage() {
    eprintln!("crepus — Vite/Tauri for GPUI");
    eprintln!();
    eprintln!("COMMANDS:");
    eprintln!("  new <name>                                    scaffold a new GPUI app");
    eprintln!("  dev [--bin NAME] [--release] [--emit-events]  hot-reload dev loop");
    eprintln!("  build [--release]                             cargo build wrapper");
    eprintln!("  preview <file.crepuss>                         live-preview a template");
    eprintln!();
    eprintln!("  webext new <name>                             scaffold browser extension");
    eprintln!("  webext build [--app PATH]                     build extension to dist/");
    eprintln!("  webext manifest [--app PATH]                  print manifest.json");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  --emit-events    emit structured JSON events to stdout (IDE integration)");
}

fn run_preview(path: std::path::PathBuf) {
    use crepusscularity_runtime::{HotReloadState, HotReloadView, TemplateContext};
    use gpui::{
        bounds, point, prelude::*, px, size, Application, WindowBackgroundAppearance, WindowKind,
        WindowOptions,
    };

    let display_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("preview")
        .to_string();

    // Also load context.toml from the same directory if present
    let mut ctx = TemplateContext::new();
    if let Some(dir) = path.parent() {
        let ctx_path = dir.join("context.toml");
        if ctx_path.exists() {
            load_context_toml(&ctx_path, &mut ctx);
        }
    }

    eprintln!("crepus preview: watching {path:?}");

    Application::new().run(move |cx: &mut gpui::App| {
        let opts = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(bounds(
                point(px(100.), px(100.)),
                size(px(1200.), px(800.)),
            ))),
            titlebar: None,
            focus: true,
            show: true,
            kind: WindowKind::Normal,
            is_movable: true,
            is_resizable: true,
            is_minimizable: true,
            display_id: None,
            window_background: WindowBackgroundAppearance::Opaque,
            app_id: Some(format!("crepuscularity.preview.{display_name}")),
            window_min_size: None,
            window_decorations: None,
            tabbing_identifier: None,
        };

        let p = path.clone();
        let c = ctx.clone();
        cx.open_window(opts, move |_window, cx| {
            let state = cx.new(|cx| HotReloadState::new(p.clone(), c.clone(), cx));
            cx.new(|_| HotReloadView::new(state))
        })
        .unwrap();
    });
}

fn load_context_toml(path: &std::path::Path, ctx: &mut crepusscularity_runtime::TemplateContext) {
    use crepusscularity_runtime::TemplateValue;
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(eq) = line.find('=') {
            let key = line[..eq].trim();
            let val = line[eq + 1..].trim();
            if val == "true" {
                ctx.set(key, TemplateValue::Bool(true));
            } else if val == "false" {
                ctx.set(key, TemplateValue::Bool(false));
            } else if let Ok(n) = val.parse::<i64>() {
                ctx.set(key, TemplateValue::Int(n));
            } else if let Ok(f) = val.parse::<f64>() {
                ctx.set(key, TemplateValue::Float(f));
            } else {
                ctx.set(key, val.trim_matches('"').trim_matches('\'').to_string());
            }
        }
    }
}
