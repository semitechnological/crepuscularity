use axum::{extract::State, response::Html};
use crepuscularity_core::{TemplateContext, TemplateValue};
use crepuscularity_web::{render_ssr_document, SsrDocument};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct SsrOptions {
    /// Template source string (content of a .crepus file).
    pub template: &'static str,
    /// Default context variables injected on every request.
    pub defaults: HashMap<String, TemplateValue>,
    /// HTML `<title>` for the rendered document.
    pub title: String,
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

    /// Axum-compatible handler: renders the template with SSR markers and wraps it in an HTML5 shell.
    pub async fn handle(State(opts): State<Arc<SsrOptions>>) -> Html<String> {
        let ctx = TemplateContext {
            vars: opts.defaults.clone(),
            base_dir: None,
            slot: None,
            virtual_files: std::sync::Arc::new(std::collections::HashMap::new()),
        };
        let html = tokio::task::spawn_blocking({
            let template = opts.template;
            let title = opts.title.clone();
            let ctx = ctx.clone();
            move || -> Result<String, String> {
                let doc = SsrDocument {
                    title: &title,
                    ..Default::default()
                };
                render_ssr_document(template, &ctx, &doc, true)
            }
        })
        .await
        .unwrap_or_else(|e| Err(format!("spawn_blocking panicked: {e}")));

        match html {
            Ok(page) => Html(page),
            Err(e) => Html(format!("<pre style='color:red'>{e}</pre>")),
        }
    }
}
