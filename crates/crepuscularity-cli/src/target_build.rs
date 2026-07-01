use std::collections::BTreeSet;
use std::path::PathBuf;

use console::style;
use crepuscularity_core::context::{TemplateContext, TemplateValue};
use crepuscularity_lvgl::{
    render_component_file_to_lvgl_xml, render_template_to_lvgl_xml_with_options, LvglOptions,
    LvglRoot,
};
use crepuscularity_native::{
    render_component_file_to_ir, render_template_to_ir, to_json, to_json_pretty,
};

use crate::build_options::BuildOptions;
use crate::crepus_toml::{pick_targets, ResolvedTarget};
use crate::ui;

pub struct ManifestBuildArgs {
    pub options: BuildOptions,
    pub target_id: Option<String>,
    pub selector: Option<String>,
    pub manifest: Option<PathBuf>,
    pub all: bool,
}

pub(crate) fn has_manifest_targets(manifest: Option<PathBuf>) -> bool {
    matches!(
        crate::crepus_toml::load_manifest_targets(manifest),
        Ok(Some(targets)) if !targets.is_empty()
    )
}

pub(crate) fn execute(args: ManifestBuildArgs) {
    let ManifestBuildArgs {
        options,
        target_id,
        selector,
        manifest,
        all,
    } = args;
    if target_id.is_some() && selector.is_some() {
        ui::error("use either --target ID or a positional selector, not both");
    }
    if all && (target_id.is_some() || selector.is_some()) {
        ui::error("use either --all or a target selector, not both");
    }
    let targets = crate::crepus_toml::load_manifest_targets(manifest)
        .unwrap_or_else(|e| ui::error(&e))
        .unwrap_or_else(|| ui::error("no crepus.toml found"));
    if targets.is_empty() {
        ui::error("crepus.toml has no [[targets]] entries");
    }
    let picked = if all {
        targets
    } else if let Some(sel) = selector.as_deref() {
        pick_targets_by_selector(&targets, sel).unwrap_or_else(|e| ui::error(&e))
    } else {
        pick_targets(&targets, target_id.as_deref()).unwrap_or_else(|e| ui::error(&e))
    };
    for target in picked {
        build_target(&target, options);
    }
}

fn pick_targets_by_selector(
    targets: &[ResolvedTarget],
    selector: &str,
) -> Result<Vec<ResolvedTarget>, String> {
    if let Ok(picked) = pick_targets(targets, Some(selector)) {
        return Ok(picked);
    }
    let normalized = normalize_selector(selector);
    let picked: Vec<ResolvedTarget> = targets
        .iter()
        .filter(|target| normalize_selector(&target.target_type) == normalized)
        .cloned()
        .collect();
    if !picked.is_empty() {
        return Ok(picked);
    }
    let ids: Vec<&str> = targets.iter().map(|t| t.id.as_str()).collect();
    let types: BTreeSet<&str> = targets.iter().map(|t| t.target_type.as_str()).collect();
    Err(format!(
        "no target id or type {selector:?} (ids: {ids:?}; types: {types:?})"
    ))
}

fn normalize_selector(selector: &str) -> &str {
    match selector {
        "extension" | "browser-extension" | "web-extension" => "webext",
        "ir" => "native",
        other => other,
    }
}

fn build_target(target: &ResolvedTarget, options: BuildOptions) {
    match target.target_type.as_str() {
        "web" => build_web(target, options),
        "webext" => {
            if let Some(manifest) = &target.webext {
                crate::webext::build_app_target(&target.dir, manifest, options);
            } else {
                crate::webext::build_app_path(&target.dir, options);
            }
        }
        "lvgl" => build_lvgl(target),
        "native" | "ir" => build_native_ir(target),
        "embedded" => build_embedded(target),
        other => ui::error(&format!(
            "target {:?} uses unsupported type {:?}",
            target.id, other
        )),
    }
}

fn build_web(target: &ResolvedTarget, options: BuildOptions) {
    crate::web::build_site_wasm(&crate::web::WebBuildArgs {
        site_dir: Some(target.dir.clone()),
        out_dir: target.out.clone(),
        entry: target.entry.clone(),
        target_id: None,
        manifest: None,
        meta: Some(target.web.clone()),
        options,
    });
}

fn build_lvgl(target: &ResolvedTarget) {
    let template_path = resolve_template_path(target);
    let template = std::fs::read_to_string(&template_path)
        .unwrap_or_else(|e| ui::error(&format!("read {}: {e}", template_path.display())));
    let ctx = target_context(target, template_path.parent().map(PathBuf::from));
    let name = target.name.clone().unwrap_or_else(|| target.id.clone());
    let root = match target.root.as_deref().unwrap_or("component") {
        "screen" => LvglRoot::Screen,
        "component" => LvglRoot::Component,
        other => ui::error(&format!(
            "lvgl root must be component or screen, got {other:?}"
        )),
    };
    let xml = if let Some(component) = &target.component {
        render_component_file_to_lvgl_xml(&template, component, &ctx)
    } else {
        render_template_to_lvgl_xml_with_options(&template, &ctx, &LvglOptions { name, root })
    }
    .unwrap_or_else(|e| ui::error(&format!("render lvgl target {:?}: {e}", target.id)));
    let out = target
        .out
        .clone()
        .unwrap_or_else(|| target.dir.join("dist").join(format!("{}.xml", target.id)));
    write_output(&out, xml.as_bytes());
    eprintln!(
        "{} {} {}",
        ui::ok(),
        style(&target.id).cyan().bold(),
        style(out.display().to_string()).dim()
    );
}

