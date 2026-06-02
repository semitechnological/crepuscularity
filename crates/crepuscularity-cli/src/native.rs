//! `crepus native` — Native mobile applications for iOS and Android.
//!
//! Scaffold and build native iOS (SwiftUI) and Android (Jetpack Compose) apps
//! that use **View IR** (`crepuscularity-native::render_template_to_ir`) to
//! render `.crepus` templates.
//!
//! The scaffold is the same source tree as `examples/native-shells/` — a
//! SwiftPM package under `<dir>/ios/` and a Gradle module under
//! `<dir>/android/`, sharing a common `fixture.json`. We *don't* embed the
//! Gradle wrapper jar; users run `gradle wrapper --gradle-version 8.10`
//! (or just open the project in Android Studio, which regenerates it
//! automatically) before the first `./gradlew` invocation.

use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use console::style;
use crepuscularity_core::context::{TemplateContext, TemplateValue};
use crepuscularity_core::{DriverCache, Fingerprint};
use crepuscularity_native::{
    render_component_file_to_ir, render_from_files, render_template_to_ir, to_json, to_json_pretty,
};
use serde::Deserialize;
use serde_json::Value;

use crate::build_options::{strip_build_options_or_exit, BuildOptions};
use crate::ui;

pub fn run(args: &[String]) {
    match args.first().map(|s| s.as_str()) {
        Some("new") => {
            let name = args.get(1).map(|s| s.as_str()).unwrap_or_else(|| {
                ui::error("Usage: crepus native new <name>");
            });
            scaffold_native_app(name);
        }
        Some("ir") => {
            run_ir(&args[1..]);
        }
        Some("sync") => {
            if let Err(e) = sync_native_fixture_inner(&args[1..]) {
                ui::error(&e);
            }
        }
        Some("build") => match args.get(1).map(|s| s.as_str()) {
            Some("ios") => {
                let options = BuildOptions::parse_or_exit(&args[2..]);
                let stripped = strip_build_options_or_exit(&args[2..]);
                let dir = parse_dir_arg(&stripped);
                build_ios(&dir, options);
            }
            Some("android") => {
                let options = BuildOptions::parse_or_exit(&args[2..]);
                let stripped = strip_build_options_or_exit(&args[2..]);
                let dir = parse_dir_arg(&stripped);
                let flavor = parse_flavor(&stripped).unwrap_or_else(|| {
                    if options.release() {
                        "Release".to_string()
                    } else {
                        "Debug".to_string()
                    }
                });
                build_android(&dir, &flavor);
            }
            _ => ui::error("Usage: crepus native build ios|android [--dir <path>]"),
        },
        Some("run") => match args.get(1).map(|s| s.as_str()) {
            Some("ios") => {
                let dir = parse_dir_arg(&args[2..]);
                run_ios_help(&dir);
            }
            Some("android") => {
                let options = BuildOptions::parse_or_exit(&args[2..]);
                let stripped = strip_build_options_or_exit(&args[2..]);
                let dir = parse_dir_arg(&stripped);
                let flavor = parse_flavor(&stripped).unwrap_or_else(|| {
                    if options.release() {
                        "Release".to_string()
                    } else {
                        "Debug".to_string()
                    }
                });
                run_android(&dir, &flavor);
            }
            _ => ui::error("Usage: crepus native run ios|android [--dir <path>]"),
        },
        _ => print_native_usage(),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IrEnvelope {
    entry: Option<String>,
    files: Option<HashMap<String, String>>,
    template: Option<String>,
    context: Option<Value>,
    component: Option<String>,
    pretty: Option<bool>,
    base_dir: Option<PathBuf>,
}

struct IrArgs {
    path: Option<PathBuf>,
    component: Option<String>,
    ctx_file: Option<PathBuf>,
    vars: Vec<(String, String)>,
    pretty: bool,
    stdin: bool,
    stdin_json: bool,
    base_dir: Option<PathBuf>,
}

struct SyncArgs {
    template: PathBuf,
    dir: PathBuf,
    outputs: Vec<PathBuf>,
    defaults: bool,
    component: Option<String>,
    ctx_file: Option<PathBuf>,
    vars: Vec<(String, String)>,
    pretty: bool,
}

fn run_ir(args: &[String]) {
    match run_ir_inner(args) {
        Ok(out) => print!("{out}"),
        Err(e) => {
            let payload = serde_json::json!({ "error": e });
            eprintln!("{payload}");
            std::process::exit(1);
        }
    }
}

fn run_ir_inner(args: &[String]) -> Result<String, String> {
    let parsed = parse_ir_args(args)?;
    if parsed.stdin && parsed.stdin_json {
        return Err("--stdin and --stdin-json are mutually exclusive".to_string());
    }

    let mut ctx = TemplateContext::new();
    if let Some(path) = &parsed.ctx_file {
        load_json_ctx(path, &mut ctx)?;
    }
    for (key, raw) in parsed.vars {
        ctx.set(key, parse_var_value(&raw));
    }

    if parsed.stdin_json {
        let mut raw = String::new();
        std::io::stdin()
            .read_to_string(&mut raw)
            .map_err(|e| format!("read stdin: {e}"))?;
        let env: IrEnvelope = serde_json::from_str(&raw).map_err(|e| format!("stdin JSON: {e}"))?;
        if let Some(value) = env.context {
            merge_json_ctx(&value, &mut ctx)?;
        }
        if let Some(base_dir) = env.base_dir {
            ctx.base_dir = Some(base_dir);
        }
        let pretty = env.pretty.unwrap_or(parsed.pretty);
        let ir = if let (Some(files), Some(entry)) = (env.files, env.entry) {
            render_from_files(&files, &entry, &ctx)?
        } else if let Some(template) = env.template {
            if let Some(component) = env.component {
                render_component_file_to_ir(&template, &component, &ctx)?
            } else {
                render_template_to_ir(&template, &ctx)?
            }
        } else {
            return Err("stdin JSON must include files+entry or template".to_string());
        };
        return stringify_ir(&ir, pretty);
    }

    if parsed.stdin {
        let mut template = String::new();
        std::io::stdin()
            .read_to_string(&mut template)
            .map_err(|e| format!("read stdin: {e}"))?;
        ctx.base_dir = parsed.base_dir;
        let ir = if let Some(component) = parsed.component {
            render_component_file_to_ir(&template, &component, &ctx)?
        } else {
            render_template_to_ir(&template, &ctx)?
        };
        return stringify_ir(&ir, parsed.pretty);
    }

    let path = parsed.path.ok_or_else(|| {
        "Usage: crepus native ir <file.crepus> [--component Name] [--ctx FILE] [--var k=v] [--pretty]".to_string()
    })?;
    let content = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    ctx.base_dir = path.parent().map(Path::to_path_buf);
    let ir = if let Some(component) = parsed.component {
        render_component_file_to_ir(&content, &component, &ctx)?
    } else {
        render_template_to_ir(&content, &ctx)?
    };
    stringify_ir(&ir, parsed.pretty)
}

fn parse_ir_args(args: &[String]) -> Result<IrArgs, String> {
    let mut parsed = IrArgs {
        path: None,
        component: None,
        ctx_file: None,
        vars: Vec::new(),
        pretty: false,
        stdin: false,
        stdin_json: false,
        base_dir: None,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--component" => {
                i += 1;
                parsed.component = args.get(i).cloned();
                if parsed.component.is_none() {
                    return Err("--component expects a name".to_string());
                }
            }
            "--ctx" => {
                i += 1;
                parsed.ctx_file = args.get(i).map(PathBuf::from);
                if parsed.ctx_file.is_none() {
                    return Err("--ctx expects a file path".to_string());
                }
            }
            "--var" => {
                i += 1;
                let Some(raw) = args.get(i) else {
                    return Err("--var expects key=value".to_string());
                };
                let Some((key, value)) = raw.split_once('=') else {
                    return Err(format!("--var expects key=value, got: {raw}"));
                };
                parsed.vars.push((key.to_string(), value.to_string()));
            }
            "--pretty" => parsed.pretty = true,
            "--stdin" => parsed.stdin = true,
            "--stdin-json" => parsed.stdin_json = true,
            "--base-dir" => {
                i += 1;
                parsed.base_dir = args.get(i).map(PathBuf::from);
                if parsed.base_dir.is_none() {
                    return Err("--base-dir expects a directory".to_string());
                }
            }
            other => {
                if other.starts_with('-') {
                    return Err(format!("unknown option: {other}"));
                }
                if parsed.path.is_some() {
                    return Err(format!("unexpected argument: {other}"));
                }
                parsed.path = Some(PathBuf::from(other));
            }
        }
        i += 1;
    }
    Ok(parsed)
}

