//! crepus — the Crepuscularity CLI
//!
//! COMMANDS
//!   crepus new NAME                              scaffold a new GPUI app
//!   crepus dev [--bin NAME] [--release] [--emit-events]  watch → rebuild → relaunch
//!   crepus build [--release]                     cargo build wrapper
//!   crepus preview FILE                          live-preview a .crepus template
//!   crepus render FILE [--ctx FILE] [--var k=v] [--component Name]
//!   crepus web new NAME                          scaffold site.json + web.toml
//!   crepus web build [--site DIR] [--json FILE] [-o FILE] [site.json]
//!   crepus web site-json [--site DIR]             pretty-print site.json
//!   crepus webext new NAME                       scaffold a browser extension
//!   crepus webext build [--app PATH]             build browser extension
//!   crepus webext manifest [--app PATH]          print manifest.json

#[cfg(feature = "desktop")]
mod builder;
#[cfg(feature = "desktop")]
mod dev;
#[cfg(feature = "desktop")]
pub mod events;
#[cfg(feature = "desktop")]
mod hud;
mod new;
mod render;
pub mod ui;
mod web;
mod webext;

use console::style;
#[cfg(feature = "desktop")]
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("--version") | Some("-V") => {
            println!("crepus cli {}", env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        }

        Some("--help") | Some("-h") => {
            print_usage();
            std::process::exit(0);
        }

        Some("new") => {
            let name = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| {
                ui::error("Usage: crepus new <name>");
            });
            new::run(name);
        }

        #[cfg(feature = "desktop")]
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

        #[cfg(feature = "desktop")]
        Some("build") => {
            let t0 = Instant::now();
            let release = args.iter().any(|a| a == "--release");
            let cwd = std::env::current_dir().unwrap();
            let sp = ui::spinner(if release {
                "cargo build --release"
            } else {
                "cargo build"
            });
            let outcome = builder::cargo_build(&cwd, release, None);
            if outcome.success {
                ui::spinner_ok(&sp, "build succeeded");
                ui::done_in(t0.elapsed());
            } else {
                sp.finish_and_clear();
                ui::error("build failed");
            }
        }

        #[cfg(feature = "desktop")]
        Some("preview") => {
            let path = args
                .get(2)
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| {
                    ui::error("Usage: crepus preview <file.crepus>");
                });
            if !path.exists() {
                ui::error(&format!("file not found: {}", path.display()));
            }
            eprintln!(
                "{} previewing {}",
                style("crepus").dim(),
                style(path.display().to_string()).cyan().bold()
            );
            run_preview(path);
        }

        Some("render") => {
            render::run(&args[2..]);
        }

        Some("web") => {
            web::run(&args[2..]);
        }

        Some("webext") => {
            webext::run(&args[2..]);
        }

        _ => {
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!(
        "{} cli {}",
        style("crepus").cyan().bold(),
        style(env!("CARGO_PKG_VERSION")).dim()
    );
    eprintln!();
    eprintln!("{}", style("COMMANDS").dim());
    eprintln!(
        "  {}  {}",
        style("new <name>                           ").green(),
        style("scaffold a new GPUI app").dim()
    );
    eprintln!(
        "  {}  {}",
        style("dev [--bin NAME] [--release]         ").green(),
        style("hot-reload dev loop").dim()
    );
    eprintln!(
        "  {}  {}",
        style("build [--release]                    ").green(),
        style("cargo build wrapper").dim()
    );
    eprintln!(
        "  {}  {}",
        style("preview <file.crepus>                ").green(),
        style("live-preview a template (GPUI window)").dim()
    );
    eprintln!(
        "  {}  {}",
        style("render <file.crepus> [--ctx] [--var] ").green(),
        style("render template to HTML on stdout").dim()
    );
    eprintln!(
        "  {}  {}",
        style("web new <name>                       ").green(),
        style("scaffold site.json + web.toml").dim()
    );
    eprintln!(
        "  {}  {}",
        style("web build [--site] [--json] [-o]     ").green(),
        style("static page HTML from site.json").dim()
    );
    eprintln!(
        "  {}  {}",
        style("web site-json [--site DIR]           ").green(),
        style("pretty-print site.json").dim()
    );
    eprintln!();
    eprintln!(
        "  {}  {}",
        style("webext new <name>                    ").green(),
        style("scaffold a browser extension").dim()
    );
    eprintln!(
        "  {}  {}",
        style("webext build [--app PATH]            ").green(),
        style("build extension to dist/unpacked/").dim()
    );
    eprintln!(
        "  {}  {}",
        style("webext manifest [--app PATH]         ").green(),
        style("print manifest.json").dim()
    );
    eprintln!();
    eprintln!("{}", style("OPTIONS").dim());
    eprintln!(
        "  {}  {}",
        style("-h, --help                           ").green(),
        style("show this help").dim()
    );
    eprintln!(
        "  {}  {}",
        style("-V, --version                        ").green(),
        style("show version").dim()
    );
    eprintln!(
        "  {}  {}",
        style("--emit-events                        ").green(),
        style("emit JSON events to stdout (IDE integration)").dim()
    );
}

#[cfg(feature = "desktop")]
fn run_preview(path: std::path::PathBuf) {
    use crepuscularity_runtime::{HotReloadState, HotReloadView, TemplateContext};
    use gpui::{
        bounds, point, prelude::*, px, size, Application, WindowBackgroundAppearance, WindowKind,
        WindowOptions,
    };

    let display_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("preview")
        .to_string();

    let mut ctx = TemplateContext::new();
    if let Some(dir) = path.parent() {
        let ctx_path = dir.join("context.toml");
        if ctx_path.exists() {
            load_context_toml(&ctx_path, &mut ctx);
        }
    }

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

#[cfg(feature = "desktop")]
fn load_context_toml(path: &std::path::Path, ctx: &mut crepuscularity_runtime::TemplateContext) {
    use crepuscularity_runtime::TemplateValue;
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
