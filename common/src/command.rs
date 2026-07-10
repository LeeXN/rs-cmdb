use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn default_uuid() -> String {
    Uuid::new_v4().to_string()
}

fn default_now() -> String {
    Utc::now().to_rfc3339()
}

/// 命令任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CommandStatus {
    /// 等待 Agent 拉取
    #[default]
    Pending,
    /// Agent 正在执行
    Running,
    /// 执行成功（exit_code == 0）
    Success,
    /// 执行失败（exit_code != 0）
    Failed,
    /// 超时前未被 Agent 拉取
    Expired,
    /// 执行超时后被强制终止
    Timeout,
}

impl std::fmt::Display for CommandStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandStatus::Pending => write!(f, "pending"),
            CommandStatus::Running => write!(f, "running"),
            CommandStatus::Success => write!(f, "success"),
            CommandStatus::Failed => write!(f, "failed"),
            CommandStatus::Expired => write!(f, "expired"),
            CommandStatus::Timeout => write!(f, "timeout"),
        }
    }
}

/// 危险等级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DangerLevel {
    #[default]
    Safe,
    Warning,
    Blocked,
}

impl std::fmt::Display for DangerLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DangerLevel::Safe => write!(f, "safe"),
            DangerLevel::Warning => write!(f, "warning"),
            DangerLevel::Blocked => write!(f, "blocked"),
        }
    }
}

/// 命令任务
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CommandTask {
    /// 任务唯一 ID
    #[serde(default = "default_uuid")]
    pub id: String,
    /// 目标 Agent 的 client_id
    pub client_id: String,
    /// 要执行的命令（主程序）
    pub command: String,
    /// 是否按 shell 脚本块执行
    #[serde(default)]
    pub shell_mode: bool,
    /// 命令参数列表（向后兼容，空时从 command 按空格分割）
    #[serde(default)]
    pub args: Vec<String>,
    /// 下发人 user_id
    pub submitted_by: String,
    /// 当前状态
    pub status: CommandStatus,
    /// 危险等级（检测时记录）
    pub danger_level: DangerLevel,
    /// 执行超时秒数（默认 300）
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// 任务创建时间
    #[serde(default = "default_now")]
    pub created_at: String,
    /// 任务过期时间（pending 状态超过此时间自动变 expired）
    pub expires_at: String,
    /// Agent 开始执行时间
    pub started_at: Option<String>,
    /// 任务完成时间
    pub completed_at: Option<String>,
    /// 退出码
    pub exit_code: Option<i32>,
    /// 是否日志被截断
    #[serde(default)]
    pub truncated: bool,
}

fn default_timeout() -> u64 {
    300
}

/// Split a command string on whitespace into a (command, args) tuple.
/// This provides backward compatibility for clients that send a flat command string.
pub fn split_command(command: &str) -> (String, Vec<String>) {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        (String::new(), vec![])
    } else {
        (parts[0].to_string(), parts[1..].iter().map(|s| s.to_string()).collect())
    }
}

/// Normalise a command + args into (command, args).
/// If args is empty, split command on whitespace.
pub fn normalise_command(command: &str, args: Vec<String>) -> (String, Vec<String>) {
    if args.is_empty() && !command.is_empty() {
        split_command(command)
    } else {
        (command.to_string(), args)
    }
}

impl CommandTask {
    pub fn new(
        client_id: String,
        command: String,
        args: Vec<String>,
        submitted_by: String,
        danger_level: DangerLevel,
        timeout_secs: Option<u64>,
    ) -> Self {
        let now = Utc::now();
        let expires_at = (now + chrono::Duration::minutes(10)).to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            client_id,
            command,
            shell_mode: false,
            args,
            submitted_by,
            status: CommandStatus::Pending,
            danger_level,
            timeout_secs: timeout_secs.unwrap_or(300),
            created_at: now.to_rfc3339(),
            expires_at,
            started_at: None,
            completed_at: None,
            exit_code: None,
            truncated: false,
        }
    }
}

/// 命令日志行
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandLogLine {
    /// 序号（用于排序）
    pub seq: u64,
    /// 日志内容
    pub line: String,
    /// 日志流（stdout / stderr）
    pub stream: LogStream,
    /// 时间戳
    #[serde(default = "default_now")]
    pub timestamp: String,
}

/// 日志流类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogStream {
    Stdout,
    Stderr,
}

