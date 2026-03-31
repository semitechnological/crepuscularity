/// File watcher using the `notify` crate.
/// Sets a shared flag when the watched file changes.
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use notify::{recommended_watcher, Event, RecursiveMode, Watcher};

/// Spawns a background thread that watches `path` for modifications.
/// When a change is detected, sets `*changed = true`.
/// Returns the thread handle (caller should hold onto it or just detach).
pub fn watch_file(path: PathBuf, changed: Arc<Mutex<bool>>) {
    thread::spawn(move || {
        let changed_inner = changed.clone();
        let mut watcher = recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                use notify::EventKind;
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) => {
                        if let Ok(mut flag) = changed_inner.lock() {
                            *flag = true;
                        }
                    }
                    _ => {}
                }
            }
        });

        match watcher {
            Ok(ref mut w) => {
                if let Err(e) = w.watch(Path::new(&path), RecursiveMode::NonRecursive) {
                    eprintln!("[crepuscularity-dev] Failed to watch {:?}: {}", path, e);
                    return;
                }
                // Keep thread alive (and watcher alive) indefinitely
                loop {
                    thread::sleep(std::time::Duration::from_secs(3600));
                }
            }
            Err(e) => {
                eprintln!("[crepuscularity-dev] Could not create watcher: {}", e);
            }
        }
    });
}
