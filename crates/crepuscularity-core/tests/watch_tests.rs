#![cfg(feature = "notify")]
use crepuscularity_core::watch;
use notify::{Event, EventKind};

#[test]
fn test_is_relevant_kind() {
    assert!(watch::is_relevant_kind(&EventKind::Modify(
        notify::event::ModifyKind::Any
    )));
    assert!(watch::is_relevant_kind(&EventKind::Create(
        notify::event::CreateKind::Any
    )));
    assert!(watch::is_relevant_kind(&EventKind::Remove(
        notify::event::RemoveKind::Any
    )));
    assert!(!watch::is_relevant_kind(&EventKind::Access(
        notify::event::AccessKind::Any
    )));
    assert!(!watch::is_relevant_kind(&EventKind::Other));
}

#[test]
fn test_event_touches_relevant_path() {
    let target = std::path::PathBuf::from("/a/b/c/target.crepus");
    let watch_root = std::path::PathBuf::from("/a/b/c");

    // Exact match
    let event = Event {
        kind: EventKind::Modify(notify::event::ModifyKind::Any),
        paths: vec![std::path::PathBuf::from("/a/b/c/target.crepus")],
        attrs: Default::default(),
    };
    assert!(watch::event_touches_relevant_path(
        &event,
        &target,
        &watch_root
    ));

    // Sibling crepus
    let event = Event {
        kind: EventKind::Modify(notify::event::ModifyKind::Any),
        paths: vec![std::path::PathBuf::from("/a/b/c/sibling.crepus")],
        attrs: Default::default(),
    };
    assert!(watch::event_touches_relevant_path(
        &event,
        &target,
        &watch_root
    ));

    // context.toml
    let event = Event {
        kind: EventKind::Modify(notify::event::ModifyKind::Any),
        paths: vec![std::path::PathBuf::from("/a/b/c/context.toml")],
        attrs: Default::default(),
    };
    assert!(watch::event_touches_relevant_path(
        &event,
        &target,
        &watch_root
    ));

    // other toml
    let event = Event {
        kind: EventKind::Modify(notify::event::ModifyKind::Any),
        paths: vec![std::path::PathBuf::from("/a/b/c/other.toml")],
        attrs: Default::default(),
    };
    assert!(!watch::event_touches_relevant_path(
        &event,
        &target,
        &watch_root
    ));

    // unrelated extension
    let event = Event {
        kind: EventKind::Modify(notify::event::ModifyKind::Any),
        paths: vec![std::path::PathBuf::from("/a/b/c/unrelated.txt")],
        attrs: Default::default(),
    };
    assert!(!watch::event_touches_relevant_path(
        &event,
        &target,
        &watch_root
    ));

    // no extension
    let event = Event {
        kind: EventKind::Modify(notify::event::ModifyKind::Any),
        paths: vec![std::path::PathBuf::from("/a/b/c/noext")],
        attrs: Default::default(),
    };
    assert!(!watch::event_touches_relevant_path(
        &event,
        &target,
        &watch_root
    ));
}

#[test]
fn test_handle_watcher_event() {
    use std::sync::{Arc, Mutex};
    let changed = Arc::new(Mutex::new(false));
    let target = std::path::PathBuf::from("/a/b/c/target.crepus");
    let watch_root = std::path::PathBuf::from("/a/b/c");

    // Ok Event matching
    let event = Event {
        kind: EventKind::Modify(notify::event::ModifyKind::Any),
        paths: vec![std::path::PathBuf::from("/a/b/c/target.crepus")],
        attrs: Default::default(),
    };
    watch::handle_watcher_event(Ok(event), &changed, &target, &watch_root, "test");
    assert!(*changed.lock().unwrap());

    *changed.lock().unwrap() = false;

    // Ok Event not matching
    let event = Event {
        kind: EventKind::Modify(notify::event::ModifyKind::Any),
        paths: vec![std::path::PathBuf::from("/a/b/c/unrelated.txt")],
        attrs: Default::default(),
    };
    watch::handle_watcher_event(Ok(event), &changed, &target, &watch_root, "test");
    assert!(!*changed.lock().unwrap());

    // Err event shouldn't panic
    watch::handle_watcher_event(
        Err(notify::Error::generic("test error")),
        &changed,
        &target,
        &watch_root,
        "test",
    );
    assert!(!*changed.lock().unwrap());
}
