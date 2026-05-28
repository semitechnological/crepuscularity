//! `crepus ios` — XcodeGen + SwiftPM **NativeShell** host app scaffolding (View IR demos).
//!
//! Requires [XcodeGen](https://github.com/yonaskolb/XcodeGen) (`brew install xcodegen`) for
//! `crepus ios generate` / `crepus ios build`.
//!
//! **`crepus.toml`** at the app root stores `[ios]` (`scheme`, `xcodegen_spec`, `destination`).
//! `generate` / `build` walk up from the current directory until they find it (or `--dir` pins the root).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use console::style;

use crate::build_options::{strip_build_options_or_exit, BuildOptions};
use crate::crepus_toml;
use crate::ui;

#[derive(Debug, Clone)]
struct IosSection {
    scheme: String,
    xcodegen_spec: String,
    ios_destination: String,
}

pub fn run(args: &[String]) {
    match args.first().map(|s| s.as_str()) {
        Some("new") => {
            let name = args.get(1).map(|s| s.as_str()).unwrap_or_else(|| {
                ui::error("Usage: crepus ios new <name>");
            });
            scaffold_ios_app(name);
        }
        Some("generate") | Some("gen") => {
            let (root, cfg) = resolve_ios_root_and_config(&parse_optional_dir(&args[1..]));
            let spec = parse_spec_override(&args[1..]).unwrap_or(cfg.xcodegen_spec);
            run_xcodegen(&root, &spec);
        }
        Some("build") => {
            let options = BuildOptions::parse_or_exit(&args[1..]);
            let stripped = strip_build_options_or_exit(&args[1..]);
            let explicit_dir = parse_optional_dir(&stripped);
            let spec_ov = parse_spec_override(&stripped);
            let scheme_ov = parse_scheme_override(&stripped);
            let dest_ov = parse_destination_override(&stripped);

            let (root, cfg) = resolve_ios_root_and_config(&explicit_dir);
            let spec = spec_ov.unwrap_or(cfg.xcodegen_spec.clone());
            let scheme = scheme_ov.unwrap_or(cfg.scheme);
            let destination = dest_ov.unwrap_or(cfg.ios_destination);

            run_xcodegen(&root, &spec);
            run_xcodebuild(&root, &scheme, &destination, options);
        }
        _ => print_ios_usage(),
    }
}

/// If `explicit_dir` is set, resolve config only under that path. Otherwise walk up from cwd.
fn resolve_ios_root_and_config(explicit_dir: &Option<PathBuf>) -> (PathBuf, IosSection) {
    if let Some(root) = explicit_dir {
        let root = normalize_root(root);
        if let Some(cfg) = load_ios_config(&root) {
            return (root, cfg);
        }
        if let Some(cfg) = legacy_config_from_project_yml(&root) {
            return (root, cfg);
        }
        ui::error(&format!(
            "no crepus.toml [ios] or project.yml in {}",
            root.display()
        ));
    }

    let start = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Some((root, cfg)) = walk_up_for_ios_config(&start) {
        return (root, cfg);
    }

    ui::error(
        "no crepus.toml with [ios] found (or project.yml legacy) — run from inside the app, a parent folder, or pass --dir PATH.\n\
         Scaffold: crepus ios new my-app && cd my-app",
    );
}

fn normalize_root(p: &Path) -> PathBuf {
    if p.as_os_str().is_empty() {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    } else {
        fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
    }
}

/// Walk from `start` (file or dir) upward looking for `crepus.toml` with `[ios]`, else `project.yml`.
fn walk_up_for_ios_config(start: &Path) -> Option<(PathBuf, IosSection)> {
    let mut dir = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };

    loop {
        if let Some(cfg) = load_ios_config(&dir) {
            return Some((dir, cfg));
        }
        if let Some(cfg) = legacy_config_from_project_yml(&dir) {
            return Some((dir, cfg));
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn load_ios_config(root: &Path) -> Option<IosSection> {
    let p = root.join("crepus.toml");
    let i = crepus_toml::try_load_ios(&p)?;
    Some(IosSection {
        scheme: i.scheme,
        xcodegen_spec: i.xcodegen_spec,
        ios_destination: i.destination,
    })
}

/// Older trees: only `project.yml` with leading `name:`
fn legacy_config_from_project_yml(root: &Path) -> Option<IosSection> {
    let yml = fs::read_to_string(root.join("project.yml")).ok()?;
    let name_line = yml.lines().find(|l| {
        let t = l.trim_start();
        t.starts_with("name:") && !t.contains("PRODUCT_BUNDLE_IDENTIFIER")
    })?;
    let rest = name_line.trim_start().strip_prefix("name:")?.trim();
    let scheme = rest.split_whitespace().next()?.to_string();
    Some(IosSection {
        scheme,
        xcodegen_spec: crepus_toml::default_xcodegen_spec(),
        ios_destination: crepus_toml::default_ios_destination(),
    })
}

fn parse_optional_dir(args: &[String]) -> Option<PathBuf> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--dir" && args.get(i + 1).is_some() {
            return Some(PathBuf::from(&args[i + 1]));
        }
        i += 1;
    }
    None
}

