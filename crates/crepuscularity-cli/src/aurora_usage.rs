use console::style;

pub fn print_usage() {
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
