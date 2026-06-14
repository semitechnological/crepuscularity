//! crepuscularity-dev — Vite-like hot-reload dev server for crepuscularity templates.
//!
//! Usage:
//!   crepus-dev <template.crepus> [--width N] [--height N]
//!
//! Opens a GPUI window that live-reloads whenever the template file is saved.
//! Optionally load context variables from a `context.toml` in the same directory.

use std::path::PathBuf;

use crepuscularity_runtime::{HotReloadState, HotReloadView, TemplateContext, TemplateValue};
use gpui::{bounds, point, prelude::*, size, Application, WindowOptions};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Parse args: crepus-dev <file> [--width N] [--height N] [--var key=value ...]
    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        eprintln!("Usage: crepus-dev <template.crepus> [--width N] [--height N] [--var key=value]");
        eprintln!();
        eprintln!("Options:");
        eprintln!("  --width N      Window width in pixels (default: 1200)");
        eprintln!("  --height N     Window height in pixels (default: 800)");
        eprintln!("  --var k=v      Set a string template variable (repeatable)");
        eprintln!("  --bool k=true  Set a boolean template variable");
        eprintln!("  --int k=42     Set an integer template variable");
        eprintln!();
        eprintln!("Example:");
        eprintln!("  crepus-dev my-view.crepus --width 800 --height 600 --var title=Hello --bool show_button=true");
        std::process::exit(0);
    }

    let template_path = PathBuf::from(&args[1]);

    if !template_path.exists() {
        eprintln!("Error: template file not found: {:?}", template_path);
        std::process::exit(1);
    }

    let mut width = 1200.0f32;
    let mut height = 800.0f32;
    let mut ctx = TemplateContext::new();

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--width" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    width = v.parse().unwrap_or(1200.0);
                }
            }
            "--height" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    height = v.parse().unwrap_or(800.0);
                }
            }
            "--var" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    if let Some(eq) = v.find('=') {
                        let key = &v[..eq];
                        let val = &v[eq + 1..];
                        ctx.set(key, val);
                    }
                }
            }
            "--bool" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    if let Some(eq) = v.find('=') {
                        let key = &v[..eq];
                        let val = v[eq + 1..].to_lowercase() == "true";
                        ctx.set(key, TemplateValue::Bool(val));
                    }
                }
            }
            "--int" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    if let Some(eq) = v.find('=') {
                        let key = &v[..eq];
                        if let Ok(n) = v[eq + 1..].parse::<i64>() {
                            ctx.set(key, TemplateValue::Int(n));
                        }
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    // Also load context.toml from the same directory if it exists
    if let Some(dir) = template_path.parent() {
        let ctx_path = dir.join("context.toml");
        if ctx_path.exists() {
            load_context_toml(&ctx_path, &mut ctx);
        }
    }

    let display_name = template_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("template")
        .to_string();

    eprintln!("crepus-dev: watching {:?}", template_path);
    eprintln!("crepus-dev: window {}x{}", width as u32, height as u32);
    eprintln!("crepus-dev: edit and save to hot-reload");

    Application::new().run(move |cx: &mut gpui::App| {
        let window_options = WindowOptions {
            app_id: Some(format!("crepuscularity.dev.{}", display_name)),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some(format!("Crepus Dev - {}", display_name).into()),
                ..Default::default()
            }),
            window_bounds: Some(gpui::WindowBounds::Windowed(bounds(
                point(gpui::px(100.), gpui::px(100.)),
                size(gpui::px(width), gpui::px(height)),
            ))),
            ..Default::default()
        };

        let path = template_path.clone();
        let ctx = ctx.clone();

        cx.open_window(window_options, move |_window, cx| {
            let state = cx.new(|cx| HotReloadState::new(path.clone(), ctx.clone(), cx));
            cx.new(|_| HotReloadView::new(state))
        })
        .unwrap();
    });
}

/// Load simple TOML key=value pairs into the context.
/// Supports: strings, booleans, integers.
fn load_context_toml(path: &std::path::Path, ctx: &mut TemplateContext) {
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
            let raw_val = line[eq + 1..].trim();

            // Boolean
            if raw_val == "true" {
                ctx.set(key, TemplateValue::Bool(true));
            } else if raw_val == "false" {
                ctx.set(key, TemplateValue::Bool(false));
            } else if let Ok(n) = raw_val.parse::<i64>() {
                // Integer
                ctx.set(key, TemplateValue::Int(n));
            } else if let Ok(f) = raw_val.parse::<f64>() {
                // Float
                ctx.set(key, TemplateValue::Float(f));
            } else {
                // String — strip surrounding quotes if present
                let s = raw_val.trim_matches('"').trim_matches('\'').to_string();
                ctx.set(key, s);
            }
        }
    }
}
