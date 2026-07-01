use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, RwLock};
use std::time::{Duration, Instant};

use console::style;
use crepuscularity_core::context::{TemplateContext, TemplateValue};
use crepuscularity_native::{
    plan_hot_reload, render_template_to_ir, to_json, HotReloadEnvelope, HotReloadMessage,
};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use serde_json::Value;

use crate::{native, ui};

const DEFAULT_PORT: u16 = 4001;
const DEFAULT_TEMPLATE: &str = "views/main.crepus";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MobilePlatform {
    Ios,
    Android,
    All,
}

#[derive(Debug)]
struct MobileDevArgs {
    dir: PathBuf,
    port: u16,
    platform: MobilePlatform,
    template: PathBuf,
    ctx_file: Option<PathBuf>,
    vars: Vec<(String, String)>,
}

struct MobileDevState {
    sequence: AtomicU64,
    root: PathBuf,
    platform: MobilePlatform,
    template_path: PathBuf,
    ctx: TemplateContext,
    last_template: RwLock<String>,
    last_ir_json: RwLock<String>,
    last_event: RwLock<HotReloadEnvelope>,
}

use crate::cli::{
    MobileCodegenPlatformArg, MobileCommands, NativeCodegenCliArgs, NativeCodegenPlatformArg,
    NativeCommands, NativeSyncArgs,
};
use crate::dispatch::mobile_platform;

pub fn execute(cmd: MobileCommands) {
    match cmd {
        MobileCommands::New { name } => {
            native::execute(NativeCommands::New { name });
        }
        MobileCommands::Sync { template, rest } => {
            native::execute(NativeCommands::Sync {
                args: NativeSyncArgs {
                    template: Some(template),
                    dir: parse_dir_from_rest(&rest).unwrap_or_else(|e| ui::error(&e)),
                    out: Vec::new(),
                    no_defaults: false,
                    component: None,
                    ctx: None,
                    vars: Vec::new(),
                    pretty: false,
                },
            });
        }
        MobileCommands::Codegen {
            template,
            platform,
            rest,
        } => run_codegen_cli(template, platform, &rest).unwrap_or_else(|e| ui::error(&e)),
        MobileCommands::Build { platform, rest } => {
            run_build_platform(mobile_platform(platform), &rest)
        }
        MobileCommands::Run { platform, rest } => {
            run_mobile_platform(mobile_platform(platform), &rest)
        }
        MobileCommands::Doctor { platform } => run_doctor_platform(mobile_platform(platform)),
        MobileCommands::Dev {
            dir,
            port,
            platform,
            template,
            ctx,
            vars,
        } => {
            let parsed = MobileDevArgs {
                dir,
                port,
                platform: mobile_platform(platform),
                template,
                ctx_file: ctx,
                vars: vars
                    .iter()
                    .map(|kv| {
                        kv.split_once('=')
                            .map(|(k, v)| (k.to_string(), v.to_string()))
                            .unwrap_or_else(|| {
                                ui::error(&format!("--var expects key=value, got: {kv}"))
                            })
                    })
                    .collect(),
            };
            run_dev(parsed).unwrap_or_else(|e| ui::error(&e));
        }
    }
}

fn parse_dir_from_rest(rest: &[String]) -> Result<PathBuf, String> {
    parse_dir_arg(rest)
}

fn run_doctor_platform(platform: MobilePlatform) {
    run_doctor_strings(platform_to_cli_args(platform))
}

fn platform_to_cli_args(platform: MobilePlatform) -> Vec<String> {
    match platform {
        MobilePlatform::Ios => vec!["--platform".into(), "ios".into()],
        MobilePlatform::Android => vec!["--platform".into(), "android".into()],
        MobilePlatform::All => vec!["--platform".into(), "all".into()],
    }
}

fn run_doctor_strings(args: Vec<String>) {
    run_doctor(&args);
}

fn run_build_platform(platform: MobilePlatform, rest: &[String]) {
    run_build_with_platform(platform, rest);
}

fn run_mobile_platform(platform: MobilePlatform, rest: &[String]) {
    run_mobile_app_with_platform(platform, rest);
}

fn run_build_with_platform(platform: MobilePlatform, args: &[String]) {
    reject_mobile_release_flag(args);
    let build_args: Vec<String> = strip_mobile_only_args(args);
    match platform {
        MobilePlatform::Ios => {
            let mut forwarded = vec!["build".to_string(), "ios".to_string()];
            forwarded.extend(build_args);
            native_forward(&forwarded);
        }
        MobilePlatform::Android => {
            let mut forwarded = vec!["build".to_string(), "android".to_string()];
            forwarded.extend(build_args);
            native_forward(&forwarded);
        }
        MobilePlatform::All => {
            run_build_with_platform(MobilePlatform::Ios, args);
            run_build_with_platform(MobilePlatform::Android, args);
        }
    }
}

