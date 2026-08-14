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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_send_and_receive() {
        let queue = FlumeMessageQueue::new();
        queue
            .send_message(Message::ClientHeartbeat("test-node".into()))
            .unwrap();
        let received = queue.receive_message(Duration::from_secs(1)).unwrap();
        assert!(received.is_some());
        match received.unwrap() {
            Message::ClientHeartbeat(id) => assert_eq!(id, "test-node"),
            _ => panic!("Expected ClientHeartbeat"),
        }
    }

    #[tokio::test]
    async fn test_is_empty() {
        let queue = FlumeMessageQueue::new();
        assert!(queue.is_empty());
        queue
            .send_message(Message::ClientHeartbeat("node-1".into()))
            .unwrap();
        assert!(!queue.is_empty());
        let _ = queue.receive_message(Duration::from_secs(1)).unwrap();
        assert!(queue.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_messages() {
        let queue = FlumeMessageQueue::new();
        queue
            .send_message(Message::ClientHeartbeat("first".into()))
            .unwrap();
        queue
            .send_message(Message::ClientHeartbeat("second".into()))
            .unwrap();
        queue
            .send_message(Message::ClientHeartbeat("third".into()))
            .unwrap();

        let msg1 = queue
            .receive_message(Duration::from_secs(1))
            .unwrap()
            .unwrap();
        let msg2 = queue
            .receive_message(Duration::from_secs(1))
            .unwrap()
            .unwrap();
        let msg3 = queue
            .receive_message(Duration::from_secs(1))
            .unwrap()
            .unwrap();

        match msg1 {
            Message::ClientHeartbeat(id) => assert_eq!(id, "first"),
            _ => panic!("Expected ClientHeartbeat"),
        }
        match msg2 {
            Message::ClientHeartbeat(id) => assert_eq!(id, "second"),
            _ => panic!("Expected ClientHeartbeat"),
        }
        match msg3 {
            Message::ClientHeartbeat(id) => assert_eq!(id, "third"),
            _ => panic!("Expected ClientHeartbeat"),
        }
    }
}
