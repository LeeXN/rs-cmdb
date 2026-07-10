use crate::db::Database;
use chrono::DateTime;
use common::command::{AuditLogEntry, CommandLogLine, CommandStatus, CommandTask};
use common::error::{CmdbError, CmdbResult};
use serde_json;
use std::sync::Arc;

#[cfg(test)]
use common::command::AuditAction;

/// Repository for remote command execution tasks and logs.
///
/// KV key schema:
///   cmd_task:{task_id}                          → CommandTask JSON
///   cmd_log:{task_id}:{seq:010}                 → CommandLogLine JSON
///   cmd_task_idx:client:{client_id}:{created_at_ts}:{task_id} → b"1"
///   audit_log:{id:010}                          → AuditLogEntry JSON
///   audit_seq                                   → i64 (last assigned id)
pub struct CommandRepository {
    db: Arc<dyn Database>,
}

impl CommandRepository {
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self { db }
    }

    // ── task helpers ──────────────────────────────────────────────────────────

    fn task_key(task_id: &str) -> String {
        format!("cmd_task:{}", task_id)
    }

    fn log_key(task_id: &str, seq: u64) -> String {
        format!("cmd_log:{}:{:010}", task_id, seq)
    }

    fn log_prefix(task_id: &str) -> String {
        format!("cmd_log:{}", task_id)
    }

    fn idx_key(client_id: &str, created_at_ts: i64, task_id: &str) -> String {
        format!(
            "cmd_task_idx:client:{}:{}:{}",
            client_id, created_at_ts, task_id
        )
    }

    fn idx_prefix_for_client(client_id: &str) -> String {
        format!("cmd_task_idx:client:{}", client_id)
    }

    // ── task CRUD ─────────────────────────────────────────────────────────────

    /// Save (insert or update) a command task.
    pub async fn save_task(&self, task: &CommandTask) -> CmdbResult<()> {
        let bytes = serde_json::to_vec(task)
            .map_err(|e| CmdbError::Serialization(format!("serialize CommandTask: {}", e)))?;
        self.db.set(&Self::task_key(&task.id), &bytes).await?;

        // Maintain the per-client time index (only needed once at creation)
        let ts = DateTime::parse_from_rfc3339(&task.created_at)
            .map(|dt| dt.timestamp())
            .unwrap_or(0);
        let idx = Self::idx_key(&task.client_id, ts, &task.id);
        self.db.set(&idx, b"1").await?;
        Ok(())
    }

    /// Fetch a single task by id.
    pub async fn get_task(&self, task_id: &str) -> CmdbResult<Option<CommandTask>> {
        match self.db.get(&Self::task_key(task_id)).await? {
            None => Ok(None),
            Some(b) => {
                let t = serde_json::from_slice(&b).map_err(|e| {
                    CmdbError::Serialization(format!("deserialize CommandTask: {}", e))
                })?;
                Ok(Some(t))
            }
        }
    }

    /// Update only the mutable fields of an existing task (status, timing, exit_code, truncated).
    pub async fn update_task(&self, task: &CommandTask) -> CmdbResult<()> {
        // We overwrite the whole record; the index key doesn't change.
        let bytes = serde_json::to_vec(task)
            .map_err(|e| CmdbError::Serialization(format!("serialize CommandTask: {}", e)))?;
        self.db.set(&Self::task_key(&task.id), &bytes).await
    }

    /// Delete a task and all its associated log lines and index entries.
    pub async fn delete_task(&self, task_id: &str) -> CmdbResult<()> {
        // Load the task first to know which index entry to remove.
        if let Some(task) = self.get_task(task_id).await? {
            let ts = DateTime::parse_from_rfc3339(&task.created_at)
                .map(|dt| dt.timestamp())
                .unwrap_or(0);
            let idx = Self::idx_key(&task.client_id, ts, &task.id);
            let _ = self.db.delete(&idx).await;
        }

        // Delete log lines
        let log_keys = self.db.list_keys(&Self::log_prefix(task_id)).await?;
        for k in log_keys {
            let _ = self.db.delete(&k).await;
        }

        // Delete task record
        self.db.delete(&Self::task_key(task_id)).await
    }

    /// List all tasks for a given client, ordered by created_at ascending (via index scan).
    pub async fn list_tasks_for_client(&self, client_id: &str) -> CmdbResult<Vec<CommandTask>> {
        let prefix = Self::idx_prefix_for_client(client_id);
        let idx_keys = self.db.list_keys(&prefix).await?;

        // idx_keys are sorted lexicographically; because created_at is a unix timestamp
        // zero-padded and task_id is UUID, the natural sort gives time-ascending order.
        let mut tasks = Vec::with_capacity(idx_keys.len());
        for k in idx_keys {
            // Extract task_id from key: cmd_task_idx:client:{client_id}:{ts}:{task_id}
            if let Some(task_id) = k.rsplit(':').next() {
                if let Some(t) = self.get_task(task_id).await? {
                    tasks.push(t);
                }
            }
        }
        Ok(tasks)
    }

    /// List ALL tasks (full scan over cmd_task: prefix).
    pub async fn list_all_tasks(&self) -> CmdbResult<Vec<CommandTask>> {
        let values = self.db.list_values("cmd_task:").await?;
        let mut tasks = Vec::with_capacity(values.len());
        for b in values {
            let t = serde_json::from_slice(&b).map_err(|e| {
                CmdbError::Serialization(format!("deserialize CommandTask: {}", e))
            })?;
            tasks.push(t);
        }
        Ok(tasks)
    }

    /// Fetch the first Pending task for a given client (for agent long-poll).
    pub async fn get_pending_task_for_client(
        &self,
        client_id: &str,
    ) -> CmdbResult<Option<CommandTask>> {
        let tasks = self.list_tasks_for_client(client_id).await?;
        Ok(tasks
            .into_iter()
            .find(|t| t.status == CommandStatus::Pending))
    }

    // ── log lines ─────────────────────────────────────────────────────────────

    /// Append a batch of log lines for a task.
    pub async fn append_logs(&self, task_id: &str, lines: &[CommandLogLine]) -> CmdbResult<()> {
        for line in lines {
            let key = Self::log_key(task_id, line.seq);
            let bytes = serde_json::to_vec(line).map_err(|e| {
                CmdbError::Serialization(format!("serialize CommandLogLine: {}", e))
            })?;
            self.db.set(&key, &bytes).await?;
        }
        Ok(())
    }

    /// Get all log lines for a task, sorted by seq.
    pub async fn get_logs(&self, task_id: &str) -> CmdbResult<Vec<CommandLogLine>> {
        let prefix = Self::log_prefix(task_id);
        let mut entries = self.db.list_entries(&prefix).await?;
        // Sort lexicographically by key (seq is zero-padded so this is numerically correct)
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));

        let mut lines = Vec::with_capacity(entries.len());
        for (_, bytes) in entries {
            let line = serde_json::from_slice(&bytes).map_err(|e| {
                CmdbError::Serialization(format!("deserialize CommandLogLine: {}", e))
            })?;
            lines.push(line);
        }
        Ok(lines)
    }

    /// Count total log bytes for a task (used to enforce 10 MB cap).
    pub async fn count_log_bytes(&self, task_id: &str) -> CmdbResult<usize> {
        let prefix = Self::log_prefix(task_id);
        let entries = self.db.list_entries(&prefix).await?;
        Ok(entries.iter().map(|(_, b)| b.len()).sum())
    }

    // ── cleanup ───────────────────────────────────────────────────────────────

    /// Return all tasks whose created_at is older than the given unix timestamp.
    pub async fn list_tasks_older_than(&self, cutoff_ts: i64) -> CmdbResult<Vec<CommandTask>> {
        let all = self.list_all_tasks().await?;
        Ok(all
            .into_iter()
            .filter(|t| {
                let ts = DateTime::parse_from_rfc3339(&t.created_at)
                    .map(|dt| dt.timestamp())
                    .unwrap_or(0);
                ts < cutoff_ts
            })
            .collect())
    }

    /// Global config key for the remote execution enabled flag.
    pub async fn get_remote_exec_enabled(&self) -> CmdbResult<bool> {
        match self.db.get("config:remote_exec_enabled").await? {
            None => Ok(false),
            Some(b) => {
                let s = String::from_utf8(b)
                    .map_err(|e| CmdbError::Serialization(format!("config parse: {}", e)))?;
                Ok(s.trim() == "true")
            }
        }
    }

    pub async fn set_remote_exec_enabled(&self, enabled: bool) -> CmdbResult<()> {
        let val = if enabled { b"true" as &[u8] } else { b"false" };
        self.db.set("config:remote_exec_enabled", val).await
    }

    // ── audit log ────────────────────────────────────────────────────────────

    /// Append an audit log entry with auto-generated ID. Returns the assigned sequence id.
    pub async fn append_audit(&self, entry: &AuditLogEntry) -> CmdbResult<u64> {
        let seq_key = "audit_seq";
        let current = match self.db.get(seq_key).await {
            Ok(Some(v)) => serde_json::from_slice::<i64>(&v).unwrap_or(0),
            _ => 0,
        };
        let next_id = (current + 1) as u64;
        self.db
            .set(seq_key, &serde_json::to_vec(&(next_id as i64)).unwrap())
            .await?;

        let key = format!("audit_log:{:010}", next_id);
        let value = serde_json::to_vec(entry).map_err(|e| CmdbError::Serialization(e.to_string()))?;
        self.db.set(&key, &value).await?;

        Ok(next_id)
    }

    #[allow(dead_code)]
    /// List audit entries ordered newest first, paginated.
    pub async fn list_audit(&self, page: usize, page_size: usize) -> CmdbResult<Vec<AuditLogEntry>> {
        let prefix = "audit_log:";
        let keys = self.db.list_keys(prefix).await?;
        let mut entries = Vec::new();
        for k in &keys {
            if let Some(v) = self.db.get(k).await.ok().flatten() {
                if let Ok(entry) = serde_json::from_slice::<AuditLogEntry>(&v) {
                    entries.push(entry);
                }
            }
        }
        entries.sort_by(|a, b| b.id.cmp(&a.id));
        let start = (page - 1) * page_size;
        let end = start + page_size;
        Ok(entries.into_iter().skip(start).take(end.saturating_sub(start)).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::setup_test_db;
    use common::command::{CommandStatus, DangerLevel, LogStream};

    fn sample_task(client_id: &str) -> CommandTask {
        CommandTask::new(
            client_id.to_string(),
            "ls".to_string(),
            vec!["-la".to_string(), "/tmp".to_string()],
            "admin".to_string(),
            DangerLevel::Safe,
            Some(300),
        )
    }

    #[tokio::test]
    async fn test_save_and_get_task() {
        let db = setup_test_db().unwrap();
        let repo = CommandRepository::new(Arc::new(db));
        let mut task = sample_task("client-001");
        task.id = "task-001".to_string();
        repo.save_task(&task).await.unwrap();

        let got = repo.get_task("task-001").await.unwrap().unwrap();
        assert_eq!(got.id, "task-001");
        assert_eq!(got.command, "ls");
        assert_eq!(got.args, vec!["-la", "/tmp"]);
        assert_eq!(got.status, CommandStatus::Pending);
    }

    #[tokio::test]
    async fn test_update_task_status() {
        let db = setup_test_db().unwrap();
        let repo = CommandRepository::new(Arc::new(db));
        let mut task = sample_task("client-001");
        task.id = "task-002".to_string();
        repo.save_task(&task).await.unwrap();

        task.status = CommandStatus::Running;
        task.started_at = Some("2025-01-01T00:00:00Z".to_string());
        repo.update_task(&task).await.unwrap();

        let got = repo.get_task("task-002").await.unwrap().unwrap();
        assert_eq!(got.status, CommandStatus::Running);
        assert!(got.started_at.is_some());
    }

    #[tokio::test]
    async fn test_append_and_get_logs() {
        let db = setup_test_db().unwrap();
        let repo = CommandRepository::new(Arc::new(db));
        let mut task = sample_task("client-001");
        task.id = "task-003".to_string();
        repo.save_task(&task).await.unwrap();

        let lines = vec![
            CommandLogLine {
                seq: 1,
                line: "stdout line 1".to_string(),
                stream: LogStream::Stdout,
                timestamp: "2025-01-01T00:00:01Z".to_string(),
            },
            CommandLogLine {
                seq: 2,
                line: "stderr line 1".to_string(),
                stream: LogStream::Stderr,
                timestamp: "2025-01-01T00:00:02Z".to_string(),
            },
        ];
        repo.append_logs("task-003", &lines).await.unwrap();

        let got = repo.get_logs("task-003").await.unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].line, "stdout line 1");
        assert_eq!(got[1].line, "stderr line 1");
        assert_eq!(got[1].stream, LogStream::Stderr);
    }

    #[tokio::test]
    async fn test_delete_task_cleans_up() {
        let db = setup_test_db().unwrap();
        let repo = CommandRepository::new(Arc::new(db));
        let mut task = sample_task("client-001");
        task.id = "task-004".to_string();
        repo.save_task(&task).await.unwrap();

        repo.delete_task("task-004").await.unwrap();
        assert!(repo.get_task("task-004").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_count_log_bytes() {
        let db = setup_test_db().unwrap();
        let repo = CommandRepository::new(Arc::new(db));
        let mut task = sample_task("client-001");
        task.id = "task-005".to_string();
        repo.save_task(&task).await.unwrap();

        let lines = vec![
            CommandLogLine {
                seq: 1,
                line: "hello world".to_string(),
                stream: LogStream::Stdout,
                timestamp: "2025-01-01T00:00:01Z".to_string(),
            },
        ];
        repo.append_logs("task-005", &lines).await.unwrap();
        let bytes = repo.count_log_bytes("task-005").await.unwrap();
        assert!(bytes > 0);
    }

    #[tokio::test]
    async fn test_remote_exec_enabled_default() {
        let db = setup_test_db().unwrap();
        let repo = CommandRepository::new(Arc::new(db));
        assert!(!repo.get_remote_exec_enabled().await.unwrap());
        repo.set_remote_exec_enabled(true).await.unwrap();
        assert!(repo.get_remote_exec_enabled().await.unwrap());
        repo.set_remote_exec_enabled(false).await.unwrap();
        assert!(!repo.get_remote_exec_enabled().await.unwrap());
    }

    #[tokio::test]
    async fn test_list_tasks_for_client() {
        let db = setup_test_db().unwrap();
        let repo = CommandRepository::new(Arc::new(db));

        let mut t1 = sample_task("client-A");
        t1.id = "task-a1".to_string();
        repo.save_task(&t1).await.unwrap();

        let mut t2 = sample_task("client-A");
        t2.id = "task-a2".to_string();
        repo.save_task(&t2).await.unwrap();

        let mut t3 = sample_task("client-B");
        t3.id = "task-b1".to_string();
        repo.save_task(&t3).await.unwrap();

        let client_a_tasks = repo.list_tasks_for_client("client-A").await.unwrap();
        assert_eq!(client_a_tasks.len(), 2);

        let client_b_tasks = repo.list_tasks_for_client("client-B").await.unwrap();
        assert_eq!(client_b_tasks.len(), 1);
    }

    #[tokio::test]
    async fn test_get_pending_task_for_client() {
        let db = setup_test_db().unwrap();
        let repo = CommandRepository::new(Arc::new(db));

        let mut task = sample_task("client-X");
        task.id = "task-x1".to_string();
        repo.save_task(&task).await.unwrap();

        let pending = repo.get_pending_task_for_client("client-X").await.unwrap();
        assert!(pending.is_some());
        assert_eq!(pending.as_ref().unwrap().id, "task-x1");

        // Update status to Running, should no longer be pending
        let mut task = pending.unwrap();
        task.status = CommandStatus::Running;
        repo.update_task(&task).await.unwrap();

        let pending2 = repo.get_pending_task_for_client("client-X").await.unwrap();
        assert!(pending2.is_none());
    }

    #[tokio::test]
    async fn test_list_tasks_older_than() {
        let db = setup_test_db().unwrap();
        let repo = CommandRepository::new(Arc::new(db));

        let mut task = sample_task("client-Y");
        task.id = "task-y1".to_string();
        task.created_at = "2020-01-01T00:00:00Z".to_string();
        repo.save_task(&task).await.unwrap();

        let old = repo.list_tasks_older_than(1577836801).await.unwrap();
        // cutoff is strictly greater than task timestamp, so task is "older"
        assert_eq!(old.len(), 1);

        // cutoff equals task timestamp, task is NOT "older" (ts < cutoff)
        let exact = repo.list_tasks_older_than(1577836800).await.unwrap();
        assert_eq!(exact.len(), 0);

        // cutoff far in future, task IS older (1577836800 < 1900000000)
        let future = repo.list_tasks_older_than(1900000000).await.unwrap();
        assert_eq!(future.len(), 1);
    }

    #[tokio::test]
    async fn test_append_audit() {
        let db = setup_test_db().unwrap();
        let repo = CommandRepository::new(Arc::new(db));

        let entry = AuditLogEntry::new(
            AuditAction::CommandCreate,
            "admin",
            "Executed: rm -rf /tmp",
        );
        let seq = repo.append_audit(&entry).await.unwrap();
        assert_eq!(seq, 1);

        let entries = repo.list_audit(1, 100).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action, AuditAction::CommandCreate);
    }
}
