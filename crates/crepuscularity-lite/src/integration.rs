//! Glue between the bridge / V8 host and **GPUI** on the UI thread.
//!
//! GPUI does not expose a global “hook this library into all events” API. You call these helpers from
//! your own `Render` handlers, `on_click` closures, or [`gpui::actions`] dispatch—wherever you already
//! have `&mut Window` and/or `&mut gpui::Context`.

use gpui::{px, size, Window, WindowDecorations};

use crate::bridge::Bridge;
use crate::host_queue::{DeferredWindowDecorations, HostDeferred};

/// Apply deferred window operations queued by native plugins (e.g. [`WindowPlugin`](crate::plugins::WindowPlugin)).
/// Call this on the **UI thread** after [`V8Host::eval`](crate::V8Host) (or any work that may have invoked `Crepus.invoke`).
pub fn apply_window_deferred(bridge: &Bridge, window: &mut Window) -> Option<String> {
    let mut latest_title = None;
    for cmd in bridge.drain_host_commands() {
        match cmd {
            HostDeferred::SetWindowTitle(t) => {
                window.set_window_title(&t);
                latest_title = Some(t);
            }
            HostDeferred::SetContentSize { width, height } => {
                window.resize(size(px(width), px(height)));
            }
            HostDeferred::ToggleFullscreen => window.toggle_fullscreen(),
            HostDeferred::Minimize => window.minimize_window(),
            HostDeferred::Zoom => window.zoom_window(),
            HostDeferred::ShowCharacterPalette => window.show_character_palette(),
            HostDeferred::SetWindowEdited(edited) => window.set_window_edited(edited),
            HostDeferred::StartWindowMove => window.start_window_move(),
            HostDeferred::RefreshWindow => window.refresh(),
            HostDeferred::RequestDecorations(style) => window.request_decorations(match style {
                DeferredWindowDecorations::Server => WindowDecorations::Server,
                DeferredWindowDecorations::Client => WindowDecorations::Client,
            }),
        }
    }
    latest_title
}

/// Returns `true` if the guest requested application exit via the **`app`** plugin (`exit` method).
/// When `true`, the flag is cleared—call [`gpui::App::quit`](gpui::App::quit) (or `cx.quit()` on a context) if you want to honor it.
pub fn take_app_exit_request(bridge: &Bridge) -> bool {
    if bridge.quit_requested() {
        bridge.clear_quit_flag();
        true
    } else {
        false
    }
}
