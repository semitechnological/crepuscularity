use std::collections::HashMap;

pub(crate) type NodeId = u32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum State {
    Clean,
    Check,
    Dirty,
}

pub(crate) struct ReactiveRuntime {
    pub current_observer: Option<NodeId>,
    pub pending_effects: Vec<NodeId>,
    pub batch_depth: u32,
    #[allow(dead_code)]
    pub flush_scheduled: bool,
    pub next_id: NodeId,
}

impl ReactiveRuntime {
    fn new() -> Self {
        ReactiveRuntime {
            current_observer: None,
            pending_effects: Vec::new(),
            batch_depth: 0,
            flush_scheduled: false,
            next_id: 1,
        }
    }
}

pub(crate) struct SignalNode {
    #[allow(dead_code)]
    pub state: State,
    pub subscribers: Vec<NodeId>,
}

pub(crate) type MemoRunFn = std::rc::Rc<dyn Fn() -> Box<dyn std::any::Any>>;
pub(crate) type MemoEqFn = std::rc::Rc<dyn Fn(&dyn std::any::Any, &dyn std::any::Any) -> bool>;

pub(crate) struct MemoNode {
    pub state: State,
    pub sources: Vec<NodeId>,
    pub subscribers: Vec<NodeId>,
    /// Returns new value as Box<dyn Any>
    pub run: MemoRunFn,
    pub cached: Option<Box<dyn std::any::Any>>,
    /// Compare old and new values; returns true if they are equal (no change)
    pub eq_fn: MemoEqFn,
}

pub(crate) struct EffectNode {
    pub state: State,
    pub sources: Vec<NodeId>,
    pub run: std::rc::Rc<dyn Fn()>,
}

pub(crate) enum AnyNode {
    Signal(SignalNode),
    Memo(MemoNode),
    Effect(EffectNode),
}

thread_local! {
    pub(crate) static RUNTIME: std::cell::RefCell<ReactiveRuntime> =
        std::cell::RefCell::new(ReactiveRuntime::new());

    pub(crate) static NODES: std::cell::RefCell<HashMap<NodeId, AnyNode>> =
        std::cell::RefCell::new(HashMap::new());
}

pub(crate) fn alloc_id() -> NodeId {
    RUNTIME.with(|rt| {
        let mut rt = rt.borrow_mut();
        let id = rt.next_id;
        rt.next_id += 1;
        id
    })
}

/// Record that `source_id` was read by the current observer.
pub(crate) fn track_read(source_id: NodeId) {
    let observer = RUNTIME.with(|rt| rt.borrow().current_observer);
    let Some(observer_id) = observer else {
        return;
    };
    NODES.with(|nodes| {
        let mut nodes = nodes.borrow_mut();

        // Add source_id to observer's sources list
        match nodes.get_mut(&observer_id) {
            Some(AnyNode::Memo(m)) => {
                if !m.sources.contains(&source_id) {
                    m.sources.push(source_id);
                }
            }
            Some(AnyNode::Effect(e)) => {
                if !e.sources.contains(&source_id) {
                    e.sources.push(source_id);
                }
            }
            _ => {}
        }

        // Add observer_id to source's subscribers list
        match nodes.get_mut(&source_id) {
            Some(AnyNode::Signal(s)) => {
                if !s.subscribers.contains(&observer_id) {
                    s.subscribers.push(observer_id);
                }
            }
            Some(AnyNode::Memo(m)) => {
                if !m.subscribers.contains(&observer_id) {
                    m.subscribers.push(observer_id);
                }
            }
            _ => {}
        }
    });
}

/// Mark subscribers of `source_id` as dirty (effects) or check (memos).
pub(crate) fn mark_subscribers_dirty(source_id: NodeId) {
    let subscribers = NODES.with(|nodes| {
        let nodes = nodes.borrow();
        match nodes.get(&source_id) {
            Some(AnyNode::Signal(s)) => s.subscribers.clone(),
            Some(AnyNode::Memo(m)) => m.subscribers.clone(),
            _ => vec![],
        }
    });

    for sub_id in subscribers {
        let is_effect = NODES.with(|nodes| {
            let nodes = nodes.borrow();
            matches!(nodes.get(&sub_id), Some(AnyNode::Effect(_)))
        });

        if is_effect {
            NODES.with(|nodes| {
                if let Some(AnyNode::Effect(e)) = nodes.borrow_mut().get_mut(&sub_id) {
                    e.state = State::Dirty;
                }
            });
            RUNTIME.with(|rt| {
                let mut rt = rt.borrow_mut();
                if !rt.pending_effects.contains(&sub_id) {
                    rt.pending_effects.push(sub_id);
                }
            });
        } else {
            // It's a memo: mark as Check and propagate
            NODES.with(|nodes| {
                if let Some(AnyNode::Memo(m)) = nodes.borrow_mut().get_mut(&sub_id) {
                    if m.state == State::Clean {
                        m.state = State::Check;
                    }
                }
            });
            // Propagate Check to memo's subscribers
            mark_subscribers_dirty(sub_id);
        }
    }
}
