use std::path::{Path, PathBuf};

use ratatui::backend::Backend;
use ratatui::layout::Rect;
use ratatui::{CompletedFrame, Frame, Terminal};

use crate::{render_template, TemplateContext};

#[derive(Clone)]
pub struct Template {
    path: PathBuf,
    source: String,
    ctx: TemplateContext,
}

#[derive(Clone, Debug)]
pub struct ElementRef {
    pub id: &'static str,
    pub content: String,
}

impl ElementRef {
    pub fn new(id: &'static str) -> Self {
        Self {
            id,
            content: String::new(),
        }
    }

    pub fn set_content(&mut self, content: impl Into<String>) -> &mut Self {
        self.content = content.into();
        self
    }

    pub fn content(&mut self, content: impl Into<String>) -> &mut Self {
        self.set_content(content)
    }

    pub fn text(&mut self, content: impl Into<String>) -> &mut Self {
        self.set_content(content)
    }

    pub fn val(&mut self, content: impl Into<String>) -> &mut Self {
        self.set_content(content)
    }

    pub fn clear(&mut self) -> &mut Self {
        self.content.clear();
        self
    }
}

impl Template {
    pub fn from_source(source: impl Into<String>) -> Self {
        Self {
            path: PathBuf::new(),
            source: source.into(),
            ctx: TemplateContext::new(),
        }
    }

    pub fn from_source_with_path(source: impl Into<String>, path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        let mut ctx = TemplateContext::new();
        ctx.base_dir = path.parent().map(Path::to_path_buf);
        Self {
            path,
            source: source.into(),
            ctx,
        }
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        let source = std::fs::read_to_string(&path)
            .map_err(|e| format!("template error: {:?}: {}", path, e))?;
        let mut ctx = TemplateContext::new();
        ctx.base_dir = path.parent().map(Path::to_path_buf);
        Ok(Self { path, source, ctx })
    }

    pub fn set(
        &mut self,
        key: impl Into<String>,
        value: impl Into<crate::TemplateValue>,
    ) -> &mut Self {
        self.ctx.set(key, value);
        self
    }

    pub fn with(mut self, key: impl Into<String>, value: impl Into<crate::TemplateValue>) -> Self {
        self.set(key, value);
        self
    }

    pub fn context(&self) -> &TemplateContext {
        &self.ctx
    }

    pub fn context_mut(&mut self) -> &mut TemplateContext {
        &mut self.ctx
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Replace the in-memory template source.
    ///
    /// Useful for live-reload pipelines that want to keep the same [`TemplateContext`]
    /// (variables, `base_dir`) but swap the rendered DSL.
    pub fn set_source(&mut self, source: impl Into<String>) -> &mut Self {
        self.source = source.into();
        self
    }

    /// Re-read the template source from [`Template::path`].
    ///
    /// Returns an error if this template was constructed without a path
    /// ([`Template::from_source`]) or if reading fails. On error the previous
    /// source is left intact so the next [`Template::draw`] still renders.
    pub fn reload(&mut self) -> Result<(), String> {
        if self.path.as_os_str().is_empty() {
            return Err("template has no path; reload requires `from_path`".to_string());
        }
        let source = std::fs::read_to_string(&self.path)
            .map_err(|e| format!("template error: {:?}: {}", self.path, e))?;
        self.source = source;
        Ok(())
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) -> Result<(), String> {
        render_template(&self.source, &self.ctx, frame, area).map_err(|e| e.to_string())
    }

    pub fn draw_full(&self, frame: &mut Frame) -> Result<(), String> {
        self.draw(frame, frame.area())
    }
}

pub fn template(path: impl AsRef<Path>) -> Result<Template, String> {
    Template::from_path(path)
}

pub fn draw<'a, B, F>(
    terminal: &'a mut Terminal<B>,
    path: impl AsRef<Path>,
    update: F,
) -> Result<CompletedFrame<'a>, String>
where
    B: Backend,
    F: FnOnce(&mut Template),
{
    let mut tpl = template(path)?;
    update(&mut tpl);
    let mut render_err = None;
    let frame = terminal
        .draw(|frame| {
            if let Err(err) = tpl.draw_full(frame) {
                render_err = Some(err);
            }
        })
        .map_err(|e| format!("terminal draw error: {e}"))?;
    if let Some(err) = render_err {
        return Err(err);
    }
    Ok(frame)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn element_ref_new_initialization() {
        let el = ElementRef::new("test-id");
        assert_eq!(el.id, "test-id");
        assert_eq!(el.content, "");
    }

    #[test]
    fn element_ref_set_content() {
        let mut el = ElementRef::new("test-id");
        el.set_content("Hello");
        assert_eq!(el.content, "Hello");
    }

    #[test]
    fn element_ref_content_alias() {
        let mut el = ElementRef::new("test-id");
        el.content("World");
        assert_eq!(el.content, "World");
    }

    #[test]
    fn element_ref_text_alias() {
        let mut el = ElementRef::new("test-id");
        el.text("TextContent");
        assert_eq!(el.content, "TextContent");
    }

    #[test]
    fn element_ref_val_alias() {
        let mut el = ElementRef::new("test-id");
        el.val("ValContent");
        assert_eq!(el.content, "ValContent");
    }

    #[test]
    fn element_ref_clear() {
        let mut el = ElementRef::new("test-id");
        el.set_content("Should be cleared");
        assert_eq!(el.content, "Should be cleared");

        el.clear();
        assert_eq!(el.content, "");
    }

    #[test]
    fn element_ref_chaining() {
        let mut el = ElementRef::new("test-id");
        el.set_content("A")
            .content("B")
            .text("C")
            .val("D")
            .clear()
            .text("Final");

        assert_eq!(el.content, "Final");
    }

    #[test]
    fn template_reload_empty_path_returns_error() {
        let mut tpl = Template::from_source("test source");
        let res = tpl.reload();
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            "template has no path; reload requires `from_path`"
        );
    }

    #[test]
    fn template_reload_read_failure_returns_error() {
        let non_existent_path = std::path::PathBuf::from("does_not_exist.crepus");
        let mut tpl = Template::from_source_with_path("old source", &non_existent_path);
        let res = tpl.reload();
        assert!(res.is_err());
        let err_msg = res.unwrap_err();
        assert!(err_msg.starts_with("template error:"));
        assert_eq!(tpl.source(), "old source");
    }
}
