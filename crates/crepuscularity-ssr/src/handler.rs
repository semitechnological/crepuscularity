use axum::{extract::State, response::Html};
use crepuscularity_core::{TemplateContext, TemplateValue};
use crepuscularity_web::render_template_to_html;
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct SsrOptions {
    /// Template source string (content of a .crepus file).
    pub template: &'static str,
    /// Default context variables injected on every request.
    pub defaults: HashMap<String, TemplateValue>,
}

pub struct SsrHandler {
    /// Shared options used to configure the Axum state for this handler.
    #[allow(dead_code)]
    opts: Arc<SsrOptions>,
}

impl SsrHandler {
    pub fn new(opts: SsrOptions) -> Self {
        Self {
            opts: Arc::new(opts),
        }
    }

    /// Axum-compatible handler: renders the template with the provided context.
    pub async fn handle(State(opts): State<Arc<SsrOptions>>) -> Html<String> {
        let ctx = TemplateContext {
            vars: opts.defaults.clone(),
            base_dir: None,
            slot: None,
            virtual_files: HashMap::new(),
        };
        let html = tokio::task::spawn_blocking({
            let template = opts.template;
            let ctx = ctx.clone();
            move || render_template_to_html(template, &ctx)
        })
        .await
        .unwrap_or_else(|e| Err(format!("spawn_blocking panicked: {e}")));

        match html {
            Ok(body) => Html(format!(
                "<!doctype html><html><head><meta charset=utf-8></head><body>{body}</body></html>"
            )),
            Err(e) => Html(format!("<pre style='color:red'>{e}</pre>")),
        }
    }
}
