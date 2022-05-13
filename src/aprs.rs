use tokio::sync::broadcast::{Receiver, Sender};

/// Defines a simple container for an inbound message
#[derive(Debug, Clone)]
pub struct InboundMessage {
    /// Sender of the message
    pub respond_to: String,
    /// Message body
    pub message: String,
}

#[derive(Debug)]
pub struct AprsMessagingClient {
    message_sender: Sender<InboundMessage>,
}

impl AprsMessagingClient {
    pub async fn connect(callsign: &str) -> Result<Self, ()> {
        // Set up channels
        let (message_sender, _) = tokio::sync::broadcast::channel(100);

        Ok(Self { message_sender })
    }

    pub fn subscribe(&self) -> Receiver<InboundMessage> {
        self.message_sender.subscribe()
    }

    

}
