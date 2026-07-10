//! Web Terminal Service
//!
//! Handles session lifecycle checks for the web terminal feature.
//! The actual WebSocket terminal is not yet implemented; this service
//! provides policy enforcement hooks ready for integration.

use crate::repository::web_terminal_policy_repository::WebTerminalPolicyRepository;
use common::entity::permission::{CommandAction, SubjectType, TargetScope, TerminalMode, WebTerminalPolicy};
use common::entity::user::Role;
use common::error::{CmdbError, CmdbResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Active session tracker (in-memory, per-user session count)
pub struct WebTerminalService {
    policy_repo: Arc<WebTerminalPolicyRepository>,
    /// user_id → active session count
    active_sessions: RwLock<HashMap<String, usize>>,
}

impl WebTerminalService {
    pub fn new(policy_repo: Arc<WebTerminalPolicyRepository>) -> Self {
        Self {
            policy_repo,
            active_sessions: RwLock::new(HashMap::new()),
        }
    }

    // ── 5.3: Policy check on session creation ────────────────────────────────

    /// Check whether a user may open a new terminal session to the given client.
    ///
    /// Returns `Ok(TerminalMode)` if allowed; `Err` if denied.
    pub async fn check_session_allowed(
        &self,
        user_id: &str,
        role: &Role,
        client_id: &str,
    ) -> CmdbResult<TerminalMode> {
        let policies = self.policy_repo.list_all().await?;

        // Find the highest-priority matching policy for this user+client
        let mut matched: Vec<&WebTerminalPolicy> = policies
            .iter()
            .filter(|p| {
                subject_matches(&p.subject_type, &p.subject_id, user_id, role)
                    && scope_matches(&p.target_scope, client_id)
            })
            .collect();

        matched.sort_by(|a, b| b.priority.cmp(&a.priority));

        let policy = match matched.first() {
            Some(p) => *p,
            None => {
                // No policy — deny by default (fail-closed)
                return Err(CmdbError::Forbidden(
                    "No terminal policy grants access to this client".into(),
                ));
            }
        };

        // 5.6: Enforce concurrent session limit
        self.check_concurrent_limit(user_id, policy.max_concurrent_sessions)?;

        Ok(policy.mode.clone())
    }

    // ── 5.4: Read-only command rule filtering ─────────────────────────────────

    /// Filter a command against the read-only terminal command rules.
    ///
    /// Returns `Ok(())` if the command is allowed; `Err(Forbidden)` if not.
    pub async fn check_command_allowed(
        &self,
        user_id: &str,
        role: &Role,
        client_id: &str,
        command: &str,
    ) -> CmdbResult<()> {
        let policies = self.policy_repo.list_all().await?;

        let best = policies
            .iter()
            .filter(|p| {
                subject_matches(&p.subject_type, &p.subject_id, user_id, role)
                    && scope_matches(&p.target_scope, client_id)
            })
            .max_by_key(|p| p.priority);

        let policy = match best {
            Some(p) => p,
            None => return Ok(()), // No policy — allow (session guard already checked)
        };

        // Only enforce command filtering in ReadOnly mode
        if policy.mode == TerminalMode::ReadOnly {
            let rules = policy.effective_terminal_command_rules();
            if rules.overrides.is_empty() && matches!(rules.default_action, CommandAction::Deny) {
                return Err(CmdbError::Forbidden(
                    "Terminal is in read-only mode with no command rules configured".into(),
                ));
            }
            let cmd_name = command.split_whitespace().next().unwrap_or(command);
            match rules.evaluate(cmd_name) {
                CommandAction::Allow => {}
                CommandAction::Deny => {
                    return Err(CmdbError::Forbidden(format!(
                        "Command '{}' is denied by terminal command rules",
                        cmd_name
                    )));
                }
                CommandAction::Warn => {
                    return Err(CmdbError::Forbidden(format!(
                        "Command '{}' requires approval and is not allowed in read-only terminal mode",
                        cmd_name
                    )));
                }
            }
        }

        Ok(())
    }

    // ── 5.5: Idle timeout ────────────────────────────────────────────────────

    /// Return the configured idle timeout for the matching policy, in seconds.
    ///
    /// Returns `0` if no policy applies (no timeout).
    #[allow(dead_code)]
    pub async fn get_session_timeout(
        &self,
        user_id: &str,
        role: &Role,
        client_id: &str,
    ) -> u64 {
        let Ok(policies) = self.policy_repo.list_all().await else {
            return 0;
        };
        policies
            .iter()
            .filter(|p| {
                subject_matches(&p.subject_type, &p.subject_id, user_id, role)
                    && scope_matches(&p.target_scope, client_id)
            })
            .max_by_key(|p| p.priority)
            .map(|p| p.session_timeout_secs)
            .unwrap_or(0)
    }

    // ── 5.6: Concurrent session limit ────────────────────────────────────────

    fn check_concurrent_limit(&self, user_id: &str, max: u32) -> CmdbResult<()> {
        if max == 0 {
            return Ok(()); // 0 means unlimited
        }
        let sessions = self.active_sessions.read().unwrap();
        let current = sessions.get(user_id).copied().unwrap_or(0);
        if current >= max as usize {
            return Err(CmdbError::Forbidden(format!(
                "Maximum of {} concurrent terminal sessions already active",
                max
            )));
        }
        Ok(())
    }

    /// Track session open (increment user's count)
    pub fn on_session_open(&self, user_id: &str) {
        let mut sessions = self.active_sessions.write().unwrap();
        *sessions.entry(user_id.to_string()).or_insert(0) += 1;
    }

    /// Track session close (decrement user's count)
    pub fn on_session_close(&self, user_id: &str) {
        let mut sessions = self.active_sessions.write().unwrap();
        if let Some(count) = sessions.get_mut(user_id) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }
}

fn subject_matches(subject_type: &SubjectType, subject_id: &str, user_id: &str, role: &Role) -> bool {
    match subject_type {
        SubjectType::User => subject_id == user_id,
        SubjectType::Group => false, // Group matching not implemented yet
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

fn scope_matches(scope: &TargetScope, client_id: &str) -> bool {
    match scope {
        TargetScope::All => true,
        TargetScope::Clients(ids) => ids.contains(&client_id.to_string()),
        TargetScope::Projects(_) | TargetScope::Tags(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::repository::web_terminal_policy_repository::WebTerminalPolicyRepository;
    use async_trait::async_trait;
    use common::entity::permission::{CommandRules, CommandOverride};
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemoryDb {
        data: Mutex<HashMap<String, Vec<u8>>>,
    }

    #[async_trait]
    impl Database for MemoryDb {
        async fn set(&self, key: &str, value: &[u8]) -> CmdbResult<()> {
            self.data.lock().unwrap().insert(key.to_string(), value.to_vec());
            Ok(())
        }

        async fn get(&self, key: &str) -> CmdbResult<Option<Vec<u8>>> {
            Ok(self.data.lock().unwrap().get(key).cloned())
        }

        async fn delete(&self, key: &str) -> CmdbResult<()> {
            self.data.lock().unwrap().remove(key);
            Ok(())
        }

        async fn list_keys(&self, prefix: &str) -> CmdbResult<Vec<String>> {
            Ok(self
                .data
                .lock()
                .unwrap()
                .keys()
                .filter(|key| key.starts_with(prefix))
                .cloned()
                .collect())
        }

        async fn list_values(&self, prefix: &str) -> CmdbResult<Vec<Vec<u8>>> {
            Ok(self
                .data
                .lock()
                .unwrap()
                .iter()
                .filter(|(key, _)| key.starts_with(prefix))
                .map(|(_, value)| value.clone())
                .collect())
        }

        async fn list_entries(&self, prefix: &str) -> CmdbResult<Vec<(String, Vec<u8>)>> {
            Ok(self
                .data
                .lock()
                .unwrap()
                .iter()
                .filter(|(key, _)| key.starts_with(prefix))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect())
        }

        async fn exists(&self, key: &str) -> CmdbResult<bool> {
            Ok(self.data.lock().unwrap().contains_key(key))
        }

        async fn update_all(
            &self,
            prefix: &str,
            callback: Box<dyn Fn(String, Vec<u8>) -> Option<Vec<u8>> + Send + Sync>,
        ) -> CmdbResult<()> {
            let snapshot = self.data.lock().unwrap().clone();
            let mut updates = Vec::new();
            for (key, value) in snapshot {
                if key.starts_with(prefix) {
                    if let Some(new_value) = callback(key.clone(), value) {
                        updates.push((key, new_value));
                    }
                }
            }
            let mut data = self.data.lock().unwrap();
            for (key, value) in updates {
                data.insert(key, value);
            }
            Ok(())
        }
    }

    async fn build_service(policy: WebTerminalPolicy) -> WebTerminalService {
        let db: Arc<dyn Database> = Arc::new(MemoryDb::default());
        let repo = Arc::new(WebTerminalPolicyRepository::new(db));
        repo.save(&policy).await.unwrap();
        WebTerminalService::new(repo)
    }

    fn base_policy() -> WebTerminalPolicy {
        WebTerminalPolicy {
            id: "terminal-policy".into(),
            name: "terminal-policy".into(),
            description: String::new(),
            subject_type: SubjectType::Role,
            subject_id: "User".into(),
            target_scope: TargetScope::All,
            mode: TerminalMode::ReadOnly,
            terminal_command_rules: CommandRules::default(),
            allowed_commands: vec![],
            session_timeout_secs: 300,
            max_concurrent_sessions: 5,
            require_approval: false,
            priority: 1,
        }
    }

    #[tokio::test]
    async fn read_only_terminal_uses_command_rules_for_deny_list() {
        let mut policy = base_policy();
        policy.terminal_command_rules = CommandRules {
            default_action: CommandAction::Allow,
            overrides: vec![CommandOverride {
                pattern: "rm".into(),
                action: CommandAction::Deny,
            }],
        };
        let svc = build_service(policy).await;

        assert!(svc
            .check_command_allowed("user-1", &Role::User, "client-1", "ls -l")
            .await
            .is_ok());

        let err = svc
            .check_command_allowed("user-1", &Role::User, "client-1", "rm -rf /tmp/demo")
            .await
            .unwrap_err();
        assert!(matches!(err, CmdbError::Forbidden(_)));
        assert!(err.log_and_user_message().contains("denied by terminal command rules"));
    }

    #[tokio::test]
    async fn read_only_terminal_falls_back_to_legacy_allowed_commands() {
        let mut policy = base_policy();
        policy.allowed_commands = vec!["ls".into(), "df".into()];
        let svc = build_service(policy).await;

        assert!(svc
            .check_command_allowed("user-1", &Role::User, "client-1", "ls -l")
            .await
            .is_ok());

        let err = svc
            .check_command_allowed("user-1", &Role::User, "client-1", "hostname -i")
            .await
            .unwrap_err();
        assert!(matches!(err, CmdbError::Forbidden(_)));
    }

}