fn load_json_ctx(path: &Path, ctx: &mut TemplateContext) -> Result<(), String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let value: Value =
        serde_json::from_str(&raw).map_err(|e| format!("context JSON {}: {e}", path.display()))?;
    merge_json_ctx(&value, ctx)
}

fn merge_json_ctx(value: &Value, ctx: &mut TemplateContext) -> Result<(), String> {
    let Some(obj) = value.as_object() else {
        return Err("context must be a JSON object".to_string());
    };
    for (key, value) in obj {
        ctx.set(key.clone(), json_to_template_value(value)?);
    }
    Ok(())
}

fn json_to_template_value(value: &Value) -> Result<TemplateValue, String> {
    match value {
        Value::Null => Ok(TemplateValue::Null),
        Value::Bool(v) => Ok(TemplateValue::Bool(*v)),
        Value::Number(v) => {
            if let Some(n) = v.as_i64() {
                Ok(TemplateValue::Int(n))
            } else if let Some(n) = v.as_f64() {
                Ok(TemplateValue::Float(n))
            } else {
                Err(format!("unsupported number: {v}"))
            }
        }
        Value::String(v) => Ok(TemplateValue::Str(v.clone())),
        Value::Array(values) => {
            let mut items = Vec::new();
            for item in values {
                let Some(obj) = item.as_object() else {
                    return Err("context arrays must contain objects".to_string());
                };
                let mut child = TemplateContext::new();
                for (key, value) in obj {
                    child.set(key.clone(), json_to_template_scalar(value)?);
                }
                items.push(child);
            }
            Ok(TemplateValue::List(items))
        }
        Value::Object(_) => {
            Err("context object values are only supported inside arrays".to_string())
        }
    }
}

