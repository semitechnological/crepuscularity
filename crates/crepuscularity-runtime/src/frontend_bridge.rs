use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendMessage {
    pub channel: String,
    #[serde(default)]
    pub topic: String,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Debug, Default, Clone)]
pub struct FrontendBridge {
    inbound: Arc<Mutex<VecDeque<FrontendMessage>>>,
    outbound: Arc<Mutex<VecDeque<FrontendMessage>>>,
}

impl FrontendBridge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn send_to_runtime(&self, message: FrontendMessage) {
        self.outbound
            .lock()
            .expect("frontend bridge outbound queue poisoned")
            .push_back(message);
    }

    pub fn recv_for_runtime(&self) -> Option<FrontendMessage> {
        self.inbound
            .lock()
            .expect("frontend bridge inbound queue poisoned")
            .pop_front()
    }

    pub fn send_to_frontend(&self, message: FrontendMessage) {
        self.inbound
            .lock()
            .expect("frontend bridge inbound queue poisoned")
            .push_back(message);
    }

    pub fn recv_for_frontend(&self) -> Option<FrontendMessage> {
        self.outbound
            .lock()
            .expect("frontend bridge outbound queue poisoned")
            .pop_front()
    }
}
