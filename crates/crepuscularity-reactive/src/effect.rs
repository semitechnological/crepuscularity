use std::rc::Rc;

use crate::runtime::{alloc_id, AnyNode, EffectNode, NodeId, State, NODES, RUNTIME};

pub struct Effect {
    #[allow(dead_code)]
    pub(crate) id: NodeId,
}

impl Effect {
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
}

pub(crate) fn run_effect(id: NodeId) {
    // 1. Clear old subscriptions: for each source in sources, remove id from source.subscribers
    let old_sources = NODES.with(|nodes| {
        let nodes = nodes.borrow();
        match nodes.get(&id) {
            Some(AnyNode::Effect(e)) => e.sources.clone(),
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

    // 2. Clear sources list
    NODES.with(|nodes| {
        if let Some(AnyNode::Effect(e)) = nodes.borrow_mut().get_mut(&id) {
            e.sources.clear();
        }
    });

    // 3. Set RUNTIME.current_observer = Some(id)
    RUNTIME.with(|rt| rt.borrow_mut().current_observer = Some(id));

    // 4. Extract the run closure and call it outside the borrow
    let run = NODES.with(|nodes| {
        let nodes = nodes.borrow();
        match nodes.get(&id) {
            Some(AnyNode::Effect(e)) => Some(Rc::clone(&e.run)),
            _ => None,
        }
    });

    if let Some(run) = run {
        run();
    }

    // 5. Restore RUNTIME.current_observer = None
    RUNTIME.with(|rt| rt.borrow_mut().current_observer = None);

    // 6. Set node state = Clean
    NODES.with(|nodes| {
        if let Some(AnyNode::Effect(e)) = nodes.borrow_mut().get_mut(&id) {
            e.state = State::Clean;
        }
    });
}
