use crepuscularity_gpui::{bounds, gpui_window_options, point, px, size};

#[test]
fn gpui_window_options_preserves_safe_default_titlebar() {
    let options = gpui_window_options(
        "crepuscularity.test",
        "Test",
        Some(gpui::WindowBounds::Windowed(bounds(
            point(px(10.), px(20.)),
            size(px(300.), px(200.)),
        ))),
        Some(size(px(200.), px(100.))),
    );

    assert_eq!(options.app_id.as_deref(), Some("crepuscularity.test"));
    assert_eq!(options.window_min_size, Some(size(px(200.), px(100.))));
    assert!(options.window_bounds.is_some());
    assert!(options.titlebar.is_some());
}
