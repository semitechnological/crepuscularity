//! Rust-first Moonshine site. Templates come from three places, all lowered to
//! the same View IR and emitted as TSX — no JavaScript is authored here.
//!
//!   * inline in this file, as a raw string literal
//!   * `templates/page.crepus`, the indentation-based syntax
//!   * `templates/badge.csx`, crepusx (JSX-flavoured, same frontend as .jsx/.tsx)
//!
//! Hand-written TypeScript and TSX live in `ts/` and are copied into the built
//! app by `crepus moon build`, so a React component you write yourself sits
//! alongside the generated ones.

use crepuscularity_native::{
    emit_moonshine_app, emit_moonshine_component, render_template_to_ir,
    render_template_to_ir_with_path, ViewIr,
};
use std::fs;
use std::io;
use std::path::Path;

/// The page — `templates/page.crepus`, embedded at compile time so a missing or
/// broken template fails the build rather than the site.
const PAGE: &str = include_str!("../templates/page.crepus");

/// crepusx: the same View IR from JSX-flavoured syntax.
const BADGE_CSX: &str = include_str!("../templates/badge.csx");

/// Inline, for the case where a template is small enough not to earn a file.
const FOOTER: &str = r###"div border-t border-zinc-800 pt-6 text-sm text-zinc-500
 span
  "Built by cargo run, then crepus moon build."
"###;

/// Lower a template, naming it in the panic so a broken one is obvious.
///
/// The path matters: the frontend is chosen from the extension, so `.csx` goes
/// through the JSX parser while `.crepus` uses the indentation parser.
fn lower(name: &str, source: &str) -> ViewIr {
    render_template_to_ir_with_path(source, &Default::default(), Some(Path::new(name)))
        .unwrap_or_else(|e| panic!("template `{name}` failed to lower to View IR: {e}"))
}

fn main() -> io::Result<()> {
    let app_tsx = emit_moonshine_app(&lower("page.crepus", PAGE));
    let badge_tsx = emit_moonshine_component(&lower("badge.csx", BADGE_CSX), "Badge");

    // No path: an inline template is always the indentation syntax.
    let footer_ir = render_template_to_ir(FOOTER, &Default::default()).expect("lower FOOTER");
    let footer_tsx = emit_moonshine_component(&footer_ir, "Footer");

    fs::create_dir_all("generated/components")?;
    fs::write("generated/app.tsx", app_tsx)?;
    fs::write("generated/components/Badge.tsx", badge_tsx)?;
    fs::write("generated/components/Footer.tsx", footer_tsx)?;

    println!("wrote generated/app.tsx            (from templates/page.crepus)");
    println!("wrote generated/components/Badge.tsx  (from templates/badge.csx)");
    println!("wrote generated/components/Footer.tsx (inline in src/main.rs)");
    println!("next: crepus moon build");
    Ok(())
}
