use std::process::{exit, Command};

use console::style;

pub fn run(args: &[String]) {
    if args.is_empty() || args.first().map_or(false, |a| a == "--help" || a == "-h") {
        print_usage();
        return;
    }

    let status = Command::new("aurorality")
        .args(args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!(
                "{} failed to run aurorality: {}. {}",
                style("crepus aurora").red().bold(),
                e,
                style("Is aurorality installed? Run: cargo install aurorality-cli").dim()
            );
            exit(1);
        });

    if !status.success() {
        exit(status.code().unwrap_or(1));
    }
}

fn print_usage() {
    eprintln!(
        "{}  {}",
        style("crepus aurora").cyan().bold(),
        style("SwiftUI + Rust shell for .crepus templates").dim()
    );
    eprintln!();
    eprintln!("{}", style("SUBCOMMANDS").dim());
    eprintln!(
        "  {}  {}",
        style("dev [--watch DIR]").green(),
        style("hot-reload dev server + live preview window").dim()
    );
    eprintln!(
        "  {}  {}",
        style("run [PROJECT] [--ios]").green(),
        style("build & launch SwiftUI app (macOS or iOS)").dim()
    );
    eprintln!(
        "  {}  {}",
        style("build --watch DIR --out DIR").green(),
        style("compile .crepus templates to JSON IR").dim()
    );
    eprintln!(
        "  {}  {}",
        style("new <name>").green(),
        style("scaffold a new aurorality project").dim()
    );
    eprintln!(
        "  {}  {}",
        style("swiftgen --view FILE --out DIR --view-name NAME").green(),
        style("generate SwiftUI source from .crepus").dim()
    );
    eprintln!(
        "  {}  {}",
        style("bundle ...").green(),
        style("bundle JS + compile templates").dim()
    );
    eprintln!(
        "  {}  {}",
        style("bindgen --input DIR --output DIR").green(),
        style("generate Swift wrappers from JS exports").dim()
    );
    eprintln!();
    eprintln!("{}", style("EXAMPLES").dim());
    eprintln!(
        "  {}  {}",
        style("crepus aurora dev").green(),
        style("live preview of views/ (opens preview window)").dim()
    );
    eprintln!(
        "  {}  {}",
        style("crepus aurora run . --macos").green(),
        style("build & run current project").dim()
    );
    eprintln!(
        "  {}  {}",
        style("crepus aurora run examples/counter --macos").green(),
        style("build & run the counter example").dim()
    );
    eprintln!();
    eprintln!(
        "{}  {}",
        style("aurorality --help").dim(),
        style("for full options").dim()
    );
}
