//! `crepus embedded` — validate templates and optional PPM snapshots for debugging.

use std::fs;
use std::path::{Path, PathBuf};

use console::style;
use crepuscularity_core::context::{TemplateContext, TemplateValue};
use crepuscularity_core::parser::{parse_component_file, parse_template};
use crepuscularity_embedded::{write_ppm, Ui};
use serde_json::Value;

pub fn run(args: &[String]) {
    match args.first().map(|s| s.as_str()) {
        Some("check") => match run_check(&args[1..]) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        },
        Some("snapshot") => match run_snapshot(&args[1..]) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        },
        _ => print_embedded_usage(),
    }
}

fn run_check(args: &[String]) -> Result<(), String> {
    let (path, component) = parse_file_args(args)?;
    let content = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    if let Some(name) = component {
        let file = parse_component_file(&content).map_err(|e| e.to_string())?;
        file.components
            .get(&name)
            .ok_or_else(|| format!("component not found: {name}"))?;
    } else {
        parse_template(&content).map_err(|e| e.to_string())?;
    }
    println!("ok: {}", path.display());
    Ok(())
}

fn run_snapshot(args: &[String]) -> Result<(), String> {
    let parsed = parse_snapshot_args(args)?;

    let mut ctx = TemplateContext::new();
    if let Some(path) = &parsed.ctx_file {
        load_json_ctx(path, &mut ctx)?;
    }
    for (key, raw) in parsed.vars {
        ctx.set(key, parse_var_value(&raw));
    }

    let content = fs::read_to_string(&parsed.path)
        .map_err(|e| format!("read {}: {e}", parsed.path.display()))?;
    let mut ui = Ui::new(parsed.width, parsed.height, &content);
    if let Some(name) = parsed.component {
        ui.set_component(name);
    }
    for (k, v) in ctx.vars {
        ui.set(k, v);
    }

    let screen = ui.screen();
    let mut ppm =
        crepuscularity_embedded::Rgb888Buffer::new(screen, crepuscularity_embedded::DEFAULT_BG);
    ui.render_into(&mut ppm)?;
    write_ppm(&parsed.out, &ppm)?;
    Ok(())
}

struct SnapshotArgs {
    path: PathBuf,
    width: u16,
    height: u16,
    out: PathBuf,
    ctx_file: Option<PathBuf>,
    vars: Vec<(String, String)>,
    component: Option<String>,
}

fn parse_file_args(args: &[String]) -> Result<(PathBuf, Option<String>), String> {
    let mut path: Option<PathBuf> = None;
    let mut component: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--component" => {
                i += 1;
                component = args.get(i).cloned();
                if component.is_none() {
                    return Err("--component expects a name".to_string());
                }
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown option: {other}"));
            }
            other => {
                if path.is_some() {
                    return Err(format!("unexpected argument: {other}"));
                }
                path = Some(PathBuf::from(other));
            }
        }
        i += 1;
    }
    let path = path.ok_or_else(|| "expected <file.crepus>".to_string())?;
    Ok((path, component))
}

fn parse_snapshot_args(args: &[String]) -> Result<SnapshotArgs, String> {
    let mut path: Option<PathBuf> = None;
    let mut width: Option<u16> = None;
    let mut height: Option<u16> = None;
    let mut out: Option<PathBuf> = None;
    let mut ctx_file: Option<PathBuf> = None;
    let mut vars: Vec<(String, String)> = Vec::new();
    let mut component: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--width" => {
                i += 1;
                width = Some(parse_u16_arg(args.get(i), "--width")?);
            }
            "--height" => {
                i += 1;
                height = Some(parse_u16_arg(args.get(i), "--height")?);
            }
            "--out" => {
                i += 1;
                out = args.get(i).map(PathBuf::from);
                if out.is_none() {
                    return Err("--out expects a file path".to_string());
                }
            }
            "--ctx" => {
                i += 1;
                ctx_file = args.get(i).map(PathBuf::from);
                if ctx_file.is_none() {
                    return Err("--ctx expects a file path".to_string());
                }
            }
            "--var" => {
                i += 1;
                let Some(raw) = args.get(i) else {
                    return Err("--var expects key=value".to_string());
                };
                let Some((key, value)) = raw.split_once('=') else {
                    return Err(format!("--var expects key=value, got: {raw}"));
                };
                vars.push((key.to_string(), value.to_string()));
            }
            "--component" => {
                i += 1;
                component = args.get(i).cloned();
                if component.is_none() {
                    return Err("--component expects a name".to_string());
                }
            }
            other => {
                if other.starts_with('-') {
                    return Err(format!("unknown option: {other}"));
                }
                if path.is_some() {
                    return Err(format!("unexpected argument: {other}"));
                }
                path = Some(PathBuf::from(other));
            }
        }
        i += 1;
    }

    let path = path.ok_or_else(|| {
        "Usage: crepus embedded snapshot <file.crepus> --width W --height H --out path.ppm"
            .to_string()
    })?;
    let width = width.ok_or_else(|| "--width is required".to_string())?;
    let height = height.ok_or_else(|| "--height is required".to_string())?;
    let out = out.ok_or_else(|| "--out is required".to_string())?;

    Ok(SnapshotArgs {
        path,
        width,
        height,
        out,
        ctx_file,
        vars,
        component,
    })
}