fn run_mobile_app_with_platform(platform: MobilePlatform, args: &[String]) {
    reject_mobile_release_flag(args);
    let run_args = strip_mobile_only_args(args);
    match platform {
        MobilePlatform::Ios => {
            let mut forwarded = vec!["run".to_string(), "ios".to_string()];
            forwarded.extend(run_args);
            native_forward(&forwarded);
        }
        MobilePlatform::Android => {
            let mut forwarded = vec!["run".to_string(), "android".to_string()];
            forwarded.extend(run_args);
            native_forward(&forwarded);
        }
        MobilePlatform::All => ui::error("crepus mobile run expects --platform ios or android"),
    }
}

fn native_forward(args: &[String]) {
    native::execute_from_argv(args);
}

fn run_codegen_cli(
    template: PathBuf,
    platform: Option<MobileCodegenPlatformArg>,
    rest: &[String],
) -> Result<(), String> {
    let dir = parse_dir_arg(rest)?;
    let template_path = resolve_template_path(&dir, &template);
    let plats = match platform {
        None => vec![MobilePlatform::Ios, MobilePlatform::Android],
        Some(MobileCodegenPlatformArg::Ios) => vec![MobilePlatform::Ios],
        Some(MobileCodegenPlatformArg::Android) => vec![MobilePlatform::Android],
    };
    for plat in plats {
        let (out, plat_arg) = match plat {
            MobilePlatform::Ios => (
                dir.join("ios/Sources/NativeShell/Generated"),
                NativeCodegenPlatformArg::SwiftUi,
            ),
            MobilePlatform::Android => (
                dir.join("android/app/src/main/java/dev/crepuscularity/nativeshell/generated"),
                NativeCodegenPlatformArg::Compose,
            ),
            MobilePlatform::All => continue,
        };
        native::execute(NativeCommands::Codegen {
            args: NativeCodegenCliArgs {
                template: Some(template_path.clone()),
                platform: Some(plat_arg),
                out: Some(out.clone()),
                view_name: Some("CrepusGeneratedView".into()),
                component: None,
                ctx: None,
                vars: Vec::new(),
            },
        });
        if plat == MobilePlatform::Android {
            prepend_kotlin_package(&out.join("CrepusGeneratedView.kt"));
        }
    }
    Ok(())
}

fn run_doctor(args: &[String]) {
    let platform = parse_platform_arg(args).unwrap_or(MobilePlatform::All);
    let mut ok = true;
    eprintln!("{}", style("crepus mobile doctor").cyan().bold());
    ok &= doctor_command("cargo", &["--version"]);
    match platform {
        MobilePlatform::Ios | MobilePlatform::All => {
            ok &= doctor_rust_target("aarch64-apple-ios");
            ok &= doctor_command("xcodebuild", &["-version"]);
            ok &= doctor_command("xcodegen", &["--version"]);
            ok &= doctor_command("xcrun", &["simctl", "list", "runtimes"]);
        }
        _ => {}
    }
    match platform {
        MobilePlatform::Android | MobilePlatform::All => {
            ok &= doctor_rust_target("aarch64-linux-android");
            ok &= doctor_command("java", &["-version"]);
            ok &= doctor_java17();
            ok &= doctor_java_home();
            ok &= doctor_command("gradle", &["--version"]);
            ok &= doctor_android_sdk();
            ok &= doctor_android_ndk();
        }
        _ => {}
    }
    if !ok {
        std::process::exit(1);
    }
}

fn doctor_command(command: &str, args: &[&str]) -> bool {
    let mut cmd = Command::new(command);
    cmd.args(args);
    if command == "gradle" {
        configure_java_home(&mut cmd);
    }
    match cmd.output() {
        Ok(out) if out.status.success() => {
            eprintln!("{} {command}", style("✓").green());
            true
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            eprintln!("{} {command}: {}", style("✗").red(), stderr.trim());
            false
        }
        Err(e) => {
            eprintln!("{} {command}: {e}", style("✗").red());
            false
        }
    }
}

fn configure_java_home(cmd: &mut Command) {
    match std::env::var("JAVA_HOME") {
        Ok(raw) if PathBuf::from(&raw).join("bin/java").exists() => {}
        _ => {
            if let Some(java_home) = discover_java_home() {
                cmd.env("JAVA_HOME", java_home);
            } else {
                cmd.env_remove("JAVA_HOME");
            }
        }
    }
}