fn parse_spec_override(args: &[String]) -> Option<String> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--spec" && args.get(i + 1).is_some() {
            return Some(args[i + 1].clone());
        }
        i += 1;
    }
    None
}

fn parse_scheme_override(args: &[String]) -> Option<String> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--scheme" && args.get(i + 1).is_some() {
            return Some(args[i + 1].clone());
        }
        i += 1;
    }
    None
}

fn parse_destination_override(args: &[String]) -> Option<String> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--destination" && args.get(i + 1).is_some() {
            return Some(args[i + 1].clone());
        }
        i += 1;
    }
    None
}

fn print_ios_usage() {
    eprintln!("{}", style("crepus ios").cyan().bold());
    eprintln!(
        "{}",
        style("XcodeGen + SwiftPM NativeShell (View IR JSON → SwiftUI)").dim()
    );
    eprintln!();
    eprintln!("{}", style("COMMANDS").dim());
    eprintln!(
        "  {}  {}",
        style("new <name>                           ").green(),
        style("scaffold crepus.toml + project.yml + NativeShell + App/").dim()
    );
    eprintln!(
        "  {}  {}",
        style("generate [--dir] [--spec]            ").green(),
        style("run xcodegen (walks up for crepus.toml [ios])").dim()
    );
    eprintln!(
        "  {}  {}",
        style("build [--dir] [--scheme] [--release]").green(),
        style("xcodegen + xcodebuild; scheme/dest from toml").dim()
    );
    eprintln!();
    eprintln!("{}", style("SETUP").dim());
    eprintln!("  {}", style("wax install xcodegen").dim());
}

fn scaffold_ios_app(name: &str) {
    let t0 = Instant::now();
    let dir = Path::new(name);
    if dir.exists() {
        ui::error(&format!("'{}' already exists", name));
    }

    let pascal = to_pascal_case(name);
    let app_target = app_target_name(&pascal);
    let mut bundle_suffix = bundle_slug(name);
    if bundle_suffix.is_empty() {
        bundle_suffix = "app".into();
    }

    let native = dir.join("NativeShell");
    let sources = native.join("Sources").join("NativeShell");

    fs::create_dir_all(&sources).unwrap_or_else(|e| {
        ui::error(&format!("create dirs: {e}"));
    });
    fs::create_dir_all(dir.join("App")).unwrap_or_else(|e| {
        ui::error(&format!("create App: {e}"));
    });

    fs::write(
        dir.join("project.yml"),
        project_yml(&app_target, &bundle_suffix),
    )
    .unwrap_or_else(|e| ui::error(&format!("write project.yml: {e}")));
    fs::write(dir.join("crepus.toml"), crepus_toml(&app_target)).unwrap_or_else(|e| {
        ui::error(&format!("write crepus.toml: {e}"));
    });
    fs::write(dir.join(".gitignore"), ios_gitignore()).unwrap_or_else(|e| {
        ui::error(&format!("write .gitignore: {e}"));
    });
    fs::write(dir.join("README.md"), readme_md(name, &app_target))
        .unwrap_or_else(|e| ui::error(&format!("write README: {e}")));

    fs::write(native.join("Package.swift"), native_package_swift())
        .unwrap_or_else(|e| ui::error(&format!("write Package.swift: {e}")));
    fs::write(
        sources.join("ViewIrModels.swift"),
        include_str!("../assets/ios/ViewIrModels.swift"),
    )
    .unwrap_or_else(|e| ui::error(&format!("write ViewIrModels: {e}")));
    fs::write(
        sources.join("ViewIrTreeView.swift"),
        include_str!("../assets/ios/ViewIrTreeView.swift"),
    )
    .unwrap_or_else(|e| ui::error(&format!("write ViewIrTreeView: {e}")));
    fs::write(
        sources.join("fixture.json"),
        include_str!("../assets/ios/fixture.json"),
    )
    .unwrap_or_else(|e| ui::error(&format!("write fixture.json: {e}")));

    fs::write(dir.join("App").join("App.swift"), app_swift(&pascal))
        .unwrap_or_else(|e| ui::error(&format!("write App.swift: {e}")));
    fs::write(
        dir.join("App").join("ContentView.swift"),
        content_view_swift(),
    )
    .unwrap_or_else(|e| ui::error(&format!("write ContentView: {e}")));

    eprintln!(
        "\n{} {}",
        ui::ok(),
        style(format!("ios scaffold `{name}`")).cyan().bold()
    );
    eprintln!();
    eprintln!("{}", style("Next:").dim());
    eprintln!("  wax install xcodegen   # once");
    eprintln!("  cd {name}");
    eprintln!("  crepus ios generate");
    eprintln!("  open {app_target}.xcodeproj");
    eprintln!();
    eprintln!(
        "{}",
        style("Or from this directory: crepus ios build").dim()
    );
    ui::done_in(t0.elapsed());
}

fn app_target_name(pascal: &str) -> String {
    if pascal == "NativeShell" {
        "NativeShellHostApp".to_string()
    } else {
        format!("{pascal}App")
    }
}

