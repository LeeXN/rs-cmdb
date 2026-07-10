use crate::repository::exec_policy_repository::ExecPolicyRepository;
use common::entity::permission::{CommandAction, ExecPolicy, SubjectType, TargetScope};
use common::entity::user::Role;
use std::sync::Arc;

pub enum PolicyDecision {
    Allowed { require_approval: bool },
    Denied(String),
    Warning,
}

pub struct ExecPolicyEngine {
    repo: Arc<ExecPolicyRepository>,
}

impl ExecPolicyEngine {
    pub fn new(repo: Arc<ExecPolicyRepository>) -> Self {
        Self { repo }
    }

    pub async fn evaluate(
        &self,
        user_id: &str,
        role: &Role,
        group_ids: &[String],
        client_id: &str,
        command: &str,
    ) -> PolicyDecision {
        let policies = match self.repo.list_all().await {
            Ok(p) => p,
            Err(_) => return PolicyDecision::Denied("No execution policies available".into()),
        };

        let mut matched: Vec<&ExecPolicy> = Vec::new();
        for p in &policies {
            if !subject_matches(&p.subject_type, &p.subject_id, user_id, role, group_ids) {
                continue;
            }
            if !scope_matches(&p.target_scope, client_id) {
                continue;
            }
            matched.push(p);
        }

        if matched.is_empty() {
            return PolicyDecision::Denied("No execution policy matches this user".into());
        }

        matched.sort_by(|a, b| b.priority.cmp(&a.priority));

        for policy in matched {
            return apply_policy(policy, command);
        }

        PolicyDecision::Denied("No execution policy matches this user".into())
    }
}

fn subject_matches(
    subject_type: &SubjectType,
    subject_id: &str,
    user_id: &str,
    role: &Role,
    group_ids: &[String],
) -> bool {
    match subject_type {
        SubjectType::User => subject_id == user_id,
        SubjectType::Group => group_ids.contains(&subject_id.to_string()),
        SubjectType::Role => {
            let role_str = match role {
                Role::Admin => "Admin",
                Role::User => "User",
                Role::Viewer => "Viewer",
            };
            subject_id == role_str
        }
    }
}

fn scope_matches(target_scope: &TargetScope, client_id: &str) -> bool {
    match target_scope {
        TargetScope::All => true,
        TargetScope::Clients(ids) => ids.contains(&client_id.to_string()),
        TargetScope::Projects(_) | TargetScope::Tags(_) => true,
    }
}

fn apply_policy(policy: &ExecPolicy, command: &str) -> PolicyDecision {
    for override_rule in &policy.command_rules.overrides {
        if command.starts_with(&override_rule.pattern) {
            match override_rule.action {
                CommandAction::Allow => {
                    return PolicyDecision::Allowed {
                        require_approval: policy.require_approval,
                    }
                }
                CommandAction::Deny => {
                    return PolicyDecision::Denied(format!(
                        "Command '{}' denied by policy override",
                        override_rule.pattern
                    ))
                }
                CommandAction::Warn => return PolicyDecision::Warning,
            }
        }
    }

    match policy.command_rules.default_action {
        CommandAction::Allow => PolicyDecision::Allowed {
            require_approval: policy.require_approval,
        },
        CommandAction::Deny => PolicyDecision::Denied("Command execution denied by policy".into()),
        CommandAction::Warn => PolicyDecision::Warning,
    }
}
