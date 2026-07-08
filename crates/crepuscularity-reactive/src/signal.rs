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
    use std::cell::RefCell;
    use std::rc::Rc;

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

        // Effect runs once on creation
        assert_eq!(*counter.borrow(), 1);

        // Setting to same value shouldn't trigger effect
        sig.set(0);
        assert_eq!(*counter.borrow(), 1);

        // Setting to new value should trigger effect
        sig.set(1);
        assert_eq!(*counter.borrow(), 2);

        // Value should be updated
        assert_eq!(sig.get(), 1);
    }
}
