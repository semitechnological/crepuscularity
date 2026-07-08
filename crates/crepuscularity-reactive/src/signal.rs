use std::cell::RefCell;
use std::rc::Rc;

use crate::batch::maybe_flush;
use crate::runtime::{
    alloc_id, mark_subscribers_dirty, track_read, AnyNode, NodeId, SignalNode, NODES,
};

/// Reactive value that notifies memos and effects when it changes.
pub struct Signal<T: Clone + PartialEq + 'static> {
    pub(crate) id: NodeId,
    value: Rc<RefCell<T>>,
}

impl<T: Clone + PartialEq + 'static> Signal<T> {
    /// Create a signal with an initial value.
    pub fn new(value: T) -> Self {
        let id = alloc_id();
        NODES.with(|n| {
            n.borrow_mut().insert(
                id,
                AnyNode::Signal(SignalNode {
                    subscribers: std::collections::HashSet::new(),
                }),
            )
        });
        Signal {
            id,
            value: Rc::new(RefCell::new(value)),
        }
    }

    /// Read the current value and subscribe the active memo or effect.
    pub fn get(&self) -> T {
        track_read(self.id);
        self.value.borrow().clone()
    }

    /// Replace the value and flush dependent effects unless a batch is active.
    pub fn set(&self, val: T) {
        {
            let mut v = self.value.borrow_mut();
            if *v == val {
                return;
            }
            *v = val;
        }
        mark_subscribers_dirty(self.id);
        maybe_flush();
    }

    /// Compute a replacement value from the previous value.
    pub fn update(&self, f: impl FnOnce(T) -> T) {
        let val = f(self.value.borrow().clone());
        self.set(val);
    }
}

impl<T: Clone + PartialEq + 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal {
            id: self.id,
            value: Rc::clone(&self.value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::Effect;
    use crate::runtime::{alloc_id, enter_observer, AnyNode, EffectNode, State, NODES};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_signal_new() {
        let sig = Signal::new(42);
        NODES.with(|nodes| {
            let nodes = nodes.borrow();
            match nodes.get(&sig.id).expect("node") {
                AnyNode::Signal(s) => assert!(s.subscribers.is_empty()),
                _ => panic!("not a signal"),
            }
        });
        assert_eq!(*sig.value.borrow(), 42);
    }

    #[test]
    fn test_signal_get() {
        assert_eq!(Signal::new(42).get(), 42);
    }

    #[test]
    fn test_signal_get_tracks_read() {
        let signal = Signal::new("hello".to_string());
        let observer_id = alloc_id();
        NODES.with(|n| {
            n.borrow_mut().insert(
                observer_id,
                AnyNode::Effect(EffectNode {
                    state: State::Clean,
                    sources: vec![],
                    run: Rc::new(|| {}),
                }),
            )
        });
        {
            let _guard = enter_observer(observer_id);
            assert_eq!(signal.get(), "hello");
        }
        let has_subscriber = NODES.with(|nodes| match nodes.borrow().get(&signal.id) {
            Some(AnyNode::Signal(s)) => s.subscribers.contains(&observer_id),
            _ => false,
        });
        assert!(has_subscriber);
        let has_source = NODES.with(|nodes| match nodes.borrow().get(&observer_id) {
            Some(AnyNode::Effect(e)) => e.sources.contains(&signal.id),
            _ => false,
        });
        assert!(has_source);
    }

    #[test]
    fn test_signal_set() {
        let sig = Signal::new(0);
        let counter = Rc::new(RefCell::new(0));
        let c_clone = counter.clone();
        let sig_clone = sig.clone();
        let _effect = Effect::new(move || {
            sig_clone.get();
            *c_clone.borrow_mut() += 1;
        });
        assert_eq!(*counter.borrow(), 1);
        sig.set(0);
        assert_eq!(*counter.borrow(), 1);
        sig.set(1);
        assert_eq!(*counter.borrow(), 2);
        assert_eq!(sig.get(), 1);
    }

    #[test]
    fn test_signal_update() {
        let signal = Signal::new(5);
        signal.update(|v| v + 10);
        assert_eq!(signal.get(), 15);
    }

    #[test]
    fn test_signal_update_multiple_times() {
        let signal = Signal::new(0);
        signal.update(|v| v + 1);
        signal.update(|v| v + 2);
        assert_eq!(signal.get(), 3);
    }

    #[test]
    #[should_panic(expected = "already borrowed")]
    fn test_signal_update_self_referential_panic() {
        let signal = Signal::new(5);
        let signal_clone = signal.clone();
        signal.update(|_| {
            signal_clone.set(10);
            10
        });
    }
}
