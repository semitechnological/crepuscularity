//! Demand-driven invalidation for active `.crepus` entries.
//!
//! Phase 10 prototype: track which entry routes/components are active in a dev
//! session, then recompute only active entries whose dependency set intersects
//! changed files. This is intentionally backend-agnostic so web, webext, TUI,
//! and GPUI dev servers can share the same invalidation rule.

use std::collections::{BTreeMap, BTreeSet};

/// Active-query invalidator for `.crepus` dev sessions.
#[derive(Debug, Default, Clone)]
pub struct ActiveInvalidator {
    active_entries: BTreeSet<String>,
    deps_by_entry: BTreeMap<String, BTreeSet<String>>,
}

impl ActiveInvalidator {
    /// Create an empty invalidator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark an entry as active. Only active entries are returned from
    /// [`Self::invalidated_entries`].
    pub fn activate(&mut self, entry: impl Into<String>) {
        self.active_entries.insert(normalize_key(&entry.into()));
    }

    /// Mark an entry as inactive.
    pub fn deactivate(&mut self, entry: &str) {
        self.active_entries.remove(&normalize_key(entry));
    }

    /// Replace the dependency set for an entry.
    pub fn set_dependencies<I, S>(&mut self, entry: impl Into<String>, deps: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let entry = normalize_key(&entry.into());
        let deps = deps
            .into_iter()
            .map(|dep| normalize_key(&dep.into()))
            .collect();
        self.deps_by_entry.insert(entry, deps);
    }

    /// Return active entries invalidated by `changed`.
    ///
    /// Entries with no dependency metadata are conservative: direct changes to
    /// the entry itself invalidate it, but unrelated files do not.
    pub fn invalidated_entries<I, S>(&self, changed: I) -> Vec<String>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let changed: BTreeSet<String> = changed
            .into_iter()
            .map(|path| normalize_key(&path.into()))
            .collect();

        self.active_entries
            .iter()
            .filter(|entry| {
                if changed.contains(*entry) {
                    return true;
                }
                self.deps_by_entry
                    .get(*entry)
                    .is_some_and(|deps| deps.iter().any(|dep| changed.contains(dep)))
            })
            .cloned()
            .collect()
    }

    /// True when `entry` is currently active.
    pub fn is_active(&self, entry: &str) -> bool {
        self.active_entries.contains(&normalize_key(entry))
    }
}

fn normalize_key(path: &str) -> String {
    path.replace('\\', "/").trim_start_matches("./").to_string()
}

#[cfg(test)]
mod tests {
    use super::ActiveInvalidator;

    #[test]
    fn active_entry_invalidates_on_direct_change() {
        let mut inv = ActiveInvalidator::new();
        inv.activate("pages/home.crepus");

        assert_eq!(
            inv.invalidated_entries(["pages/home.crepus"]),
            vec!["pages/home.crepus".to_string()]
        );
    }

    #[test]
    fn inactive_entry_is_not_invalidated() {
        let mut inv = ActiveInvalidator::new();
        inv.set_dependencies("pages/home.crepus", ["components/card.crepus"]);

        assert!(inv
            .invalidated_entries(["components/card.crepus"])
            .is_empty());
    }

    #[test]
    fn active_entry_invalidates_on_dependency_change() {
        let mut inv = ActiveInvalidator::new();
        inv.activate("pages/home.crepus");
        inv.set_dependencies("pages/home.crepus", ["components/card.crepus"]);

        assert_eq!(
            inv.invalidated_entries(["components/card.crepus"]),
            vec!["pages/home.crepus".to_string()]
        );
    }

    #[test]
    fn unrelated_change_does_not_invalidate_active_entry() {
        let mut inv = ActiveInvalidator::new();
        inv.activate("pages/home.crepus");
        inv.set_dependencies("pages/home.crepus", ["components/card.crepus"]);

        assert!(inv.invalidated_entries(["pages/about.crepus"]).is_empty());
    }

    #[test]
    fn normalizes_windows_paths() {
        let mut inv = ActiveInvalidator::new();
        inv.activate("pages/home.crepus");
        inv.set_dependencies("pages/home.crepus", ["components/card.crepus"]);

        assert_eq!(
            inv.invalidated_entries(["components\\card.crepus"]),
            vec!["pages/home.crepus".to_string()]
        );
    }
}
