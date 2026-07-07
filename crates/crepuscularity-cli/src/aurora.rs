use std::path::Path;
use std::process::{exit, Child, Command};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use console::style;

pub fn run_forwarded(args: &[String]) {
    if args.is_empty() {
        crate::aurora_usage::print_usage();
        return;
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| ".".into());
    let app_name = aurora_app_name(&cwd);
    let child_slot: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    let child_for_signal = Arc::clone(&child_slot);
    let app_name_for_signal = app_name.clone();

    if let Err(e) = ctrlc::set_handler(move || {
        stop_aurora_child(&child_for_signal);
        stop_aurora_app(app_name_for_signal.as_deref());
    }) {
        eprintln!(
            "{} could not install Ctrl-C handler: {}",
            style("crepus aurora").yellow(),
            e
        );
    }

    let child = Command::new("aurorality")
        .args(args)
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!(
                "{} failed to run aurorality: {}. {}",
                style("crepus aurora").red().bold(),
                e,
                style("Is aurorality installed? Run: cargo install aurorality-cli").dim()
            );
            exit(1);
        });

    {
        let mut slot = child_slot.lock().unwrap_or_else(|e| e.into_inner());
        *slot = Some(child);
    }

    let status = loop {
        let mut finished = None;
        {
            let mut slot = child_slot.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(child) = slot.as_mut() {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        finished = Some(status);
                    }
                    Ok(None) => {}
                    Err(e) => {
                        eprintln!(
                            "{} aurorality wait failed: {}",
                            style("crepus aurora").red(),
                            e
                        );
                        exit(1);
                    }
                }
            } else {
                stop_aurora_app(app_name.as_deref());
                exit(130);
            }
        }
        if let Some(status) = finished {
            let mut slot = child_slot.lock().unwrap_or_else(|e| e.into_inner());
            *slot = None;
            break status;
        }
        thread::sleep(Duration::from_millis(100));
    };

    if !status.success() {
        if args.first().is_some_and(|arg| arg == "dev") {
            stop_aurora_app(app_name.as_deref());
        }
        exit(status.code().unwrap_or(1));
    }
}

fn stop_aurora_child(child_slot: &Arc<Mutex<Option<Child>>>) {
    let mut slot = child_slot.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(mut child) = slot.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn stop_aurora_app(app_name: Option<&str>) {
    if let Some(app_name) = app_name {
        let _ = Command::new("pkill").args(["-f", app_name]).status();
    }
}

fn aurora_app_name(cwd: &Path) -> Option<String> {
    let content = std::fs::read_to_string(cwd.join("crepus.toml")).ok()?;
    let value = content.parse::<toml::Value>().ok()?;
    value
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(|name| name.as_str())
        .map(ToOwned::to_owned)
}
