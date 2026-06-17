//! crepuscularity-dev — debugging and development tools for `.crepus` templates.
//!
//! # Modes
//!
//! - **Default (GPUI viewer)**: opens a live-reload GPUI window (requires `gpui-viewer` feature).
//! - `--ast FILE`: dump the parsed AST of a template to stdout.
//! - `--render FILE [--var k=v ...]`: render a template to HTML on stdout.
//! - `--ir FILE`: dump the View IR (JSON) for a template.
//! - `--ctx FILE`: inspect context.toml variables.
//!
//! # Usage
//!
//! ```sh
//! crepus-dev my-view.crepus --ast          # dump AST
//! crepus-dev my-view.crepus --render       # render to stdout
//! crepus-dev my-view.crepus --ir           # View IR JSON
//! crepus-dev my-view.crepus --var name=Alice --render  # with variables
//! crepus-dev my-view.crepus                # GPUI live viewer (default)
//! ```

use std::path::{Path, PathBuf};

use crepuscularity_core::context::{TemplateContext, TemplateValue};
use crepuscularity_core::parser::parse_template;

fn print_usage() {
    eprintln!("Usage: crepus-dev <template.crepus> [OPTIONS]");
    eprintln!();
    eprintln!("Modes:");
    eprintln!("  (default)             Open GPUI live-reload viewer");
    eprintln!("  --ast                 Dump parsed AST to stdout");
    eprintln!("  --render              Render template to HTML on stdout");
    eprintln!("  --ir                  Dump View IR (JSON) to stdout");
    eprintln!("  --ctx                 Inspect context.toml variables");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --width N             Window width (GPUI mode, default 1200)");
    eprintln!("  --height N            Window height (GPUI mode, default 800)");
    eprintln!("  --var k=v             Set a string variable (repeatable)");
    eprintln!("  --bool k=true         Set a boolean variable");
    eprintln!("  --int k=42            Set an integer variable");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        print_usage();
        std::process::exit(0);
    }

    let template_path = PathBuf::from(&args[1]);
    if !template_path.exists() {
        eprintln!("Error: template not found: {}", template_path.display());
        std::process::exit(1);
    }

    let source = std::fs::read_to_string(&template_path).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {e}", template_path.display());
        std::process::exit(1);
    });

    // Parse flags
    let mut mode = Mode::Viewer;
    let mut width = 1200.0f32;
    let mut height = 800.0f32;
    let mut vars: Vec<(String, TemplateValue)> = Vec::new();

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--ast" => mode = Mode::Ast,
            "--render" => mode = Mode::Render,
            "--ir" => mode = Mode::Ir,
            "--ctx" => mode = Mode::Ctx,
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
                        vars.push((v[..eq].to_string(), v[eq + 1..].to_string().into()));
                    }
                }
            }
            "--bool" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    if let Some(eq) = v.find('=') {
                        let val = v[eq + 1..].parse::<bool>().unwrap_or(false);
                        vars.push((v[..eq].to_string(), TemplateValue::Bool(val)));
                    }
                }
            }
            "--int" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    if let Some(eq) = v.find('=') {
                        let val = v[eq + 1..].parse::<i64>().unwrap_or(0);
                        vars.push((v[..eq].to_string(), TemplateValue::Int(val)));
                    }
                }
            }
            other => {
                eprintln!("Unknown option: {other}");
                print_usage();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    match mode {
        Mode::Ast => dump_ast(&source),
        Mode::Render => render_to_stdout(&source, &vars),
        Mode::Ir => dump_ir(&source, &template_path, &vars),
        Mode::Ctx => dump_ctx(&template_path),
        Mode::Viewer => launch_viewer(template_path, width, height, vars),
    }
}

enum Mode {
    Ast,
    Render,
    Ir,
    Ctx,
    Viewer,
}

fn dump_ast(source: &str) {
    match parse_template(source) {
        Ok(ast) => println!("{:#?}", ast),
        Err(e) => {
            eprintln!("Parse error: {e}");
            std::process::exit(1);
        }
    }
}

fn render_to_stdout(source: &str, vars: &[(String, TemplateValue)]) {
    let mut ctx = TemplateContext::new();
    for (k, v) in vars {
        ctx.set(k, v.clone());
    }
    match crepuscularity_web::render_template_to_html(source, &ctx) {
        Ok(html) => print!("{html}"),
        Err(e) => {
            eprintln!("Render error: {e}");
            std::process::exit(1);
        }
    }
}

fn dump_ir(source: &str, path: &PathBuf, vars: &[(String, TemplateValue)]) {
    let mut ctx = TemplateContext::new();
    for (k, v) in vars {
        ctx.set(k, v.clone());
    }
    let component = path.file_stem().and_then(|s| s.to_str()).unwrap_or("root");
    match crepuscularity_native::render_component_file_to_ir(source, component, &ctx) {
        Ok(ir) => {
            let json = crepuscularity_native::to_json_pretty(&ir).unwrap_or_else(|e| {
                eprintln!("IR serialization error: {e}");
                std::process::exit(1);
            });
            print!("{json}");
        }
        Err(e) => {
            eprintln!("IR error: {e}");
            std::process::exit(1);
        }
    }
}

fn dump_ctx(template_path: &Path) {
    let dir = template_path.parent().unwrap_or(std::path::Path::new("."));
    let ctx_path = dir.join("context.toml");
    if !ctx_path.exists() {
        eprintln!("No context.toml found in {}", dir.display());
        eprintln!("Create one with key=value pairs:");
        eprintln!("  name = \"Alice\"");
        eprintln!("  count = 42");
        eprintln!("  show_header = true");
        return;
    }
    let content = std::fs::read_to_string(&ctx_path).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {e}", ctx_path.display());
        std::process::exit(1);
    });
    eprintln!("=== {} ===", ctx_path.display());
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(eq) = line.find('=') {
            let key = line[..eq].trim();
            let raw_val = line[eq + 1..].trim();
            let typed = if raw_val == "true" || raw_val == "false" {
                format!("bool: {raw_val}")
            } else if raw_val.parse::<i64>().is_ok() {
                format!("int: {raw_val}")
            } else if raw_val.parse::<f64>().is_ok() {
                format!("float: {raw_val}")
            } else {
                format!("string: {raw_val}")
            };
            eprintln!("  {key} = {typed}");
        }
    }
}

#[cfg(feature = "gpui-viewer")]
fn launch_viewer(
    template_path: PathBuf,
    width: f32,
    height: f32,
    vars: Vec<(String, TemplateValue)>,
) {
    use crepuscularity_runtime::{HotReloadState, HotReloadView};
    use gpui::{bounds, point, prelude::*, size, Application, WindowOptions};

    let mut ctx = TemplateContext::new();
    for (k, v) in vars {
        ctx.set(k, v);
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

#[cfg(not(feature = "gpui-viewer"))]
fn launch_viewer(
    _template_path: PathBuf,
    _width: f32,
    _height: f32,
    _vars: Vec<(String, TemplateValue)>,
) {
    eprintln!("Error: GPUI viewer not available (enable the `gpui-viewer` feature).");
    eprintln!("Use --ast, --render, or --ir instead.");
    std::process::exit(1);
}
