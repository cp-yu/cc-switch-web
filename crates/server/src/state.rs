use std::sync::Arc;
use crate::events::EventSender;

pub struct ServerState {
    pub auth_token: Option<String>,
    pub event_bus: EventSender,
}

impl ServerState {
    pub fn new(auth_token: Option<String>, event_bus: EventSender) -> Arc<Self> {
        Arc::new(Self {
            auth_token,
            event_bus,
        })
    }
}
