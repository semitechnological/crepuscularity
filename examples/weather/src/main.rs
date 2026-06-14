use crepuscularity_gpui::prelude::*;
use gpui::{actions, bounds, point, size, Application, ClickEvent};

actions!(weather, [FetchWeather]);

struct WeatherView {
    temp_c: i32,
    condition: String,
    suggestion: String,
    _city: String,
    is_loading: bool,
}

impl WeatherView {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            temp_c: 22,
            condition: "Sunny".to_string(),
            suggestion: "Grab some sunscreen".to_string(),
            _city: String::new(),
            is_loading: false,
        }
    }

    fn fetch_weather(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.is_loading = true;
        cx.notify();

        // Simulate a fetch with a short background task
        cx.spawn(async move |this, cx: &mut gpui::AsyncApp| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(800))
                .await;

            this.update(cx, |view, cx| {
                view.temp_c = 18;
                view.condition = "Cloudy".to_string();
                view.suggestion = "Bring a light jacket".to_string();
                view.is_loading = false;
                cx.notify();
            })
        })
        .detach();
    }
}

impl Render for WeatherView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let temp_c = self.temp_c;
        let condition = self.condition.clone();
        let suggestion = self.suggestion.clone();
        let is_loading = self.is_loading;

        view! {r#"
            div w-full h-full bg-black text-white flex justify-center items-center font-['Instrument_Sans']

                # Main content container
                div w-[640px] flex flex-col gap-2

                    # Temperature
                    div self-stretch
                        div text-9xl font-bold leading-none
                            "{temp_c}°"

                    # Condition
                    div flex items-center gap-6
                        div text-5xl font-bold
                            "{condition}"
                        div w-16 h-16 bg-zinc-300 rounded-full

                    # Suggestion
                    div self-stretch
                        div text-4xl font-bold text-zinc-400 leading-tight
                            "{suggestion}"

                    # Fetch button
                    div mt-16 flex gap-4 items-center
                        button bg-white text-black font-bold text-xl py-3 px-8 rounded-xl hover:bg-zinc-200 @click=fetch_weather
                            if {is_loading}
                                "Scanning..."
                            else
                                "Check Weather"
        "#}
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let window_options = gpui_window_options(
            "crepuscularity.weather",
            "Weather",
            Some(gpui::WindowBounds::Windowed(bounds(
                point(gpui::px(0.), gpui::px(0.)),
                size(gpui::px(800.), gpui::px(600.)),
            ))),
            None,
        );

        cx.open_window(window_options, |_window, cx| cx.new(WeatherView::new))
            .unwrap();
    });
}
