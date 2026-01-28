use super::{Message, MessageQueue};
use common::error::CmdbResult;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct MockMessageQueue {
    messages: Arc<Mutex<Vec<Message>>>,
}

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
