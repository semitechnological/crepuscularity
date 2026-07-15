use std::path::{Path, PathBuf};

use console::style;
use crepuscularity_core::context::{TemplateContext, TemplateValue};
use crepuscularity_embedded::{write_ppm, Ui};
use crepuscularity_lvgl::{
    render_component_file_to_lvgl_xml, render_template_to_lvgl_xml_with_options, LvglOptions,
    LvglRoot,
};
use crepuscularity_native::{
    render_component_file_to_ir, render_template_to_ir, to_json, to_json_pretty,
};

use crate::build_options::BuildOptions;
use crate::crepus_toml::ResolvedTarget;
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
        crate::crepus_toml::pick_targets(&targets, target_id.as_deref()).unwrap_or_else(|e| ui::error(&e))
    };
    for target in picked {
        build_target(&target, options);
    }
}

fn pick_targets_by_selector(
    targets: &[ResolvedTarget],
    selector: &str,
) -> Result<Vec<ResolvedTarget>, String> {
    if let Ok(picked) = crate::crepus_toml::pick_targets(targets, Some(selector)) {
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
    Err(format!(
        "no target id or type {selector:?} (ids: {ids:?})"
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
    let result = match target.target_type.as_str() {
        "web" => {
            crate::web::build_site_wasm(&crate::web::WebBuildArgs {
                site_dir: Some(target.dir.clone()),
                out_dir: target.out.clone(),
                entry: target.entry.clone(),
                target_id: None,
                manifest: None,
                meta: Some(target.web.clone()),
                options,
            });
            Ok(())
        }
        "webext" => {
            if let Some(manifest) = &target.webext {
                crate::webext::build_app_target(
                    &target.dir,
                    manifest,
                    &target.webext_config,
                    options,
                );
            } else {
                crate::webext::build_app_path(&target.dir, options);
            }
            Ok(())
        }
        "lvgl" => build_lvgl_target(target, options),
        "native" | "ir" => build_native_target(target, options),
        "embedded" => build_embedded_target(target, options),
        other => Err(format!("unsupported target type {other:?}")),
    };

    if let Err(e) = result {
        ui::error(&e);
    }
}

fn build_lvgl_target(target: &ResolvedTarget, _options: BuildOptions) -> Result<(), String> {
    let template_path = resolve_template_path(target);
    let template = std::fs::read_to_string(&template_path)
        .map_err(|e| format!("read {}: {e}", template_path.display()))?;
    let ctx = target_context(target, template_path.parent().map(PathBuf::from));
    let name = target.name.clone().unwrap_or_else(|| target.id.clone());
    let root = match target.root.as_deref().unwrap_or("component") {
        "screen" => LvglRoot::Screen,
        "component" => LvglRoot::Component,
        other => return Err(format!("lvgl root must be component or screen, got {other:?}")),
    };
    let xml = if let Some(component) = &target.component {
        render_component_file_to_lvgl_xml(&template, component, &ctx)
    } else {
        render_template_to_lvgl_xml_with_options(&template, &ctx, &LvglOptions { name, root })
    }
    .map_err(|e| format!("render lvgl target {:?}: {e}", target.id))?;
    let out = target.out.clone().unwrap_or_else(|| {
        target
            .dir
            .join("dist")
            .join(format!("{}.xml", target.id))
    });
    write_output(&out, xml.as_bytes());
    eprintln!(
        "{} {} {}",
        ui::ok(),
        style(&target.id).cyan().bold(),
        style(out.display().to_string()).dim()
    );
    Ok(())
}

fn build_native_target(target: &ResolvedTarget, _options: BuildOptions) -> Result<(), String> {
    let template_path = resolve_template_path(target);
    let template = std::fs::read_to_string(&template_path)
        .map_err(|e| format!("read {}: {e}", template_path.display()))?;
    let ctx = target_context(target, template_path.parent().map(PathBuf::from));
    let ir = if let Some(component) = &target.component {
        render_component_file_to_ir(&template, component, &ctx)
    } else {
        render_template_to_ir(&template, &ctx)
    }
    .map_err(|e| format!("render native target {:?}: {e}", target.id))?;
    let json = if target.root.as_deref() == Some("pretty") {
        to_json_pretty(&ir)
    } else {
        to_json(&ir)
    }
    .map_err(|e| format!("serialize native target {:?}: {e}", target.id))?;
    let out = target.out.clone().unwrap_or_else(|| {
        target
            .dir
            .join("dist")
            .join(format!("{}.json", target.id))
    });
    write_output(&out, json.as_bytes());
    eprintln!(
        "{} {} {}",
        ui::ok(),
        style(&target.id).cyan().bold(),
        style(out.display().to_string()).dim()
    );
    Ok(())
}

fn build_embedded_target(target: &ResolvedTarget, _options: BuildOptions) -> Result<(), String> {
    let width = target
        .width
        .ok_or_else(|| "embedded target needs width".to_string())?;
    let height = target
        .height
        .ok_or_else(|| "embedded target needs height".to_string())?;
    let template_path = resolve_template_path(target);
    let template = std::fs::read_to_string(&template_path)
        .map_err(|e| format!("read {}: {e}", template_path.display()))?;
    let ctx = target_context(target, template_path.parent().map(PathBuf::from));
    let mut ui_view = Ui::new(width, height, &template);
    if let Some(component) = &target.component {
        ui_view.set_component(component.clone());
    }
    for (key, value) in &ctx.vars {
        ui_view.set(key, value.clone());
    }
    let screen = ui_view.screen();
    let mut ppm =
        crepuscularity_embedded::Rgb888Buffer::new(screen, crepuscularity_embedded::DEFAULT_BG);
    ui_view
        .render_into(&mut ppm)
        .map_err(|e| format!("render embedded target {:?}: {e}", target.id))?;
    let out = target.out.clone().unwrap_or_else(|| {
        target
            .dir
            .join("dist")
            .join(format!("{}.ppm", target.id))
    });
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    write_ppm(&out, &ppm).map_err(|e| format!("write {}: {e}", out.display()))?;
    eprintln!(
        "{} {} {}",
        ui::ok(),
        style(&target.id).cyan().bold(),
        style(out.display().to_string()).dim()
    );
    Ok(())
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

fn write_output(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .unwrap_or_else(|e| ui::error(&format!("create {}: {e}", parent.display())));
    }
    std::fs::write(path, bytes)
        .unwrap_or_else(|e| ui::error(&format!("write {}: {e}", path.display())));
}