fn build_native_ir(target: &ResolvedTarget) {
    let template_path = resolve_template_path(target);
    let template = std::fs::read_to_string(&template_path)
        .unwrap_or_else(|e| ui::error(&format!("read {}: {e}", template_path.display())));
    let ctx = target_context(target, template_path.parent().map(PathBuf::from));
    let ir = if let Some(component) = &target.component {
        render_component_file_to_ir(&template, component, &ctx)
    } else {
        render_template_to_ir(&template, &ctx)
    }
    .unwrap_or_else(|e| ui::error(&format!("render native target {:?}: {e}", target.id)));
    let json = if target.root.as_deref() == Some("pretty") {
        to_json_pretty(&ir)
    } else {
        to_json(&ir)
    }
    .unwrap_or_else(|e| ui::error(&format!("serialize native target {:?}: {e}", target.id)));
    let out = target
        .out
        .clone()
        .unwrap_or_else(|| target.dir.join("dist").join(format!("{}.json", target.id)));
    write_output(&out, json.as_bytes());
    eprintln!(
        "{} {} {}",
        ui::ok(),
        style(&target.id).cyan().bold(),
        style(out.display().to_string()).dim()
    );
}

fn build_embedded(target: &ResolvedTarget) {
    let width = target
        .width
        .unwrap_or_else(|| ui::error("embedded target needs width"));
    let height = target
        .height
        .unwrap_or_else(|| ui::error("embedded target needs height"));
    let template_path = resolve_template_path(target);
    let template = std::fs::read_to_string(&template_path)
        .unwrap_or_else(|e| ui::error(&format!("read {}: {e}", template_path.display())));
    let ctx = target_context(target, template_path.parent().map(PathBuf::from));
    let mut ui_view = crepuscularity_embedded::Ui::new(width, height, &template);
    if let Some(component) = &target.component {
        ui_view.set_component(component.clone());
    }
    for (key, value) in ctx.vars {
        ui_view.set(key, value);
    }
    let screen = ui_view.screen();
    let mut ppm =
        crepuscularity_embedded::Rgb888Buffer::new(screen, crepuscularity_embedded::DEFAULT_BG);
    ui_view
        .render_into(&mut ppm)
        .unwrap_or_else(|e| ui::error(&format!("render embedded target {:?}: {e}", target.id)));
    let out = target
        .out
        .clone()
        .unwrap_or_else(|| target.dir.join("dist").join(format!("{}.ppm", target.id)));
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)
            .unwrap_or_else(|e| ui::error(&format!("create {}: {e}", parent.display())));
    }
    crepuscularity_embedded::write_ppm(&out, &ppm)
        .unwrap_or_else(|e| ui::error(&format!("write {}: {e}", out.display())));
    eprintln!(
        "{} {} {}",
        ui::ok(),
        style(&target.id).cyan().bold(),
        style(out.display().to_string()).dim()
    );
}

fn resolve_template_path(target: &ResolvedTarget) -> PathBuf {
    target
        .template
        .clone()
        .or_else(|| target.entry.as_ref().map(|entry| target.dir.join(entry)))
        .unwrap_or_else(|| target.dir.join("ui.crepus"))
}

fn target_context(target: &ResolvedTarget, base_dir: Option<PathBuf>) -> TemplateContext {
    let mut ctx = TemplateContext::new();
    ctx.base_dir = base_dir;
    if let Some(path) = &target.ctx {
        load_ctx_file(path, &mut ctx);
    }
    for (key, value) in &target.vars {
        ctx.set(key, toml_to_template_value(value));
    }
    ctx
}

fn load_ctx_file(path: &PathBuf, ctx: &mut TemplateContext) {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| ui::error(&format!("read {}: {e}", path.display())));
    let table = raw
        .parse::<toml::Table>()
        .unwrap_or_else(|e| ui::error(&format!("parse {}: {e}", path.display())));
    for (key, value) in table {
        ctx.set(key, toml_to_template_value(&value));
    }
}

fn toml_to_template_value(value: &toml::Value) -> TemplateValue {
    match value {
        toml::Value::String(s) => TemplateValue::Str(s.clone()),
        toml::Value::Integer(n) => TemplateValue::Int(*n),
        toml::Value::Float(n) => TemplateValue::Float(*n),
        toml::Value::Boolean(b) => TemplateValue::Bool(*b),
        toml::Value::Array(items) => TemplateValue::List(
            items
                .iter()
                .map(|item| {
                    let mut row = TemplateContext::new();
                    row.set("value", toml_to_template_value(item));
                    row
                })
                .collect(),
        ),
        toml::Value::Table(table) => {
            let mut ctx = TemplateContext::new();
            for (key, value) in table {
                ctx.set(key, toml_to_template_value(value));
            }
            TemplateValue::Scope(ctx)
        }
        toml::Value::Datetime(dt) => TemplateValue::Str(dt.to_string()),
    }
}

fn write_output(path: &PathBuf, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .unwrap_or_else(|e| ui::error(&format!("create {}: {e}", parent.display())));
    }
    std::fs::write(path, bytes)
        .unwrap_or_else(|e| ui::error(&format!("write {}: {e}", path.display())));
}