fn json_to_template_scalar(value: &Value) -> Result<TemplateValue, String> {
    match value {
        Value::Array(_) | Value::Object(_) => {
            Err("loop item fields must be scalar JSON values".to_string())
        }
        _ => json_to_template_value(value),
    }
}

fn parse_var_value(raw: &str) -> TemplateValue {
    match raw {
        "true" => TemplateValue::Bool(true),
        "false" => TemplateValue::Bool(false),
        "null" => TemplateValue::Null,
        _ => raw
            .parse::<i64>()
            .map(TemplateValue::Int)
            .or_else(|_| raw.parse::<f64>().map(TemplateValue::Float))
            .unwrap_or_else(|_| TemplateValue::Str(raw.to_string())),
    }
}

fn stringify_ir(ir: &crepuscularity_native::ViewIr, pretty: bool) -> Result<String, String> {
    if pretty {
        to_json_pretty(ir).map_err(|e| format!("serialize IR: {e}"))
    } else {
        to_json(ir).map_err(|e| format!("serialize IR: {e}"))
    }
}

fn sync_native_fixture_inner(args: &[String]) -> Result<(), String> {
    let parsed = parse_sync_args(args)?;
    if !parsed.dir.exists() {
        return Err(format!(
            "native scaffold dir not found: {}",
            parsed.dir.display()
        ));
    }
    let root = fs::canonicalize(&parsed.dir).unwrap_or(parsed.dir);
    let template_path = resolve_template_path(&root, &parsed.template);
    let template = fs::read_to_string(&template_path)
        .map_err(|e| format!("read {}: {e}", template_path.display()))?;

    let mut ctx = TemplateContext::new();
    if let Some(path) = &parsed.ctx_file {
        load_json_ctx(path, &mut ctx)?;
    }
    for (key, raw) in parsed.vars {
        ctx.set(key, parse_var_value(&raw));
    }
    ctx.base_dir = template_path.parent().map(Path::to_path_buf);

    let component_ref = parsed.component.clone();
    let ir = if let Some(component) = &parsed.component {
        render_component_file_to_ir(&template, component, &ctx)?
    } else {
        render_template_to_ir(&template, &ctx)?
    };
    let mut json = stringify_ir(&ir, parsed.pretty)?;
    if !json.ends_with('\n') {
        json.push('\n');
    }

    // Cache skip: avoid rewriting unchanged fixtures (prevents git dirty
    // markers on `crepus native sync` re-runs).
    let cache = DriverCache::open(&root);
    let fp = Fingerprint::new(&template, component_ref.as_deref(), "native-ir");

    let mut written = 0;
    if parsed.defaults {
        for path in default_fixture_output_paths(&root) {
            if let Some(parent) = path.parent() {
                if parent.exists() {
                    if path.is_file() && cache.is_up_to_date(&fp, &json) {
                        written += 1;
                        continue;
                    }
                    fs::write(&path, &json)
                        .map_err(|e| format!("write {}: {e}", path.display()))?;
                    written += 1;
                }
            }
        }
    }
    for path in explicit_fixture_output_paths(&root, &parsed.outputs) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
        }
        if path.is_file() && cache.is_up_to_date(&fp, &json) {
            written += 1;
            continue;
        }
        fs::write(&path, &json).map_err(|e| format!("write {}: {e}", path.display()))?;
        written += 1;
    }
    cache.record(&fp, &json);
    if written == 0 {
        return Err(format!(
            "no native fixture directories found under {}",
            root.display()
        ));
    }
    ui::success(&format!(
        "synced View IR fixture from {}",
        template_path.display()
    ));
    Ok(())
}

