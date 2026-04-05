use std::cell::RefCell;
use std::rc::Rc;

use crate::batch::maybe_flush;
use crate::runtime::{
    alloc_id, mark_subscribers_dirty, track_read, AnyNode, NodeId, SignalNode, State, NODES,
};

pub struct Signal<T: Clone + PartialEq + 'static> {
    pub(crate) id: NodeId,
    value: Rc<RefCell<T>>,
}

impl<T: Clone + PartialEq + 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        let id = alloc_id();
        NODES.with(|n| {
            n.borrow_mut().insert(
                id,
                AnyNode::Signal(SignalNode {
                    state: State::Clean,
                    subscribers: vec![],
                }),
            )
        });
        Signal {
            id,
            value: Rc::new(RefCell::new(value)),
        }
    }

    pub fn get(&self) -> T {
        track_read(self.id);
        self.value.borrow().clone()
    }

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
