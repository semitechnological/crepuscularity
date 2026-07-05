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