fn parse_sync_args(args: &[String]) -> Result<SyncArgs, String> {
    let mut template = None;
    let mut dir = PathBuf::from(".");
    let mut outputs = Vec::new();
    let mut defaults = true;
    let mut component = None;
    let mut ctx_file = None;
    let mut vars = Vec::new();
    let mut pretty = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dir" => {
                i += 1;
                dir = args
                    .get(i)
                    .map(PathBuf::from)
                    .ok_or_else(|| "--dir expects a path".to_string())?;
            }
            "--component" => {
                i += 1;
                component = args.get(i).cloned();
                if component.is_none() {
                    return Err("--component expects a name".to_string());
                }
            }
            "--out" => {
                i += 1;
                outputs.push(
                    args.get(i)
                        .map(PathBuf::from)
                        .ok_or_else(|| "--out expects a file path".to_string())?,
                );
            }
            "--ctx" => {
                i += 1;
                ctx_file = args.get(i).map(PathBuf::from);
                if ctx_file.is_none() {
                    return Err("--ctx expects a file path".to_string());
                }
            }
            "--var" => {
                i += 1;
                let Some(raw) = args.get(i) else {
                    return Err("--var expects key=value".to_string());
                };
                let Some((key, value)) = raw.split_once('=') else {
                    return Err(format!("--var expects key=value, got: {raw}"));
                };
                vars.push((key.to_string(), value.to_string()));
            }
            "--pretty" => pretty = true,
            "--no-defaults" => defaults = false,
            other => {
                if other.starts_with("--dir=") {
                    dir = PathBuf::from(other.trim_start_matches("--dir="));
                } else if other.starts_with("--out=") {
                    outputs.push(PathBuf::from(other.trim_start_matches("--out=")));
                } else if other.starts_with('-') {
                    return Err(format!("unknown option: {other}"));
                } else if template.is_some() {
                    return Err(format!("unexpected argument: {other}"));
                } else {
                    template = Some(PathBuf::from(other));
                }
            }
        }
        i += 1;
    }

    let template = template.ok_or_else(|| {
        "Usage: crepus native sync <file.crepus> [--dir <project>] [--out FILE] [--no-defaults] [--ctx FILE] [--var k=v] [--pretty]".to_string()
    })?;
    Ok(SyncArgs {
        template,
        dir,
        outputs,
        defaults,
        component,
        ctx_file,
        vars,
        pretty,
    })
}

