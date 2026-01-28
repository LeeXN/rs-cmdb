use super::{Message, MessageQueue};
use common::error::{CmdbError, CmdbResult};
use flume::{Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::time::Duration;

/// Flume-based implementation of MessageQueue
pub struct FlumeMessageQueue {
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

impl FlumeMessageQueue {
    /// Create a new flume-based message queue
    pub fn new() -> Self {
        let (sender, receiver) = flume::unbounded();
        Self { sender, receiver }
    }

    /// Get a cloned sender for this queue
    #[allow(dead_code)]
    pub fn get_sender(&self) -> Sender<Message> {
        self.sender.clone()
    }

    /// Get a cloned receiver for this queue
    #[allow(dead_code)]
    pub fn get_receiver(&self) -> Receiver<Message> {
        self.receiver.clone()
    }
}

impl MessageQueue for FlumeMessageQueue {
    fn send_message(&self, message: Message) -> CmdbResult<()> {
        self.sender
            .send(message)
            .map_err(|e| CmdbError::Other(format!("Failed to send message: {}", e)))
    }

    fn receive_message(&self, timeout: Duration) -> CmdbResult<Option<Message>> {
        match self.receiver.recv_timeout(timeout) {
            Ok(message) => Ok(Some(message)),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(e) => Err(CmdbError::Other(format!(
                "Failed to receive message: {}",
                e
            ))),
        }
    }

    fn is_empty(&self) -> bool {
        self.receiver.is_empty()
    }
}

/// A factory for creating message queues of different types
pub struct MessageQueueFactory;

impl MessageQueueFactory {
    /// Create a new flume-based message queue
    pub fn create_flume_queue() -> Arc<dyn MessageQueue> {
        Arc::new(FlumeMessageQueue::new())
    }
}
