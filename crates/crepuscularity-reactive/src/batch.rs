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
    use crate::effect::Effect;
    use crate::runtime::{alloc_id, AnyNode, EffectNode, State, NODES, RUNTIME};
    use crate::signal::Signal;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    #[test]
    fn test_batch_begin_increments_depth() {
        let initial_depth = RUNTIME.with(|rt| rt.borrow().batch_depth);
        batch_begin();
        assert_eq!(
            RUNTIME.with(|rt| rt.borrow().batch_depth),
            initial_depth + 1
        );
        batch_end();
        assert_eq!(RUNTIME.with(|rt| rt.borrow().batch_depth), initial_depth);
    }

    #[test]
    fn test_nested_batches() {
        let initial_depth = RUNTIME.with(|rt| rt.borrow().batch_depth);
        batch_begin();
        batch_begin();
        assert_eq!(
            RUNTIME.with(|rt| rt.borrow().batch_depth),
            initial_depth + 2
        );
        batch_end();
        assert_eq!(
            RUNTIME.with(|rt| rt.borrow().batch_depth),
            initial_depth + 1
        );
        batch_end();
        assert_eq!(RUNTIME.with(|rt| rt.borrow().batch_depth), initial_depth);
    }

    #[test]
    fn test_batch_end_saturating_sub() {
        RUNTIME.with(|rt| rt.borrow_mut().batch_depth = 0);
        batch_end();
        assert_eq!(RUNTIME.with(|rt| rt.borrow().batch_depth), 0);
    }

    #[test]
    fn test_batch_defers_effects() {
        let signal = Signal::new(0);
        let run_count = Rc::new(Cell::new(0));
        let run_count_clone = run_count.clone();
        let signal_clone = signal.clone();
        let _effect = Effect::new(move || {
            signal_clone.get();
            run_count_clone.set(run_count_clone.get() + 1);
        });
        assert_eq!(run_count.get(), 1);
        batch_begin();
        signal.set(1);
        signal.set(2);
        assert_eq!(run_count.get(), 1);
        batch_end();
        assert_eq!(run_count.get(), 2);
    }

    #[test]
    fn test_flush_mock_effects() {
        let run_count = Rc::new(RefCell::new(0));
        let rc1 = Rc::clone(&run_count);
        let id1 = alloc_id();
        NODES.with(|n| {
            n.borrow_mut().insert(
                id1,
                AnyNode::Effect(EffectNode {
                    state: State::Dirty,
                    sources: vec![],
                    run: Rc::new(move || *rc1.borrow_mut() += 1),
                }),
            )
        });
        let rc2 = Rc::clone(&run_count);
        let id2 = alloc_id();
        NODES.with(|n| {
            n.borrow_mut().insert(
                id2,
                AnyNode::Effect(EffectNode {
                    state: State::Dirty,
                    sources: vec![],
                    run: Rc::new(move || *rc2.borrow_mut() += 1),
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
        assert!(RUNTIME.with(|rt| rt.borrow().pending_effects.is_empty()));
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
        let rc2 = Rc::clone(&run_count);
        NODES.with(|n| {
            n.borrow_mut().insert(
                id2,
                AnyNode::Effect(EffectNode {
                    state: State::Dirty,
                    sources: vec![],
                    run: Rc::new(move || *rc2.borrow_mut() += 1),
                }),
            )
        });
        let id1 = alloc_id();
        let rc1 = Rc::clone(&run_count);
        NODES.with(|n| {
            n.borrow_mut().insert(
                id1,
                AnyNode::Effect(EffectNode {
                    state: State::Dirty,
                    sources: vec![],
                    run: Rc::new(move || {
                        *rc1.borrow_mut() += 1;
                        RUNTIME.with(|rt| rt.borrow_mut().pending_effects.push(id2));
                    }),
                }),
            )
        });
        RUNTIME.with(|rt| rt.borrow_mut().pending_effects.push(id1));
        flush();
        assert_eq!(*run_count.borrow(), 2);
        assert!(RUNTIME.with(|rt| rt.borrow().pending_effects.is_empty()));
        NODES.with(|n| {
            let mut n = n.borrow_mut();
            n.remove(&id1);
            n.remove(&id2);
        });
    }
}
