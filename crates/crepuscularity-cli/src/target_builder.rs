use std::path::PathBuf;

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

pub(crate) trait TargetBuilder {
    fn build(&self, options: &BuildOptions) -> Result<(), String>;
    fn output_path(&self, options: &BuildOptions) -> PathBuf;
    #[allow(dead_code)]
    fn target_triple(&self) -> Option<&str>;
}

pub(crate) struct WebBuilder<'a> {
    target: &'a ResolvedTarget,
}

pub(crate) struct WebextBuilder<'a> {
    target: &'a ResolvedTarget,
}

pub(crate) struct LvglBuilder<'a> {
    target: &'a ResolvedTarget,
}

pub(crate) struct NativeBuilder<'a> {
    target: &'a ResolvedTarget,
}

pub(crate) struct EmbeddedBuilder<'a> {
    target: &'a ResolvedTarget,
}

pub(crate) fn builder_for(target: &ResolvedTarget) -> Box<dyn TargetBuilder + '_> {
    match target.target_type.as_str() {
        "web" => Box::new(WebBuilder { target }),
        "webext" => Box::new(WebextBuilder { target }),
        "lvgl" => Box::new(LvglBuilder { target }),
        "native" | "ir" => Box::new(NativeBuilder { target }),
        "embedded" => Box::new(EmbeddedBuilder { target }),
        other => panic!("unsupported target type {other:?}"),
    }
}

impl TargetBuilder for WebBuilder<'_> {
    fn build(&self, options: &BuildOptions) -> Result<(), String> {
        crate::web::build_site_wasm(&crate::web::WebBuildArgs {
            site_dir: Some(self.target.dir.clone()),
            out_dir: self.target.out.clone(),
            entry: self.target.entry.clone(),
            target_id: None,
            manifest: None,
            meta: Some(self.target.web.clone()),
            options: *options,
        });
        Ok(())
    }

    fn output_path(&self, _options: &BuildOptions) -> PathBuf {
        self.target
            .out
            .clone()
            .unwrap_or_else(|| self.target.dir.join("dist"))
    }

    fn target_triple(&self) -> Option<&str> {
        Some("wasm32-unknown-unknown")
    }
}

impl TargetBuilder for WebextBuilder<'_> {
    fn build(&self, options: &BuildOptions) -> Result<(), String> {
        if let Some(manifest) = &self.target.webext {
            crate::webext::build_app_target(
                &self.target.dir,
                manifest,
                &self.target.webext_config,
                *options,
            );
        } else {
            crate::webext::build_app_path(&self.target.dir, *options);
        }
        Ok(())
    }

    fn output_path(&self, _options: &BuildOptions) -> PathBuf {
        self.target.dir.join("dist").join("unpacked")
    }

    fn target_triple(&self) -> Option<&str> {
        Some("wasm32-unknown-unknown")
    }
}

impl TargetBuilder for LvglBuilder<'_> {
    fn build(&self, options: &BuildOptions) -> Result<(), String> {
        let template_path = resolve_template_path(self.target);
        let template = std::fs::read_to_string(&template_path)
            .map_err(|e| format!("read {}: {e}", template_path.display()))?;
        let ctx = target_context(self.target, template_path.parent().map(PathBuf::from));
        let name = self
            .target
            .name
            .clone()
            .unwrap_or_else(|| self.target.id.clone());
        let root = match self.target.root.as_deref().unwrap_or("component") {
            "screen" => LvglRoot::Screen,
            "component" => LvglRoot::Component,
            other => {
                return Err(format!(
                    "lvgl root must be component or screen, got {other:?}"
                ))
            }
        };
        let xml = if let Some(component) = &self.target.component {
            render_component_file_to_lvgl_xml(&template, component, &ctx)
        } else {
            render_template_to_lvgl_xml_with_options(&template, &ctx, &LvglOptions { name, root })
        }
        .map_err(|e| format!("render lvgl target {:?}: {e}", self.target.id))?;
        let out = self.output_path(options);
        write_output(&out, xml.as_bytes());
        eprintln!(
            "{} {} {}",
            ui::ok(),
            style(&self.target.id).cyan().bold(),
            style(out.display().to_string()).dim()
        );
        Ok(())
    }

    fn output_path(&self, _options: &BuildOptions) -> PathBuf {
        self.target.out.clone().unwrap_or_else(|| {
            self.target
                .dir
                .join("dist")
                .join(format!("{}.xml", self.target.id))
        })
    }

    fn target_triple(&self) -> Option<&str> {
        None
    }
}

impl TargetBuilder for NativeBuilder<'_> {
    fn build(&self, options: &BuildOptions) -> Result<(), String> {
        let template_path = resolve_template_path(self.target);
        let template = std::fs::read_to_string(&template_path)
            .map_err(|e| format!("read {}: {e}", template_path.display()))?;
        let ctx = target_context(self.target, template_path.parent().map(PathBuf::from));
        let ir = if let Some(component) = &self.target.component {
            render_component_file_to_ir(&template, component, &ctx)
        } else {
            render_template_to_ir(&template, &ctx)
        }
        .map_err(|e| format!("render native target {:?}: {e}", self.target.id))?;
        let json = if self.target.root.as_deref() == Some("pretty") {
            to_json_pretty(&ir)
        } else {
            to_json(&ir)
        }
        .map_err(|e| format!("serialize native target {:?}: {e}", self.target.id))?;
        let out = self.output_path(options);
        write_output(&out, json.as_bytes());
        eprintln!(
            "{} {} {}",
            ui::ok(),
            style(&self.target.id).cyan().bold(),
            style(out.display().to_string()).dim()
        );
        Ok(())
    }

    fn output_path(&self, _options: &BuildOptions) -> PathBuf {
        self.target.out.clone().unwrap_or_else(|| {
            self.target
                .dir
                .join("dist")
                .join(format!("{}.json", self.target.id))
        })
    }

    fn target_triple(&self) -> Option<&str> {
        None
    }
}

impl TargetBuilder for EmbeddedBuilder<'_> {
    fn build(&self, options: &BuildOptions) -> Result<(), String> {
        let width = self
            .target
            .width
            .ok_or_else(|| "embedded target needs width".to_string())?;
        let height = self
            .target
            .height
            .ok_or_else(|| "embedded target needs height".to_string())?;
        let template_path = resolve_template_path(self.target);
        let template = std::fs::read_to_string(&template_path)
            .map_err(|e| format!("read {}: {e}", template_path.display()))?;
        let ctx = target_context(self.target, template_path.parent().map(PathBuf::from));
        let mut ui_view = Ui::new(width, height, &template);
        if let Some(component) = &self.target.component {
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
            .map_err(|e| format!("render embedded target {:?}: {e}", self.target.id))?;
        let out = self.output_path(options);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("create {}: {e}", parent.display()))?;
        }
        write_ppm(&out, &ppm).map_err(|e| format!("write {}: {e}", out.display()))?;
        eprintln!(
            "{} {} {}",
            ui::ok(),
            style(&self.target.id).cyan().bold(),
            style(out.display().to_string()).dim()
        );
        Ok(())
    }

    fn output_path(&self, _options: &BuildOptions) -> PathBuf {
        self.target.out.clone().unwrap_or_else(|| {
            self.target
                .dir
                .join("dist")
                .join(format!("{}.ppm", self.target.id))
        })
    }

    fn target_triple(&self) -> Option<&str> {
        None
    }
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
