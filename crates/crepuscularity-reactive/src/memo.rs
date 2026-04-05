use std::rc::Rc;

use crate::runtime::{
    alloc_id, mark_subscribers_dirty, track_read, AnyNode, MemoEqFn, MemoNode, MemoRunFn, NodeId,
    State, NODES, RUNTIME,
};

pub struct Memo<T: Clone + PartialEq + 'static> {
    pub(crate) id: NodeId,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Clone + PartialEq + 'static> Memo<T> {
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
                    subscribers: vec![],
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
    // Clear old subscriptions
    let old_sources = NODES.with(|nodes| {
        let nodes = nodes.borrow();
        match nodes.get(&id) {
            Some(AnyNode::Memo(m)) => m.sources.clone(),
            _ => vec![],
        }
    });

    for source_id in &old_sources {
        NODES.with(|nodes| {
            let mut nodes = nodes.borrow_mut();
            match nodes.get_mut(source_id) {
                Some(AnyNode::Signal(s)) => s.subscribers.retain(|&x| x != id),
                Some(AnyNode::Memo(m)) => m.subscribers.retain(|&x| x != id),
                _ => {}
            }
        });
    }

    // Clear sources
    NODES.with(|nodes| {
        if let Some(AnyNode::Memo(m)) = nodes.borrow_mut().get_mut(&id) {
            m.sources.clear();
        }
    });

    // Set as current observer
    RUNTIME.with(|rt| rt.borrow_mut().current_observer = Some(id));

    // Extract and run the closure (outside the borrow)
    let run = NODES.with(|nodes| {
        let nodes = nodes.borrow();
        match nodes.get(&id) {
            Some(AnyNode::Memo(m)) => Some(Rc::clone(&m.run)),
            _ => None,
        }
    });

    let new_value = run.map(|f| f());

    // Restore observer
    RUNTIME.with(|rt| rt.borrow_mut().current_observer = None);

    if let Some(new_val) = new_value {
        // Compare with old cached value using the stored eq_fn
        let changed = NODES.with(|nodes| {
            let nodes = nodes.borrow();
            match nodes.get(&id) {
                Some(AnyNode::Memo(m)) => match &m.cached {
                    Some(old) => !(m.eq_fn)(old.as_ref(), new_val.as_ref()),
                    None => true,
                },
                _ => true,
            }
        });

        // Store new cached value
        NODES.with(|nodes| {
            if let Some(AnyNode::Memo(m)) = nodes.borrow_mut().get_mut(&id) {
                m.cached = Some(new_val);
                m.state = State::Clean;
            }
        });

        if changed {
            mark_subscribers_dirty(id);
        }
    }
}
