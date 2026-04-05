//! Static marketing-site commands (`unthought-sites` payloads → HTML).

use console::style;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::ui;

// ── entry ────────────────────────────────────────────────────────────────────

pub fn run(args: &[String]) {
    match args.first().map(|s| s.as_str()) {
        Some("new") => {
            let name = args.get(1).map(|s| s.as_str()).unwrap_or_else(|| {
                ui::error("Usage: crepus web new <name>");
            });
            scaffold_site(name);
        }
        Some("build") => run_build(&args[1..]),
        Some("--help") | Some("-h") => print_web_usage(),
        _ => print_web_usage(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebBuildPlan {
    pub json_path: PathBuf,
    pub out_path: Option<PathBuf>,
}

pub fn parse_web_build_args(args: &[String]) -> Result<WebBuildPlan, String> {
    let mut site_dir: Option<PathBuf> = None;
    let mut json_flag: Option<PathBuf> = None;
    let mut out_path: Option<PathBuf> = None;
    let mut positional: Vec<&str> = Vec::new();

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--site" => {
                let dir = args
                    .get(i + 1)
                    .ok_or_else(|| "--site requires a directory path".to_string())?;
                site_dir = Some(PathBuf::from(dir));
                i += 2;
            }
            "--json" => {
                let path = args
                    .get(i + 1)
                    .ok_or_else(|| "--json requires a file path".to_string())?;
                json_flag = Some(PathBuf::from(path));
                i += 2;
            }
            "-o" | "--out" => {
                let path = args
                    .get(i + 1)
                    .ok_or_else(|| "--out requires a file path".to_string())?;
                out_path = Some(PathBuf::from(path));
                i += 2;
            }
            "-" => {
                positional.push("-");
                i += 1;
            }
            s if s.starts_with('-') => {
                return Err(format!("unknown flag: {s}"));
            }
            other => {
                positional.push(other);
                i += 1;
            }
        }
    }

    if positional.len() > 1 {
        return Err("expected at most one JSON file path".to_string());
    }
    if json_flag.is_some() && !positional.is_empty() {
        return Err("use either --json or a positional path, not both".to_string());
    }
    if site_dir.is_some() && !positional.is_empty() {
        return Err(
            "use either --site (for site.json) or a positional JSON path, not both".to_string(),
        );
    }

    let json_path = if let Some(p) = json_flag {
        p
    } else if let Some(p) = positional.first() {
        PathBuf::from(*p)
    } else {
        let base = site_dir.unwrap_or_else(|| PathBuf::from("."));
        base.join("site.json")
    };

    Ok(WebBuildPlan {
        json_path,
        out_path,
    })
}

fn print_web_usage() {
    eprintln!("{}", style("crepus web").cyan().bold());
    eprintln!(
        "{}",
        style("Static sites from SiteBuilder JSON (unthought-sites crate).").dim()
    );
    eprintln!();
    eprintln!("{}", style("COMMANDS").dim());
    eprintln!(
        "  {}  {}",
        style("new <name>                      ").green(),
        style("scaffold site.json starter project").dim()
    );
    eprintln!(
        "  {}  {}",
        style("build [OPTS] [site.json]          ").green(),
        style("render HTML (stdout or --out)").dim()
    );
    eprintln!();
    eprintln!("{}", style("build OPTIONS").dim());
    eprintln!(
        "  {}  {}",
        style("--site <dir>                      ").green(),
        style("read <dir>/site.json").dim()
    );
    eprintln!(
        "  {}  {}",
        style("--json <file>                     ").green(),
        style("explicit JSON path").dim()
    );
    eprintln!(
        "  {}  {}",
        style("-o, --out <file>                  ").green(),
        style("write HTML to file").dim()
    );
    eprintln!(
        "  {}  {}",
        style("Use `-` as JSON path                           ").green(),
        style("read SiteBuilder JSON from stdin").dim()
    );
}

fn slugify_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
}