fn bundle_slug(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

fn crepus_toml(app_target: &str) -> String {
    format!(
        r#"# Crepuscularity — `crepus ios generate` / `crepus ios build` read this (walks up from cwd).
[ios]
scheme = "{app_target}"
xcodegen_spec = "project.yml"
destination = "platform=iOS Simulator,name=iPhone 16,OS=latest"
"#
    )
}

fn project_yml(app_target: &str, bundle_suffix: &str) -> String {
    // Keep keys stable for XcodeGen; scheme name matches the app target.
    format!(
        r#"name: {app_target}
options:
  bundleIdPrefix: dev.crepuscularity
  deploymentTarget:
    iOS: "17.0"
  createIntermediateGroups: true
settings:
  base:
    SWIFT_VERSION: "5.9"
    TARGETED_DEVICE_FAMILY: "1,2"
packages:
  NativeShell:
    path: NativeShell
targets:
  {app_target}:
    type: application
    platform: iOS
    sources:
      - path: App
    settings:
      base:
        PRODUCT_BUNDLE_IDENTIFIER: dev.crepuscularity.{bundle_suffix}
    dependencies:
      - package: NativeShell
        product: NativeShell
"#
    )
}

fn native_package_swift() -> String {
    r#"// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "NativeShell",
    platforms: [.iOS(.v17), .macOS(.v14)],
    products: [
        .library(name: "NativeShell", targets: ["NativeShell"]),
    ],
    targets: [
        .target(
            name: "NativeShell",
            path: "Sources/NativeShell",
            resources: [.copy("fixture.json")]
        ),
    ]
)
"#
    .to_string()
}

fn app_swift(pascal: &str) -> String {
    format!(
        r#"import SwiftUI

@main
struct {pascal}App: App {{
    var body: some Scene {{
        WindowGroup {{
            ContentView()
        }}
    }}
}}
"#
    )
}

fn content_view_swift() -> String {
    r#"import SwiftUI
import NativeShell

struct ContentView: View {
    var body: some View {
        FixtureRootView()
    }
}
"#
    .to_string()
}

fn ios_gitignore() -> String {
    r#"DerivedData/
.build/
xcuserdata/
*.xcuserstate
*.xcodeproj/
*.xcworkspace/
.DS_Store
"#
    .to_string()
}

fn readme_md(name: &str, app_target: &str) -> String {
    format!(
        r#"# {name} (Crepuscularity iOS shell)

Generated with `crepus ios new {name}`.

- **XcodeGen** produces `{app_target}.xcodeproj` from `project.yml`.
- **NativeShell** is a local Swift package that decodes View IR JSON (`fixture.json`).

## Commands

`crepus.toml` stores the Xcode scheme and simulator destination; run commands from this directory (or any subfolder).

```bash
wax install xcodegen
crepus ios generate
open {app_target}.xcodeproj
```

Build from the CLI (after XcodeGen):

```bash
crepus ios build
```

Replace `fixture.json` under `NativeShell/Sources/NativeShell/` when you change templates; rebuild the app.

"#
    )
}

fn run_xcodegen(dir: &Path, spec: &str) {
    let spec_path = dir.join(spec);
    if !spec_path.is_file() {
        ui::error(&format!(
            "no XcodeGen spec `{}` in {} — fix [ios] xcodegen_spec or run `crepus ios new`",
            spec,
            dir.display()
        ));
    }
    let mut cmd = Command::new("xcodegen");
    cmd.current_dir(dir);
    cmd.arg("generate").args(["--spec", spec]);
    let status = cmd.status().unwrap_or_else(|e| {
        ui::error(&format!(
            "failed to run `xcodegen`: {e}\nInstall: wax install xcodegen"
        ));
    });
    if !status.success() {
        ui::error("xcodegen failed");
    }
    eprintln!("{}", style("xcodegen: ok").green());
}

fn run_xcodebuild(dir: &Path, scheme: &str, destination: &str, options: BuildOptions) {
    let proj = find_xcodeproj(dir).unwrap_or_else(|| {
        ui::error(&format!(
            "no .xcodeproj in {} — run `crepus ios generate` first",
            dir.display()
        ));
    });

    let status = Command::new("xcodebuild")
        .current_dir(dir)
        .args([
            "-project",
            proj.file_name().unwrap().to_str().unwrap(),
            "-scheme",
            scheme,
            "-destination",
            destination,
            "-configuration",
            if options.release() {
                "Release"
            } else {
                "Debug"
            },
            "build",
        ])
        .status()
        .unwrap_or_else(|e| ui::error(&format!("xcodebuild: {e}")));

    if !status.success() {
        ui::error("xcodebuild failed");
    }
    eprintln!("{}", style("xcodebuild: ok").green());
}

fn find_xcodeproj(dir: &Path) -> Option<PathBuf> {
    let rd = fs::read_dir(dir).ok()?;
    for ent in rd.flatten() {
        let p = ent.path();
        if p.extension().is_some_and(|e| e == "xcodeproj") {
            return Some(p);
        }
    }
    None
}

fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| c == '-' || c == '_' || c.is_whitespace())
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}
