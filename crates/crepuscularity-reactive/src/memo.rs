use std::rc::Rc;

use crate::runtime::{
    alloc_id, clear_observer_sources, enter_observer, mark_subscribers_dirty, remove_node,
    track_read, AnyNode, MemoEqFn, MemoNode, MemoRunFn, NodeId, State, NODES,
};

/// Cached reactive computation that notifies dependents only when its value changes.
pub struct Memo<T: Clone + PartialEq + 'static> {
    pub(crate) id: NodeId,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Clone + PartialEq + 'static> Memo<T> {
    /// Create a lazy memo. The closure runs on the first [`Memo::get`] call.
    pub fn new(f: impl Fn() -> T + 'static) -> Self {
        let id = alloc_id();
        let run: MemoRunFn = Rc::new(move || Box::new(f()) as Box<dyn std::any::Any>);
        let eq_fn: MemoEqFn = Rc::new(|a: &dyn std::any::Any, b: &dyn std::any::Any| {
            match (a.downcast_ref::<T>(), b.downcast_ref::<T>()) {
                (Some(a), Some(b)) => a == b,
                _ => false,
            }
        });
        NODES.with(|n| {
            n.borrow_mut().insert(
                id,
                AnyNode::Memo(MemoNode {
                    state: State::Dirty,
                    sources: vec![],
                    subscribers: std::collections::HashSet::new(),
                    run,
                    cached: None,
                    eq_fn,
                }),
            )
        });
        Memo {
            id,
            _marker: std::marker::PhantomData,
        }
    }

    /// Return the current value, recomputing if an upstream dependency changed.
    pub fn get(&self) -> T {
        track_read(self.id);
        run_memo_if_needed(self.id);
        NODES.with(|nodes| {
            let nodes = nodes.borrow();
            match nodes.get(&self.id) {
                Some(AnyNode::Memo(m)) => m
                    .cached
                    .as_ref()
                    .and_then(|v| v.downcast_ref::<T>())
                    .cloned()
                    .expect("memo cached value must be T"),
                _ => panic!("memo node not found"),
            }
        })
    }

    /// Remove this memo from the reactive graph.
    ///
    /// All clones share the same graph node. Disposing one clone invalidates the others.
    pub fn dispose(self) {
        remove_node(self.id);
    }
}

impl<T: Clone + PartialEq + 'static> Clone for Memo<T> {
    fn clone(&self) -> Self {
        Memo {
            id: self.id,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Recompute memo if its state is not Clean.
pub(crate) fn run_memo_if_needed(id: NodeId) {
    let state = NODES.with(|nodes| {
        let nodes = nodes.borrow();
        match nodes.get(&id) {
            Some(AnyNode::Memo(m)) => m.state,
            _ => State::Clean,
        }
    });

    match state {
        State::Clean => {
            // Nothing to do — cached value is valid.
        }
        State::Check | State::Dirty => {
            // In both cases re-run the computation.
            // (Check means an upstream changed; we run and compare to decide if
            //  our own subscribers need notification. Dirty means we're definitely stale.)
            run_memo(id);
        }
    }
}

/// Run the memo computation, update cached value, and notify subscribers if the value changed.
pub(crate) fn run_memo(id: NodeId) {
    clear_observer_sources(id);

    let run = NODES.with(|nodes| {
        let nodes = nodes.borrow();
        match nodes.get(&id) {
            Some(AnyNode::Memo(m)) => Some(Rc::clone(&m.run)),
            _ => None,
        }
    });

    let new_value = run.map(|f| {
        let _observer = enter_observer(id);
        f()
    });

    if let Some(new_val) = new_value {
        let (had_cached, changed) = NODES.with(|nodes| {
            let nodes = nodes.borrow();
            match nodes.get(&id) {
                Some(AnyNode::Memo(m)) => match &m.cached {
                    Some(old) => (true, !(m.eq_fn)(old.as_ref(), new_val.as_ref())),
                    None => (false, true),
                },
                _ => (false, true),
            }
        });

        NODES.with(|nodes| {
            if let Some(AnyNode::Memo(m)) = nodes.borrow_mut().get_mut(&id) {
                m.cached = Some(new_val);
                m.state = State::Clean;
            }
        });

        if had_cached && changed {
            mark_subscribers_dirty(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{AnyNode, State, NODES};

    #[test]
    fn test_memo_new() {
        let memo = Memo::new(|| 42);
        NODES.with(|nodes| {
            let nodes = nodes.borrow();
            let node = nodes.get(&memo.id).expect("node");
            if let AnyNode::Memo(m) = node {
                assert_eq!(m.state, State::Dirty);
                assert!(m.cached.is_none());
                assert!(m.sources.is_empty());
                assert!(m.subscribers.is_empty());
            } else {
                panic!("not a memo");
            }
        });
    }

    #[test]
    fn test_memo_dispose() {
        let memo = Memo::new(|| 42);
        let id = memo.id;
        NODES.with(|nodes| assert!(nodes.borrow().contains_key(&id)));
        memo.dispose();
        NODES.with(|nodes| assert!(!nodes.borrow().contains_key(&id)));
    }

    #[test]
    fn test_memo_get_basic() {
        let memo = Memo::new(|| 100);
        assert_eq!(memo.get(), 100);
    }

    #[test]
    fn test_memo_get_reacts_to_signal() {
        use crate::signal::Signal;
        let sig = Signal::new(1);
        let sig_clone = sig.clone();
        let memo = Memo::new(move || sig_clone.get() * 2);

        assert_eq!(memo.get(), 2);

        sig.set(5);
        assert_eq!(memo.get(), 10);
    }

    #[test]
    fn test_memo_get_caches_value() {
        use crate::signal::Signal;
        use std::cell::RefCell;
        use std::rc::Rc;

        let sig = Signal::new(1);
        let sig_clone = sig.clone();
        let run_count = Rc::new(RefCell::new(0));
        let rc_clone = run_count.clone();

        let memo = Memo::new(move || {
            *rc_clone.borrow_mut() += 1;
            sig_clone.get() * 10
        });

        assert_eq!(*run_count.borrow(), 0);

        // First get triggers run
        assert_eq!(memo.get(), 10);
        assert_eq!(*run_count.borrow(), 1);

        // Second get uses cache
        assert_eq!(memo.get(), 10);
        assert_eq!(*run_count.borrow(), 1);

        // Update signal triggers run on next get
        sig.set(2);
        assert_eq!(memo.get(), 20);
        assert_eq!(*run_count.borrow(), 2);
    }
}
