pub mod message_queue;
pub mod mock_queue;

use common::error::CmdbResult;
use common::models::{Client, ClientHardwareInfo, PullRequest, PullResponse};
use std::time::Duration;

/// Message types for the message queue
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    /// Client registration message
    ClientRegistration(Client),
    /// Client hardware info message
    ClientHardwareInfo(ClientHardwareInfo),
    /// Pull request to a client
    PullRequest(PullRequest, String), // PullRequest and target client ID
    /// Pull response from a client
    PullResponse(PullResponse),
    /// Client heartbeat message (to update last seen timestamp)
    ClientHeartbeat(String), // Client ID
}

/// Message queue interface
pub trait MessageQueue: Send + Sync + 'static {
    /// Send a message to the queue
    fn send_message(&self, message: Message) -> CmdbResult<()>;

    /// Receive a message from the queue with timeout
    fn receive_message(&self, timeout: Duration) -> CmdbResult<Option<Message>>;

    /// Check if the queue is empty
    #[allow(dead_code)]
    fn is_empty(&self) -> bool;
}
