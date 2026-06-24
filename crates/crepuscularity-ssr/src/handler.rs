use axum::{extract::State, response::Html};
use crepuscularity_core::ast::Node;
use crepuscularity_core::{TemplateContext, TemplateValue};
use crepuscularity_web::{render_ssr_document, SsrDocument};
use std::{collections::HashMap, sync::Arc};

/// Escape HTML special characters in error messages before injecting into HTML.
fn escape_html_error(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Configuration for [`SsrHandler`].
///
/// The `nodes` field holds the pre-parsed template AST so the template is not re-parsed
/// on every HTTP request. Use [`SsrOptions::new`] to construct — it pre-parses the template
/// at call time so parse errors surface immediately at startup.
#[derive(Clone)]
pub struct SsrOptions {
    /// Template source string (content of a .crepus file).
    pub template: &'static str,
    /// Default context variables injected on every request.
    pub defaults: HashMap<String, TemplateValue>,
    /// HTML `<title>` for the rendered document.
    pub title: String,
    /// Pre-parsed template AST, cached at construction time to avoid re-parsing on every request.
    pub nodes: Arc<Vec<Node>>,
}

impl SsrOptions {
    /// Create new SSR options, pre-parsing the template at construction time.
    ///
    /// Panics if the template fails to parse — this is intentional so invalid templates
    /// are caught at startup rather than at request time.
    pub fn new(template: &'static str, title: impl Into<String>) -> Self {
        let nodes = crepuscularity_core::ast_cache::parse_content(template)
            .expect("SsrOptions::new: failed to parse template");
        Self {
            template,
            defaults: HashMap::new(),
            title: title.into(),
            nodes,
        }
    }

    /// Set default context variables injected on every request.
    pub fn with_defaults(mut self, defaults: HashMap<String, TemplateValue>) -> Self {
        self.defaults = defaults;
        self
    }
}

/// SSR handler with pre-parsed template caching.
///
/// Construct with [`SsrHandler::new`] and obtain the Axum state via [`SsrHandler::state`].
#[non_exhaustive]
pub struct SsrHandler {
    opts: Arc<SsrOptions>,
}

impl SsrHandler {
    /// Create a new handler, caching the pre-parsed template from `opts`.
    ///
    /// The `opts` should be constructed via [`SsrOptions::new`] which pre-parses the template.
    /// The resulting `Arc<SsrOptions>` is stored and can be obtained via [`SsrHandler::state`]
    /// for use as Axum state.
    pub fn new(opts: SsrOptions) -> Self {
        Self {
            opts: Arc::new(opts),
        }
    }

    /// Returns the cached `SsrOptions` wrapped in `Arc`, suitable for use as Axum state.
    ///
    /// ```rust,no_run
    /// use axum::{routing::get, Router};
    /// use crepuscularity_ssr::{SsrHandler, SsrOptions};
    /// use std::sync::Arc;
    ///
    /// let handler = SsrHandler::new(SsrOptions::new(r#"div "Hello""#, "Title"));
    /// let app: Router = Router::new()
    ///     .route("/", get(SsrHandler::handle))
    ///     .with_state(handler.state());
    /// ```
    pub fn state(&self) -> Arc<SsrOptions> {
        Arc::clone(&self.opts)
    }

    /// Axum-compatible handler: renders the template with SSR markers and wraps it in an HTML5 shell.
    ///
    /// The synchronous rendering work runs inside `tokio::task::spawn_blocking` so the async
    /// runtime event loop is not blocked. The pre-parsed template AST (`opts.nodes`) is available
    /// in the state and is shared across all requests via `Arc`.
    pub async fn handle(State(opts): State<Arc<SsrOptions>>) -> Html<String> {
        let ctx = TemplateContext {
            vars: opts.defaults.clone(),
            base_dir: None,
            slot: None,
            virtual_files: std::sync::Arc::new(HashMap::new()),
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
                render_ssr_document(template, &ctx, &doc, true).map_err(|e| e.to_string())
            }
        })
        .await
        .unwrap_or_else(|e| Err(format!("spawn_blocking panicked: {e}")));

        match html {
            Ok(page) => Html(page),
            Err(e) => Html(format!(
                "<pre style='color:red'>{}</pre>",
                escape_html_error(&e)
            )),
        }
    }
}
