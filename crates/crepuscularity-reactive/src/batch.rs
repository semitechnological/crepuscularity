use crate::effect::run_effect;
use crate::runtime::RUNTIME;

pub fn batch_begin() {
    RUNTIME.with(|rt| rt.borrow_mut().batch_depth += 1);
}

pub fn batch_end() {
    let should = RUNTIME.with(|rt| {
        let mut rt = rt.borrow_mut();
        rt.batch_depth = rt.batch_depth.saturating_sub(1);
        rt.batch_depth == 0
    });
    if should {
        flush();
    }
}

pub(crate) fn maybe_flush() {
    let should = RUNTIME.with(|rt| rt.borrow().batch_depth == 0);
    if should {
        flush();
    }
}

pub fn flush() {
    loop {
        let effects: Vec<u32> =
            RUNTIME.with(|rt| std::mem::take(&mut rt.borrow_mut().pending_effects));
        if effects.is_empty() {
            break;
        }
        for id in effects {
            run_effect(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{alloc_id, AnyNode, EffectNode, State, NODES, RUNTIME};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_flush_mock_effects() {
        let run_count = Rc::new(RefCell::new(0));

        let rc_clone1 = Rc::clone(&run_count);
        let id1 = alloc_id();
        NODES.with(|n| {
            n.borrow_mut().insert(
                id1,
                AnyNode::Effect(EffectNode {
                    state: State::Dirty,
                    sources: vec![],
                    run: Rc::new(move || {
                        *rc_clone1.borrow_mut() += 1;
                    }),
                }),
            )
        });

        let rc_clone2 = Rc::clone(&run_count);
        let id2 = alloc_id();
        NODES.with(|n| {
            n.borrow_mut().insert(
                id2,
                AnyNode::Effect(EffectNode {
                    state: State::Dirty,
                    sources: vec![],
                    run: Rc::new(move || {
                        *rc_clone2.borrow_mut() += 1;
                    }),
                }),
            )
        });

        RUNTIME.with(|rt| {
            let mut rt = rt.borrow_mut();
            rt.pending_effects.push(id1);
            rt.pending_effects.push(id2);
        });

        flush();

        assert_eq!(*run_count.borrow(), 2);

        let pending_empty = RUNTIME.with(|rt| rt.borrow().pending_effects.is_empty());
        assert!(pending_empty, "flush should clear pending effects");

        NODES.with(|n| {
            let mut n = n.borrow_mut();
            n.remove(&id1);
            n.remove(&id2);
        });
    }

    #[test]
    fn test_flush_nested_effects() {
        let run_count = Rc::new(RefCell::new(0));

        let id2 = alloc_id();
        let rc_clone2 = Rc::clone(&run_count);
        NODES.with(|n| {
            n.borrow_mut().insert(
                id2,
                AnyNode::Effect(EffectNode {
                    state: State::Dirty,
                    sources: vec![],
                    run: Rc::new(move || {
                        *rc_clone2.borrow_mut() += 1;
                    }),
                }),
            )
        });

        let id1 = alloc_id();
        let rc_clone1 = Rc::clone(&run_count);
        NODES.with(|n| {
            n.borrow_mut().insert(
                id1,
                AnyNode::Effect(EffectNode {
                    state: State::Dirty,
                    sources: vec![],
                    run: Rc::new(move || {
                        *rc_clone1.borrow_mut() += 1;
                        RUNTIME.with(|rt| rt.borrow_mut().pending_effects.push(id2));
                    }),
                }),
            )
        });

        RUNTIME.with(|rt| {
            rt.borrow_mut().pending_effects.push(id1);
        });

        flush();

        assert_eq!(*run_count.borrow(), 2);

        let pending_empty = RUNTIME.with(|rt| rt.borrow().pending_effects.is_empty());
        assert!(pending_empty, "flush should handle nested enqueues");

        NODES.with(|n| {
            let mut n = n.borrow_mut();
            n.remove(&id1);
            n.remove(&id2);
        });
    }
}