fn discover_java_home() -> Option<String> {
    for candidate in [
        "/opt/homebrew/opt/openjdk@17/libexec/openjdk.jdk/Contents/Home",
        "/opt/homebrew/opt/openjdk/libexec/openjdk.jdk/Contents/Home",
    ] {
        let path = Path::new(candidate);
        if path.join("bin/java").exists() {
            return Some(candidate.to_string());
        }
    }
    if let Ok(out) = Command::new("/usr/libexec/java_home")
        .args(["-v", "17"])
        .output()
    {
        if out.status.success() {
            let path = String::from_utf8(out.stdout).ok()?;
            let trimmed = path.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    let java = std::fs::canonicalize("/opt/homebrew/bin/java")
        .or_else(|_| std::fs::canonicalize("/usr/bin/java"))
        .ok()?;
    let home = java.parent()?.parent()?;
    if home.join("bin/java").exists() {
        Some(home.display().to_string())
    } else {
        None
    }
}

fn doctor_rust_target(target: &str) -> bool {
    match Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
    {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.lines().any(|line| line.trim() == target) {
                eprintln!("{} rust target {target}", style("✓").green());
                true
            } else {
                eprintln!(
                    "{} rust target {target}: install with `rustup target add {target}`",
                    style("✗").red()
                );
                false
            }
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            eprintln!("{} rustup: {}", style("✗").red(), stderr.trim());
            false
        }
        Err(e) => {
            eprintln!("{} rustup: {e}", style("✗").red());
            false
        }
    }
}

fn doctor_java17() -> bool {
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let java = PathBuf::from(&java_home).join("bin/java");
        if java_version_at_least(&java, 17) {
            eprintln!("{} Java 17 {}", style("✓").green(), java_home);
            true
        } else if java_version_at_least(Path::new("java"), 17) {
            eprintln!("{} Java 17 java on PATH", style("✓").green());
            true
        } else {
            eprintln!(
                "{} Java 17: JAVA_HOME does not point to Java 17+",
                style("✗").red()
            );
            false
        }
    } else {
        match Command::new("/usr/libexec/java_home")
            .args(["-v", "17"])
            .output()
        {
            Ok(out) if out.status.success() => {
                let path = String::from_utf8_lossy(&out.stdout);
                eprintln!("{} Java 17 {}", style("✓").green(), path.trim());
                true
            }
            _ if java_version_at_least(Path::new("java"), 17) => {
                eprintln!("{} Java 17 java on PATH", style("✓").green());
                true
            }
            _ => {
                eprintln!(
                    "{} Java 17: install openjdk@17 and expose it to Gradle",
                    style("✗").red()
                );
                false
            }
        }
    }
}

fn java_version_at_least(java: &Path, major: u32) -> bool {
    let Ok(out) = Command::new(java).arg("-version").output() else {
        return false;
    };
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    text.split(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
        .find_map(parse_java_major)
        .is_some_and(|found| found >= major)
}

fn parse_java_major(raw: &str) -> Option<u32> {
    if raw.is_empty() {
        return None;
    }
    let first = raw.split('.').next()?;
    first.parse().ok()
}

fn doctor_java_home() -> bool {
    match std::env::var("JAVA_HOME") {
        Ok(raw) => {
            let path = PathBuf::from(&raw);
            if path.join("bin/java").exists() {
                eprintln!("{} JAVA_HOME {}", style("✓").green(), path.display());
                true
            } else if Path::new("java").exists() || java_version_at_least(Path::new("java"), 17) {
                eprintln!(
                    "{} JAVA_HOME invalid; Gradle will use java from PATH",
                    style("✓").green()
                );
                true
            } else {
                eprintln!(
                    "{} JAVA_HOME {} does not contain bin/java",
                    style("✗").red(),
                    path.display()
                );
                false
            }
        }
        Err(_) => {
            eprintln!(
                "{} JAVA_HOME not set; Gradle will use java from PATH",
                style("✓").green()
            );
            true
        }
    }
}

fn doctor_android_sdk() -> bool {
    let sdk = std::env::var("ANDROID_HOME")
        .or_else(|_| std::env::var("ANDROID_SDK_ROOT"))
        .ok()
        .map(PathBuf::from);
    match sdk {
        Some(path) if path.join("platforms").exists() => {
            eprintln!("{} Android SDK {}", style("✓").green(), path.display());
            true
        }
        Some(path) => {
            eprintln!(
                "{} Android SDK {} missing platforms/",
                style("✗").red(),
                path.display()
            );
            false
        }
        None => {
            eprintln!(
                "{} Android SDK: set ANDROID_HOME or ANDROID_SDK_ROOT",
                style("✗").red()
            );
            false
        }
    }
}

fn doctor_android_ndk() -> bool {
    if let Ok(path) = std::env::var("ANDROID_NDK_HOME").map(PathBuf::from) {
        if android_ndk_clang(&path).is_some() {
            eprintln!("{} Android NDK {}", style("✓").green(), path.display());
            return true;
        }
    }
    let sdk = std::env::var("ANDROID_HOME")
        .or_else(|_| std::env::var("ANDROID_SDK_ROOT"))
        .ok()
        .map(PathBuf::from);
    if let Some(sdk) = sdk {
        let ndk = sdk.join("ndk");
        if let Some(path) = latest_android_ndk(&ndk) {
            eprintln!("{} Android NDK {}", style("✓").green(), path.display());
            return true;
        }
    }
    eprintln!(
        "{} Android NDK: install via Android Studio SDK Manager or set ANDROID_NDK_HOME",
        style("✗").red()
    );
    false
}

fn latest_android_ndk(ndk_dir: &Path) -> Option<PathBuf> {
    let mut entries = ndk_dir
        .read_dir()
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && android_ndk_clang(path).is_some())
        .collect::<Vec<_>>();
    entries.sort();
    entries.pop()
}

