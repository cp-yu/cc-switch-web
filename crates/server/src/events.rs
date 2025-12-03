use serde_json::Value;
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub struct ServerEvent {
    pub name: String,
    pub payload: Value,
}

pub type EventSender = broadcast::Sender<ServerEvent>;
pub type EventReceiver = broadcast::Receiver<ServerEvent>;

pub fn create_event_bus(capacity: usize) -> EventSender {
    let (tx, _) = broadcast::channel(capacity);
    tx
}
