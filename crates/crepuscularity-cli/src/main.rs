//! crepus — the Crepuscularity CLI

mod apple_project;
mod aurora;
#[cfg(feature = "benchmark")]
mod benchmark;
#[cfg(feature = "benchmark")]
mod benchmark_tui;
mod build_options;
#[cfg(feature = "desktop")]
mod builder;
mod cli;
mod components;
mod crepus_toml;
#[cfg(feature = "desktop")]
mod dev;
mod dispatch;
mod docs_generator;
mod embedded;
mod error;
#[cfg(feature = "desktop")]
pub mod events;
mod flutter;
#[cfg(feature = "desktop")]
mod hud;
mod inspect;
mod ios;
mod moonshine;
mod native;
mod new;
mod plugins;
mod preview;
mod render;
mod scaffold;
mod target_build;
mod tauri;
mod tui;
pub mod ui;
mod wasm_bundle;
mod web;
mod web_docs_hook;
mod web_islands;
mod web_serve;
mod webext;

fn init_tracing() {
    let mut filter = tracing_subscriber::EnvFilter::from_default_env();
    match "crepuscularity=info".parse() {
        Ok(dir) => {
            filter = filter.add_directive(dir);
        }
        Err(e) => {
            eprintln!("Warning: Failed to parse default tracing directive: {}", e);
        }
    }

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

fn main() {
    init_tracing();
    let cli = cli::parse();
    if let Err(err) = dispatch::run(cli) {
        ui::report_error(&err);
    }
}
