//! Hot-reload GPUI view component: reads a template file, renders it, and
//! watches for changes via the parent-directory `notify` watcher in
//! [`crate::watcher`].
//!
//! # What triggers a reload
//!
//! The watcher is rooted at the *directory* that contains the entry template.
//! A reload fires when:
//!
//! - the entry template itself is modified, created, or removed (covers
//!   editor atomic-save patterns that rename a temp file over the original),
//! - any other `.crepus` file in that subtree changes (so editing an
//!   `include`d component reloads the entry), or
//! - a `context.toml` next to the template is updated.
//!
//! Renderer parse errors render an in-window red error block instead of
//! panicking, so an in-progress edit cannot crash the dev window.
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use gpui::{
    div, rems, rgb, AsyncApp, Context, Entity, IntoElement, ParentElement, Render, Styled,
    WeakEntity, Window,
};

use crate::ast::Node;
use crate::context::TemplateContext;
use crate::parser::parse_template;
use crate::renderer::render_nodes;
use crate::watcher::watch_file;

/// State model for the hot-reload view.
pub struct HotReloadState {
    pub path: PathBuf,
    pub template: Result<Vec<Node>, String>,
    pub context: TemplateContext,
    pub changed: Arc<Mutex<bool>>,
}

impl HotReloadState {
    pub fn new(path: PathBuf, mut context: TemplateContext, cx: &mut Context<Self>) -> Self {
        // Set base_dir so `include` directives inside the template can resolve relative paths.
        if context.base_dir.is_none() {
            context.base_dir = path.parent().map(|p| p.to_path_buf());
        }

        let template = load_template(&path);
        let changed = Arc::new(Mutex::new(false));

        // Start file watcher
        watch_file(path.clone(), changed.clone());

        // Poll for changes and trigger re-renders
        let changed_poll = changed.clone();
        cx.spawn(
            async move |this: WeakEntity<HotReloadState>, cx: &mut AsyncApp| loop {
                let is_changed = {
                    changed_poll
                        .lock()
                        .map(|mut g| {
                            let v = *g;
                            *g = false;
                            v
                        })
                        .unwrap_or(false)
                };

                if is_changed {
                    this.update(cx, |state, cx| {
                        state.template = load_template(&state.path);
                        cx.notify();
                    })
                    .ok();
                }

                cx.background_executor()
                    .timer(std::time::Duration::from_millis(100))
                    .await;
            },
        )
        .detach();

        Self {
            path,
            template,
            context,
            changed,
        }
    }

    pub fn reload(&mut self) {
        self.template = load_template(&self.path);
    }
}

fn load_template(path: &PathBuf) -> Result<Vec<Node>, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Could not read {:?}: {}", path, e))?;
    parse_template(&content)
}

/// A GPUI view that renders a hot-reloaded template.
pub struct HotReloadView {
    pub state: Entity<HotReloadState>,
}

impl HotReloadView {
    pub fn new(state: Entity<HotReloadState>) -> Self {
        Self { state }
    }
}

impl Render for HotReloadView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        match &state.template {
            Ok(nodes) => render_nodes(nodes, &state.context),
            Err(err) => {
                let msg = format!("Parse error:\n\n{}", err);
                div()
                    .w_full()
                    .h_full()
                    .bg(rgb(0x1a0000))
                    .p(rems(2.))
                    .flex()
                    .flex_col()
                    .gap(rems(1.))
                    .child(
                        div()
                            .text_color(rgb(0xff4444))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_size(rems(1.125))
                            .child("Template Error"),
                    )
                    .child(
                        div()
                            .text_color(rgb(0xff8888))
                            .text_size(rems(0.875))
                            .child(msg),
                    )
                    .into_any_element()
            }
        }
    }
}
