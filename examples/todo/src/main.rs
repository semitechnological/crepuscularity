use crepuscularity_gpui::prelude::*;
use gpui::{bounds, point, px, size, App, Application, ClickEvent, WindowOptions};

const DEMO_ITEMS: &[&str] = &[
    "Buy groceries",
    "Write documentation",
    "Review pull requests",
    "Fix that weird bug",
    "Update dependencies",
    "Write more tests",
    "Deploy to staging",
    "Coffee break",
    "Read the RFC",
    "Refactor auth module",
];

struct TodoItem {
    text: String,
    done: bool,
}

struct TodoView {
    items: Vec<TodoItem>,
    next_id: usize,
    demo_idx: usize,
}

impl TodoView {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            items: vec![
                TodoItem {
                    text: "Buy groceries".into(),
                    done: false,
                },
                TodoItem {
                    text: "Write documentation".into(),
                    done: true,
                },
                TodoItem {
                    text: "Review pull requests".into(),
                    done: false,
                },
            ],
            next_id: 3,
            demo_idx: 3,
        }
    }

    fn add_todo(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let text = DEMO_ITEMS[self.demo_idx % DEMO_ITEMS.len()];
        self.demo_idx += 1;
        self.items.push(TodoItem {
            text: text.into(),
            done: false,
        });
        self.next_id += 1;
        cx.notify();
    }

    fn check_first(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(item) = self.items.iter_mut().find(|i| !i.done) {
            item.done = true;
            cx.notify();
        }
    }

    fn clear_done(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.items.retain(|i| !i.done);
        cx.notify();
    }
}

impl Render for TodoView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let total = self.items.len();
        let done_count = self.items.iter().filter(|i| i.done).count();
        let pending = total - done_count;
        let has_done = done_count > 0;
        let has_pending = pending > 0;

        // Build the item list using GPUI's div builder.
        // Per-item styling (strikethrough, opacity) requires building elements
        // dynamically — the view! macro is for static layout skeletons.
        let item_list = div()
            .flex()
            .flex_col()
            .gap_0p5()
            .children(self.items.iter().map(|item| {
                let done = item.done;
                let text = item.text.clone();
                let indicator = if done {
                    div()
                        .w_4()
                        .h_4()
                        .rounded_full()
                        .bg(rgb(0x059669)) // emerald-600
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_shrink_0()
                        .child(
                            div()
                                .w_2()
                                .h_2()
                                .rounded_full()
                                .bg(white()),
                        )
                } else {
                    div()
                        .w_4()
                        .h_4()
                        .rounded_full()
                        .border_1()
                        .border_color(rgb(0x52525b)) // zinc-600
                        .flex_shrink_0()
                };
                let label = if done {
                    div()
                        .text_sm()
                        .text_color(rgb(0x71717a)) // zinc-500
                        .line_through()
                        .child(text)
                } else {
                    div()
                        .text_sm()
                        .text_color(rgb(0xf4f4f5)) // zinc-100
                        .child(text)
                };
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .py_2()
                    .border_b_1()
                    .border_color(rgb(0x27272a)) // zinc-800
                    .child(indicator)
                    .child(label)
            }));

        // Use view! for the outer shell — static layout with interpolated scalars.
        view! {r#"
            div w-full h-full bg-zinc-950 text-white flex flex-col items-center pt-16

                div w-[480px] flex flex-col gap-6

                    div flex flex-col gap-1
                        div text-4xl font-bold text-white
                            "todo"
                        div text-sm text-zinc-500
                            "{pending} remaining · {done_count} done · {total} total"

                    div flex flex-row gap-3

                        button bg-blue-600 text-white font-medium text-sm px-4 py-2 rounded-lg @click=add_todo
                            "Add item"

                        if {has_pending}
                            button bg-zinc-800 text-zinc-300 font-medium text-sm px-4 py-2 rounded-lg @click=check_first
                                "Check first"

                        if {has_done}
                            button bg-zinc-800 text-rose-400 font-medium text-sm px-4 py-2 rounded-lg @click=clear_done
                                "Clear done"

                    {item_list}
        "#}
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let opts = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(bounds(
                point(px(100.), px(100.)),
                size(px(640.), px(700.)),
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
            app_id: Some("crepuscularity.todo".to_string()),
            window_min_size: None,
            window_decorations: None,
            tabbing_identifier: None,
        };

        cx.open_window(opts, |_window, cx| cx.new(TodoView::new))
            .unwrap();
    });
}