fn android_ndk_clang(ndk: &Path) -> Option<PathBuf> {
    let prebuilt = ndk.join("toolchains/llvm/prebuilt");
    prebuilt
        .read_dir()
        .ok()?
        .flatten()
        .map(|entry| {
            entry
                .path()
                .join("bin")
                .join("aarch64-linux-android26-clang")
        })
        .find(|path| path.exists())
}

fn run_build(args: &[String]) {
    let platform = parse_platform_arg(args).unwrap_or(MobilePlatform::All);
    let stripped = strip_mobile_only_args(args);
    reject_mobile_release_flag(&stripped);
    let build_args = stripped;
    match platform {
        MobilePlatform::Ios => {
            let mut forwarded = vec!["build".to_string(), "ios".to_string()];
            forwarded.extend(build_args);
            native::execute_from_argv(&forwarded);
        }
        MobilePlatform::Android => {
            let mut forwarded = vec!["build".to_string(), "android".to_string()];
            forwarded.extend(build_args);
            native::execute_from_argv(&forwarded);
        }
        MobilePlatform::All => {
            let mut ios = vec!["build".to_string(), "ios".to_string()];
            ios.extend(build_args.clone());
            native::execute_from_argv(&ios);
            let mut android = vec!["build".to_string(), "android".to_string()];
            android.extend(build_args);
            native::execute_from_argv(&android);
        }
    }
}

fn run_mobile_app(args: &[String]) {
    let platform = parse_platform_arg(args).unwrap_or(MobilePlatform::Android);
    let stripped = strip_mobile_only_args(args);
    reject_mobile_release_flag(&stripped);
    let run_args = stripped;
    match platform {
        MobilePlatform::Ios => {
            let mut forwarded = vec!["run".to_string(), "ios".to_string()];
            forwarded.extend(run_args);
            native::execute_from_argv(&forwarded);
        }
        MobilePlatform::Android => {
            let mut forwarded = vec!["run".to_string(), "android".to_string()];
            forwarded.extend(run_args);
            native::execute_from_argv(&forwarded);
        }
        MobilePlatform::All => ui::error("crepus mobile run expects --platform ios or android"),
    }
}

fn reject_mobile_release_flag(args: &[String]) {
    if args.iter().any(|arg| arg == "--release") {
        ui::error("crepus mobile build does not use --release; use --configuration Release for iOS or --flavor Release for Android");
    }
}

fn run_codegen(args: &[String]) -> Result<(), String> {
    let platform = parse_platform_arg(args).unwrap_or(MobilePlatform::All);
    let dir = parse_dir_arg(args)?;
    let stripped = strip_mobile_codegen_args(args);
    let template = stripped
        .first()
        .filter(|arg| !arg.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| DEFAULT_TEMPLATE.to_string());
    let extra = if stripped
        .first()
        .filter(|arg| !arg.starts_with('-'))
        .is_some()
    {
        stripped[1..].to_vec()
    } else {
        stripped
    };
    let template_path = resolve_template_path(&dir, Path::new(&template));
    match platform {
        MobilePlatform::Ios | MobilePlatform::All => {
            let mut forwarded = vec![
                "codegen".to_string(),
                template_path.display().to_string(),
                "--platform".to_string(),
                "swiftui".to_string(),
                "--out".to_string(),
                dir.join("ios/Sources/NativeShell/Generated")
                    .display()
                    .to_string(),
                "--view-name".to_string(),
                "CrepusGeneratedView".to_string(),
            ];
            forwarded.extend(extra.clone());
            native::execute_from_argv(&forwarded);
        }
        _ => {}
    }
    match platform {
        MobilePlatform::Android | MobilePlatform::All => {
            let out_dir =
                dir.join("android/app/src/main/java/dev/crepuscularity/nativeshell/generated");
            let mut forwarded = vec![
                "codegen".to_string(),
                template_path.display().to_string(),
                "--platform".to_string(),
                "compose".to_string(),
                "--out".to_string(),
                out_dir.display().to_string(),
                "--view-name".to_string(),
                "CrepusGeneratedView".to_string(),
            ];
            forwarded.extend(extra);
            native::execute_from_argv(&forwarded);
            prepend_kotlin_package(&out_dir.join("CrepusGeneratedView.kt"));
        }
        _ => {}
    }
    Ok(())
}

