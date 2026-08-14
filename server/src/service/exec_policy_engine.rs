use crate::middleware::permission::PermissionContext;
use crate::repository::client_repository::ClientRepository;
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
    client_repo: Option<Arc<ClientRepository>>,
}

impl ExecPolicyEngine {
    pub fn new(repo: Arc<ExecPolicyRepository>) -> Self {
        Self {
            repo,
            client_repo: None,
        }
    }

    pub fn with_client_repo(mut self, client_repo: Arc<ClientRepository>) -> Self {
        self.client_repo = Some(client_repo);
        self
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
            if !scope_matches(&p.target_scope, client_id, self.client_repo.as_ref()).await {
                continue;
            }
            matched.push(p);
        }

        if matched.is_empty() {
            return PolicyDecision::Denied("No execution policy matches this user".into());
        }

        matched.sort_by(|a, b| b.priority.cmp(&a.priority));

        if let Some(policy) = matched.into_iter().next() {
            return apply_policy(policy, command);
        }

        PolicyDecision::Denied("No execution policy matches this user".into())
    }

    /// Evaluate every command in a multiline task. A policy decision for the
    /// first line must not implicitly authorize later lines in a shell-mode
    /// script.
    pub async fn evaluate_script(
        &self,
        user_id: &str,
        role: &Role,
        group_ids: &[String],
        client_id: &str,
        script: &str,
    ) -> PolicyDecision {
        let mut require_approval = false;
        let mut saw_command = false;
        for line in script
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
        {
            saw_command = true;
            match self
                .evaluate(user_id, role, group_ids, client_id, line)
                .await
            {
                PolicyDecision::Denied(message) => return PolicyDecision::Denied(message),
                PolicyDecision::Warning => return PolicyDecision::Warning,
                PolicyDecision::Allowed {
                    require_approval: required,
                } => {
                    require_approval |= required;
                }
            }
        }
        if !saw_command {
            return PolicyDecision::Denied("Command script is empty".into());
        }
        PolicyDecision::Allowed { require_approval }
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

async fn scope_matches(
    target_scope: &TargetScope,
    client_id: &str,
    client_repo: Option<&Arc<ClientRepository>>,
) -> bool {
    match target_scope {
        TargetScope::All => true,
        TargetScope::Clients(ids) => ids.contains(&client_id.to_string()),
        TargetScope::Projects(project_ids) => {
            let Some(repo) = client_repo else {
                return false;
            };
            repo.get(client_id)
                .await
                .ok()
                .flatten()
                .and_then(|client| client.project_id)
                .is_some_and(|project_id| project_ids.contains(&project_id))
        }
        TargetScope::Tags(tags) => {
            let Some(repo) = client_repo else {
                return false;
            };
            let Some(client) = repo.get(client_id).await.ok().flatten() else {
                return false;
            };
            PermissionContext::client_tags(&client)
                .iter()
                .any(|tag| tags.iter().any(|required| required == tag))
        }
    }
}

fn apply_policy(policy: &ExecPolicy, command: &str) -> PolicyDecision {
    for override_rule in &policy.command_rules.overrides {
        if command.starts_with(&override_rule.pattern) {
            match override_rule.action {
                CommandAction::Allow => {
                    return PolicyDecision::Allowed {
                        require_approval: policy.require_approval,
                    };
                }
                CommandAction::Deny => {
                    return PolicyDecision::Denied(format!(
                        "Command '{}' denied by policy override",
                        override_rule.pattern
                    ));
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
