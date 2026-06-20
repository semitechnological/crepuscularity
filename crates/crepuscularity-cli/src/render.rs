/// `crepus render <file.crepus>` — render a template to HTML on stdout.
///
/// OPTIONS:
///   --ctx <file>    load context variables from a TOML or JSON file
///   --var key=value set a single context variable (repeatable)
///   --component Name render a named component from a multi-component file
use std::path::{Path, PathBuf};

use crepuscularity_core::context::{TemplateContext, TemplateValue};
use crepuscularity_web::{render_component_file_to_html, render_template_to_html};
use serde_json::Value;

use crate::ui;

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
                        ui::error(&format!("--var expects key=value, got: {kv}"));
                    }
                }
            }
            "--component" => {
                i += 1;
                component = args.get(i).cloned();
            }
            other => {
                if other.starts_with('-') {
                    ui::error(&format!("unknown option: {other}"));
                } else if path.is_none() {
                    path = Some(PathBuf::from(other));
                } else {
                    ui::error(&format!("unexpected argument: {other}"));
                }
            }
        }
        i += 1;
    }

    let path = path.unwrap_or_else(|| {
        eprintln!("Usage: crepus render <file.crepus> [--ctx FILE] [--var k=v] [--component Name]");
        std::process::exit(1);
    });

    if !path.exists() {
        ui::error(&format!("file not found: {}", path.display()));
    }

    let mut ctx = TemplateContext::new();
    ctx.base_dir = path.parent().map(|p| p.to_path_buf());

    if let Some(ctx_path) = ctx_file {
        load_ctx_file(&ctx_path, &mut ctx);
    } else if let Some(dir) = path.parent() {
        let auto = dir.join("context.toml");
        if auto.exists() {
            load_ctx_file(&auto, &mut ctx);
        }
    }

    for (k, v) in vars {
        ctx.set(k, v);
    }

    let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        ui::error(&format!("failed to read {}: {e}", path.display()));
    });

    let html = match component {
        Some(ref name) => render_component_file_to_html(&content, name, &ctx),
        None => render_template_to_html(&content, &ctx),
    };

    match html {
        Ok(out) => print!("{out}"),
        Err(e) => ui::error(&format!("Render error: {e}")),
    }
}

fn load_toml_ctx(path: &Path, ctx: &mut TemplateContext) {
    let Ok(content) = std::fs::read_to_string(path) else {
        ui::warning(&format!("could not read context file: {}", path.display()));
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

fn load_ctx_file(path: &Path, ctx: &mut TemplateContext) {
    let Ok(content) = std::fs::read_to_string(path) else {
        ui::warning(&format!("could not read context file: {}", path.display()));
        return;
    };
    // Try JSON first, fall back to TOML
    if let Ok(value) = serde_json::from_str::<Value>(&content) {
        load_json_ctx_value(&value, ctx);
    } else {
        load_toml_ctx(path, ctx);
    }
}

fn load_json_ctx_value(value: &Value, ctx: &mut TemplateContext) {
    let Some(obj) = value.as_object() else {
        return;
    };
    for (key, val) in obj {
        if let Ok(v) = json_to_template_value(val) {
            ctx.set(key.clone(), v);
        }
    }
}

fn json_to_template_value(value: &Value) -> Result<TemplateValue, ()> {
    match value {
        Value::Null => Ok(TemplateValue::Null),
        Value::Bool(v) => Ok(TemplateValue::Bool(*v)),
        Value::Number(v) => {
            if let Some(n) = v.as_i64() {
                Ok(TemplateValue::Int(n))
            } else if let Some(n) = v.as_f64() {
                Ok(TemplateValue::Float(n))
            } else {
                Err(())
            }
        }
        Value::String(v) => Ok(TemplateValue::Str(v.clone())),
        Value::Array(values) => {
            // Check if array items are scalars (strings/numbers/bools) or objects
            let mut items = Vec::new();
            for item in values {
                match item {
                    Value::Object(obj) => {
                        // Each object becomes a child context
                        let mut child = TemplateContext::new();
                        for (key, value) in obj {
                            if let Ok(v) = json_to_template_scalar(value) {
                                child.set(key.clone(), v);
                            }
                        }
                        items.push(child);
                    }
                    Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null => {
                        // Scalar values become item contexts with "value" key
                        let mut child = TemplateContext::new();
                        if let Ok(v) = json_to_template_scalar(item) {
                            child.set("value", v);
                        }
                        items.push(child);
                    }
                    _ => return Err(()),
                }
            }
            Ok(TemplateValue::List(items))
        }
        Value::Object(_) => Err(()),
    }
}

fn json_to_template_scalar(value: &Value) -> Result<TemplateValue, ()> {
    match value {
        Value::Array(_) | Value::Object(_) => Err(()),
        _ => json_to_template_value(value),
    }
}