fn scaffold_site(name: &str) {
    let t0 = Instant::now();
    let business_name = name.trim();
    if business_name.is_empty() {
        ui::error("name must not be empty");
    }

    let slug = slugify_name(business_name);
    let base = PathBuf::from(&slug);
    if base.exists() {
        ui::error(&format!("directory already exists: {slug}"));
    }

    fs::create_dir_all(&base).unwrap_or_else(|e| {
        ui::error(&format!("could not create directories: {e}"));
    });

    let payload = unthought_sites::starter_site_payload(business_name);
    let json = serde_json::to_string_pretty(&payload)
        .unwrap_or_else(|e| ui::error(&format!("could not serialize site.json: {e}")));
    fs::write(base.join("site.json"), json).unwrap_or_else(|e| {
        ui::error(&format!("could not write site.json: {e}"));
    });

    fs::write(base.join(".gitignore"), "dist/\n").unwrap_or_else(|e| {
        ui::error(&format!("could not write .gitignore: {e}"));
    });

    eprintln!(
        "\n{} created {}",
        ui::ok(),
        style(format!("{slug}/")).cyan().bold()
    );
    eprintln!();
    eprintln!("{}", style("Next steps:").dim());
    eprintln!("  cd {slug}");
    eprintln!("  crepus web build --site . -o dist/index.html");
    ui::done_in(t0.elapsed());
}

fn run_build(args: &[String]) {
    let plan = match parse_web_build_args(args) {
        Ok(p) => p,
        Err(e) => ui::error(&format!("crepus web build: {e}")),
    };

    let t0 = Instant::now();
    let json_raw = read_json_payload(&plan.json_path);
    let result = match unthought_sites::build_site_from_json(&json_raw) {
        Ok(r) => r,
        Err(e) => ui::error(&format!("build failed: {e}")),
    };

    for w in &result.warnings {
        ui::warning(w);
    }

    match &plan.out_path {
        Some(out) => {
            if let Some(parent) = out.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent).unwrap_or_else(|e| {
                        ui::error(&format!("could not create {}: {}", parent.display(), e));
                    });
                }
            }
            fs::write(out, &result.html).unwrap_or_else(|e| {
                ui::error(&format!("could not write {}: {}", out.display(), e));
            });
            eprintln!(
                "\n{} wrote {}",
                ui::ok(),
                style(out.display().to_string()).cyan().bold()
            );
            ui::done_in(t0.elapsed());
        }
        None => {
            print!("{}", result.html);
        }
    }
}

fn read_json_payload(path: &Path) -> String {
    if path.as_os_str() == "-" {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .unwrap_or_else(|e| ui::error(&format!("read stdin: {e}")));
        return buf;
    }

    if !path.exists() {
        ui::error(&format!("file not found: {}", path.display()));
    }

    fs::read_to_string(path).unwrap_or_else(|e| {
        ui::error(&format!("could not read {}: {}", path.display(), e));
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_site_defaults_to_site_json() {
        let p = parse_web_build_args(&[]).expect("parse");
        assert_end_with_site_json(&p.json_path);
        assert_eq!(p.out_path, None);
    }

    #[test]
    fn parse_site_with_custom_dir() {
        let p = parse_web_build_args(&["--site".into(), "abc".into()]).expect("parse");
        assert_eq!(p.json_path, PathBuf::from("abc").join("site.json"));
    }

    #[test]
    fn parse_positional_json() {
        let p = parse_web_build_args(&["payload.json".into(), "-o".into(), "out.html".into()])
            .expect("parse");
        assert_eq!(p.json_path, PathBuf::from("payload.json"));
        assert_eq!(p.out_path, Some(PathBuf::from("out.html")));
    }

    #[test]
    fn parse_json_flag() {
        let p = parse_web_build_args(&[
            "--json".into(),
            "x.json".into(),
            "--out".into(),
            "z.html".into(),
        ])
        .expect("parse");
        assert_eq!(p.json_path, PathBuf::from("x.json"));
        assert_eq!(p.out_path, Some(PathBuf::from("z.html")));
    }

    #[test]
    fn parse_rejects_site_plus_positional() {
        let e = parse_web_build_args(&["--site".into(), "a".into(), "b.json".into()]).unwrap_err();
        assert!(e.contains("not both"));
    }

    #[test]
    fn parse_rejects_two_positionals() {
        assert!(parse_web_build_args(&["a.json".into(), "b.json".into()]).is_err());
    }

    #[test]
    fn parse_rejects_unknown_flag() {
        assert!(parse_web_build_args(&["--nope".into()]).is_err());
    }

    #[test]
    fn parse_rejects_json_flag_and_positional() {
        assert!(
            parse_web_build_args(&["--json".into(), "a.json".into(), "b.json".into()]).is_err()
        );
    }

    #[test]
    fn parse_dash_as_stdin_json_path() {
        let p = parse_web_build_args(&["-".into()]).expect("parse");
        assert_eq!(p.json_path, PathBuf::from("-"));
    }

    fn assert_end_with_site_json(p: &Path) {
        assert_eq!(p.file_name().and_then(|n| n.to_str()), Some("site.json"));
    }
}
