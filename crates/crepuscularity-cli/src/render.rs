/// `crepus render <file.crepus>` — render a template to HTML on stdout.
///
/// Useful for server-side rendering pipelines, CI snapshot tests, and
/// inspecting template output without a GPUI window.
///
/// OPTIONS:
///   --ctx <file.toml>    load context variables from a TOML file
///   --var key=value      set a single context variable (repeatable)
///   --component Name     render a named component from a multi-component file
use std::path::{Path, PathBuf};

use crepuscularity_core::context::{TemplateContext, TemplateValue};
use crepuscularity_web::{render_component_file_to_html, render_template_to_html};

pub fn run(args: &[String]) {
    let mut path: Option<PathBuf> = None;
    let mut ctx_file: Option<PathBuf> = None;
    let mut vars: Vec<(String, String)> = Vec::new();
    let mut component: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--ctx" => {
                i += 1;
                ctx_file = args.get(i).map(PathBuf::from);
            }
            "--var" => {
                i += 1;
                if let Some(kv) = args.get(i) {
                    if let Some(eq) = kv.find('=') {
                        vars.push((kv[..eq].to_string(), kv[eq + 1..].to_string()));
                    } else {
                        eprintln!("--var expects key=value, got: {kv}");
                        std::process::exit(1);
                    }
                }
            }
            "--component" => {
                i += 1;
                component = args.get(i).cloned();
            }
            other => {
                if other.starts_with('-') {
                    eprintln!("Unknown option: {other}");
                    print_usage();
                    std::process::exit(1);
                } else if path.is_none() {
                    path = Some(PathBuf::from(other));
                } else {
                    eprintln!("Unexpected argument: {other}");
                    print_usage();
                    std::process::exit(1);
                }
            }
        }
        i += 1;
    }

    let path = path.unwrap_or_else(|| {
        print_usage();
        std::process::exit(1);
    });

    if !path.exists() {
        eprintln!("File not found: {}", path.display());
        std::process::exit(1);
    }

    let mut ctx = TemplateContext::new();
    ctx.base_dir = path.parent().map(|p| p.to_path_buf());

    // Load --ctx file
    if let Some(ctx_path) = ctx_file {
        load_toml_ctx(&ctx_path, &mut ctx);
    } else {
        // Auto-load context.toml from same directory
        if let Some(dir) = path.parent() {
            let auto = dir.join("context.toml");
            if auto.exists() {
                load_toml_ctx(&auto, &mut ctx);
            }
        }
    }

    // Apply --var overrides
    for (k, v) in vars {
        ctx.set(k, v);
    }

    let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {e}", path.display());
        std::process::exit(1);
    });

    let html = match component {
        Some(ref name) => render_component_file_to_html(&content, name, &ctx),
        None => render_template_to_html(&content, &ctx),
    };

    match html {
        Ok(out) => print!("{out}"),
        Err(e) => {
            eprintln!("Render error: {e}");
            std::process::exit(1);
        }
    }
}

fn load_toml_ctx(path: &Path, ctx: &mut TemplateContext) {
    let Ok(content) = std::fs::read_to_string(path) else {
        eprintln!("Could not read context file: {}", path.display());
        return;
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
            continue;
        }
        if let Some(eq) = line.find('=') {
            let key = line[..eq].trim();
            let val = line[eq + 1..].trim();
            let value = if val == "true" {
                TemplateValue::Bool(true)
            } else if val == "false" {
                TemplateValue::Bool(false)
            } else if let Ok(n) = val.parse::<i64>() {
                TemplateValue::Int(n)
            } else if let Ok(f) = val.parse::<f64>() {
                TemplateValue::Float(f)
            } else {
                TemplateValue::Str(val.trim_matches('"').trim_matches('\'').to_string())
            };
            ctx.set(key, value);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: crepus render <file.crepus> [OPTIONS]");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  --ctx <file.toml>    load context variables from a TOML file");
    eprintln!("  --var key=value      set a context variable (repeatable)");
    eprintln!("  --component Name     render a specific named component");
}
