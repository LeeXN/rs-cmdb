use serde::{Deserialize, Serialize};
use std::fmt;

use crate::entity::execution::ExecutionType;
use crate::entity::user::Role;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SubjectType {
    User,
    Group,
    Role,
}

impl fmt::Display for SubjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubjectType::User => write!(f, "User"),
            SubjectType::Group => write!(f, "Group"),
            SubjectType::Role => write!(f, "Role"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResourceType {
    Client,
    Component,
    Rack,
    Person,
    Project,
    Dictionary,
    Command,
    User,
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceType::Client => write!(f, "Client"),
            ResourceType::Component => write!(f, "Component"),
            ResourceType::Rack => write!(f, "Rack"),
            ResourceType::Person => write!(f, "Person"),
            ResourceType::Project => write!(f, "Project"),
            ResourceType::Dictionary => write!(f, "Dictionary"),
            ResourceType::Command => write!(f, "Command"),
            ResourceType::User => write!(f, "User"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionAction {
    View,
    Create,
    Update,
    Delete,
}

impl fmt::Display for PermissionAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionAction::View => write!(f, "View"),
            PermissionAction::Create => write!(f, "Create"),
            PermissionAction::Update => write!(f, "Update"),
            PermissionAction::Delete => write!(f, "Delete"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScopeConstraint {
    All,
    Owned,
    Project(Vec<String>),
    Tag(Vec<String>),
    None,
}

impl fmt::Display for ScopeConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScopeConstraint::All => write!(f, "All"),
            ScopeConstraint::Owned => write!(f, "Owned"),
            ScopeConstraint::Project(ids) => write!(f, "Project({})", ids.join(",")),
            ScopeConstraint::Tag(tags) => write!(f, "Tag({})", tags.join(",")),
            ScopeConstraint::None => write!(f, "None"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissionRule {
    pub id: String,
    pub subject_type: SubjectType,
    pub subject_id: String,
    pub resource_type: ResourceType,
    pub actions: Vec<PermissionAction>,
    pub constraint: ScopeConstraint,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub member_ids: Vec<String>,
}

// ── Approval structures ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ApprovalCommandPayload {
    pub client_id: String,
    pub command: String,
    pub submitted_role: Role,
    #[serde(default)]
    pub group_ids: Vec<String>,
    #[serde(default)]
    pub shell_mode: bool,
    #[serde(default)]
    pub args: Option<Vec<String>>,
    #[serde(default)]
    pub execution_type: Option<ExecutionType>,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ApprovalPayload {
    Command { request: ApprovalCommandPayload },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingApproval {
    pub id: String,
    pub policy_type: String,
    pub policy_id: String,
    pub user_id: String,
    pub username: String,
    pub target_client_id: String,
    pub command: String,
    #[serde(default)]
    pub payload: Option<ApprovalPayload>,
    pub status: ApprovalStatus,
    pub created_at: String,
    pub expires_at: String,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<String>,
    #[serde(default)]
    pub executed_task_id: Option<String>,
    /// Set when an approved request is being materialized into a command.
    /// This makes approval retries idempotent and distinguishes a failed
    /// materialization from a request that was never approved.
    #[serde(default)]
    pub execution_started_at: Option<String>,
    /// Unique owner of the current task-materialization claim. Cleanup and
    /// task attachment must present the same claim so an expired worker
    /// cannot roll back a newer retry.
    #[serde(default)]
    pub execution_claim_id: Option<String>,
}

// ── ExecPolicy structures ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TargetScope {
    All,
    Clients(Vec<String>),
    Projects(Vec<String>),
    Tags(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommandAction {
    Allow,
    Deny,
    Warn,
}

impl fmt::Display for CommandAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandAction::Allow => write!(f, "Allow"),
            CommandAction::Deny => write!(f, "Deny"),
            CommandAction::Warn => write!(f, "Warn"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandOverride {
    pub pattern: String,
    pub action: CommandAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandRules {
    pub default_action: CommandAction,
    pub overrides: Vec<CommandOverride>,
}

impl Default for CommandRules {
    fn default() -> Self {
        Self {
            default_action: CommandAction::Allow,
            overrides: Vec::new(),
        }
    }
}

impl CommandRules {
    pub fn allow_list(commands: Vec<String>) -> Self {
        Self {
            default_action: CommandAction::Deny,
            overrides: commands
                .into_iter()
                .map(|pattern| CommandOverride {
                    pattern,
                    action: CommandAction::Allow,
                })
                .collect(),
        }
    }

    pub fn deny_list(commands: Vec<String>) -> Self {
        Self {
            default_action: CommandAction::Allow,
            overrides: commands
                .into_iter()
                .map(|pattern| CommandOverride {
                    pattern,
                    action: CommandAction::Deny,
                })
                .collect(),
        }
    }

    pub fn evaluate(&self, command: &str) -> CommandAction {
        for override_rule in &self.overrides {
            if command == override_rule.pattern {
                return override_rule.action.clone();
            }
        }
        self.default_action.clone()
    }

    pub fn listed_commands(&self) -> Vec<String> {
        self.overrides
            .iter()
            .map(|rule| rule.pattern.clone())
            .collect()
    }

    pub fn is_allow_all(&self) -> bool {
        matches!(self.default_action, CommandAction::Allow) && self.overrides.is_empty()
    }

    pub fn is_deny_all(&self) -> bool {
        matches!(self.default_action, CommandAction::Deny) && self.overrides.is_empty()
    }
}

// ── WebTerminalPolicy structures ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TerminalMode {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebTerminalPolicy {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub subject_type: SubjectType,
    pub subject_id: String,
    pub target_scope: TargetScope,
    pub mode: TerminalMode,
    #[serde(default, alias = "command_rules")]
    pub terminal_command_rules: CommandRules,
    #[serde(default)]
    pub allowed_commands: Vec<String>,
    pub session_timeout_secs: u64,
    pub max_concurrent_sessions: u32,
    pub require_approval: bool,
    pub priority: i32,
}

impl WebTerminalPolicy {
    pub fn effective_terminal_command_rules(&self) -> CommandRules {
        if !self.terminal_command_rules.overrides.is_empty() {
            return self.terminal_command_rules.clone();
        }

        if !self.allowed_commands.is_empty() {
            return CommandRules::allow_list(self.allowed_commands.clone());
        }

        self.terminal_command_rules.clone()
    }

    pub fn effective_command_rules(&self) -> CommandRules {
        self.effective_terminal_command_rules()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecPolicy {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub subject_type: SubjectType,
    pub subject_id: String,
    pub target_scope: TargetScope,
    pub command_rules: CommandRules,
    pub require_approval: bool,
    pub priority: i32,
}
