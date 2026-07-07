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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::{batch_begin, batch_end};
    use crate::runtime::{AnyNode, NODES, RUNTIME};
    use crate::signal::Signal;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_effect_dispose() {
        let effect = Effect::new(|| {});
        let id = effect.id;

        // Verify it was added to NODES
        let exists_before = NODES.with(|nodes| nodes.borrow().contains_key(&id));
        assert!(exists_before, "Effect should exist in NODES after creation");

        effect.dispose();

        // Verify it was removed from NODES
        let exists_after = NODES.with(|nodes| nodes.borrow().contains_key(&id));
        assert!(
            !exists_after,
            "Effect should be removed from NODES after dispose"
        );
    }

    #[test]
    fn test_effect_dispose_clears_subscriptions() {
        let signal = Signal::new(10);
        let signal_id = signal.id;
        let signal_clone = signal.clone();

        let effect = Effect::new(move || {
            let _ = signal_clone.get();
        });
        let effect_id = effect.id;

        let is_subscribed = NODES.with(|nodes| {
            let nodes = nodes.borrow();
            if let Some(AnyNode::Signal(s)) = nodes.get(&signal_id) {
                s.subscribers.contains(&effect_id)
            } else {
                false
            }
        });
        assert!(is_subscribed, "Effect should be subscribed to signal");

        effect.dispose();

        let is_subscribed_after = NODES.with(|nodes| {
            let nodes = nodes.borrow();
            if let Some(AnyNode::Signal(s)) = nodes.get(&signal_id) {
                s.subscribers.contains(&effect_id)
            } else {
                false
            }
        });
        assert!(
            !is_subscribed_after,
            "Effect should be removed from signal's subscribers after dispose"
        );
    }

    #[test]
    fn test_effect_dispose_removes_from_pending_effects() {
        let signal = Signal::new(10);
        let signal_clone = signal.clone();
        let effect = Effect::new(move || {
            let _ = signal_clone.get();
        });
        let effect_id = effect.id;

        batch_begin();
        signal.set(20);

        let is_pending = RUNTIME.with(|rt| rt.borrow().pending_effects.contains(&effect_id));
        assert!(
            is_pending,
            "Effect should be in pending_effects before dispose"
        );

        effect.dispose();

        let is_pending_after = RUNTIME.with(|rt| rt.borrow().pending_effects.contains(&effect_id));
        assert!(
            !is_pending_after,
            "Effect should be removed from pending_effects after dispose"
        );

        batch_end();
    }

    #[test]
    fn test_effect_dispose_during_execution() {
        let effect_handle: Rc<RefCell<Option<Effect>>> = Rc::new(RefCell::new(None));
        let handle_clone = Rc::clone(&effect_handle);
        let signal = Signal::new(10);
        let signal_clone = signal.clone();
        let runs = Rc::new(RefCell::new(0));
        let runs_clone = Rc::clone(&runs);

        let effect = Effect::new(move || {
            let _ = signal_clone.get();
            *runs_clone.borrow_mut() += 1;

            if let Some(e) = handle_clone.borrow_mut().take() {
                e.dispose();
            }
        });

        *effect_handle.borrow_mut() = Some(effect);
        signal.set(20);
        assert_eq!(*runs.borrow(), 2);
    }
}
