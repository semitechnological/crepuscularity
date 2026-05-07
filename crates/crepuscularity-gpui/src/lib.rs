/// GPUI backend for Crepuscularity.
///
/// Re-exports the GPUI-oriented `view!` macro and prelude from the split backend crate.
pub use crepuscularity_core::build;
pub use crepuscularity_macros::view;

pub mod prelude {
    pub use crepuscularity_macros::view;
    pub use gpui::prelude::*;
    pub use gpui::{
        black, div, px, relative, rems, rgb, white, App, AppContext, Application, Context, Entity,
        FontWeight, IntoElement, Render, SharedString, Window, WindowOptions,
    };
}
