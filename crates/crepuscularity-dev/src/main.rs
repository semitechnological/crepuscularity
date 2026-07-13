//! Deprecated. Use `crepus inspect` instead.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    eprintln!("crepus-dev is deprecated; use `crepus inspect`.");
    eprintln!("example: crepus inspect FILE --mode ast|render|ir|ctx|preview");
    if args.is_empty() {
        std::process::exit(2);
    }
    let mut cmd = std::process::Command::new("crepus");
    cmd.arg("inspect");
    // old: crepus-dev FILE [--ast|--render|--ir|--ctx] ...
    // new: crepus inspect FILE --mode ...
    let mut file = None;
    let mut mode = "preview";
    let mut rest = Vec::new();
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--ast" => mode = "ast",
            "--render" => mode = "render",
            "--ir" => mode = "ir",
            "--ctx" => mode = "ctx",
            "--help" | "-h" => {
                eprintln!("Usage: crepus-dev <file.crepus> [--ast|--render|--ir|--ctx] ...");
                std::process::exit(0);
            }
            other if !other.starts_with('-') && file.is_none() => file = Some(other.to_string()),
            other => {
                rest.push(other);
                if other == "--width" || other == "--height" || other == "--var" || other == "--bool" || other == "--int" {
                    if let Some(v) = it.next() { rest.push(v); }
                }
            }
        }
    }
    let Some(file) = file else {
        eprintln!("missing template path");
        std::process::exit(2);
    };
    cmd.arg(file).arg("--mode").arg(mode).args(rest);
    match cmd.status() {
        Ok(st) => std::process::exit(st.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("failed to run crepus: {e}");
            eprintln!("install: cargo install --path crates/crepuscularity-cli");
            std::process::exit(1);
        }
    }
}
