//! crepus — the Crepuscularity CLI
//!
//! COMMANDS
//!   crepus new NAME                              scaffold a new GPUI app
//!   crepus init KIND NAME                        scaffold web, webext, tui, native, ios, or gpui
//!   crepus dev [--bin NAME] [--release] [--emit-events]  watch → rebuild → relaunch
//!   crepus build [TYPE|ID] [--target ID] [--manifest FILE] build crepus.toml targets, or cargo fallback
//!   crepus preview FILE                          live-preview a .crepus template
//!   crepus render FILE [--ctx FILE] [--var k=v] [--component Name]
//!   crepus web new NAME                          scaffold index.crepus + runtime/ + crepus.toml
//!   crepus web build [--site DIR] [--out-dir DIR]   dist/ with WASM + crepus-bundle.json
//!   crepus web site-json [--site DIR]             pretty-print site.json (deprecated)
//!   crepus webext new NAME                       scaffold a browser extension
//!   crepus webext build [--app PATH]             build browser extension
//!   crepus webext manifest [--app PATH]          print manifest.json
//!   crepus ios new NAME                      XcodeGen + SwiftPM NativeShell host app
//!   crepus ios generate [--dir PATH]         run xcodegen (brew install xcodegen)
//!   crepus ios build [--dir] [--scheme] [...]  xcodegen + xcodebuild; toml = defaults
//!   crepus tui new NAME                         scaffold TUI app with Ratatui
//!   crepus tui build [--release]                 build TUI app
//!   crepus tui run                               run TUI app
//!   crepus tui preview FILE                      hot-reload preview a .crepus template in the terminal
//!   crepus native new NAME                       scaffold cross-platform native app
//!   crepus native ir FILE                        emit View IR JSON
//!   crepus native build ios [--scheme S]         build iOS app
//!   crepus native build android [--flavor F]     build Android app
//!   crepus native run ios                        run iOS app (Xcode)
//!   crepus native run android                    install Android app
//!   crepus embedded check FILE
//!   crepus embedded snapshot FILE --width W --height H --out path.ppm
//!   crepus aurora dev [--watch DIR] [--port N]
//!   crepus aurora build --watch DIR --out DIR
//!   crepus aurora new NAME
//!   crepus aurora swiftgen --view FILE --out DIR --view-name NAME
//!   crepus benchmark [all|run|check] [flags…]    benchmark.toml run or prereq check (examples/benchmarks)

mod aurora;
mod benchmark;
mod benchmark_tui;
mod build_options;
#[cfg(feature = "desktop")]
mod builder;
mod crepus_toml;
#[cfg(feature = "desktop")]
mod dev;
mod embedded;
#[cfg(feature = "desktop")]
pub mod events;
#[cfg(feature = "desktop")]
mod hud;
mod ios;
mod native;
mod new;
mod render;
mod target_build;
mod tui;
pub mod ui;
mod wasm_bundle;
mod web;
mod web_docs_hook;
mod web_islands;
mod web_serve;
mod webext;

use console::style;
use std::time::Instant;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("crepuscularity=info".parse().unwrap()),
        )
        .with_target(false)
        .init();

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

        Some("init") => {
            run_init(&args[2..]);
        }

        #[cfg(feature = "desktop")]
        Some("dev") => {
            let options = build_options::BuildOptions::parse_or_exit(&args[2..]);
            let mut bin: Option<String> = None;
            let mut emit_events = false;
            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--bin" => {
                        i += 1;
                        bin = args.get(i).cloned();
                    }
                    "--debug" | "--dev" | "--release" => {}
                    "--opt-level" => i += 1,
                    arg if arg.starts_with("--opt-level=") => {}
                    "--emit-events" => emit_events = true,
                    _ => {}
                }
                i += 1;
            }
            dev::run(bin, options, emit_events);
        }

        Some("build") => {
            let options = build_options::BuildOptions::parse_or_exit(&args[2..]);
            let build_args = build_options::strip_build_options_or_exit(&args[2..]);
            let manifest_arg = manifest_arg(&build_args);
            if build_args
                .iter()
                .any(|a| matches!(a.as_str(), "--target" | "-t" | "--manifest" | "--all"))
                || build_args
                    .iter()
                    .any(|a| !a.starts_with('-') && a.as_str() != "release")
                || target_build::has_manifest_targets(manifest_arg)
            {
                target_build::run(&args[2..]);
                return;
            }
            let t0 = Instant::now();
            let cwd = std::env::current_dir().unwrap_or_else(|e| {
                ui::error(&format!("cannot determine current directory: {e}"));
            });
            let sp = ui::spinner(if options.release() {
                "cargo build --release"
            } else {
                "cargo build"
            });
            #[cfg(feature = "desktop")]
            let outcome = builder::cargo_build(&cwd, options, None);
            #[cfg(not(feature = "desktop"))]
            let outcome = cargo_build_fallback(&cwd, options);
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

        Some("ios") => {
            ios::run(&args[2..]);
        }

        Some("tui") => {
            tui::run(&args[2..]);
        }

        Some("native") => {
            native::run(&args[2..]);
        }

        Some("aurora") => {
            aurora::run(&args[2..]);
        }

        Some("embedded") => {
            embedded::run(&args[2..]);
        }

        Some("benchmark") => match args.get(2).map(|s| s.as_str()) {
            Some("check") => benchmark::run_check(args.get(3..).unwrap_or(&[])),
            _ => benchmark::run(args.get(2..).unwrap_or(&[])),
        },

        _ => {
            print_usage();
            std::process::exit(1);
        }
    }
}

fn manifest_arg(args: &[String]) -> Option<std::path::PathBuf> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--manifest" {
            return args.get(i + 1).map(std::path::PathBuf::from);
        }
        i += 1;
    }
    None
}

