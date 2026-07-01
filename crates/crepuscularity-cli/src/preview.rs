#[cfg(feature = "desktop")]
pub(crate) fn run_preview(path: std::path::PathBuf) {
    use console::style;
    use crepuscularity_runtime::{HotReloadState, HotReloadView, TemplateContext};
    use gpui::{bounds, point, prelude::*, px, size, Application, WindowOptions};

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

    eprintln!(
        "{} previewing {}",
        style("crepus").dim(),
        style(path.display().to_string()).cyan().bold()
    );

    Application::new().run(move |cx: &mut gpui::App| {
        let opts = WindowOptions {
            app_id: Some(format!("crepuscularity.preview.{display_name}")),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some(format!("Crepus Preview - {display_name}").into()),
                ..Default::default()
            }),
            window_bounds: Some(gpui::WindowBounds::Windowed(bounds(
                point(px(100.), px(100.)),
                size(px(1200.), px(800.)),
            ))),
            ..Default::default()
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

#[cfg(not(feature = "desktop"))]
pub(crate) struct CargoBuildOutcome {
    pub success: bool,
}

#[cfg(not(feature = "desktop"))]
pub(crate) fn cargo_build_fallback(
    cwd: &std::path::Path,
    options: crate::build_options::BuildOptions,
) -> CargoBuildOutcome {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build").current_dir(cwd);
    if options.release() {
        cmd.arg("--release");
    }
    let success = cmd.status().map(|status| status.success()).unwrap_or(false);
    CargoBuildOutcome { success }
}
