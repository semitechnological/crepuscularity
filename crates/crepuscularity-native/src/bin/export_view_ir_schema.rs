//! Emit JSON Schema for [`crepuscularity_native::ir::ViewIr`] (requires `--features schema`).
//!
//! Usage: `export-view-ir-schema [--output path]` (default: stdout).

use std::path::PathBuf;

fn main() {
    let mut args = std::env::args().skip(1);
    let mut out_path: Option<PathBuf> = None;
    while let Some(a) = args.next() {
        if a == "--output" || a == "-o" {
            out_path = args.next().map(PathBuf::from);
        } else if a == "--help" || a == "-h" {
            eprintln!("Usage: export-view-ir-schema [--output PATH]");
            std::process::exit(0);
        }
    }

    let schema = schemars::schema_for!(crepuscularity_native::ViewIr);
    let json = serde_json::to_string_pretty(&schema).expect("schema to JSON");

    if let Some(p) = out_path {
        std::fs::write(&p, json).unwrap_or_else(|e| {
            eprintln!("write {}: {e}", p.display());
            std::process::exit(1);
        });
        eprintln!("wrote {}", p.display());
    } else {
        println!("{json}");
    }
}