fn run_dev(args: MobileDevArgs) -> Result<(), String> {
    let root = fs::canonicalize(&args.dir).unwrap_or(args.dir);
    let template_path = resolve_template_path(&root, &args.template);
    let mut ctx = TemplateContext::new();
    if let Some(path) = &args.ctx_file {
        load_json_ctx(path, &mut ctx)?;
    }
    for (key, raw) in args.vars {
        ctx.set(key, parse_var_value(&raw));
    }
    ctx.base_dir = template_path.parent().map(Path::to_path_buf);

    let initial_template = fs::read_to_string(&template_path)
        .map_err(|e| format!("read {}: {e}", template_path.display()))?;
    let ir = render_template_to_ir(&initial_template, &ctx).map_err(|e| e.to_string())?;
    let ir_json = to_json(&ir).map_err(|e| format!("serialize IR: {e}"))?;
    let event = HotReloadEnvelope {
        sequence: 0,
        message: HotReloadMessage::FullReload {
            ir,
            reason: "initial load".to_string(),
        },
    };
    let state = Arc::new(MobileDevState {
        sequence: AtomicU64::new(0),
        root: root.clone(),
        platform: args.platform,
        template_path: template_path.clone(),
        ctx,
        last_template: RwLock::new(initial_template),
        last_ir_json: RwLock::new(ir_json),
        last_event: RwLock::new(event),
    });

    sync_outputs(&root, &template_path, args.platform);
    start_mobile_watcher(root.clone(), state.clone());

    let addr = format!("127.0.0.1:{}", args.port);
    let listener = TcpListener::bind(&addr).map_err(|e| format!("bind {addr}: {e}"))?;
    eprintln!(
        "{} mobile dev server listening on http://{}",
        style("crepus").cyan().bold(),
        addr
    );
    eprintln!(
        "  {} watching {}",
        style("→").dim(),
        template_path.display()
    );
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let state = state.clone();
                std::thread::spawn(move || {
                    let _ = handle_mobile_dev_stream(&mut stream, &state);
                });
            }
            Err(e) => eprintln!("  {} mobile dev connection error: {e}", style("⚠").yellow()),
        }
    }
    Ok(())
}

