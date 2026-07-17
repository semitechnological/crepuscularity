/// `crepus dev` — hot-reload dev loop with multi-target + UnoCSS hint support.
///
/// Usage:
///   crepus dev                      # cwd, HUD mode
///   crepus dev gpui                 # resolve ui/gui, HUD mode
///   crepus dev tui                  # resolve ui/tui, terminal mode
///   crepus dev tui:h-1 gpui:h-20px  # both simultaneously
///
/// Hint syntax (UnoCSS-style, colon-separated after target name):
///   h-N    → CREPUS_DEV_HEIGHT=N
///   h-Npx  → CREPUS_DEV_HEIGHT=N
///   w-N    → CREPUS_DEV_WIDTH=N
///   w-Npx  → CREPUS_DEV_WIDTH=N
///
/// Thread layout:
///   main thread  → GPUI Application::run (DevHUD) for gpui target, or
///                  terminal loop for tui-only
///   background   → file watcher + cargo build + child process per target
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};

use crate::build_options::BuildOptions;
use crate::builder::{cargo_build, find_bin_name, kill_child};
use crate::crepus_toml::{find_manifest_upward, load_manifest_targets};
use crate::events::CompilerEvent;
use crate::hud::{open_hud_window, DevStatus, HudState};
use crate::ui;

// ── Hint parsing ───────────────────────────────────────────────────────────

/// Parsed dev target spec: name + optional UnoCSS-style hints.
#[derive(Clone)]
struct DevTargetSpec {
    name: String,
    height: Option<String>,
    width: Option<String>,
}

/// Parse a target string like "tui:h-1" or "gpui:w-400px:h-20px" into name + hints.
fn parse_target_spec(s: &str) -> DevTargetSpec {
    let (name, rest) = match s.split_once(':') {
        Some((n, r)) => (n.to_string(), r),
        None => (s.to_string(), ""),
    };

    let mut spec = DevTargetSpec {
        name,
        height: None,
        width: None,
    };

    // Parse UnoCSS-style utilities: h-1, h-20px, w-400, w-400px
    for part in rest.split(':') {
        if let Some(val) = part.strip_prefix("h-") {
            spec.height = Some(strip_px(val));
        } else if let Some(val) = part.strip_prefix("w-") {
            spec.width = Some(strip_px(val));
        }
    }
    spec
}

/// Strip trailing "px" from a value: "20px" → "20", "1" → "1".
fn strip_px(s: &str) -> String {
    s.trim_end_matches("px").to_string()
}

/// Build env vars from a DevTargetSpec for the child process.
fn hint_env(spec: &DevTargetSpec) -> Vec<(&str, String)> {
    let mut env = Vec::new();
    if let Some(ref h) = spec.height {
        env.push(("CREPUS_DEV_HEIGHT", h.clone()));
    }
    if let Some(ref w) = spec.width {
        env.push(("CREPUS_DEV_WIDTH", w.clone()));
    }
    env
}

// ── Target resolution ──────────────────────────────────────────────────────

