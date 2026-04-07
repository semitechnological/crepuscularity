//! Multi-route SSR router: maps URL paths to .crepus templates.
use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, State},
    response::Html,
    routing::get,
};
use crepuscularity_core::TemplateContext;
use crepuscularity_web::{SsrDocument, render_ssr_document};

/// A single route entry pairing a template source with a page title.
pub struct RouteEntry {
    pub template: String,
    pub title: String,
}

/// Builds an Axum `Router` from a declarative map of URL paths → `.crepus` templates.
///
/// # Example
/// ```rust,no_run
/// use crepuscularity_ssr::SsrRouter;
///
/// let router = SsrRouter::new()
///     .route("/", r#"div "Home""#, "Home")
///     .route("/about", r#"div "About""#, "About")
///     .into_axum_router();
/// ```
pub struct SsrRouter {
    routes: HashMap<String, RouteEntry>,
}

impl Default for SsrRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl SsrRouter {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    /// Register a URL path with its template source and page title.
    pub fn route(
        mut self,
        path: &str,
        template: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        self.routes.insert(
            path.to_string(),
            RouteEntry {
                template: template.into(),
                title: title.into(),
            },
        );
        self
    }

    /// Build an Axum [`Router`] from the registered routes.
    pub fn into_axum_router(self) -> Router {
        let routes = Arc::new(self.routes);
        Router::new()
            .route("/*path", get(handle_route))
            .route("/", get(handle_root))
            .with_state(routes)
    }
}

async fn handle_root(
    State(routes): State<Arc<HashMap<String, RouteEntry>>>,
) -> Html<String> {
    render_entry(&routes, "/")
}

async fn handle_route(
    State(routes): State<Arc<HashMap<String, RouteEntry>>>,
    Path(path): Path<String>,
) -> Html<String> {
    let path = format!("/{path}");
    render_entry(&routes, &path)
}

fn render_entry(routes: &HashMap<String, RouteEntry>, path: &str) -> Html<String> {
    let entry = match routes.get(path).or_else(|| routes.get("/")) {
        Some(e) => e,
        None => return Html("<h1>404 Not Found</h1>".to_string()),
    };
    let template = entry.template.clone();
    let title = entry.title.clone();
    let ctx = TemplateContext::new();

    let doc = SsrDocument {
        title: &title,
        ..Default::default()
    };
    let result = render_ssr_document(&template, &ctx, &doc, true);

    match result {
        Ok(h) => Html(h),
        Err(e) => Html(format!("<pre style='color:red'>{e}</pre>")),
    }
}
