use crepuscularity_gpui::prelude::*;
use gpui::{bounds, point, size, App, Application, WindowOptions};

struct TextFeaturesView;

impl TextFeaturesView {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for TextFeaturesView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        view! {r#"
            div w-full h-full bg-zinc-950 text-white flex justify-center items-start py-10 px-8 font-['Helvetica']

                div w-[1320px] flex flex-col gap-8

                    div flex flex-col gap-2
                        div text-5xl font-bold tracking-tight
                            "Text Features"
                        div text-sm text-zinc-400 leading-relaxed max-w-[720px]
                            "Visual verification app for forked GPUI text transforms and letter spacing. Case transforms now preserve byte length instead of restricting themselves to plain ASCII, so accented letters like É and À transform when the Unicode mapping stays one codepoint and the same UTF-8 width."

                    div grid grid-cols-2 gap-6

                        div bg-zinc-900 rounded-2xl border border-zinc-800 p-6 flex flex-col gap-4
                            div text-sm font-semibold uppercase tracking-widest text-zinc-400
                                "Tracking Scale"
                            div flex flex-col gap-3
                                div
                                    div text-xs text-zinc-500
                                        "tracking-tighter"
                                    div text-4xl tracking-tighter
                                        "Spacing Check"
                                div
                                    div text-xs text-zinc-500
                                        "tracking-normal"
                                    div text-4xl tracking-normal
                                        "Spacing Check"
                                div
                                    div text-xs text-zinc-500
                                        "tracking-wide"
                                    div text-4xl tracking-wide
                                        "Spacing Check"
                                div
                                    div text-xs text-zinc-500
                                        "tracking-widest"
                                    div text-4xl tracking-widest
                                        "Spacing Check"
                                div
                                    div text-xs text-zinc-500
                                        "tracking-[3px]"
                                    div text-4xl tracking-[3px]
                                        "Spacing Check"
                                div text-xs text-zinc-500 leading-relaxed
                                    "Named tracking classes use larger fixed pixel values now so the difference is obvious in the forked GPUI demo."

                        div bg-zinc-900 rounded-2xl border border-zinc-800 p-6 flex flex-col gap-4
                            div text-sm font-semibold uppercase tracking-widest text-zinc-400
                                "Case Transforms"
                            div flex flex-col gap-3
                                div
                                    div text-xs text-zinc-500
                                        "uppercase"
                                    div text-3xl uppercase
                                        "hello world déjà vu à la carte"
                                div
                                    div text-xs text-zinc-500
                                        "lowercase"
                                    div text-3xl lowercase
                                        "HELLO WORLD DÉJÀ VU À LA CARTE"
                                div
                                    div text-xs text-zinc-500
                                        "capitalize"
                                    div text-3xl capitalize
                                        "hello world déjà vu à la carte 123abc"
                                div
                                    div text-xs text-zinc-500
                                        "normal-case"
                                    div text-3xl normal-case
                                        "MiXeD Case stays as typed"

                    div grid grid-cols-2 gap-6

                        div bg-black rounded-2xl border border-zinc-800 p-6 flex flex-col gap-4
                            div text-sm font-semibold uppercase tracking-widest text-zinc-400
                                "Combined Styles"
                            div flex flex-col gap-4
                                div text-6xl font-black uppercase tracking-tight leading-none
                                    "signal / noise"
                                div text-5xl font-semibold capitalize tracking-wide leading-tight
                                    "kinetic type demo"
                                div text-4xl lowercase tracking-[2px] text-zinc-300
                                    "WIDE LETTERS WITH SOFT TONE"

                        div bg-black rounded-2xl border border-zinc-800 p-6 flex flex-col gap-4
                            div text-sm font-semibold uppercase tracking-widest text-zinc-400
                                "Transform Rules"
                            div text-base text-zinc-300 leading-relaxed
                                "A character is transformed only when its Unicode case-mapped form stays one character wide and preserves the same UTF-8 byte length. That keeps selection, wrapping, and highlight offsets stable."
                            div text-2xl capitalize tracking-normal
                                "naïve façade déjà vu coöperate à la carte"
                            div text-xs text-zinc-500
                                "Expected: déjà becomes Déjà and À/É can change case, while expanding mappings like ß are left alone."
        "#}
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let window_options = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(bounds(
                point(gpui::px(80.), gpui::px(80.)),
                size(gpui::px(1600.), gpui::px(1120.)),
            ))),
            titlebar: None,
            focus: true,
            show: true,
            kind: gpui::WindowKind::Normal,
            is_movable: true,
            is_resizable: true,
            is_minimizable: true,
            display_id: None,
            window_background: gpui::WindowBackgroundAppearance::Opaque,
            app_id: Some("crepuscularity.text-features".to_string()),
            window_min_size: Some(size(gpui::px(1320.), gpui::px(980.))),
            window_decorations: None,
            tabbing_identifier: None,
        };

        cx.open_window(window_options, |_window, cx| cx.new(TextFeaturesView::new))
            .unwrap();
    });
}