fn start_mobile_watcher(root: PathBuf, state: Arc<MobileDevState>) {
    let (tx, rx) = mpsc::channel::<PathBuf>();
    let watch_root = root.join("views");
    std::thread::spawn(move || {
        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<Event>| match res {
                Ok(event) => {
                    if is_template_event(&event) {
                        for path in event.paths {
                            let _ = tx.send(path);
                        }
                    }
                }
                Err(e) => eprintln!("  {} mobile watcher error: {e}", style("⚠").yellow()),
            })
            .expect("crepus mobile dev: cannot create file watcher");
        let _ = watcher.watch(&watch_root, RecursiveMode::Recursive);
        loop {
            std::thread::sleep(Duration::from_secs(3600));
        }
    });

    std::thread::spawn(move || {
        while rx.recv().is_ok() {
            let start = Instant::now();
            while start.elapsed() < Duration::from_millis(50) {
                if rx.try_recv().is_err() {
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            refresh_mobile_state(&state);
        }
    });
}

fn refresh_mobile_state(state: &MobileDevState) {
    let new_template = match fs::read_to_string(&state.template_path) {
        Ok(value) => value,
        Err(e) => {
            store_event(
                state,
                HotReloadMessage::Error {
                    message: format!("read {}: {e}", state.template_path.display()),
                },
            );
            return;
        }
    };
    let old_template = state
        .last_template
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let rendered_ir = match render_template_to_ir(&new_template, &state.ctx) {
        Ok(ir) => ir,
        Err(e) => {
            store_event(
                state,
                HotReloadMessage::Error {
                    message: e.to_string(),
                },
            );
            return;
        }
    };
    let message = plan_hot_reload(&old_template, &new_template, &state.ctx);
    match &message {
        HotReloadMessage::Patch { .. } | HotReloadMessage::FullReload { ir: _, .. } => {
            if let Ok(json) = to_json(&rendered_ir) {
                *state
                    .last_ir_json
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = json;
                *state
                    .last_template
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = new_template;
                sync_outputs(&state.root, &state.template_path, state.platform);
            }
        }
        _ => {}
    }
    store_event(state, message);
}

fn store_event(state: &MobileDevState, message: HotReloadMessage) {
    let sequence = state.sequence.fetch_add(1, Ordering::SeqCst) + 1;
    let envelope = HotReloadEnvelope { sequence, message };
    *state
        .last_event
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = envelope;
}

fn handle_mobile_dev_stream(stream: &mut TcpStream, state: &MobileDevState) -> std::io::Result<()> {
    let mut buf = [0; 4096];
    let n = stream.read(&mut buf)?;
    let req = String::from_utf8_lossy(&buf[..n]);
    let path = req
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");
    let response = mobile_dev_response(path, state);
    stream.write_all(response.as_bytes())
}

fn mobile_dev_response(path: &str, state: &MobileDevState) -> String {
    match path {
        "/health" => {
            let payload = serde_json::json!({
                "ok": true,
                "sequence": state.sequence.load(Ordering::SeqCst)
            });
            http_response("200 OK", "application/json", &payload.to_string())
        }
        "/ir" => {
            let body = state
                .last_ir_json
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            http_response("200 OK", "application/json", &body)
        }
        "/events" => {
            let envelope = state
                .last_event
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            let body = format!(
                "event: crepus-mobile\ndata: {}\n\n",
                serde_json::to_string(&envelope).unwrap_or_else(|_| "{}".to_string())
            );
            http_response("200 OK", "text/event-stream", &body)
        }
        _ => http_response("404 Not Found", "text/plain", "not found"),
    }
}

fn http_response(status: &str, content_type: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn sync_outputs(root: &Path, template_path: &Path, platform: MobilePlatform) {
    let template = template_path
        .strip_prefix(root)
        .unwrap_or(template_path)
        .display()
        .to_string();
    let mut sync_args = vec![
        "sync".to_string(),
        template,
        "--dir".to_string(),
        root.display().to_string(),
        "--pretty".to_string(),
    ];
    native::execute_from_argv(&sync_args);
    match platform {
        MobilePlatform::Ios | MobilePlatform::All => {
            sync_args = vec![
                "codegen".to_string(),
                template_path.display().to_string(),
                "--platform".to_string(),
                "swiftui".to_string(),
                "--out".to_string(),
                root.join("ios/Sources/NativeShell/Generated")
                    .display()
                    .to_string(),
                "--view-name".to_string(),
                "CrepusGeneratedView".to_string(),
            ];
            native::execute_from_argv(&sync_args);
        }
        _ => {}
    }
    match platform {
        MobilePlatform::Android | MobilePlatform::All => {
            let out_dir =
                root.join("android/app/src/main/java/dev/crepuscularity/nativeshell/generated");
            let args = vec![
                "codegen".to_string(),
                template_path.display().to_string(),
                "--platform".to_string(),
                "compose".to_string(),
                "--out".to_string(),
                out_dir.display().to_string(),
                "--view-name".to_string(),
                "CrepusGeneratedView".to_string(),
            ];
            native::execute_from_argv(&args);
            prepend_kotlin_package(&out_dir.join("CrepusGeneratedView.kt"));
        }
        _ => {}
    }
}

fn prepend_kotlin_package(path: &Path) {
    let Ok(source) = fs::read_to_string(path) else {
        return;
    };
    if source.starts_with("package ") {
        return;
    }
    let updated = format!("package dev.crepuscularity.nativeshell\n\n{source}");
    let _ = fs::write(path, updated);
}

fn with_default_template_arg(command: &str, args: &[String]) -> Vec<String> {
    let mut forwarded = vec![command.to_string()];
    if args.first().is_none_or(|arg| arg.starts_with('-')) {
        forwarded.push(DEFAULT_TEMPLATE.to_string());
    }
    forwarded.extend(args.iter().cloned());
    forwarded
}

fn parse_dev_args(args: &[String]) -> Result<MobileDevArgs, String> {
    let mut dir = PathBuf::from(".");
    let mut port = DEFAULT_PORT;
    let mut platform = MobilePlatform::All;
    let mut template = PathBuf::from(DEFAULT_TEMPLATE);
    let mut ctx_file = None;
    let mut vars = Vec::new();
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
            "--port" => {
                i += 1;
                let raw = args
                    .get(i)
                    .ok_or_else(|| "--port expects a value".to_string())?;
                port = raw
                    .parse()
                    .map_err(|_| format!("invalid --port value: {raw}"))?;
            }
            "--platform" => {
                i += 1;
                platform = parse_platform_value(
                    args.get(i)
                        .ok_or_else(|| "--platform expects ios, android, or all".to_string())?,
                )?;
            }
            "--template" => {
                i += 1;
                template = args
                    .get(i)
                    .map(PathBuf::from)
                    .ok_or_else(|| "--template expects a file path".to_string())?;
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
                let raw = args
                    .get(i)
                    .ok_or_else(|| "--var expects key=value".to_string())?;
                let Some((key, value)) = raw.split_once('=') else {
                    return Err(format!("--var expects key=value, got: {raw}"));
                };
                vars.push((key.to_string(), value.to_string()));
            }
            other => {
                if let Some(value) = other.strip_prefix("--dir=") {
                    dir = PathBuf::from(value);
                } else if let Some(value) = other.strip_prefix("--port=") {
                    port = value
                        .parse()
                        .map_err(|_| format!("invalid --port value: {value}"))?;
                } else if let Some(value) = other.strip_prefix("--platform=") {
                    platform = parse_platform_value(value)?;
                } else if other.starts_with('-') {
                    return Err(format!("unknown option: {other}"));
                } else {
                    return Err(format!("unexpected argument: {other}"));
                }
            }
        }
        i += 1;
    }
    Ok(MobileDevArgs {
        dir,
        port,
        platform,
        template,
        ctx_file,
        vars,
    })
}

