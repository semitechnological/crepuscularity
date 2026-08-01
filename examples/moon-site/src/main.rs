//! Rust-first Moonshine site: every template below is inlined here as a raw
//! string literal, lowered to View IR by crepuscularity-native and emitted as
//! JSX. Edit a template, run `cargo run`, and the generated app changes — no
//! JavaScript is authored anywhere in this project.
//!
//! Drop TypeScript modules in `ts/` — they are copied into the built app by
//! `crepus moon build` and stay importable from the generated TSX.

use crepuscularity_native::{
    emit_moonshine_app, emit_moonshine_component, render_template_to_ir, ViewIr,
};
use std::fs;
use std::io;

/// The page — becomes `generated/app.tsx` (the Moonshine app entry).
const PAGE: &str = r###"div w-full min-h-screen bg-zinc-950 text-zinc-100 px-6 py-16 flex flex-col gap-10
 div flex flex-col gap-3
  div text-4xl font-bold tracking-tight
   "moon-site"
  div text-lg text-zinc-400 max-w-2xl
   "A Moonshine site whose templates live in Rust. This sentence is a raw string literal in src/main.rs."
 div flex flex-col gap-4
  div text-sm uppercase tracking-widest text-zinc-500
   "How it works"
  ol
   li
    "cargo run lowers each inlined template to View IR."
   li
    "The IR is emitted as real JSX, class tokens intact."
   li
    "crepus moon build runs vite and writes dist/."
 div flex flex-col gap-4
  div text-sm uppercase tracking-widest text-zinc-500
   "Links"
  a href="https://crates.io/crates/crepuscularity-cli" text-sky-400 underline
   "crepuscularity-cli on crates.io"
  a href="https://github.com/tschk/moonshine" text-sky-400 underline
   "moonshine on GitHub"
 hr
 div text-sm text-zinc-500
  "Generated from Rust. Add TypeScript under ts/ when you want it."
"###;

/// A reusable component — becomes `generated/components/FeatureCard.tsx`.
const FEATURE_CARD: &str = r###"div rounded-xl border border-zinc-800 bg-zinc-900 p-5 flex flex-col gap-2
 div text-base font-semibold text-zinc-100
  "Rust-first"
 div text-sm text-zinc-400
  "Templates are compiled from Rust source, so a broken template is a build failure rather than a blank page."
"###;

/// Lower one template, naming it in the panic so a broken template is obvious.
fn lower(name: &str, source: &str) -> ViewIr {
    render_template_to_ir(source, &Default::default())
        .unwrap_or_else(|e| panic!("template `{name}` failed to lower to View IR: {e}"))
}

fn main() -> io::Result<()> {
    let app_tsx = emit_moonshine_app(&lower("PAGE", PAGE));
    let card_tsx = emit_moonshine_component(&lower("FEATURE_CARD", FEATURE_CARD), "FeatureCard");

    fs::create_dir_all("generated/components")?;
    fs::write("generated/app.tsx", app_tsx)?;
    fs::write("generated/components/FeatureCard.tsx", card_tsx)?;

    println!("wrote generated/app.tsx");
    println!("wrote generated/components/FeatureCard.tsx");
    println!("next: crepus moon build");
    Ok(())
}