/// 审计操作类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    CommandCreate,
    CommandBlocked,
    CommandCompleted,
    ConfigChange,
    UserCreated,
    UserDeleted,
    UserRoleChanged,
    ClientCreated,
    ClientUpdated,
    ClientDeleted,
    ComponentCreated,
    ComponentUpdated,
    ComponentDeleted,
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditAction::CommandCreate => write!(f, "command_create"),
            AuditAction::CommandBlocked => write!(f, "command_blocked"),
            AuditAction::CommandCompleted => write!(f, "command_completed"),
            AuditAction::ConfigChange => write!(f, "config_change"),
            AuditAction::UserCreated => write!(f, "user_created"),
            AuditAction::UserDeleted => write!(f, "user_deleted"),
            AuditAction::UserRoleChanged => write!(f, "user_role_changed"),
            AuditAction::ClientCreated => write!(f, "client_created"),
            AuditAction::ClientUpdated => write!(f, "client_updated"),
            AuditAction::ClientDeleted => write!(f, "client_deleted"),
            AuditAction::ComponentCreated => write!(f, "component_created"),
            AuditAction::ComponentUpdated => write!(f, "component_updated"),
            AuditAction::ComponentDeleted => write!(f, "component_deleted"),
        }
    }
}

/// 审计日志条目 — 记录各种操作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditLogEntry {
    /// 自增序号（0 表示待分配）
    pub id: u64,
    /// 操作类型
    pub action: AuditAction,
    /// 操作人
    pub operator: String,
    /// 关联的命令任务 ID（如无则为空）
    pub task_id: Option<String>,
    /// 目标 client_id
    pub client_id: Option<String>,
    /// 操作详情
    pub detail: String,
    /// 时间戳
    #[serde(default = "default_now")]
    pub created_at: String,
}