fn parse_dir_arg(args: &[String]) -> Result<PathBuf, String> {
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--dir" {
            return args
                .get(i + 1)
                .map(PathBuf::from)
                .ok_or_else(|| "--dir expects a path".to_string());
        }
        if let Some(value) = arg.strip_prefix("--dir=") {
            return Ok(PathBuf::from(value));
        }
        i += 1;
    }
    Ok(PathBuf::from("."))
}

fn parse_platform_arg(args: &[String]) -> Option<MobilePlatform> {
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--platform" {
            return args
                .get(i + 1)
                .and_then(|raw| parse_platform_value(raw).ok());
        }
        if let Some(value) = arg.strip_prefix("--platform=") {
            return parse_platform_value(value).ok();
        }
        i += 1;
    }
    None
}

fn parse_platform_value(raw: &str) -> Result<MobilePlatform, String> {
    match raw {
        "ios" | "swift" | "swiftui" => Ok(MobilePlatform::Ios),
        "android" | "compose" | "kotlin" => Ok(MobilePlatform::Android),
        "all" => Ok(MobilePlatform::All),
        _ => Err(format!("unknown mobile platform: {raw}")),
    }
}

fn strip_mobile_codegen_args(args: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--platform" || arg == "--dir" {
            i += 2;
            continue;
        }
        if arg.starts_with("--platform=") || arg.starts_with("--dir=") {
            i += 1;
            continue;
        }
        out.push(arg.clone());
        i += 1;
    }
    out
}

fn strip_mobile_only_args(args: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--platform" {
            i += 2;
            continue;
        }
        if arg.starts_with("--platform=") {
            i += 1;
            continue;
        }
        out.push(arg.clone());
        i += 1;
    }
    out
}

fn is_template_event(event: &Event) -> bool {
    matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    ) && event
        .paths
        .iter()
        .any(|path| path.extension().and_then(|e| e.to_str()) == Some("crepus"))
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
        Value::Array(_) => Err("mobile dev context arrays are not supported yet".to_string()),
        Value::Object(_) => Err("context object values are not supported".to_string()),
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

