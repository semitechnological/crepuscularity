/// Crepuscularity — declarative GPUI UI with Tailwind-like syntax.
///
/// Re-exports the `view!` macro and GPUI's prelude so consumers only need
/// one `use` statement.
///
/// ```ignore
/// use crepuscularity::prelude::*;
///
/// impl Render for MyView {
///     fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
///         view! {r#"
///         div w-full h-full bg-zinc-900 flex items-center justify-center
///             span text-4xl font-bold text-white
///                 "{self.message}"
///         "#}
///     }
/// }
/// ```
pub use crepuscularity_macros::view;

pub mod prelude {
    pub use crepuscularity_macros::view;
    pub use gpui::prelude::*;
    pub use gpui::{
        App, AppContext, Context, Entity, FontWeight, IntoElement, Render, SharedString, Window,
        div, px, rems, rgb, black, white, relative,
    };
}
