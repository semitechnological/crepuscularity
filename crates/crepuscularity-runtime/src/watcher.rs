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

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};

/// Spawn a background thread that watches the directory containing `path`
/// recursively, and flips `*changed = true` when:
///
/// - the watched template itself is modified / created / removed, or
/// - any sibling/descendant `.crepus` file changes (so `include`d components
///   trigger reload), or
/// - a `context.toml` next to the template is updated.
///
/// The thread runs for the rest of the process lifetime — `notify` requires
/// the [`Watcher`] value to stay alive for events to fire, so we park inside
/// the thread on a long sleep loop.
pub fn watch_file(path: PathBuf, changed: Arc<Mutex<bool>>) {
    let canonical_target = path.canonicalize().unwrap_or_else(|_| path.clone());
    let watch_dir = canonical_target
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    thread::spawn(move || {
        let changed_inner = Arc::clone(&changed);
        let target = canonical_target.clone();
        let root = watch_dir.clone();

        let mut watcher = match recommended_watcher(move |res: notify::Result<Event>| match res {
            Ok(event) => {
                if !is_relevant_kind(&event.kind) {
                    return;
                }
                if !event_touches_relevant_path(&event, &target, &root) {
                    return;
                }
                if let Ok(mut flag) = changed_inner.lock() {
                    *flag = true;
                }
            }
            Err(e) => {
                eprintln!("[crepuscularity-dev] watcher error: {e}");
            }
        }) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("[crepuscularity-dev] could not create watcher: {e}");
                return;
            }
        };

        if let Err(e) = watcher.watch(&watch_dir, RecursiveMode::Recursive) {
            eprintln!(
                "[crepuscularity-dev] failed to watch {}: {e}",
                watch_dir.display()
            );
            return;
        }

        loop {
            thread::sleep(std::time::Duration::from_secs(3600));
        }
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
}
