/// `crepu dev` — hot-reload dev loop.
///
/// Thread layout:
///   main thread  → GPUI Application::run (owns DevHUD window, blocks until closed)
///   background   → file watcher + cargo build + child process management
///
/// Communication: `Arc<Mutex<HudState>>` (polled by GPUI entity every 100ms)
/// Shutdown:      `Arc<AtomicBool>` set when GPUI window closes

use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use notify::{Event, EventKind, RecursiveMode, Watcher, recommended_watcher};

use crate::builder::{cargo_build, find_bin_name, kill_child};
use crate::hud::{DevStatus, HudState, open_hud_window};

pub fn run(bin_override: Option<String>, release: bool) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let bin_name = find_bin_name(&cwd, bin_override.as_deref()).unwrap_or_else(|| {
        eprintln!("[crepu dev] Could not determine binary name. Add a [[bin]] entry to Cargo.toml or use --bin.");
        std::process::exit(1);
    });

    let shared = Arc::new(Mutex::new(HudState::new(bin_name.clone())));
    let shutdown = Arc::new(AtomicBool::new(false));

    // Spawn background build+watch thread
    {
        let shared = shared.clone();
        let shutdown = shutdown.clone();
        let cwd = cwd.clone();
        let bin_name = bin_name.clone();
        std::thread::spawn(move || background_loop(shared, shutdown, cwd, bin_name, release));
    }

    // Run GPUI DevHUD on main thread (blocks until window is closed)
    {
        let shared = shared.clone();
        let shutdown = shutdown.clone();
        gpui::Application::new().run(move |cx: &mut gpui::App| {
            open_hud_window(shared, shutdown, cx);
        });
    }

    // Window closed — tell background thread to stop
    shutdown.store(true, Ordering::Relaxed);
}

// ── Background loop ────────────────────────────────────────────────────────

fn background_loop(
    shared: Arc<Mutex<HudState>>,
    shutdown: Arc<AtomicBool>,
    cwd: PathBuf,
    bin_name: String,
    release: bool,
) {
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    let tx_notify = tx.clone();

    let mut watcher = match recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(ev) = res {
            match ev.kind {
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                    let _ = tx_notify.send(());
                }
                _ => {}
            }
        }
    }) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("[crepu dev] Could not create file watcher: {e}");
            return;
        }
    };

    let src = cwd.join("src");
    if src.exists() {
        watcher.watch(&src, RecursiveMode::Recursive).ok();
    }
    watcher.watch(&cwd.join("Cargo.toml"), RecursiveMode::NonRecursive).ok();

    // Initial build + launch
    let mut child = do_build_launch(&shared, &cwd, &bin_name, release, None);

    loop {
        if shutdown.load(Ordering::Relaxed) {
            if let Some(mut c) = child { kill_child(&mut c); }
            break;
        }

        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(_) => {
                // Debounce: drain events for 300 ms
                let t = Instant::now();
                while t.elapsed() < Duration::from_millis(300) {
                    while rx.try_recv().is_ok() {}
                    std::thread::sleep(Duration::from_millis(30));
                }
                eprintln!("[crepu dev] Change detected — rebuilding…");
                child = do_build_launch(&shared, &cwd, &bin_name, release, child);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Check whether child exited on its own
                if let Some(ref mut c) = child {
                    match c.try_wait() {
                        Ok(Some(status)) => {
                            let code = status.code();
                            eprintln!("[crepu dev] App exited with code {code:?}");
                            if let Ok(mut s) = shared.lock() {
                                s.status = DevStatus::Exited { code };
                            }
                            child = None;
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

/// Find the compiled binary, checking the workspace root target dir first.
///
/// In a workspace, `cargo build` places binaries in `{workspace_root}/target/`
/// rather than in the crate's own directory. We use `cargo locate-project
/// --workspace` to find the workspace root, then fall back to `{cwd}/target/`.
fn locate_binary(cwd: &std::path::Path, profile: &str, bin_name: &str) -> PathBuf {
    // Ask cargo where the workspace root is
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
    release: bool,
    old_child: Option<Child>,
) -> Option<Child> {
    // Signal building
    if let Ok(mut s) = shared.lock() {
        s.status = DevStatus::Building;
    }

    // Kill old child
    if let Some(mut c) = old_child {
        kill_child(&mut c);
    }

    let t0 = Instant::now();
    let outcome = cargo_build(cwd, release, Some(shared.clone()));
    let elapsed_ms = t0.elapsed().as_millis() as u64;

    if outcome.success {
        if let Ok(mut s) = shared.lock() {
            s.status = DevStatus::Running { elapsed_ms };
        }
        let profile = if release { "release" } else { "debug" };
        let bin_path = locate_binary(cwd, profile, bin_name);

        eprintln!("[crepu dev] Built in {elapsed_ms} ms — launching {bin_name}");

        match Command::new(&bin_path).current_dir(cwd).spawn() {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("[crepu dev] Failed to launch {bin_name}: {e}");
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
        let count = outcome.errors.len();
        eprintln!("[crepu dev] Build failed with {count} error(s)");
        if let Ok(mut s) = shared.lock() {
            s.status = DevStatus::Failed { errors: outcome.errors, count };
        }
        None
    }
}