impl AuditLogEntry {
    pub fn new(action: AuditAction, operator: &str, detail: &str) -> Self {
        Self {
            id: 0,
            action,
            operator: operator.to_string(),
            task_id: None,
            client_id: None,
            detail: detail.to_string(),
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_task_default() {
        let task = CommandTask::default();
        assert_eq!(task.status, CommandStatus::Pending);
        assert!(task.id.is_empty());
        assert_eq!(task.timeout_secs, 0);
    }

    #[test]
    fn test_default_timeout_fn() {
        assert_eq!(default_timeout(), 300);
    }

    #[test]
    fn test_command_task_new() {
        let task = CommandTask::new(
            "client-1".into(),
            "ls".into(),
            vec!["-la".into()],
            "admin".into(),
            DangerLevel::Warning,
            Some(600),
        );
        assert_eq!(task.client_id, "client-1");
        assert_eq!(task.command, "ls");
        assert_eq!(task.submitted_by, "admin");
        assert_eq!(task.danger_level, DangerLevel::Warning);
        assert_eq!(task.timeout_secs, 600);
        assert_eq!(task.status, CommandStatus::Pending);
        assert!(task.expires_at > task.created_at);
    }

    #[test]
    fn test_command_task_new_default_timeout() {
        let task = CommandTask::new(
            "c1".into(),
            "echo".into(),
            vec!["hi".into()],
            "user".into(),
            DangerLevel::Safe,
            None,
        );
        assert_eq!(task.timeout_secs, 300);
    }

    #[test]
    fn test_command_task_serialization() {
        let task = CommandTask::new(
            "c1".into(),
            "df".into(),
            vec!["-h".into()],
            "admin".into(),
            DangerLevel::Safe,
            None,
        );
        let json = serde_json::to_string(&task).unwrap();
        let deserialized: CommandTask = serde_json::from_str(&json).unwrap();
        assert_eq!(task.id, deserialized.id);
        assert_eq!(task.status, deserialized.status);
        assert_eq!(task.danger_level, deserialized.danger_level);
    }

    #[test]
    fn test_command_status_display() {
        assert_eq!(CommandStatus::Pending.to_string(), "pending");
        assert_eq!(CommandStatus::Running.to_string(), "running");
        assert_eq!(CommandStatus::Success.to_string(), "success");
        assert_eq!(CommandStatus::Failed.to_string(), "failed");
        assert_eq!(CommandStatus::Expired.to_string(), "expired");
        assert_eq!(CommandStatus::Timeout.to_string(), "timeout");
    }

    #[test]
    fn test_danger_level_display() {
        assert_eq!(DangerLevel::Safe.to_string(), "safe");
        assert_eq!(DangerLevel::Warning.to_string(), "warning");
        assert_eq!(DangerLevel::Blocked.to_string(), "blocked");
    }

    #[test]
    fn test_command_log_line_round_trip() {
        let line = CommandLogLine {
            seq: 1,
            line: "hello".into(),
            stream: LogStream::Stdout,
            timestamp: Utc::now().to_rfc3339(),
        };
        let json = serde_json::to_string(&line).unwrap();
        let deserialized: CommandLogLine = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.seq, 1);
        assert_eq!(deserialized.stream, LogStream::Stdout);
    }

    #[test]
    fn test_audit_log_entry_round_trip() {
        let entry = AuditLogEntry {
            id: 42,
            action: AuditAction::CommandCreate,
            operator: "admin".into(),
            task_id: Some("task-1".into()),
            client_id: Some("client-1".into()),
            detail: "created command".into(),
            created_at: Utc::now().to_rfc3339(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: AuditLogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, 42);
        assert_eq!(deserialized.task_id, Some("task-1".into()));
        assert_eq!(deserialized.action, AuditAction::CommandCreate);
    }

    #[test]
    fn test_audit_log_entry_new() {
        let entry = AuditLogEntry::new(AuditAction::UserCreated, "admin", "created user test");
        assert_eq!(entry.id, 0);
        assert_eq!(entry.action, AuditAction::UserCreated);
        assert_eq!(entry.operator, "admin");
        assert_eq!(entry.detail, "created user test");
        assert!(entry.client_id.is_none());
        assert!(entry.task_id.is_none());
        assert!(!entry.created_at.is_empty());
    }

    #[test]
    fn test_audit_action_display() {
        assert_eq!(AuditAction::CommandCreate.to_string(), "command_create");
        assert_eq!(AuditAction::UserCreated.to_string(), "user_created");
        assert_eq!(AuditAction::ComponentDeleted.to_string(), "component_deleted");
    }

    #[test]
    fn test_audit_action_serde() {
        let json = serde_json::to_string(&AuditAction::UserRoleChanged).unwrap();
        assert_eq!(json, "\"user_role_changed\"");
        let des: AuditAction = serde_json::from_str("\"user_role_changed\"").unwrap();
        assert_eq!(des, AuditAction::UserRoleChanged);
    }

    #[test]
    fn test_split_command_empty() {
        let (cmd, args) = split_command("");
        assert_eq!(cmd, "");
        assert!(args.is_empty());
    }

    #[test]
    fn test_split_command_no_args() {
        let (cmd, args) = split_command("ping");
        assert_eq!(cmd, "ping");
        assert!(args.is_empty());
    }

    #[test]
    fn test_split_command_with_args() {
        let (cmd, args) = split_command("ping -c 4 8.8.8.8");
        assert_eq!(cmd, "ping");
        assert_eq!(args, vec!["-c", "4", "8.8.8.8"]);
    }

    #[test]
    fn test_split_command_multiple_spaces() {
        let (cmd, args) = split_command("echo   hello   world");
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["hello", "world"]);
    }

    #[test]
    fn test_normalise_command_with_args() {
        let (cmd, args) = normalise_command("ls", vec!["-la".into()]);
        assert_eq!(cmd, "ls");
        assert_eq!(args, vec!["-la"]);
    }

    #[test]
    fn test_normalise_command_empty_args_fallback() {
        let (cmd, args) = normalise_command("df -h /tmp", vec![]);
        assert_eq!(cmd, "df");
        assert_eq!(args, vec!["-h", "/tmp"]);
    }

    #[test]
    fn test_command_task_args_field() {
        let task = CommandTask::new("c1".into(), "ping".into(), vec!["-c".into(), "1".into(), "8.8.8.8".into()], "u".into(), DangerLevel::Safe, None);
        assert_eq!(task.args, vec!["-c", "1", "8.8.8.8"]);
    }

    #[test]
    fn test_command_task_backward_compat_deser() {
        let old_json = r#"{"id":"x","client_id":"c1","command":"ls -la","submitted_by":"admin","status":"pending","danger_level":"safe","timeout_secs":60,"created_at":"2024-01-01T00:00:00Z","expires_at":"2024-01-01T00:10:00Z","started_at":null,"completed_at":null,"exit_code":null,"truncated":false}"#;
        let task: CommandTask = serde_json::from_str(old_json).unwrap();
        assert_eq!(task.command, "ls -la");
        assert!(!task.shell_mode);
        assert!(task.args.is_empty(), "old data without args field should default to empty vec");
    }
}
