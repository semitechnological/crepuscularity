/// GPUI backend for Crepuscularity.
///
/// Re-exports the upstream GPUI API plus Crepuscularity's GPUI-oriented `view!` macro and build
/// helpers so existing GPUI consumers can migrate incrementally.
pub use crepuscularity_core::build;
pub use crepuscularity_macros::view;
pub use gpui::*;
pub use pollster::block_on;

pub const GPUI_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

#[derive(
    Clone, Debug, PartialEq, serde::Deserialize, gpui::private::schemars::JsonSchema, gpui::Action,
)]
#[action(namespace = zed)]
pub struct Unbind(pub SharedString);

#[cfg(feature = "symbols")]
pub use gpui_symbols::Icon;

pub mod prelude {
    pub use crate::{
        ZedAnchoredCompat, ZedForegroundExecutorCompat, ZedGpuiCompat, ZedStyledTextCompat,
        ZedSvgCompat, ZedWindowCompat,
    };
    pub use crepuscularity_macros::view;
    pub use gpui::prelude::*;
    pub use gpui::{
        black, div, px, relative, rems, rgb, white, App, AppContext, Application, Context, Entity,
        FontWeight, IntoElement, Render, SharedString, Window, WindowOptions,
    };
    #[cfg(feature = "symbols")]
    pub use gpui_symbols::Icon;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Anchor {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    LeftCenter,
    RightCenter,
}

impl From<Anchor> for Corner {
    fn from(anchor: Anchor) -> Self {
        match anchor {
            Anchor::TopLeft | Anchor::TopCenter | Anchor::LeftCenter => Corner::TopLeft,
            Anchor::TopRight | Anchor::RightCenter => Corner::TopRight,
            Anchor::BottomLeft => Corner::BottomLeft,
            Anchor::BottomCenter | Anchor::BottomRight => Corner::BottomRight,
        }
    }
}

pub trait ZedAnchoredCompat: Sized {
    fn anchor(self, anchor: Anchor) -> Self;
}

impl ZedAnchoredCompat for Anchored {
    fn anchor(self, anchor: Anchor) -> Self {
        Anchored::anchor(self, anchor.into())
    }
}

pub trait ZedSvgCompat: Sized {
    fn external_path(self, path: impl Into<SharedString>) -> Self;
}

pub trait ZedWindowCompat {
    fn pixel_snap_bounds(&self, bounds: Bounds<Pixels>) -> Bounds<Pixels>;
    fn pixel_snap_point(&self, point: Point<Pixels>) -> Point<Pixels>;
    fn input_latency_snapshot(&self) -> InputLatencySnapshot;
}

pub trait ZedForegroundExecutorCompat {
    fn block_on<F>(&self, future: F) -> F::Output
    where
        F: std::future::Future;
}

impl ZedForegroundExecutorCompat for ForegroundExecutor {
    fn block_on<F>(&self, future: F) -> F::Output
    where
        F: std::future::Future,
    {
        pollster::block_on(future)
    }
}

impl ZedWindowCompat for Window {
    fn pixel_snap_bounds(&self, bounds: Bounds<Pixels>) -> Bounds<Pixels> {
        bounds
    }

    fn pixel_snap_point(&self, point: Point<Pixels>) -> Point<Pixels> {
        point
    }

