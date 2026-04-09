use crepuscularity_gpui::prelude::*;
use gpui::{App, Application, WindowOptions};

struct BenchView {
    count: i32,
}

impl BenchView {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self { count: 0 }
    }

    fn increment(&mut self, _: &gpui::ClickEvent, _: &mut gpui::Window, cx: &mut Context<Self>) {
        self.count += 1;
        cx.notify();
    }
}

impl Render for BenchView {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.count;
        view! {r#"
            div w-full h-full bg-zinc-950 text-white flex flex-col items-center justify-center gap-6
                div text-6xl font-bold
                    "{count}"
                button bg-white text-black font-semibold px-6 py-2 rounded-lg @click=increment
                    "increment"
        "#}
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        use gpui::prelude::*;
        cx.open_window(WindowOptions::default(), |_win, cx| cx.new(BenchView::new))
            .unwrap();
    });
}
