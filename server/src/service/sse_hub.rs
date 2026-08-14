//! SSE hub for broadcasting command log lines to browser subscribers.
//!
//! Each task has a channel. Clients subscribe by task_id; the agent
//! side publishes new log lines. When the task completes the channel is closed.

use axum::response::sse::Event;
use common::command::CommandLogLine;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

/// Number of log events buffered per channel before older ones are dropped.
const CHANNEL_CAPACITY: usize = 512;

/// A single per-task broadcast channel.
type Sender = broadcast::Sender<Arc<CommandLogLine>>;

/// The hub is shared via `Arc<SseHub>`.
pub struct SseHub {
    channels: Mutex<HashMap<String, Sender>>,
}

impl SseHub {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            channels: Mutex::new(HashMap::new()),
        })
    }

    /// Subscribe to log events for a task. Creates the channel if it doesn't
    /// exist yet (early subscriber before agent starts posting).
    pub async fn subscribe(&self, task_id: &str) -> broadcast::Receiver<Arc<CommandLogLine>> {
        let mut map = self.channels.lock().await;
        if let Some(tx) = map.get(task_id) {
            tx.subscribe()
        } else {
            let (tx, rx) = broadcast::channel(CHANNEL_CAPACITY);
            map.insert(task_id.to_string(), tx);
            rx
        }
    }

    /// Publish a batch of log lines to all browser subscribers of a task.
    pub async fn publish(&self, task_id: &str, lines: &[CommandLogLine]) {
        let map = self.channels.lock().await;
        if let Some(tx) = map.get(task_id) {
            for line in lines {
                // Ignore send errors (no subscribers yet / all disconnected).
                let _ = tx.send(Arc::new(line.clone()));
            }
        }
    }

    /// Close and remove the channel for a finished task.
    pub async fn close_task(&self, task_id: &str) {
        let mut map = self.channels.lock().await;
        map.remove(task_id);
    }

    /// Convert a `CommandLogLine` to an SSE `Event`.
    pub fn line_to_event(line: &CommandLogLine) -> Event {
        let data = serde_json::to_string(line).unwrap_or_default();
        Event::default().event("log").data(data)
    }

    /// Build a terminal-close SSE event (sent when task completes/fails).
    pub fn done_event(task_id: &str, exit_code: Option<i32>) -> Event {
        let payload = serde_json::json!({
            "task_id": task_id,
            "exit_code": exit_code,
        })
        .to_string();
        Event::default().event("done").data(payload)
    }
}

impl Default for SseHub {
    fn default() -> Self {
        Self {
            channels: Mutex::new(HashMap::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::command::LogStream;

    #[tokio::test]
    async fn test_subscribe_publish() {
        let hub = SseHub::new();
        let mut rx = hub.subscribe("task-1").await;

        let lines = vec![CommandLogLine {
            seq: 1,
            line: "hello".to_string(),
            stream: LogStream::Stdout,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }];
        hub.publish("task-1", &lines).await;

        let received = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv()).await;
        assert!(received.is_ok(), "Should receive message within timeout");
        let line = received.unwrap().unwrap();
        assert_eq!(line.seq, 1);
        assert_eq!(line.line, "hello");
    }

    #[tokio::test]
    async fn test_close_task() {
        let hub = SseHub::new();
        let _rx = hub.subscribe("task-2").await;

        hub.close_task("task-2").await;

        let lines = vec![CommandLogLine {
            seq: 1,
            line: "after close".to_string(),
            stream: LogStream::Stdout,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }];
        // Should not panic even though channel was removed
        hub.publish("task-2", &lines).await;
    }

    #[test]
    fn test_line_to_event() {
        let line = CommandLogLine {
            seq: 42,
            line: "test content".to_string(),
            stream: LogStream::Stdout,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
        };
        // Verify JSON serialization (core logic of line_to_event)
        let json = serde_json::to_string(&line).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["seq"], 42);
        assert_eq!(parsed["line"], "test content");
        assert_eq!(parsed["stream"], "stdout");

        // Verify Event construction does not panic
        let _event = SseHub::line_to_event(&line);
    }

    #[test]
    fn test_done_event() {
        // Verify JSON payload construction (core logic of done_event)
        let payload = serde_json::json!({
            "task_id": "task-99",
            "exit_code": 0,
        })
        .to_string();
        let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(parsed["task_id"], "task-99");
        assert_eq!(parsed["exit_code"], 0);

        // Verify Event construction does not panic
        let _event = SseHub::done_event("task-99", Some(0));
    }
}