fn resolve_template_path(root: &Path, template: &Path) -> PathBuf {
    if template.is_absolute() {
        return template.to_path_buf();
    }
    let rooted = root.join(template);
    if rooted.exists() {
        rooted
    } else {
        template.to_path_buf()
    }
}

fn default_fixture_output_paths(root: &Path) -> Vec<PathBuf> {
    vec![
        root.join("fixture.json"),
        root.join("ios/Sources/NativeShell/fixture.json"),
        root.join("NativeShell/Sources/NativeShell/fixture.json"),
        root.join("android/app/src/main/assets/fixture.json"),
    ]
}

fn explicit_fixture_output_paths(root: &Path, explicit: &[PathBuf]) -> Vec<PathBuf> {
    explicit
        .iter()
        .map(|path| {
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                root.join(path)
            }
        })
        .collect()
}

/// Each entry is `(relative path within the scaffold root, file content)`.
///
/// Embedding via `include_str!` keeps the templates next to the published
/// crate source without an explicit `[package].include` list — `cargo
/// publish` walks the source tree by default and picks them up.
const TEMPLATE_FILES: &[(&str, &str)] = &[
    ("README.md", include_str!("../templates/native/README.md")),
    (
        "views/main.crepus",
        include_str!("../templates/native/views/main.crepus"),
    ),
    ("fixture.json", include_str!("../templates/native/fixture.json")),
    ("ios/Package.swift", include_str!("../templates/native/ios/Package.swift")),
    (
        "ios/Sources/NativeShell/ViewIrModels.swift",
        include_str!("../templates/native/ios/Sources/NativeShell/ViewIrModels.swift"),
    ),
    (
        "ios/Sources/NativeShell/ViewIrTreeView.swift",
        include_str!("../templates/native/ios/Sources/NativeShell/ViewIrTreeView.swift"),
    ),
    (
        "ios/Sources/NativeShell/fixture.json",
        include_str!("../templates/native/ios/Sources/NativeShell/fixture.json"),
    ),
    (
        "android/build.gradle.kts",
        include_str!("../templates/native/android/build.gradle.kts"),
    ),
    (
        "android/settings.gradle.kts",
        include_str!("../templates/native/android/settings.gradle.kts"),
    ),
    (
        "android/gradle.properties",
        include_str!("../templates/native/android/gradle.properties"),
    ),
    (
        "android/gradle/wrapper/gradle-wrapper.properties",
        include_str!("../templates/native/android/gradle/wrapper/gradle-wrapper.properties"),
    ),
    (
        "android/app/build.gradle.kts",
        include_str!("../templates/native/android/app/build.gradle.kts"),
    ),
    (
        "android/app/src/main/AndroidManifest.xml",
        include_str!("../templates/native/android/app/src/main/AndroidManifest.xml"),
    ),
    (
        "android/app/src/main/assets/fixture.json",
        include_str!("../templates/native/android/app/src/main/assets/fixture.json"),
    ),
    (
        "android/app/src/main/java/dev/crepuscularity/nativeshell/MainActivity.kt",
        include_str!(
            "../templates/native/android/app/src/main/java/dev/crepuscularity/nativeshell/MainActivity.kt"
        ),
    ),
    (
        "android/app/src/main/java/dev/crepuscularity/nativeshell/ViewIr.kt",
        include_str!(
            "../templates/native/android/app/src/main/java/dev/crepuscularity/nativeshell/ViewIr.kt"
        ),
    ),
    (
        "android/app/src/main/java/dev/crepuscularity/nativeshell/ViewIrTree.kt",
        include_str!(
            "../templates/native/android/app/src/main/java/dev/crepuscularity/nativeshell/ViewIrTree.kt"
        ),
    ),
    (
        "android/app/src/main/res/values/themes.xml",
        include_str!("../templates/native/android/app/src/main/res/values/themes.xml"),
    ),
];