#[cfg(not(feature = "desktop"))]
struct CargoBuildOutcome {
    success: bool,
}

#[cfg(not(feature = "desktop"))]
fn cargo_build_fallback(
    cwd: &std::path::Path,
    options: build_options::BuildOptions,
) -> CargoBuildOutcome {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build").current_dir(cwd);
    if options.release() {
        cmd.arg("--release");
    }
    let success = cmd.status().map(|status| status.success()).unwrap_or(false);
    CargoBuildOutcome { success }
}

fn run_init(args: &[String]) {
    let kind = args
        .first()
        .map(|s| s.as_str())
        .unwrap_or_else(|| ui::error("Usage: crepus init <kind> <name>"));
    let name = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or_else(|| ui::error("Usage: crepus init <kind> <name>"));
    if args.len() > 2 {
        ui::error("Usage: crepus init <kind> <name>");
    }
    let sub_args = vec!["new".to_string(), name.to_string()];
    match kind {
        "web" => web::run(&sub_args),
        "webext" | "extension" | "browser-extension" => webext::run(&sub_args),
        "tui" => tui::run(&sub_args),
        "native" => native::run(&sub_args),
        "ios" => ios::run(&sub_args),
        "gpui" | "app" | "desktop" => new::run(name),
        other => ui::error(&format!(
            "unknown init kind {other:?}; expected web, webext, tui, native, ios, or gpui"
        )),
    }
}

fn print_usage() {
    eprintln!(
        "{} cli {}",
        style("crepus").cyan().bold(),
        style(env!("CARGO_PKG_VERSION")).dim()
    );
    eprintln!();

    let commands = [
        (
            "new <name>                           ",
            "scaffold a new GPUI app",
        ),
        (
            "init <kind> <name>                    ",
            "scaffold web, webext, tui, native, ios, or gpui",
        ),
        (
            "dev [--bin NAME] [--debug|--dev|--release]",
            "hot-reload dev loop",
        ),
        (
            "build [TYPE|ID] [--debug|--dev|--release]",
            "build crepus.toml targets, or cargo fallback",
        ),
        (
            "preview <file.crepus>                ",
            "live-preview a template (GPUI window)",
        ),
        (
            "render <file.crepus> [--ctx] [--var] ",
            "render template to HTML on stdout",
        ),
        (
            "web new <name>                       ",
            "scaffold .crepus site + WASM runtime/",
        ),
        (
            "web build [--site] [--release]       ",
            "static dist/ — HTML shell + WASM bundle",
        ),
        (
            "web site-json [--site DIR]           ",
            "pretty-print site.json",
        ),
        (
            "web dev [--site DIR] [--port N]      ",
            "live-reload dev server (alias: serve)",
        ),
        ("", ""),
        (
            "webext new <name>                    ",
            "scaffold a browser extension",
        ),
        (
            "webext build [--app PATH] [--release]",
            "build extension to dist/unpacked/",
        ),
        (
            "webext manifest [--app PATH]         ",
            "print manifest.json",
        ),
        (
            "ios new <name>                         ",
            "XcodeGen + NativeShell SwiftPM app",
        ),
        (
            "ios generate [--dir] [--spec]          ",
            "xcodegen; finds crepus.toml [ios] up-tree",
        ),
        (
            "ios build [--dir] [--scheme] [--release]",
            "xcodegen + xcodebuild; toml = defaults",
        ),
        (
            "tui new <name>                         ",
            "scaffold TUI app with Ratatui",
        ),
        ("tui build [--debug|--dev|--release]     ", "build TUI app"),
        (
            "tui preview <file.crepus>               ",
            "hot-reload preview a .crepus template in the terminal",
        ),
        (
            "native new <name>                       ",
            "scaffold native iOS/Android app",
        ),
        (
            "native ir <file.crepus>                  ",
            "emit View IR JSON",
        ),
        (
            "native build ios [--scheme]             ",
            "build native iOS app",
        ),
        (
            "native build android [--flavor]         ",
            "build native Android app",
        ),
        (
            "embedded check <file> [--component Name]  ",
            "validate .crepus for CI / build.rs",
        ),
        (
            "embedded snapshot <file> -W -H --out ppm",
            "debug PPM only (use Rust API in firmware)",
        ),
        (
            "aurora dev [--watch DIR]           ",
            "hot-reload preview window (SwiftUI Runner).",
        ),
        (
            "aurora run [PROJECT] [--macos|--ios]",
            "build & launch a SwiftUI project.",
        ),
        (
            "benchmark [all|run|check] [flags…]   ",
            "benchmark.toml run or prereq probe (see examples/benchmarks)",
        ),
    ];

    eprintln!("{}", style("COMMANDS").dim());
    for (cmd, desc) in commands {
        if cmd.is_empty() {
            eprintln!();
        } else {
            eprintln!("  {}  {}", style(cmd).green(), style(desc).dim());
        }
    }
    eprintln!();

    let options = [
        ("-h, --help                           ", "show this help"),
        ("-V, --version                        ", "show version"),
        (
            "--emit-events                        ",
            "emit JSON events to stdout (IDE integration)",
        ),
    ];

    eprintln!("{}", style("OPTIONS").dim());
    for (opt, desc) in options {
        if opt.is_empty() {
            eprintln!();
        } else {
            eprintln!("  {}  {}", style(opt).green(), style(desc).dim());
        }
    }
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
        match cx.open_window(opts, move |_window, cx| {
            let state = cx.new(|cx| HotReloadState::new(p.clone(), c.clone(), cx));
            cx.new(|_| HotReloadView::new(state))
        }) {
            Ok(_) => {}
            Err(e) => eprintln!("preview: failed to open window: {e}"),
        }
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
