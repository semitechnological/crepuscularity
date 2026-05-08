//! File watcher for the GPUI hot-reload path.
//!
//! Sets a shared flag when the watched template (or a sibling `.crepus`
//! component / `context.toml`) is modified. The flag is later consumed by
//! [`crate::HotReloadState`]'s polling loop, which re-parses and re-renders.
//!
//! # Why we watch the parent directory
//!
//! Watching a single file with `notify` is fragile: most editors save via
//! "write to a temp file, then `rename` over the original" (Vim with
//! `writebackup`, Helix's `tempfile::persist`, JetBrains' safe-write, modern
//! VS Code's `files.atomicSave`, etc.). On Linux/inotify that ties the watch
//! to the *inode* of the original file — after rename the inode is gone and
//! no further events fire.
//!
//! `notify`'s recommended workaround, which we follow here, is to watch the
//! parent directory recursively and filter events by path. This also lets us
//! transparently pick up edits to `include`d components (`include card.crepus`
//! living next to the entry file) without each include needing its own
//! watcher.
//!
//! # Lifetime
//!
//! [`create_watcher`] returns a boxed [`Watcher`] that the caller owns and
//! drops when it no longer needs change events; dropping the value tears down
//! `notify`'s internal worker threads cleanly. The legacy [`watch_file`]
//! helper preserves the old fire-and-forget behaviour by leaking the watcher
//! into a parked background thread — it remains for backward compatibility
//! and is no longer used by this crate.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};

/// Create and start a file-system watcher rooted at the parent directory of
/// `path`. The watcher flips `*changed = true` when:
///
/// - the watched template itself is modified / created / removed, or
/// - any sibling/descendant `.crepus` file changes (so `include`d components
///   trigger reload), or
/// - a `context.toml` next to the template is updated.
///
/// The returned value owns the `notify` worker; drop it to stop watching.
pub fn create_watcher(
    path: PathBuf,
    changed: Arc<Mutex<bool>>,
) -> Result<Box<dyn Watcher + Send>, String> {
    let canonical_target = path.canonicalize().unwrap_or_else(|_| path.clone());
    let watch_dir = canonical_target
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let target = canonical_target.clone();
    let root = watch_dir.clone();

    let mut watcher = recommended_watcher(move |res: notify::Result<Event>| match res {
        Ok(event) => {
            if !is_relevant_kind(&event.kind) {
                return;
            }
            if !event_touches_relevant_path(&event, &target, &root) {
                return;
            }
            if let Ok(mut flag) = changed.lock() {
                *flag = true;
            }
        }
        Err(e) => {
            eprintln!("[crepuscularity-runtime] watcher error: {e}");
        }
    })
    .map_err(|e| format!("could not create watcher: {e}"))?;

    watcher
        .watch(&watch_dir, RecursiveMode::Recursive)
        .map_err(|e| format!("failed to watch {}: {e}", watch_dir.display()))?;

    Ok(Box::new(watcher))
}

/// Legacy fire-and-forget wrapper around [`create_watcher`].
///
/// Spawns a background thread that owns the [`Watcher`] for the rest of the
/// process — convenient for once-per-process tools, but leaks the underlying
/// worker if the caller is instantiated more than once. New code should call
/// [`create_watcher`] and store the returned handle alongside the polling
/// state so it is dropped when the parent goes away.
#[deprecated(
    since = "0.4.3",
    note = "Use `create_watcher` and own the returned `Watcher` so it drops with your hot-reload state."
)]
pub fn watch_file(path: PathBuf, changed: Arc<Mutex<bool>>) {
    let watcher = match create_watcher(path, changed) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("[crepuscularity-runtime] {e}");
            return;
        }
    };
    thread::spawn(move || {
        let _keep_alive = watcher;
        thread::park();
    });
}

fn is_relevant_kind(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
    )
}

/// `true` when the event refers to either the canonicalized target template
/// or to a sibling/descendant `.crepus` / `context.toml` inside `watch_root`.
///
/// Pure function — split out for unit testing without spinning up a real
/// `notify` watcher.
pub(crate) fn event_touches_relevant_path(event: &Event, target: &Path, watch_root: &Path) -> bool {
    for path in &event.paths {
        if path_matches_target(path, target) {
            return true;
        }
        if path_is_relevant_sibling(path, watch_root) {
            return true;
        }
    }
    false
}

fn path_matches_target(path: &Path, target: &Path) -> bool {
    if path == target {
        return true;
    }
    let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    canon == *target
}