fn scaffold_native_app(name: &str) {
    let root = PathBuf::from(name);
    if root.exists() {
        ui::error(&format!(
            "destination '{}' already exists; pick a fresh name or remove it first",
            root.display()
        ));
    }

    for (rel, content) in TEMPLATE_FILES {
        let target = root.join(rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|e| {
                ui::error(&format!("failed to create '{}': {e}", parent.display()));
            });
        }
        fs::write(&target, content).unwrap_or_else(|e| {
            ui::error(&format!("failed to write '{}': {e}", target.display()));
        });
    }

    let gitignore = "# Build outputs and IDE caches kept out of source control.\n\
                     ios/.build/\n\
                     android/.gradle/\n\
                     android/build/\n\
                     android/app/build/\n\
                     android/local.properties\n\
                     .idea/\n\
                     *.iml\n";
    fs::write(root.join(".gitignore"), gitignore).unwrap_or_else(|e| {
        ui::error(&format!("failed to write .gitignore: {e}"));
    });

    ui::success(&format!(
        "scaffolded native app '{}' at '{}'",
        name,
        root.display()
    ));
    eprintln!();
    eprintln!("{}", style("Next steps").dim());
    eprintln!(
        "  iOS:     cd {dir}/ios && swift build              # or open Package.swift in Xcode",
        dir = name
    );
    eprintln!(
        "  Android: cd {dir}/android && gradle wrapper --gradle-version 8.10 && \\\n           ./gradlew :app:assembleDebug",
        dir = name
    );
    eprintln!(
        "  Build via crepus: crepus native build ios --dir {dir}",
        dir = name
    );
    eprintln!(
        "                    crepus native build android --dir {dir}",
        dir = name
    );
}

fn parse_dir_arg(args: &[String]) -> PathBuf {
    for window in args.windows(2) {
        if window[0] == "--dir" {
            return PathBuf::from(&window[1]);
        }
    }
    for arg in args {
        if let Some(rest) = arg.strip_prefix("--dir=") {
            return PathBuf::from(rest);
        }
    }
    PathBuf::from(".")
}

fn parse_flavor(args: &[String]) -> Option<String> {
    for window in args.windows(2) {
        if window[0] == "--flavor" {
            return Some(window[1].clone());
        }
    }
    for arg in args {
        if let Some(rest) = arg.strip_prefix("--flavor=") {
            return Some(rest.to_string());
        }
    }
    None
}

fn build_ios(dir: &Path, options: BuildOptions) {
    let ios_dir = dir.join("ios");
    if !ios_dir.join("Package.swift").exists() {
        ui::error(&format!(
            "no Package.swift at '{}'. Pass --dir <path-to-scaffold-root> if the project lives elsewhere.",
            ios_dir.display()
        ));
    }
    let mut cmd = Command::new("swift");
    cmd.arg("build").current_dir(&ios_dir);
    if options.release() {
        cmd.args(["-c", "release"]);
    } else {
        cmd.args(["-c", "debug"]);
    }
    delegate(cmd, "swift build");
}

fn build_android(dir: &Path, flavor: &str) {
    let android_dir = dir.join("android");
    let gradlew = android_dir.join("gradlew");
    if !android_dir.join("settings.gradle.kts").exists() {
        ui::error(&format!(
            "no settings.gradle.kts at '{}'. Pass --dir <path-to-scaffold-root> if the project lives elsewhere.",
            android_dir.display()
        ));
    }

    let task = format!(":app:assemble{}", capitalize_ascii(flavor));
    let mut cmd = if gradlew.exists() {
        let mut c = Command::new(&gradlew);
        c.current_dir(&android_dir);
        c.arg(&task);
        c
    } else {
        // Fall back to system `gradle`. Print a hint either way so users know
        // why we're not invoking ./gradlew.
        eprintln!(
            "{} no ./gradlew at {}; using system `gradle` (run `gradle wrapper --gradle-version 8.10` to generate the wrapper)",
            style("note:").yellow(),
            gradlew.display()
        );
        let mut c = Command::new("gradle");
        c.current_dir(&android_dir);
        c.arg(&task);
        c
    };
    cmd.arg("--quiet"); // don't drown the user in gradle log spam
    delegate(cmd, "gradle build");
}

