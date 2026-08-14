//! Web Terminal Service
//!
//! Handles session lifecycle checks for the web terminal feature.
//! The actual WebSocket terminal is not yet implemented; this service
//! provides policy enforcement hooks ready for integration.

use crate::middleware::permission::PermissionContext;
use crate::repository::client_repository::ClientRepository;
use crate::repository::web_terminal_policy_repository::WebTerminalPolicyRepository;
use common::entity::permission::{
    CommandAction, PermissionAction, ResourceType, ScopeConstraint, SubjectType, TargetScope,
    TerminalMode, WebTerminalPolicy,
};
use common::entity::user::Role;
use common::error::{CmdbError, CmdbResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Active session tracker (in-memory, per-user session count)
pub struct WebTerminalService {
    policy_repo: Arc<WebTerminalPolicyRepository>,
    client_repo: Option<Arc<ClientRepository>>,
    /// user_id → active session count
    active_sessions: RwLock<HashMap<String, usize>>,
}

impl WebTerminalService {
    pub fn new(policy_repo: Arc<WebTerminalPolicyRepository>) -> Self {
        Self {
            policy_repo,
            client_repo: None,
            active_sessions: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_client_repo(mut self, client_repo: Arc<ClientRepository>) -> Self {
        self.client_repo = Some(client_repo);
        self
    }

    /// Apply project/tag client permission scopes to terminal operations.
    ///
    /// Terminal policies remain the authority for whether a session may be
    /// opened at all. This check closes the gap for an already-created
    /// session when the generic Client permission is project/tag scoped. A
    /// missing generic Client rule is left to the terminal policy, preserving
    /// deployments that use terminal policies independently of CMDB CRUD
    /// permissions.
    pub async fn allows_client_permission_scope(
        &self,
        perm_ctx: &PermissionContext,
        client_id: &str,
        action: &PermissionAction,
    ) -> bool {
        if perm_ctx.is_admin() {
            return true;
        }

        let scope = perm_ctx
            .evaluate(&ResourceType::Client, action)
            .unwrap_or(ScopeConstraint::None);
        match scope {
            ScopeConstraint::Project(_) | ScopeConstraint::Tag(_) => {
                let Some(repo) = self.client_repo.as_ref() else {
                    return false;
                };
                let Some(client) = repo.get(client_id).await.ok().flatten() else {
                    return false;
                };
                PermissionContext::matches_scope(
                    &scope,
                    &perm_ctx.user_id,
                    client.created_by.as_deref(),
                    client.project_id.as_deref(),
                    &PermissionContext::client_tags(&client),
                )
            }
            // All/Owned are covered by the session owner/admin check. A
            // missing generic rule is intentionally not made a second deny
            // point because WebTerminalPolicy is independently enforced.
            ScopeConstraint::All | ScopeConstraint::Owned | ScopeConstraint::None => true,
        }
    }

    // ── 5.3: Policy check on session creation ────────────────────────────────

    /// Check whether a user may open a new terminal session to the given client.
    ///
    /// Returns `Ok(TerminalMode)` if allowed; `Err` if denied.
    #[allow(dead_code)]
    pub async fn check_session_allowed(
        &self,
        user_id: &str,
        role: &Role,
        client_id: &str,
    ) -> CmdbResult<TerminalMode> {
        self.check_session_allowed_for_groups(user_id, role, &[], client_id)
            .await
    }

    #[allow(dead_code)]
    pub async fn check_session_allowed_for_groups(
        &self,
        user_id: &str,
        role: &Role,
        group_ids: &[String],
        client_id: &str,
    ) -> CmdbResult<TerminalMode> {
        let policies = self.policy_repo.list_all().await?;

        // Find the highest-priority matching policy for this user+client
        let mut matched = self
            .matching_policies(&policies, user_id, role, group_ids, client_id)
            .await;

        matched.sort_by_key(|policy| std::cmp::Reverse(policy.priority));

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

    /// Check policy and reserve a concurrent-session slot atomically with the
    /// limit check. Session creation uses this variant so two simultaneous
    /// requests cannot both pass the read-only counter check.
    pub async fn reserve_session_slot_for_groups(
        &self,
        user_id: &str,
        role: &Role,
        group_ids: &[String],
        client_id: &str,
    ) -> CmdbResult<TerminalMode> {
        let policies = self.policy_repo.list_all().await?;
        let mut matched = self
            .matching_policies(&policies, user_id, role, group_ids, client_id)
            .await;
        matched.sort_by_key(|policy| std::cmp::Reverse(policy.priority));
        let policy = matched.first().copied().ok_or_else(|| {
            CmdbError::Forbidden("No terminal policy grants access to this client".into())
        })?;

        let mut sessions = self.active_sessions.write().unwrap();
        let current = sessions.get(user_id).copied().unwrap_or(0);
        if policy.max_concurrent_sessions > 0 && current >= policy.max_concurrent_sessions as usize
        {
            return Err(CmdbError::Forbidden(format!(
                "Maximum of {} concurrent terminal sessions already active",
                policy.max_concurrent_sessions
            )));
        }
        *sessions.entry(user_id.to_string()).or_insert(0) += 1;
        Ok(policy.mode.clone())
    }

    // ── 5.4: Read-only command rule filtering ─────────────────────────────────

    /// Filter a command against the read-only terminal command rules.
    ///
    /// Returns `Ok(())` if the command is allowed; `Err(Forbidden)` if not.
    #[allow(dead_code)]
    pub async fn check_command_allowed(
        &self,
        user_id: &str,
        role: &Role,
        client_id: &str,
        command: &str,
    ) -> CmdbResult<()> {
        self.check_command_allowed_for_groups(user_id, role, &[], client_id, command)
            .await
    }

    pub async fn check_command_allowed_for_groups(
        &self,
        user_id: &str,
        role: &Role,
        group_ids: &[String],
        client_id: &str,
        command: &str,
    ) -> CmdbResult<()> {
        let policies = self.policy_repo.list_all().await?;

        let best = self
            .matching_policies(&policies, user_id, role, group_ids, client_id)
            .await
            .into_iter()
            .max_by_key(|p| p.priority);

        let policy = match best {
            Some(p) => p,
            None => {
                return Err(CmdbError::Forbidden(
                    "Terminal policy no longer grants access to this session".into(),
                ));
            }
        };

        // Only enforce command filtering in ReadOnly mode
        if policy.mode == TerminalMode::ReadOnly {
            if command
                .chars()
                .any(|ch| matches!(ch, '|' | ';' | '&' | '$' | '`' | '>' | '<'))
            {
                return Err(CmdbError::Forbidden(
                    "shell operators are not allowed in read-only terminal mode".into(),
                ));
            }
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
    pub async fn get_session_timeout(&self, user_id: &str, role: &Role, client_id: &str) -> u64 {
        self.get_session_timeout_for_groups(user_id, role, &[], client_id)
            .await
    }

    pub async fn get_session_timeout_for_groups(
        &self,
        user_id: &str,
        role: &Role,
        group_ids: &[String],
        client_id: &str,
    ) -> u64 {
        let Ok(policies) = self.policy_repo.list_all().await else {
            return 0;
        };
        self.matching_policies(&policies, user_id, role, group_ids, client_id)
            .await
            .into_iter()
            .max_by_key(|p| p.priority)
            .map(|p| p.session_timeout_secs)
            .unwrap_or(0)
    }

    // ── 5.6: Concurrent session limit ────────────────────────────────────────

    #[allow(dead_code)]
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

    /// Track session close (decrement user's count)
    pub fn on_session_close(&self, user_id: &str) {
        let mut sessions = self.active_sessions.write().unwrap();
        if let Some(count) = sessions.get_mut(user_id) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }

    async fn matching_policies<'a>(
        &self,
        policies: &'a [WebTerminalPolicy],
        user_id: &str,
        role: &Role,
        group_ids: &[String],
        client_id: &str,
    ) -> Vec<&'a WebTerminalPolicy> {
        let mut matched = Vec::new();
        for policy in policies {
            if subject_matches(
                &policy.subject_type,
                &policy.subject_id,
                user_id,
                role,
                group_ids,
            ) && scope_matches(&policy.target_scope, client_id, self.client_repo.as_ref()).await
            {
                matched.push(policy);
            }
        }
        matched
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
        SubjectType::Group => group_ids.iter().any(|group_id| group_id == subject_id),
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
    scope: &TargetScope,
    client_id: &str,
    client_repo: Option<&Arc<ClientRepository>>,
) -> bool {
    match scope {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::repository::web_terminal_policy_repository::WebTerminalPolicyRepository;
    use async_trait::async_trait;
    use common::entity::permission::{CommandOverride, CommandRules};
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemoryDb {
        data: Mutex<HashMap<String, Vec<u8>>>,
    }

    #[async_trait]
    impl Database for MemoryDb {
        async fn set(&self, key: &str, value: &[u8]) -> CmdbResult<()> {
            self.data
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_vec());
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

        assert!(
            svc.check_command_allowed("user-1", &Role::User, "client-1", "ls -l")
                .await
                .is_ok()
        );

        let err = svc
            .check_command_allowed("user-1", &Role::User, "client-1", "rm -rf /tmp/demo")
            .await
            .unwrap_err();
        assert!(matches!(err, CmdbError::Forbidden(_)));
        assert!(
            err.log_and_user_message()
                .contains("denied by terminal command rules")
        );
    }

    #[tokio::test]
    async fn read_only_terminal_falls_back_to_legacy_allowed_commands() {
        let mut policy = base_policy();
        policy.allowed_commands = vec!["ls".into(), "df".into()];
        let svc = build_service(policy).await;

        assert!(
            svc.check_command_allowed("user-1", &Role::User, "client-1", "ls -l")
                .await
                .is_ok()
        );

        let err = svc
            .check_command_allowed("user-1", &Role::User, "client-1", "hostname -i")
            .await
            .unwrap_err();
        assert!(matches!(err, CmdbError::Forbidden(_)));
    }

    #[tokio::test]
    async fn group_subject_policy_matches_only_members() {
        let mut policy = base_policy();
        policy.subject_type = SubjectType::Group;
        policy.subject_id = "ops".into();
        let svc = build_service(policy).await;

        let groups = vec!["ops".to_string()];
        assert!(
            svc.check_session_allowed_for_groups("user-1", &Role::User, &groups, "client-1")
                .await
                .is_ok()
        );
        assert!(
            svc.check_session_allowed_for_groups("user-1", &Role::User, &[], "client-1")
                .await
                .is_err()
        );
    }
}
