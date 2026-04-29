//! Deferred work that must run on the GPUI thread with a live window (e.g. native title APIs).

use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum HostDeferred {
    SetWindowTitle(String),
    /// Content size in **logical pixels** (GPUI `px`).
    SetContentSize {
        width: f32,
        height: f32,
    },
    ToggleFullscreen,
    Minimize,
    Zoom,
    /// [`gpui::Window::show_character_palette`].
    ShowCharacterPalette,
    /// macOS “document edited” dot; maps to [`gpui::Window::set_window_edited`].
    SetWindowEdited(bool),
    /// [`gpui::Window::start_window_move`].
    StartWindowMove,
    /// [`gpui::Window::refresh`].
    RefreshWindow,
    /// [`gpui::Window::request_decorations`].
    RequestDecorations(DeferredWindowDecorations),
}

/// Decoration style for [`HostDeferred::RequestDecorations`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeferredWindowDecorations {
    Server,
    Client,
}

pub struct HostCommandQueue {
    inner: Mutex<Vec<HostDeferred>>,
}

impl HostCommandQueue {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(Vec::new()),
        })
    }

    pub fn push(&self, cmd: HostDeferred) {
        self.inner
            .lock()
            .expect("HostCommandQueue mutex poisoned")
            .push(cmd);
    }

    pub fn drain(&self) -> Vec<HostDeferred> {
        let mut g = self.inner.lock().expect("HostCommandQueue mutex poisoned");
        std::mem::take(&mut *g)
    }
}