fn run_ios_help(dir: &Path) {
    eprintln!(
        "{}",
        style("crepus native run ios — open in Xcode").cyan().bold()
    );
    eprintln!();
    eprintln!("  open {dir}/ios/Package.swift", dir = dir.display());
    eprintln!();
    eprintln!(
        "{} SwiftPM does not run apps directly; opening Package.swift in Xcode lets you pick a simulator and Run.",
        style("note:").yellow()
    );
    eprintln!(
        "{} for a fresh iOS app target with a generated `.xcodeproj`, see `crepus ios new`.",
        style("hint:").dim()
    );
}

fn run_android(dir: &Path, flavor: &str) {
    let android_dir = dir.join("android");
    let gradlew = android_dir.join("gradlew");
    let task = format!(":app:install{}", capitalize_ascii(flavor));

    let mut cmd = if gradlew.exists() {
        let mut c = Command::new(&gradlew);
        c.current_dir(&android_dir);
        c.arg(&task);
        c
    } else {
        let mut c = Command::new("gradle");
        c.current_dir(&android_dir);
        c.arg(&task);
        c
    };
    cmd.arg("--quiet");
    delegate(cmd, "gradle install");

    eprintln!(
        "\n{} APK installed; launch with:\n  adb shell am start -n dev.crepuscularity.nativeshell/.MainActivity",
        style("note:").dim()
    );
}

fn delegate(mut cmd: Command, label: &str) {
    match cmd.status() {
        Ok(status) if status.success() => {}
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(e) => ui::error(&format!(
            "failed to invoke `{label}`: {e}. Is the toolchain installed and on PATH?"
        )),
    }
}

fn capitalize_ascii(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    if let Some(c) = chars.next() {
        out.extend(c.to_uppercase());
    }
    out.extend(chars);
    out
}

