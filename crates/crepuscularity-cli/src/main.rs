//! crepus — the Crepuscularity CLI

#[cfg(feature = "aurora")]
mod aurora;
mod benchmark;
mod benchmark_tui;
mod build_options;
#[cfg(feature = "desktop")]
mod builder;
mod cli;
mod crepus_toml;
#[cfg(feature = "desktop")]
mod dev;
mod dispatch;
mod docs_generator;
mod embedded;
#[cfg(feature = "desktop")]
pub mod events;
#[cfg(feature = "desktop")]
mod hud;
mod ios;
mod mobile;
mod native;
mod new;
mod preview;
mod render;
mod target_build;
mod tui;
pub mod ui;
mod wasm_bundle;
mod web;
mod web_docs_hook;
mod web_islands;
mod web_serve;
mod webext;

fn main() {
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

    let cli = cli::parse();
    dispatch::run(cli);
}