fn parse_u16_arg(raw: Option<&String>, flag: &str) -> Result<u16, String> {
    let Some(s) = raw else {
        return Err(format!("{flag} expects a positive integer"));
    };
    s.parse::<u16>()
        .map_err(|_| format!("{flag} expects a positive integer, got: {s}"))
}

fn load_json_ctx(path: &Path, ctx: &mut TemplateContext) -> Result<(), String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let value: Value =
        serde_json::from_str(&raw).map_err(|e| format!("context JSON {}: {e}", path.display()))?;
    merge_json_ctx(&value, ctx)
}

fn merge_json_ctx(value: &Value, ctx: &mut TemplateContext) -> Result<(), String> {
    let Some(obj) = value.as_object() else {
        return Err("context must be a JSON object".to_string());
    };
    for (key, value) in obj {
        ctx.set(key.clone(), json_to_template_value(value)?);
    }
    Ok(())
}

fn json_to_template_value(value: &Value) -> Result<TemplateValue, String> {
    match value {
        Value::Null => Ok(TemplateValue::Null),
        Value::Bool(v) => Ok(TemplateValue::Bool(*v)),
        Value::Number(v) => {
            if let Some(n) = v.as_i64() {
                Ok(TemplateValue::Int(n))
            } else if let Some(n) = v.as_f64() {
                Ok(TemplateValue::Float(n))
            } else {
                Err(format!("unsupported number: {v}"))
            }
        }
        Value::String(v) => Ok(TemplateValue::Str(v.clone())),
        Value::Array(values) => {
            let mut items = Vec::new();
            for item in values {
                let Some(obj) = item.as_object() else {
                    return Err("context arrays must contain objects".to_string());
                };
                let mut child = TemplateContext::new();
                for (key, value) in obj {
                    child.set(key.clone(), json_to_template_scalar(value)?);
                }
                items.push(child);
            }
            Ok(TemplateValue::List(items))
        }
        Value::Object(_) => {
            Err("context object values are only supported inside arrays".to_string())
        }
    }
}

fn json_to_template_scalar(value: &Value) -> Result<TemplateValue, String> {
    match value {
        Value::Array(_) | Value::Object(_) => {
            Err("loop item fields must be scalar JSON values".to_string())
        }
        _ => json_to_template_value(value),
    }
}

fn parse_var_value(raw: &str) -> TemplateValue {
    match raw {
        "true" => TemplateValue::Bool(true),
        "false" => TemplateValue::Bool(false),
        "null" => TemplateValue::Null,
        _ => raw
            .parse::<i64>()
            .map(TemplateValue::Int)
            .or_else(|_| raw.parse::<f64>().map(TemplateValue::Float))
            .unwrap_or_else(|_| TemplateValue::Str(raw.to_string())),
    }
}

pub fn print_embedded_usage() {
    eprintln!(
        "{}",
        style("crepus embedded — framebuffer UI for firmware")
            .cyan()
            .bold()
    );
    eprintln!();
    eprintln!("{}", style("COMMANDS").dim());
    eprintln!(
        "  {}  {}",
        style("check <file.crepus> [--component Name]").green(),
        style("validate template (CI / build.rs)").dim()
    );
    eprintln!(
        "  {}  {}",
        style("snapshot <file> --width W --height H --out file.ppm").green(),
        style("debug: write RGB888 PPM (not for production firmware)").dim()
    );
    eprintln!();
    eprintln!("{}", style("RUST INTEGRATION").dim());
    eprintln!(
        "  Add {} and call {} in your firmware loop.",
        style("crepuscularity-embedded").green(),
        style("Ui::render() / ui.rgb565()").green()
    );
    eprintln!(
        "  Use {} in build.rs for compile-time checks.",
        style("crepuscularity_core::build::compile_crepus").green()
    );
    eprintln!();
    eprintln!("{}", style("OPTIONS (snapshot)").dim());
    eprintln!(
        "  {}  {}",
        style("--ctx FILE                         ").green(),
        style("JSON object of template variables").dim()
    );
    eprintln!(
        "  {}  {}",
        style("--var key=value                    ").green(),
        style("set a single variable (repeatable)").dim()
    );
    eprintln!();
    eprintln!("{}", style("EXAMPLES").dim());
    eprintln!("  crepus embedded check ui/dashboard.crepus");
    eprintln!(
        "  crepus embedded snapshot ui.crepus --width 240 --height 320 --out /tmp/preview.ppm"
    );
}
