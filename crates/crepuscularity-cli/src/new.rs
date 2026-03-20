/// `crepu new <name>` — scaffold a new GPUI application.

use std::fs;
use std::path::Path;

pub fn run(name: &str) {
    let dir = Path::new(name);

    if dir.exists() {
        eprintln!("Error: '{}' already exists", name);
        std::process::exit(1);
    }

    fs::create_dir_all(dir.join("src")).unwrap_or_else(|e| {
        eprintln!("Error creating directories: {e}");
        std::process::exit(1);
    });

    fs::write(dir.join("Cargo.toml"), cargo_toml(name)).unwrap();
    fs::write(dir.join("src").join("main.rs"), main_rs(name)).unwrap();
    fs::write(dir.join(".gitignore"), "/target\n").unwrap();

    eprintln!("\x1b[32m✓\x1b[0m Created \x1b[1m{name}\x1b[0m");
    eprintln!();
    eprintln!("  cd {name}");
    eprintln!("  crepu dev");
}

fn cargo_toml(name: &str) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{name}"
path = "src/main.rs"

[dependencies]
gpui = {{ version = "0.2", default-features = false, features = ["font-kit"] }}
crepuscularity = {{ git = "https://github.com/semitechnological/crepuscularity", branch = "runtime-dev" }}
"#
    )
}

fn main_rs(name: &str) -> String {
    let pascal = to_pascal_case(name);
    // Use r##"..."## so the inner r#"..."# delimiters don't close the outer string.
    // Inside format!(), {{ → { and }} → } in the output.
    format!(
        r##"use crepuscularity::prelude::*;
use gpui::{{App, Application, WindowOptions}};

struct {pascal}View {{
    count: i32,
}}

impl {pascal}View {{
    fn new(_cx: &mut Context<Self>) -> Self {{
        Self {{ count: 0 }}
    }}

    fn increment(&mut self, _: &gpui::ClickEvent, _: &mut gpui::Window, cx: &mut Context<Self>) {{
        self.count += 1;
        cx.notify();
    }}
}}

impl Render for {pascal}View {{
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {{
        let count = self.count;
        view! {{r#"
            div w-full h-full bg-zinc-950 text-white flex flex-col items-center justify-center gap-6
                div text-8xl font-bold leading-none
                    "{{count}}"
                button bg-white text-black font-semibold px-6 py-2 rounded-lg @click=increment
                    "increment"
        "#}}
    }}
}}

fn main() {{
    Application::new().run(|cx: &mut App| {{
        use gpui::prelude::*;
        cx.open_window(WindowOptions::default(), |_win, cx| {{
            cx.new({pascal}View::new)
        }})
        .unwrap();
    }});
}}
"##,
        pascal = pascal,
    )
}

fn to_pascal_case(s: &str) -> String {
    s.split(&['-', '_', ' '][..])
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}