fn path_is_relevant_sibling(path: &Path, watch_root: &Path) -> bool {
    let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let canon_root = watch_root
        .canonicalize()
        .unwrap_or_else(|_| watch_root.to_path_buf());
    if !canon.starts_with(&canon_root) {
        return false;
    }
    match canon.extension().and_then(|e| e.to_str()) {
        Some("crepus") => true,
        Some("toml") if canon.file_name().and_then(|n| n.to_str()) == Some("context.toml") => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{CreateKind, ModifyKind, RemoveKind};
    use std::fs;

    fn ev(kind: EventKind, paths: Vec<PathBuf>) -> Event {
        Event {
            kind,
            paths,
            attrs: Default::default(),
        }
    }

    #[test]
    fn relevant_kinds_include_modify_create_remove() {
        assert!(is_relevant_kind(&EventKind::Modify(ModifyKind::Any)));
        assert!(is_relevant_kind(&EventKind::Create(CreateKind::Any)));
        assert!(is_relevant_kind(&EventKind::Remove(RemoveKind::Any)));
        assert!(!is_relevant_kind(&EventKind::Access(
            notify::event::AccessKind::Any
        )));
    }

    #[test]
    fn matches_when_event_path_equals_target() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("ui.crepus");
        fs::write(&target, "div\n  \"x\"").unwrap();
        let target = target.canonicalize().unwrap();

        let e = ev(
            EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Content)),
            vec![target.clone()],
        );
        assert!(event_touches_relevant_path(&e, &target, dir.path()));
    }

    #[test]
    fn matches_sibling_crepus_file_for_include_changes() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("ui.crepus");
        let included = dir.path().join("card.crepus");
        fs::write(&target, "div\n  \"x\"").unwrap();
        fs::write(&included, "div\n  \"y\"").unwrap();
        let target = target.canonicalize().unwrap();
        let included = included.canonicalize().unwrap();

        let e = ev(EventKind::Modify(ModifyKind::Any), vec![included.clone()]);
        assert!(event_touches_relevant_path(&e, &target, dir.path()));
    }

    #[test]
    fn matches_nested_include_in_subdirectory() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("components")).unwrap();
        let target = dir.path().join("ui.crepus");
        let nested = dir.path().join("components").join("button.crepus");
        fs::write(&target, "div").unwrap();
        fs::write(&nested, "button").unwrap();
        let target = target.canonicalize().unwrap();
        let nested = nested.canonicalize().unwrap();

        let e = ev(EventKind::Modify(ModifyKind::Any), vec![nested]);
        assert!(event_touches_relevant_path(&e, &target, dir.path()));
    }

    #[test]
    fn matches_context_toml_changes() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("ui.crepus");
        let toml = dir.path().join("context.toml");
        fs::write(&target, "div").unwrap();
        fs::write(&toml, "name = \"a\"").unwrap();
        let target = target.canonicalize().unwrap();
        let toml = toml.canonicalize().unwrap();

        let e = ev(EventKind::Modify(ModifyKind::Any), vec![toml]);
        assert!(event_touches_relevant_path(&e, &target, dir.path()));
    }

    #[test]
    fn ignores_unrelated_files_in_dir() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("ui.crepus");
        let unrelated = dir.path().join("notes.txt");
        fs::write(&target, "div").unwrap();
        fs::write(&unrelated, "...").unwrap();
        let target = target.canonicalize().unwrap();
        let unrelated = unrelated.canonicalize().unwrap();

        let e = ev(EventKind::Modify(ModifyKind::Any), vec![unrelated]);
        assert!(!event_touches_relevant_path(&e, &target, dir.path()));
    }

    #[test]
    fn ignores_random_toml_that_is_not_context_toml() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("ui.crepus");
        let other = dir.path().join("Cargo.toml");
        fs::write(&target, "div").unwrap();
        fs::write(&other, "[package]").unwrap();
        let target = target.canonicalize().unwrap();
        let other = other.canonicalize().unwrap();

        let e = ev(EventKind::Modify(ModifyKind::Any), vec![other]);
        assert!(!event_touches_relevant_path(&e, &target, dir.path()));
    }

    #[test]
    fn matches_target_via_remove_event_for_atomic_save() {
        // Simulates the atomic-save pattern: editor removes the original
        // and (in a separate event) creates a new file at the same path.
        // The Remove event still names the same path, so we react.
        let dir = tempfile::tempdir().unwrap();
        let target_path = dir.path().join("ui.crepus");
        fs::write(&target_path, "div").unwrap();
        let target_canon = target_path.canonicalize().unwrap();

        // After remove the path can't be canonicalized anymore — make sure
        // we still match by *path string* equality with the canonical target.
        fs::remove_file(&target_path).unwrap();
        let e = ev(
            EventKind::Remove(RemoveKind::File),
            vec![target_canon.clone()],
        );
        assert!(event_touches_relevant_path(&e, &target_canon, dir.path()));
    }

    #[test]
    fn create_watcher_is_droppable_without_leaking_a_thread() {
        // Smoke test: instantiate and drop many watchers in a row. Before
        // moving the `notify::Watcher` ownership into the caller this would
        // permanently leak one parked OS thread per call.
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("ui.crepus");
        fs::write(&target, "div").unwrap();

        for _ in 0..16 {
            let flag = Arc::new(Mutex::new(false));
            let watcher =
                create_watcher(target.clone(), Arc::clone(&flag)).expect("watcher should start");
            drop(watcher);
        }
    }

    #[test]
    fn create_watcher_observes_modifications_through_the_flag() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("ui.crepus");
        fs::write(&target, "div\n  \"x\"").unwrap();

        let flag = Arc::new(Mutex::new(false));
        let _watcher =
            create_watcher(target.clone(), Arc::clone(&flag)).expect("watcher should start");

        // Give the OS watcher a moment to begin observing. notify takes a
        // few ms to subscribe to events on most platforms.
        thread::sleep(std::time::Duration::from_millis(150));

        fs::write(&target, "div\n  \"y\"").unwrap();

        // Poll briefly for the flag to flip; on busy CI a single sleep can
        // race the notify worker.
        let mut saw_change = false;
        for _ in 0..40 {
            if *flag.lock().unwrap() {
                saw_change = true;
                break;
            }
            thread::sleep(std::time::Duration::from_millis(50));
        }
        assert!(saw_change, "watcher never flipped the changed flag");
    }
}
