/// DevHUD — a small floating status window rendered via GPUI.
///
/// `HudState` lives in an `Arc<Mutex<_>>` shared between the background build
/// thread and the GPUI entity. The entity polls every 100 ms and calls
/// `cx.notify()` whenever the state has changed.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use gpui::{
    div, prelude::*, px, rems, rgb, size, AsyncApp, Context, IntoElement, ParentElement, Render,
    Styled, WeakEntity, Window,
};

// ── Shared data types ──────────────────────────────────────────────────────

#[derive(Clone, Debug, Default)]
pub struct BuildError {
    pub level: String,
    pub message: String,
    pub file: String,
    pub line: u32,
    pub rendered: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub enum DevStatus {
    #[default]
    Starting,
    Building,
    Running {
        elapsed_ms: u64,
    },
    Failed {
        errors: Vec<BuildError>,
        count: usize,
    },
    Exited {
        code: Option<i32>,
    },
}

#[derive(Clone, Debug)]
pub struct HudState {
    pub project_name: String,
    pub status: DevStatus,
}

impl HudState {
    pub fn new(project_name: String) -> Self {
        Self {
            project_name,
            status: DevStatus::Starting,
        }
    }
}

// ── GPUI entity ────────────────────────────────────────────────────────────

const SPINNER: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠏"];

pub struct DevHud {
    shared: Arc<Mutex<HudState>>,
    /// Local rendering snapshot, updated each poll.
    project_name: String,
    status_text: String,
    status_color: u32,
    frame: usize,
}

impl DevHud {
    pub fn new(
        shared: Arc<Mutex<HudState>>,
        shutdown: Arc<AtomicBool>,
        cx: &mut Context<Self>,
    ) -> Self {
        let (project_name, status_text, status_color) = snapshot(&shared, 0);

        let shared_poll = shared.clone();
        cx.spawn(async move |this: WeakEntity<DevHud>, cx: &mut AsyncApp| {
            let mut frame = 0usize;
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(100))
                    .await;

                if shutdown.load(Ordering::Relaxed) {
                    break;
                }

                frame = frame.wrapping_add(1);
                let (name, text, color) = snapshot(&shared_poll, frame);

                this.update(cx, |hud, cx| {
                    hud.frame = frame;
                    hud.project_name = name;
                    hud.status_text = text;
                    hud.status_color = color;
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();

        Self {
            shared,
            project_name,
            status_text,
            status_color,
            frame: 0,
        }
    }
}

fn snapshot(shared: &Arc<Mutex<HudState>>, frame: usize) -> (String, String, u32) {
    let state = shared.lock().unwrap();
    let spinner = SPINNER[frame % SPINNER.len()];
    let (text, color) = match &state.status {
        DevStatus::Starting => (format!("{spinner} starting…"), 0x888888u32),
        DevStatus::Building => (format!("{spinner} building…"), 0xfbbf24u32),
        DevStatus::Running { elapsed_ms } => (format!("✓ running  ({elapsed_ms} ms)"), 0x4ade80u32),
        DevStatus::Failed { count, errors } => {
            let loc = errors
                .first()
                .and_then(|e| {
                    if !e.file.is_empty() {
                        Some(format!("  {}", e.file.trim_start_matches("src/")))
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            (
                format!(
                    "✗ {count} error{}  {loc}",
                    if *count == 1 { "" } else { "s" }
                ),
                0xf87171u32,
            )
        }
        DevStatus::Exited { code } => (
            format!(
                "⚑ exited ({})",
                code.map(|c| c.to_string()).unwrap_or_else(|| "?".into())
            ),
            0xa78bfa,
        ),
    };
    (state.project_name.clone(), text, color)
}

impl Render for DevHud {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let name = self.project_name.clone();
        let text = self.status_text.clone();
        let color = self.status_color;

        div()
            .w(px(400.))
            .h(px(72.))
            .bg(rgb(0x0f172a))
            .flex()
            .flex_col()
            .justify_center()
            .px(rems(1.25))
            .gap(px(3.))
            // Project name row
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .child(
                        div()
                            .text_color(rgb(0x818cf8))
                            .text_size(rems(0.7))
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("crepus"),
                    )
                    .child(
                        div()
                            .text_color(rgb(0x1e293b))
                            .text_size(rems(0.7))
                            .child("·"),
                    )
                    .child(
                        div()
                            .text_color(rgb(0xcbd5e1))
                            .text_size(rems(0.7))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(name),
                    ),
            )
            // Status row
            .child(
                div()
                    .text_color(rgb(color))
                    .text_size(rems(0.8))
                    .child(text),
            )
    }
}

// ── Window bootstrap ───────────────────────────────────────────────────────

pub fn open_hud_window(
    shared: Arc<Mutex<HudState>>,
    shutdown: Arc<AtomicBool>,
    cx: &mut gpui::App,
) {
    use gpui::{bounds, point, WindowBackgroundAppearance, WindowKind, WindowOptions};

    let window_options = WindowOptions {
        window_bounds: Some(gpui::WindowBounds::Windowed(bounds(
            point(px(20.), px(20.)),
            size(px(400.), px(72.)),
        ))),
        titlebar: None,
        focus: false,
        show: true,
        kind: WindowKind::Normal,
        is_movable: true,
        is_resizable: false,
        is_minimizable: false,
        display_id: None,
        window_background: WindowBackgroundAppearance::Opaque,
        app_id: Some("crepuscularity.crepus.hud".to_string()),
        window_min_size: Some(size(px(400.), px(72.))),
        window_decorations: None,
        tabbing_identifier: None,
    };

    cx.open_window(window_options, move |_window, cx| {
        cx.new(|cx| DevHud::new(shared.clone(), shutdown.clone(), cx))
    })
    .unwrap();
}