    fn input_latency_snapshot(&self) -> InputLatencySnapshot {
        InputLatencySnapshot::default()
    }
}

pub struct InputLatencySnapshot {
    pub latency_histogram: hdrhistogram::Histogram<u64>,
    pub events_per_frame_histogram: hdrhistogram::Histogram<u64>,
    pub mid_draw_events_dropped: u64,
}

impl Default for InputLatencySnapshot {
    fn default() -> Self {
        Self {
            latency_histogram: hdrhistogram::Histogram::new(3)
                .expect("valid input latency histogram precision"),
            events_per_frame_histogram: hdrhistogram::Histogram::new(3)
                .expect("valid input event histogram precision"),
            mid_draw_events_dropped: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowButton {
    Minimize,
    Maximize,
    Close,
}

pub const MAX_BUTTONS_PER_SIDE: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowButtonLayout {
    pub left: [Option<WindowButton>; MAX_BUTTONS_PER_SIDE],
    pub right: [Option<WindowButton>; MAX_BUTTONS_PER_SIDE],
}

impl WindowButtonLayout {
    pub fn linux_default() -> Self {
        Self {
            left: [None; MAX_BUTTONS_PER_SIDE],
            right: [
                Some(WindowButton::Minimize),
                Some(WindowButton::Maximize),
                Some(WindowButton::Close),
            ],
        }
    }

    pub fn parse(layout_string: &str) -> Result<Self> {
        let layout_string = layout_string.trim();
        if layout_string.is_empty() {
            return Ok(Self::linux_default());
        }

        let (left, right) = layout_string
            .split_once(':')
            .map_or(("", layout_string), |(left, right)| (left, right));

        Ok(Self {
            left: parse_window_button_side(left)?,
            right: parse_window_button_side(right)?,
        })
    }
}

fn parse_window_button_side(side: &str) -> Result<[Option<WindowButton>; MAX_BUTTONS_PER_SIDE]> {
    let mut buttons = [None; MAX_BUTTONS_PER_SIDE];
    let mut index = 0;

    for raw_button in side.split(',') {
        let button = raw_button.trim();
        if button.is_empty() || button == "appmenu" {
            continue;
        }
        if index >= MAX_BUTTONS_PER_SIDE {
            return Err(gpui::private::anyhow::anyhow!(
                "too many window buttons in layout side: {side}"
            ));
        }
        buttons[index] = Some(match button {
            "minimize" => WindowButton::Minimize,
            "maximize" => WindowButton::Maximize,
            "close" => WindowButton::Close,
            other => {
                return Err(gpui::private::anyhow::anyhow!(
                    "unknown window button: {other}"
                ))
            }
        });
        index += 1;
    }

    Ok(buttons)
}

impl ZedSvgCompat for Svg {
    fn external_path(self, path: impl Into<SharedString>) -> Self {
        self.path(path)
    }
}

pub trait ZedStyledTextCompat: Sized {
    fn with_font_family_overrides(
        self,
        font_family_overrides: impl IntoIterator<Item = (std::ops::Range<usize>, SharedString)>,
    ) -> Self;
}

impl ZedStyledTextCompat for StyledText {
    fn with_font_family_overrides(
        self,
        font_family_overrides: impl IntoIterator<Item = (std::ops::Range<usize>, SharedString)>,
    ) -> Self {
        drop(font_family_overrides);
        self
    }
}

pub trait ZedGpuiCompat: Sized {
    fn flex_grow_1(self) -> Self;
    fn flex_shrink_1(self) -> Self;
    fn focus_visible(self, style: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self;
    fn text_ellipsis_start(self) -> Self;
}

impl<T> ZedGpuiCompat for T
where
    T: Styled,
{
    fn flex_grow_1(self) -> Self {
        self.flex_grow()
    }

    fn flex_shrink_1(self) -> Self {
        self.flex_shrink()
    }

    fn focus_visible(self, style: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self {
        drop(style);
        self
    }

    fn text_ellipsis_start(self) -> Self {
        self.text_ellipsis()
    }
}

pub trait TaskExt<T, E> {
    fn detach_and_log_err(self, cx: &App);
    fn detach_and_log_err_with_backtrace(self, cx: &App);
}

impl<T, E> TaskExt<T, E> for Task<Result<T, E>>
where
    T: 'static + Send,
    E: 'static + std::fmt::Display + std::fmt::Debug + Send,
{
    fn detach_and_log_err(self, cx: &App) {
        cx.background_spawn(async move {
            if let Err(error) = self.await {
                log::error!("{error}");
            }
        })
        .detach();
    }

    fn detach_and_log_err_with_backtrace(self, cx: &App) {
        cx.background_spawn(async move {
            if let Err(error) = self.await {
                log::error!("{error:?}");
            }
        })
        .detach();
    }
}

#[cfg(test)]
mod tests {
    use crate::{WindowButton, WindowButtonLayout};

    #[test]
    fn parses_linux_window_button_layout() {
        let layout = WindowButtonLayout::parse(":minimize,maximize,close").unwrap();

        assert_eq!(layout, WindowButtonLayout::linux_default());
    }

    #[test]
    fn parses_left_and_right_window_button_layout() {
        let layout = WindowButtonLayout::parse("close:minimize,maximize").unwrap();

        assert_eq!(
            layout,
            WindowButtonLayout {
                left: [Some(WindowButton::Close), None, None],
                right: [
                    Some(WindowButton::Minimize),
                    Some(WindowButton::Maximize),
                    None,
                ],
            }
        );
    }

    #[test]
    fn rejects_unknown_window_button_layout_tokens() {
        let error = WindowButtonLayout::parse(":minimize,fullscreen").unwrap_err();

        assert!(error.to_string().contains("unknown window button"));
    }
}