/// Resolve a target name to a working directory by looking up crepus.toml.
fn resolve_target_dir(target: &str) -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;

    if let Some(mpath) = find_manifest_upward(&cwd) {
        if let Ok(Some(targets)) = load_manifest_targets(Some(mpath)) {
            for t in &targets {
                if t.target_type == target || t.id == target {
                    return Some(t.dir.clone());
                }
            }
        }
    }

    let fallbacks: &[(&str, &[&str])] = &[
        ("gpui", &["ui/gui", "gui"]),
        ("tui", &["ui/tui", "tui"]),
        ("web", &["ui/web", "web"]),
        ("ios", &["ui/ios", "ios"]),
    ];
    for (kind, dirs) in fallbacks {
        if kind == &target {
            for dir in *dirs {
                let candidate = cwd.join(dir);
                if candidate.join("Cargo.toml").exists() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

fn is_terminal_target(name: &str) -> bool {
    name == "tui" || name == "terminal"
}

// ── Entry point ────────────────────────────────────────────────────────────

pub fn run(
    targets: Vec<String>,
    bin_override: Option<String>,
    options: BuildOptions,
    emit_events: bool,
) {
    if targets.is_empty() {
        // No target — original behavior: dev in cwd with HUD
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let bin_name = find_bin_name(&cwd, bin_override.as_deref()).unwrap_or_else(|| {
            ui::error("could not determine binary name — add [[bin]] to Cargo.toml or use --bin");
        });
        run_hud(cwd, bin_name, options, emit_events, &[]);
        return;
    }

    let specs: Vec<DevTargetSpec> = targets.iter().map(|s| parse_target_spec(s)).collect();

    // Resolve all target directories upfront
    let mut resolved: Vec<(DevTargetSpec, PathBuf, String)> = Vec::new();
    for spec in specs {
        match resolve_target_dir(&spec.name) {
            Some(dir) => {
                let bin_name = find_bin_name(&dir, bin_override.as_deref()).unwrap_or_else(|| {
                    ui::error(&format!(
                        "could not determine binary name for target '{}' — add [[bin]] to Cargo.toml or use --bin",
                        spec.name
                    ));
                });
                resolved.push((spec, dir, bin_name));
            }
            None => {
                ui::error(&format!(
                    "could not resolve target '{}' — no matching [[targets]] in crepus.toml and no ui/{}/ directory",
                    spec.name, spec.name
                ));
            }
        }
    }

    if resolved.is_empty() {
        return;
    }

    // Partition: terminal targets → background threads, gpui/other → main thread HUD
    let mut terminal: Vec<_> = resolved
        .iter()
        .filter(|(s, _, _)| is_terminal_target(&s.name))
        .cloned()
        .collect();
    let hud: Vec<_> = resolved
        .iter()
        .filter(|(s, _, _)| !is_terminal_target(&s.name))
        .cloned()
        .collect();

    let shutdown = Arc::new(AtomicBool::new(false));

    if hud.is_empty() && !terminal.is_empty() {
        // All terminal — keep first for main thread, rest in background
        let main_terminal = terminal.remove(0);
        for (spec, cwd, bin_name) in &terminal {
            let spec = spec.clone();
            let cwd = cwd.clone();
            let bin_name = bin_name.clone();
            let options = options.clone();
            let shutdown = shutdown.clone();
            std::thread::spawn(move || {
                run_terminal(spec, cwd, bin_name, options, emit_events, shutdown);
            });
        }
        run_terminal(main_terminal.0, main_terminal.1, main_terminal.2, options, emit_events, shutdown);
    } else {
        // Spawn all terminal targets in background threads
        for (spec, cwd, bin_name) in &terminal {
            let spec = spec.clone();
            let cwd = cwd.clone();
            let bin_name = bin_name.clone();
            let options = options.clone();
            let shutdown = shutdown.clone();
            std::thread::spawn(move || {
                run_terminal(spec, cwd, bin_name, options, emit_events, shutdown);
            });
        }
        // Main thread: first HUD target
        if let Some((spec, cwd, bin_name)) = hud.first() {
            let env = hint_env(spec);
            run_hud(cwd.clone(), bin_name.clone(), options, emit_events, &env);
            shutdown.store(true, Ordering::Relaxed);
        }
    }
}

// ── Terminal dev loop ──────────────────────────────────────────────────────

fn run_terminal(
    spec: DevTargetSpec,
    cwd: PathBuf,
    bin_name: String,
    options: BuildOptions,
    emit_events: bool,
    shutdown: Arc<AtomicBool>,
) {
    let env = hint_env(&spec);
    let hint_str = if env.is_empty() {
        String::new()
    } else {
        format!(
            " [{}]",
            env.iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    eprintln!(
        "  {} dev (terminal) — {bin_name} in {}{hint_str}",
        ui::arrow(),
        cwd.display()
    );

    let (tx, rx) = std::sync::mpsc::channel::<PathBuf>();
    let tx_notify = tx.clone();

    let mut watcher = match recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(ev) = res {
            match ev.kind {
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                    if let Some(path) = ev.paths.into_iter().next() {
                        let _ = tx_notify.send(path);
                    }
                }
                _ => {}
            }
        }
    }) {
        Ok(w) => w,
        Err(e) => {
            ui::error(&format!("could not create file watcher: {e}"));
        }
    };

    let src = cwd.join("src");
    if src.exists() {
        watcher.watch(&src, RecursiveMode::Recursive).ok();
    }
    watcher
        .watch(&cwd.join("Cargo.toml"), RecursiveMode::NonRecursive)
        .ok();

    let mut child = do_build_launch_terminal(&cwd, &bin_name, options, None, emit_events, &env);

    loop {
        if shutdown.load(Ordering::Relaxed) {
            if let Some(mut c) = child {
                kill_child(&mut c);
            }
            break;
        }

        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(_changed_path) => {
                let t = Instant::now();
                while t.elapsed() < Duration::from_millis(300) {
                    while rx.try_recv().is_ok() {}
                    std::thread::sleep(Duration::from_millis(30));
                }
                eprintln!("  {} change detected — rebuilding {bin_name}…", ui::arrow());
                child = do_build_launch_terminal(&cwd, &bin_name, options, child, emit_events, &env);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if let Some(ref mut c) = child {
                    match c.try_wait() {
                        Ok(Some(status)) => {
                            let code = status.code();
                            eprintln!("  {} {bin_name} exited ({code:?})", ui::warn());
                            if emit_events {
                                CompilerEvent::process_exited(c.id(), code).emit();
                            }
                            child = None;
                            if code.is_some() {
                                break;
                            }
                        }
                        Ok(None) => {}
                        Err(_) => child = None,
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
}

fn do_build_launch_terminal(
    cwd: &PathBuf,
    bin_name: &str,
    options: BuildOptions,
    old_child: Option<Child>,
    emit_events: bool,
    env: &[(&str, String)],
) -> Option<Child> {
    if let Some(mut c) = old_child {
        let pid = c.id();
        kill_child(&mut c);
        if emit_events {
            CompilerEvent::process_exited(pid, None).emit();
        }
    }

    if emit_events {
        CompilerEvent::compilation_started(vec![cwd.join("src")], Some("file_change".to_string()))
            .emit();
    }

    let t0 = Instant::now();
    let outcome = cargo_build(cwd, options, None);
    let elapsed_ms = t0.elapsed().as_millis() as u64;

    if outcome.success {
        if emit_events {
            let output = locate_binary(cwd, options.cargo_profile(), bin_name);
            CompilerEvent::compilation_success(elapsed_ms, output).emit();
        }

        eprintln!("  {} built in {elapsed_ms} ms — launching {bin_name}", ui::ok());

        let bin_path = locate_binary(cwd, options.cargo_profile(), bin_name);
        let mut cmd = Command::new(&bin_path);
        cmd.current_dir(cwd);
        for (key, val) in env {
            cmd.env(key, val);
        }
        match cmd.spawn() {
            Ok(c) => {
                if emit_events {
                    CompilerEvent::process_launched(c.id(), bin_path).emit();
                }
                Some(c)
            }
            Err(e) => {
                ui::error(&format!("failed to launch {bin_name}: {e}"));
            }
        }
    } else {
        let count = outcome.errors.len();
        eprintln!("  {} build failed — {count} error(s)", ui::err());
        if emit_events {
            CompilerEvent::compilation_error(elapsed_ms, outcome.errors.clone()).emit();
        }
        None
    }
}

// ── GPUI HUD dev loop ──────────────────────────────────────────────────────

fn run_hud(
    cwd: PathBuf,
    bin_name: String,
    options: BuildOptions,
    emit_events: bool,
    env: &[(&str, String)],
) {
    let shared = Arc::new(Mutex::new(HudState::new(bin_name.clone())));
    let shutdown = Arc::new(AtomicBool::new(false));

    if emit_events {
        CompilerEvent::dev_server_started(
            bin_name.clone(),
            vec![cwd.join("src"), cwd.join("Cargo.toml")],
        )
        .emit();
    }

    // Spawn background build+watch thread
    {
        let shared = shared.clone();
        let shutdown = shutdown.clone();
        let cwd = cwd.clone();
        let bin_name = bin_name.clone();
        let env: Vec<(String, String)> = env.iter().map(|(k, v)| (k.to_string(), v.clone())).collect();
        std::thread::spawn(move || {
            background_loop(shared, shutdown, cwd, bin_name, options, emit_events, &env)
        });
    }

    // Run GPUI DevHUD on main thread (blocks until window is closed)
    {
        let shared = shared.clone();
        let shutdown = shutdown.clone();
        gpui::Application::new().run(move |cx: &mut gpui::App| {
            open_hud_window(shared, shutdown, cx);
        });
    }

    shutdown.store(true, Ordering::Relaxed);
}

fn background_loop(
    shared: Arc<Mutex<HudState>>,
    shutdown: Arc<AtomicBool>,
    cwd: PathBuf,
    bin_name: String,
    options: BuildOptions,
    emit_events: bool,
    env: &[(String, String)],
) {
    let (tx, rx) = std::sync::mpsc::channel::<PathBuf>();
    let tx_notify = tx.clone();

    let mut watcher = match recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(ev) = res {
            match ev.kind {
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                    if let Some(path) = ev.paths.into_iter().next() {
                        let _ = tx_notify.send(path);
                    }
                }
                _ => {}
            }
        }
    }) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("  {} could not create file watcher: {e}", crate::ui::err());
            return;
        }
    };

    let src = cwd.join("src");
    if src.exists() {
        watcher.watch(&src, RecursiveMode::Recursive).ok();
    }
    watcher
        .watch(&cwd.join("Cargo.toml"), RecursiveMode::NonRecursive)
        .ok();

    let mut child = do_build_launch(&shared, &cwd, &bin_name, options, None, emit_events, env);

    loop {
        if shutdown.load(Ordering::Relaxed) {
            if let Some(mut c) = child {
                kill_child(&mut c);
                if emit_events {
                    CompilerEvent::process_exited(c.id(), None).emit();
                }
            }
            break;
        }

        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(changed_path) => {
                if emit_events {
                    CompilerEvent::file_changed(changed_path).emit();
                }

                let t = Instant::now();
                while t.elapsed() < Duration::from_millis(300) {
                    while rx.try_recv().is_ok() {}
                    std::thread::sleep(Duration::from_millis(30));
                }
                eprintln!("  {} change detected — rebuilding…", crate::ui::arrow());
                child = do_build_launch(&shared, &cwd, &bin_name, options, child, emit_events, env);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if let Some(ref mut c) = child {
                    match c.try_wait() {
                        Ok(Some(status)) => {
                            let code = status.code();
                            eprintln!("  {} app exited ({code:?})", crate::ui::warn());
                            if emit_events {
                                CompilerEvent::process_exited(c.id(), code).emit();
                            }
                            if let Ok(mut s) = shared.lock() {
                                s.status = DevStatus::Exited { code };
                            }
                            child = None;
                            if code.is_some() {
                                break;
                            }
                        }
                        Ok(None) => {}
                        Err(_) => child = None,
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
}

fn locate_binary(cwd: &std::path::Path, profile: &str, bin_name: &str) -> PathBuf {
    let workspace_root = std::process::Command::new("cargo")
        .args(["locate-project", "--workspace", "--message-format", "plain"])
        .current_dir(cwd)
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                let s = String::from_utf8(out.stdout).ok()?;
                let p = PathBuf::from(s.trim());
                p.parent().map(|d| d.to_path_buf())
            } else {
                None
            }
        });

    let target_base = workspace_root.unwrap_or_else(|| cwd.to_path_buf());
    target_base.join("target").join(profile).join(bin_name)
}

fn do_build_launch(
    shared: &Arc<Mutex<HudState>>,
    cwd: &PathBuf,
    bin_name: &str,
    options: BuildOptions,
    old_child: Option<Child>,
    emit_events: bool,
    env: &[(String, String)],
) -> Option<Child> {
    if let Ok(mut s) = shared.lock() {
        s.status = DevStatus::Building;
    }

    if let Some(mut c) = old_child {
        let pid = c.id();
        kill_child(&mut c);
        if emit_events {
            CompilerEvent::process_exited(pid, None).emit();
        }
    }

    if emit_events {
        CompilerEvent::compilation_started(vec![cwd.join("src")], Some("file_change".to_string()))
            .emit();
    }

    let t0 = Instant::now();
    let outcome = cargo_build(cwd, options, Some(shared.clone()));
    let elapsed_ms = t0.elapsed().as_millis() as u64;

    if outcome.success {
        if emit_events {
            let output = locate_binary(cwd, options.cargo_profile(), bin_name);
            CompilerEvent::compilation_success(elapsed_ms, output).emit();
        }

        if let Ok(mut s) = shared.lock() {
            s.status = DevStatus::Running { elapsed_ms };
        }
        let bin_path = locate_binary(cwd, options.cargo_profile(), bin_name);

        eprintln!(
            "  {} built in {elapsed_ms} ms — launching {bin_name}",
            crate::ui::ok()
        );

        let mut cmd = Command::new(&bin_path);
        cmd.current_dir(cwd);
        for (key, val) in env {
            cmd.env(key, val);
        }
        match cmd.spawn() {
            Ok(c) => {
                if emit_events {
                    CompilerEvent::process_launched(c.id(), bin_path).emit();
                }
                Some(c)
            }
            Err(e) => {
                eprintln!("  {} failed to launch {bin_name}: {e}", crate::ui::err());
                if let Ok(mut s) = shared.lock() {
                    s.status = DevStatus::Failed {
                        errors: vec![crate::hud::BuildError {
                            level: "error".into(),
                            message: format!("Failed to launch binary: {e}"),
                            ..Default::default()
                        }],
                        count: 1,
                    };
                }
                None
            }
        }
    } else {
        if emit_events {
            CompilerEvent::compilation_error(elapsed_ms, outcome.errors.clone()).emit();
        }

        let count = outcome.errors.len();
        eprintln!("  {} build failed — {count} error(s)", crate::ui::err());
        if let Ok(mut s) = shared.lock() {
            s.status = DevStatus::Failed {
                errors: outcome.errors,
                count,
            };
        }
        None
    }
}
