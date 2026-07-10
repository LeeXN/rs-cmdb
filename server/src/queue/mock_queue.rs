use super::{Message, MessageQueue};
use common::error::CmdbResult;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[allow(dead_code)]
pub struct MockMessageQueue {
    messages: Arc<Mutex<Vec<Message>>>,
}

#[allow(dead_code)]
impl MockMessageQueue {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_messages(&self) -> Vec<Message> {
        self.messages.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.messages.lock().unwrap().clear();
    }
}

impl MessageQueue for MockMessageQueue {
    fn send_message(&self, message: Message) -> CmdbResult<()> {
        self.messages.lock().unwrap().push(message);
        Ok(())
    }

    fn receive_message(&self, _timeout: Duration) -> CmdbResult<Option<Message>> {
        let mut messages = self.messages.lock().unwrap();
        if messages.is_empty() {
            Ok(None)
        } else {
            Ok(Some(messages.remove(0)))
        }
    }

    fn is_empty(&self) -> bool {
        self.messages.lock().unwrap().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_send_and_receive() {
        let queue = MockMessageQueue::new();
        queue.send_message(Message::ClientHeartbeat("test-node".into())).unwrap();
        let received = queue.receive_message(Duration::from_secs(1)).unwrap();
        assert!(received.is_some());
        match received.unwrap() {
            Message::ClientHeartbeat(id) => assert_eq!(id, "test-node"),
            _ => panic!("Expected ClientHeartbeat"),
        }
    }

    #[tokio::test]
    async fn test_get_messages() {
        let queue = MockMessageQueue::new();
        queue.send_message(Message::ClientHeartbeat("first".into())).unwrap();
        queue.send_message(Message::ClientHeartbeat("second".into())).unwrap();
        let messages = queue.get_messages();
        assert_eq!(messages.len(), 2);
        match &messages[0] {
            Message::ClientHeartbeat(id) => assert_eq!(id, "first"),
            _ => panic!("Expected ClientHeartbeat"),
        }
        match &messages[1] {
            Message::ClientHeartbeat(id) => assert_eq!(id, "second"),
            _ => panic!("Expected ClientHeartbeat"),
        }
    }

    #[tokio::test]
    async fn test_clear() {
        let queue = MockMessageQueue::new();
        queue.send_message(Message::ClientHeartbeat("node-1".into())).unwrap();
        assert!(!queue.is_empty());
        queue.clear();
        assert!(queue.is_empty());
    }

    #[tokio::test]
    async fn test_receive_empty() {
        let queue = MockMessageQueue::new();
        let received = queue.receive_message(Duration::from_secs(1)).unwrap();
        assert!(received.is_none());
    }
}
