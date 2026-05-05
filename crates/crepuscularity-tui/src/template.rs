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

impl Template {
    pub fn from_source(source: impl Into<String>) -> Self {
        Self {
            path: PathBuf::new(),
            source: source.into(),
            ctx: TemplateContext::new(),
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

    pub fn draw(&self, frame: &mut Frame, area: Rect) -> Result<(), String> {
        render_template(&self.source, &self.ctx, frame, area)
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
