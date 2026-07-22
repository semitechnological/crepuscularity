use crepuscularity_lite::bridge::Bridge;
use crepuscularity_lite::integration::{apply_window_deferred, take_app_exit_request};
use gpui::div;
use gpui::{AppContext, Render, TestAppContext, WindowOptions};

struct DummyView;

impl Render for DummyView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
    }
}

#[gpui::test]
fn test_apply_window_deferred(cx: &mut TestAppContext) {
    let bridge = Bridge::default_arc();
    let _ = bridge.invoke_envelope(
        "window",
        "setTitle",
        &serde_json::json!({ "title": "New Title 1" }),
    );
    let _ = bridge.invoke_envelope(
        "window",
        "setTitle",
        &serde_json::json!({ "title": "New Title 2" }),
    );
    let _ = bridge.invoke_envelope(
        "window",
        "setContentSize",
        &serde_json::json!({ "width": 800.0, "height": 600.0 }),
    );
    let _ = bridge.invoke_envelope(
        "window",
        "setDocumentEdited",
        &serde_json::json!({ "edited": true }),
    );
    let _ = bridge.invoke_envelope(
        "window",
        "requestDecorations",
        &serde_json::json!({ "style": "client" }),
    );
    let _ = bridge.invoke_envelope(
        "window",
        "requestDecorations",
        &serde_json::json!({ "style": "server" }),
    );

    // Commands that don't panic on TestAppContext
    // minimize, zoom, startWindowMove cause panic because they are not implemented in gpui TestWindow

    cx.update(|cx| {
        let window = cx
            .open_window(WindowOptions::default(), |_, cx| cx.new(|_| DummyView))
            .unwrap();

        window
            .update(cx, |_, window, _| {
                // Apply deferred commands
                let latest_title = apply_window_deferred(&bridge, window);
                assert_eq!(latest_title, Some("New Title 2".to_string()));

                // Check that it's drained
                assert!(bridge.drain_host_commands().is_empty());

                // Calling it again on empty queue should return None
                assert_eq!(apply_window_deferred(&bridge, window), None);
            })
            .unwrap();
    });
}

#[test]
fn test_take_app_exit_request() {
    let bridge = Bridge::default_arc();
    assert_eq!(take_app_exit_request(&bridge), false);
    let _ = bridge.invoke_envelope("app", "exit", &serde_json::json!({}));
    assert_eq!(take_app_exit_request(&bridge), true);
    assert_eq!(take_app_exit_request(&bridge), false);
}
