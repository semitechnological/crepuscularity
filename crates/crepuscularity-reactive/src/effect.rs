use std::rc::Rc;

use crate::runtime::{
    alloc_id, clear_observer_sources, enter_observer, remove_node, AnyNode, EffectNode, NodeId,
    State, NODES,
};

/// Reactive side effect that re-runs when signals or memos read inside it change.
pub struct Effect {
    pub(crate) id: NodeId,
}

impl Effect {
    /// Create an effect and run it once immediately to establish subscriptions.
    pub fn new(f: impl Fn() + 'static) -> Self {
        let id = alloc_id();
        NODES.with(|n| {
            n.borrow_mut().insert(
                id,
                AnyNode::Effect(EffectNode {
                    state: State::Dirty,
                    sources: vec![],
                    run: Rc::new(f),
                }),
            )
        });
        run_effect(id);
        Effect { id }
    }

    /// Remove this effect from the reactive graph.
    pub fn dispose(self) {
        remove_node(self.id);
    }
}

pub(crate) fn run_effect(id: NodeId) {
    clear_observer_sources(id);

    let run = NODES.with(|nodes| {
        let nodes = nodes.borrow();
        match nodes.get(&id) {
            Some(AnyNode::Effect(e)) => Some(Rc::clone(&e.run)),
            _ => None,
        }
    });

    let Some(run) = run else {
        return;
    };

    {
        let _observer = enter_observer(id);
        run();
    }

    NODES.with(|nodes| {
        if let Some(AnyNode::Effect(e)) = nodes.borrow_mut().get_mut(&id) {
            e.state = State::Clean;
        }
    });
}
