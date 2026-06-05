/// GPUI backend for Crepuscularity.
///
/// Re-exports the upstream GPUI API plus Crepuscularity's GPUI-oriented `view!` macro and build
/// helpers so existing GPUI consumers can migrate incrementally.
pub use crepuscularity_core::build;
pub use crepuscularity_macros::view;
pub use gpui::Corner as Anchor;
pub use gpui::*;
pub use pollster::block_on;

#[cfg(feature = "symbols")]
pub use gpui_symbols::Icon;

pub mod prelude {
    pub use crepuscularity_macros::view;
    pub use gpui::prelude::*;
    pub use gpui::{
        black, div, px, relative, rems, rgb, white, App, AppContext, Application, Context, Entity,
        FontWeight, IntoElement, Render, SharedString, Window, WindowOptions,
    };
    #[cfg(feature = "symbols")]
    pub use gpui_symbols::Icon;
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
