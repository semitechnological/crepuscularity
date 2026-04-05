//! CLI entry point for unthought-sites
//!
//! Usage: unthought-sites [--json <file>] [<json-string>]
//!
//! Reads a SitePayload JSON from a file or argument, renders HTML to stdout.

use std::{
    env, fs,
    io::{self, Read},
};
use unthought_sites::build_site_from_json;

fn main() {
    let args: Vec<String> = env::args().collect();

    let json = if args.len() > 2 && args[1] == "--json" {
        // Read from file
        fs::read_to_string(&args[2]).unwrap_or_else(|e| {
            eprintln!("Error reading file {}: {}", args[2], e);
            std::process::exit(1);
        })
    } else if args.len() > 1 && args[1] != "--help" && args[1] != "-h" {
        // JSON passed as argument
        args[1].clone()
    } else if atty::isnt(atty::Stream::Stdin) {
        // Read from stdin
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap_or_else(|e| {
            eprintln!("Error reading stdin: {}", e);
            std::process::exit(1);
        });
        buffer
    } else {
        eprintln!("Usage: unthought-sites [--json <file>] [<json-string>]");
        eprintln!("       echo '<json>' | unthought-sites");
        std::process::exit(1);
    };

    match build_site_from_json(&json) {
        Ok(result) => {
            // Print warnings to stderr
            for warning in &result.warnings {
                eprintln!("Warning: {}", warning);
            }
            // Print HTML to stdout
            print!("{}", result.html);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