fn print_native_usage() {
    eprintln!(
        "{}",
        style("crepus native — Native mobile applications (iOS + Android)")
            .cyan()
            .bold()
    );
    eprintln!();
    eprintln!("{}", style("COMMANDS").dim());
    eprintln!(
        "  {}  {}",
        style("new <name>                  ").green(),
        style("scaffold an iOS (SwiftPM) + Android (Gradle) project").dim()
    );
    eprintln!(
        "  {}  {}",
        style("ir <file.crepus> [--pretty]      ").green(),
        style("emit View IR JSON for plugins and native shells").dim()
    );
    eprintln!(
        "  {}  {}",
        style("sync <file.crepus> [--dir P] [--out FILE]").green(),
        style("write View IR fixture JSON into a native scaffold").dim()
    );
    eprintln!(
        "  {}  {}",
        style("build ios [--dir <path>]    ").green(),
        style("swift build inside <dir>/ios").dim()
    );
    eprintln!(
        "  {}  {}",
        style("build android [--dir P] [--flavor F]").green(),
        style("./gradlew :app:assemble<Flavor> inside <dir>/android").dim()
    );
    eprintln!(
        "  {}  {}",
        style("run ios [--dir <path>]      ").green(),
        style("print Xcode-open instructions for the SwiftPM package").dim()
    );
    eprintln!(
        "  {}  {}",
        style("run android [--dir P]       ").green(),
        style("./gradlew :app:install<Flavor> + adb launch hint").dim()
    );
    eprintln!();
    eprintln!("{}", style("EXAMPLES").dim());
    eprintln!("  crepus native new my-mobile-app");
    eprintln!("  crepus native ir views/main.crepus --ctx context.json --pretty");
    eprintln!("  crepus native sync views/main.crepus --dir my-mobile-app --out app/Resources/dashboard.view-ir.json --no-defaults --var name=Ada --pretty");
    eprintln!("  crepus native build ios --dir my-mobile-app");
    eprintln!("  crepus native build android --dir my-mobile-app --flavor Debug");
    eprintln!("  crepus native run android --dir my-mobile-app");
    eprintln!();
    eprintln!(
        "{} Android needs the Gradle wrapper. After scaffolding, run \
         `cd <name>/android && gradle wrapper --gradle-version 8.10` once \
         (or open the project in Android Studio, which regenerates it).",
        style("note:").dim()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capitalize_ascii_basic() {
        assert_eq!(capitalize_ascii("debug"), "Debug");
        assert_eq!(capitalize_ascii("Release"), "Release");
        assert_eq!(capitalize_ascii(""), "");
        assert_eq!(capitalize_ascii("a"), "A");
    }

    #[test]
    fn parse_dir_arg_handles_both_styles() {
        let v = vec!["--dir".to_string(), "/tmp/x".to_string()];
        assert_eq!(parse_dir_arg(&v), PathBuf::from("/tmp/x"));
        let v = vec!["--dir=/tmp/y".to_string()];
        assert_eq!(parse_dir_arg(&v), PathBuf::from("/tmp/y"));
        let v: Vec<String> = vec![];
        assert_eq!(parse_dir_arg(&v), PathBuf::from("."));
    }

    #[test]
    fn parse_flavor_handles_both_styles() {
        let v = vec!["--flavor".to_string(), "Release".to_string()];
        assert_eq!(parse_flavor(&v), Some("Release".to_string()));
        let v = vec!["--flavor=Debug".to_string()];
        assert_eq!(parse_flavor(&v), Some("Debug".to_string()));
        let v: Vec<String> = vec![];
        assert_eq!(parse_flavor(&v), None);
    }

    #[test]
    fn sync_fixture_writes_native_shell_outputs() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("app");
        fs::create_dir_all(root.join("views")).unwrap();
        fs::create_dir_all(root.join("ios/Sources/NativeShell")).unwrap();
        fs::create_dir_all(root.join("android/app/src/main/assets")).unwrap();
        fs::write(
            root.join("views/main.crepus"),
            "div flex flex-col\n  span\n    \"Hello {name}\"",
        )
        .unwrap();

        sync_native_fixture_inner(&[
            "views/main.crepus".to_string(),
            "--dir".to_string(),
            root.display().to_string(),
            "--var".to_string(),
            "name=Cupboard".to_string(),
            "--out".to_string(),
            "linux/share/dashboard.view-ir.json".to_string(),
            "--pretty".to_string(),
        ])
        .unwrap();

        let root_fixture = fs::read_to_string(root.join("fixture.json")).unwrap();
        let ios_fixture =
            fs::read_to_string(root.join("ios/Sources/NativeShell/fixture.json")).unwrap();
        let android_fixture =
            fs::read_to_string(root.join("android/app/src/main/assets/fixture.json")).unwrap();
        let linux_fixture =
            fs::read_to_string(root.join("linux/share/dashboard.view-ir.json")).unwrap();

        assert_eq!(root_fixture, ios_fixture);
        assert_eq!(root_fixture, android_fixture);
        assert_eq!(root_fixture, linux_fixture);
        assert!(root_fixture.contains("Hello Cupboard"));
        assert!(root_fixture.contains("\"kind\": \"stack\""));
    }

    #[test]
    fn sync_fixture_can_write_only_explicit_outputs() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("app");
        fs::create_dir_all(root.join("views")).unwrap();
        fs::write(
            root.join("views/main.crepus"),
            "div flex flex-col\n  span\n    \"Hello {name}\"",
        )
        .unwrap();

        sync_native_fixture_inner(&[
            "views/main.crepus".to_string(),
            "--dir".to_string(),
            root.display().to_string(),
            "--no-defaults".to_string(),
            "--out".to_string(),
            "desktop/dashboard.view-ir.json".to_string(),
            "--var".to_string(),
            "name=Cupboard".to_string(),
            "--pretty".to_string(),
        ])
        .unwrap();

        let desktop_fixture =
            fs::read_to_string(root.join("desktop/dashboard.view-ir.json")).unwrap();
        assert!(!root.join("fixture.json").exists());
        assert!(desktop_fixture.contains("Hello Cupboard"));
    }

    #[test]
    fn template_files_present() {
        // Smoke-test: every embedded file is non-empty so we know `include_str!`
        // is wired correctly to existing template files.
        for (rel, content) in TEMPLATE_FILES {
            assert!(!content.is_empty(), "empty template content at {rel}");
        }
    }

    #[test]
    fn template_files_have_unique_paths() {
        use std::collections::BTreeSet;
        let mut seen = BTreeSet::new();
        for (rel, _) in TEMPLATE_FILES {
            assert!(seen.insert(*rel), "duplicate template entry: {rel}");
        }
    }
}
