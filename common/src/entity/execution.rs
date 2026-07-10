use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn default_uuid() -> String {
    Uuid::new_v4().to_string()
}

fn default_now() -> String {
    Utc::now().to_rfc3339()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    #[default]
    Pending,
    Running,
    Success,
    Failed,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionType {
    Batch,
    #[default]
    Terminal,
}

impl std::fmt::Display for ExecutionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionType::Batch => write!(f, "batch"),
            ExecutionType::Terminal => write!(f, "terminal"),
        }
    }
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Pending => write!(f, "pending"),
            SessionStatus::Running => write!(f, "running"),
            SessionStatus::Success => write!(f, "success"),
            SessionStatus::Failed => write!(f, "failed"),
            SessionStatus::Partial => write!(f, "partial"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionSession {
    #[serde(default = "default_uuid")]
    pub session_id: String,
    pub user_id: String,
    pub username: String,
    pub client_ids: Vec<String>,
    #[serde(default)]
    pub execution_type: ExecutionType,
    pub command: String,
    pub status: SessionStatus,
    #[serde(default = "default_now")]
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_secs: Option<u64>,
    pub cast_file_path: Option<String>,
}

impl ExecutionSession {
    pub fn new(
        user_id: &str,
        username: &str,
        client_ids: Vec<String>,
        command: &str,
        execution_type: ExecutionType,
    ) -> Self {
        let now = default_now();
        Self {
            session_id: default_uuid(),
            user_id: user_id.to_string(),
            username: username.to_string(),
            client_ids,
            execution_type,
            command: command.to_string(),
            status: SessionStatus::Pending,
            start_time: now,
            end_time: None,
            duration_secs: None,
            cast_file_path: None,
        }
    }
}