fn print_mobile_usage() {
    eprintln!("{}", style("crepus mobile").cyan().bold());
    eprintln!("{}", style("Unified iOS + Android mobile workflow").dim());
    eprintln!();
    eprintln!("{}", style("COMMANDS").dim());
    eprintln!(
        "  {}  {}",
        style("new <name>                                      ").green(),
        style("scaffold iOS + Android shells with dev runtime").dim()
    );
    eprintln!(
        "  {}  {}",
        style("dev [--platform ios|android|all] [--dir P]    ").green(),
        style("watch templates and serve View IR hot reload").dim()
    );
    eprintln!(
        "  {}  {}",
        style("build [--platform ios|android|all]            ").green(),
        style("build native app targets").dim()
    );
    eprintln!(
        "  {}  {}",
        style("run --platform ios|android                    ").green(),
        style("run/install one native target").dim()
    );
    eprintln!(
        "  {}  {}",
        style("doctor [--platform ios|android|all]           ").green(),
        style("check Rust, Xcode, Java, Android SDK/NDK prerequisites").dim()
    );
    eprintln!(
        "  {}  {}",
        style("sync [views/main.crepus]                      ").green(),
        style("mirror View IR fixtures into native shells").dim()
    );
    eprintln!(
        "  {}  {}",
        style("codegen [file.crepus] --platform ios|android  ").green(),
        style("write generated SwiftUI or Compose source").dim()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crepuscularity_core::context::TemplateContext;

    fn test_state() -> MobileDevState {
        let ir = render_template_to_ir("div\n  \"Hi\"", &TemplateContext::new()).unwrap();
        let ir_json = to_json(&ir).unwrap();
        MobileDevState {
            sequence: AtomicU64::new(7),
            root: PathBuf::from("."),
            platform: MobilePlatform::All,
            template_path: PathBuf::from("views/main.crepus"),
            ctx: TemplateContext::new(),
            last_template: RwLock::new("div\n  \"Hi\"".to_string()),
            last_ir_json: RwLock::new(ir_json),
            last_event: RwLock::new(HotReloadEnvelope {
                sequence: 7,
                message: HotReloadMessage::FullReload {
                    ir,
                    reason: "test".to_string(),
                },
            }),
        }
    }

    fn temp_state(template: &str) -> (tempfile::TempDir, MobileDevState) {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();
        let views = root.join("views");
        std::fs::create_dir_all(&views).unwrap();
        let template_path = views.join("main.crepus");
        std::fs::write(&template_path, template).unwrap();
        let ctx = TemplateContext::new();
        let ir = render_template_to_ir(template, &ctx).unwrap();
        let ir_json = to_json(&ir).unwrap();
        (
            temp,
            MobileDevState {
                sequence: AtomicU64::new(0),
                root,
                platform: MobilePlatform::Ios,
                template_path,
                ctx,
                last_template: RwLock::new(template.to_string()),
                last_ir_json: RwLock::new(ir_json),
                last_event: RwLock::new(HotReloadEnvelope {
                    sequence: 0,
                    message: HotReloadMessage::FullReload {
                        ir,
                        reason: "test".to_string(),
                    },
                }),
            },
        )
    }

    #[test]
    fn health_response_reports_sequence() {
        let response = mobile_dev_response("/health", &test_state());
        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("\"ok\":true"));
        assert!(response.contains("\"sequence\":7"));
    }

    #[test]
    fn ir_response_returns_latest_ir() {
        let expected = test_state()
            .last_ir_json
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let response = mobile_dev_response("/ir", &test_state());
        assert!(response.contains(&expected));
    }

    #[test]
    fn events_response_streams_hot_reload_envelope() {
        let response = mobile_dev_response("/events", &test_state());
        assert!(response.contains("Content-Type: text/event-stream"));
        assert!(response.contains("event: crepus-mobile"));
        assert!(response.contains("\"sequence\":7"));
        assert!(response.contains("\"kind\":\"fullReload\""));
    }

    #[test]
    fn refresh_emits_patch_for_literal_change() {
        let (_temp, state) = temp_state("div\n  \"Hi\"");
        std::fs::write(&state.template_path, "div\n  \"Bye\"").unwrap();
        refresh_mobile_state(&state);
        let event = state
            .last_event
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        assert!(matches!(event.message, HotReloadMessage::Patch { .. }));
        assert!(state
            .last_ir_json
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains("Bye"));
    }

    #[test]
    fn refresh_error_keeps_last_good_ir() {
        let (_temp, state) = temp_state("div\n  \"Hi\"");
        std::fs::write(&state.template_path, "include missing.crepus").unwrap();
        refresh_mobile_state(&state);
        let event = state
            .last_event
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        assert!(matches!(event.message, HotReloadMessage::Error { .. }));
        let ir = state
            .last_ir_json
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        assert!(ir.contains("Hi"));
        assert!(!ir.contains("bad indent"));
    }

    #[test]
    fn refresh_render_error_does_not_emit_patch() {
        let (_temp, state) = temp_state("div\n  \"Hi\"");
        std::fs::write(&state.template_path, "div\n  include missing.crepus").unwrap();
        refresh_mobile_state(&state);
        let event = state
            .last_event
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        assert!(matches!(event.message, HotReloadMessage::Error { .. }));
        let ir = state
            .last_ir_json
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        assert!(ir.contains("Hi"));
        assert!(!ir.contains("missing.crepus"));
    }
}
