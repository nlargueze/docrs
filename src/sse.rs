//! SSE manager

use axum::response::sse;
use tokio::sync::broadcast;

/// SSE event manager
#[derive(Debug, Clone)]
pub struct SeeEventManager {
    /// sender
    sender: broadcast::Sender<sse::Event>,
}

impl SeeEventManager {
    /// Instantiates a [SeeEventManager]
    pub fn new(sender: broadcast::Sender<sse::Event>) -> Self {
        Self { sender }
    }

    /// Sends a reload event
    pub fn reload(&self) {
        eprintln!("↳ SSE reload event");
        let event = sse::Event::default().event("reload").data("hey");
        self.sender.send(event).unwrap();
    }
}
