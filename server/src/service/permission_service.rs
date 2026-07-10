use crate::repository::permission_repository::PermissionRepository;
use common::entity::permission::{
    Group, PermissionAction, PermissionRule, ResourceType, ScopeConstraint, SubjectType,
};
use common::entity::user::Role;
use common::error::{CmdbError, CmdbResult};
use std::sync::{Arc, RwLock};

pub struct PermissionService {
    repo: Arc<PermissionRepository>,
    rules_cache: Arc<RwLock<Vec<PermissionRule>>>,
}

impl std::fmt::Debug for PermissionService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PermissionService")
            .field("repo", &self.repo)
            .finish()
    }
}

impl PermissionService {
    pub fn new(repo: Arc<PermissionRepository>) -> Self {
        Self {
            repo,
            rules_cache: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Load all rules from DB into cache
    pub async fn refresh_cache(&self) -> CmdbResult<()> {
        let rules = self.repo.list_rules().await?;
        let mut cache = self.rules_cache.write().map_err(|e| {
            CmdbError::Internal(format!("Permission cache lock error: {}", e))
        })?;
        *cache = rules;
        Ok(())
    }

    fn get_cached_rules(&self) -> CmdbResult<Vec<PermissionRule>> {
        let cache = self.rules_cache.read().map_err(|e| {
            CmdbError::Internal(format!("Permission cache lock error: {}", e))
        })?;
        Ok(cache.clone())
    }

    // ── Rule CRUD ────────────────────────────────────────────────────────────

    #[allow(dead_code)]
    pub async fn create_rule(&self, rule: &PermissionRule) -> CmdbResult<()> {
        self.repo.save_rule(rule).await?;
        self.refresh_cache().await
    }

    #[allow(dead_code)]
    pub async fn get_rule(&self, id: &str) -> CmdbResult<Option<PermissionRule>> {
        self.repo.get_rule(id).await
    }

    #[allow(dead_code)]
    pub async fn update_rule(&self, rule: &PermissionRule) -> CmdbResult<()> {
        self.repo.save_rule(rule).await?;
        self.refresh_cache().await
    }

    #[allow(dead_code)]
    pub async fn delete_rule(&self, id: &str) -> CmdbResult<()> {
        self.repo.delete_rule(id).await?;
        self.refresh_cache().await
    }

    #[allow(dead_code)]
    pub async fn list_rules(&self) -> CmdbResult<Vec<PermissionRule>> {
        self.repo.list_rules().await
    }

    // ── Group CRUD ───────────────────────────────────────────────────────────

    #[allow(dead_code)]
    pub async fn create_group(&self, group: &Group) -> CmdbResult<()> {
        self.repo.save_group(group).await
    }

    #[allow(dead_code)]
    pub async fn get_group(&self, id: &str) -> CmdbResult<Option<Group>> {
        self.repo.get_group(id).await
    }

    #[allow(dead_code)]
    pub async fn update_group(&self, group: &Group) -> CmdbResult<()> {
        self.repo.save_group(group).await?;
        self.refresh_cache().await
    }

    #[allow(dead_code)]
    pub async fn delete_group(&self, id: &str) -> CmdbResult<()> {
        self.repo.delete_group(id).await
    }

    #[allow(dead_code)]
    pub async fn list_groups(&self) -> CmdbResult<Vec<Group>> {
        self.repo.list_groups().await
    }

    // ── Permission Matching ──────────────────────────────────────────────────

    /// Find the effective scope constraint for a given user + resource type + action
    pub fn evaluate(
        &self,
        user_id: &str,
        role: &Role,
        group_ids: &[String],
        resource_type: &ResourceType,
        action: &PermissionAction,
    ) -> CmdbResult<ScopeConstraint> {
        let rules = self.get_cached_rules()?;
        let mut matched_rules: Vec<&PermissionRule> = Vec::new();

        for rule in &rules {
            if &rule.resource_type != resource_type {
                continue;
            }
            if !rule.actions.contains(action) {
                continue;
            }
            let subject_match = match &rule.subject_type {
                SubjectType::User => &rule.subject_id == user_id,
                SubjectType::Group => group_ids.contains(&rule.subject_id),
                SubjectType::Role => {
                    let role_str = role.to_string();
                    &rule.subject_id == &role_str
                }
            };
            if subject_match {
                matched_rules.push(rule);
            }
        }

        // Sort by priority descending: highest priority wins
        matched_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        match matched_rules.first() {
            Some(rule) => Ok(rule.constraint.clone()),
            None => Ok(ScopeConstraint::None),
        }
    }

    /// Check if a user has a specific permission on a resource type
    #[allow(dead_code)]
    pub fn has_permission(
        &self,
        user_id: &str,
        role: &Role,
        group_ids: &[String],
        resource_type: &ResourceType,
        action: &PermissionAction,
    ) -> CmdbResult<bool> {
        let scope = self.evaluate(user_id, role, group_ids, resource_type, action)?;
        Ok(scope != ScopeConstraint::None)
    }

    /// Check if user can access a specific resource by ID
    #[allow(dead_code)]
    pub fn can_access_resource(
        &self,
        user_id: &str,
        role: &Role,
        group_ids: &[String],
        resource_type: &ResourceType,
        action: &PermissionAction,
        resource_owner_id: &Option<String>,
    ) -> CmdbResult<bool> {
        let scope = self.evaluate(user_id, role, group_ids, resource_type, action)?;
        match scope {
            ScopeConstraint::None => Ok(false),
            ScopeConstraint::All => Ok(true),
            ScopeConstraint::Owned => {
                Ok(resource_owner_id.as_deref() == Some(user_id))
            }
            ScopeConstraint::Project(_) | ScopeConstraint::Tag(_) => {
                Ok(true)
            }
        }
    }

    // ── Default presets ──────────────────────────────────────────────────────

    /// Create default preset rules if none exist (called at server startup)
    pub async fn ensure_default_rules(&self) -> CmdbResult<()> {
        if self.repo.rule_count().await? > 0 {
            return Ok(());
        }

        let defaults = vec![
            PermissionRule {
                id: "default-admin".to_string(),
                subject_type: SubjectType::Role,
                subject_id: "Admin".to_string(),
                resource_type: ResourceType::Client,
                actions: vec![
                    PermissionAction::View,
                    PermissionAction::Create,
                    PermissionAction::Update,
                    PermissionAction::Delete,
                ],
                constraint: ScopeConstraint::All,
                priority: 100,
            },
            PermissionRule {
                id: "default-admin-command".to_string(),
                subject_type: SubjectType::Role,
                subject_id: "Admin".to_string(),
                resource_type: ResourceType::Command,
                actions: vec![
                    PermissionAction::View,
                    PermissionAction::Create,
                    PermissionAction::Update,
                    PermissionAction::Delete,
                ],
                constraint: ScopeConstraint::All,
                priority: 100,
            },
            PermissionRule {
                id: "default-user".to_string(),
                subject_type: SubjectType::Role,
                subject_id: "User".to_string(),
                resource_type: ResourceType::Client,
                actions: vec![
                    PermissionAction::View,
                    PermissionAction::Create,
                    PermissionAction::Update,
                    PermissionAction::Delete,
                ],
                constraint: ScopeConstraint::Owned,
                priority: 50,
            },
            PermissionRule {
                id: "default-user-component".to_string(),
                subject_type: SubjectType::Role,
                subject_id: "User".to_string(),
                resource_type: ResourceType::Component,
                actions: vec![
                    PermissionAction::View,
                    PermissionAction::Create,
                    PermissionAction::Update,
                    PermissionAction::Delete,
                ],
                constraint: ScopeConstraint::Owned,
                priority: 50,
            },
            PermissionRule {
                id: "default-user-rack".to_string(),
                subject_type: SubjectType::Role,
                subject_id: "User".to_string(),
                resource_type: ResourceType::Rack,
                actions: vec![
                    PermissionAction::View,
                    PermissionAction::Create,
                    PermissionAction::Update,
                    PermissionAction::Delete,
                ],
                constraint: ScopeConstraint::Owned,
                priority: 50,
            },
            PermissionRule {
                id: "default-user-person".to_string(),
                subject_type: SubjectType::Role,
                subject_id: "User".to_string(),
                resource_type: ResourceType::Person,
                actions: vec![
                    PermissionAction::View,
                    PermissionAction::Create,
                    PermissionAction::Update,
                    PermissionAction::Delete,
                ],
                constraint: ScopeConstraint::Owned,
                priority: 50,
            },
            PermissionRule {
                id: "default-user-project".to_string(),
                subject_type: SubjectType::Role,
                subject_id: "User".to_string(),
                resource_type: ResourceType::Project,
                actions: vec![
                    PermissionAction::View,
                    PermissionAction::Create,
                    PermissionAction::Update,
                    PermissionAction::Delete,
                ],
                constraint: ScopeConstraint::Owned,
                priority: 50,
            },
            PermissionRule {
                id: "default-admin-rack".to_string(),
                subject_type: SubjectType::Role,
                subject_id: "Admin".to_string(),
                resource_type: ResourceType::Rack,
                actions: vec![
                    PermissionAction::View,
                    PermissionAction::Create,
                    PermissionAction::Update,
                    PermissionAction::Delete,
                ],
                constraint: ScopeConstraint::All,
                priority: 100,
            },
            PermissionRule {
                id: "default-admin-person".to_string(),
                subject_type: SubjectType::Role,
                subject_id: "Admin".to_string(),
                resource_type: ResourceType::Person,
                actions: vec![
                    PermissionAction::View,
                    PermissionAction::Create,
                    PermissionAction::Update,
                    PermissionAction::Delete,
                ],
                constraint: ScopeConstraint::All,
                priority: 100,
            },
            PermissionRule {
                id: "default-admin-project".to_string(),
                subject_type: SubjectType::Role,
                subject_id: "Admin".to_string(),
                resource_type: ResourceType::Project,
                actions: vec![
                    PermissionAction::View,
                    PermissionAction::Create,
                    PermissionAction::Update,
                    PermissionAction::Delete,
                ],
                constraint: ScopeConstraint::All,
                priority: 100,
            },
            PermissionRule {
                id: "default-admin-component".to_string(),
                subject_type: SubjectType::Role,
                subject_id: "Admin".to_string(),
                resource_type: ResourceType::Component,
                actions: vec![
                    PermissionAction::View,
                    PermissionAction::Create,
                    PermissionAction::Update,
                    PermissionAction::Delete,
                ],
                constraint: ScopeConstraint::All,
                priority: 100,
            },
            PermissionRule {
                id: "default-user-project".to_string(),
                subject_type: SubjectType::Role,
                subject_id: "User".to_string(),
                resource_type: ResourceType::Project,
                actions: vec![
                    PermissionAction::View,
                    PermissionAction::Create,
                    PermissionAction::Update,
                    PermissionAction::Delete,
                ],
                constraint: ScopeConstraint::Owned,
                priority: 50,
            },
            PermissionRule {
                id: "default-user-dictionary".to_string(),
                subject_type: SubjectType::Role,
                subject_id: "User".to_string(),
                resource_type: ResourceType::Dictionary,
                actions: vec![
                    PermissionAction::View,
                    PermissionAction::Create,
                    PermissionAction::Update,
                    PermissionAction::Delete,
                ],
                constraint: ScopeConstraint::Owned,
                priority: 50,
            },
            PermissionRule {
                id: "default-viewer".to_string(),
                subject_type: SubjectType::Role,
                subject_id: "Viewer".to_string(),
                resource_type: ResourceType::Client,
                actions: vec![PermissionAction::View],
                constraint: ScopeConstraint::Owned,
                priority: 10,
            },
            PermissionRule {
                id: "default-viewer-component".to_string(),
                subject_type: SubjectType::Role,
                subject_id: "Viewer".to_string(),
                resource_type: ResourceType::Component,
                actions: vec![PermissionAction::View],
                constraint: ScopeConstraint::Owned,
                priority: 10,
            },
            PermissionRule {
                id: "default-viewer-rack".to_string(),
                subject_type: SubjectType::Role,
                subject_id: "Viewer".to_string(),
                resource_type: ResourceType::Rack,
                actions: vec![PermissionAction::View],
                constraint: ScopeConstraint::Owned,
                priority: 10,
            },
            PermissionRule {
                id: "default-viewer-person".to_string(),
                subject_type: SubjectType::Role,
                subject_id: "Viewer".to_string(),
                resource_type: ResourceType::Person,
                actions: vec![PermissionAction::View],
                constraint: ScopeConstraint::Owned,
                priority: 10,
            },
            PermissionRule {
                id: "default-viewer-project".to_string(),
                subject_type: SubjectType::Role,
                subject_id: "Viewer".to_string(),
                resource_type: ResourceType::Project,
                actions: vec![PermissionAction::View],
                constraint: ScopeConstraint::Owned,
                priority: 10,
            },
            PermissionRule {
                id: "default-viewer-dictionary".to_string(),
                subject_type: SubjectType::Role,
                subject_id: "Viewer".to_string(),
                resource_type: ResourceType::Dictionary,
                actions: vec![PermissionAction::View],
                constraint: ScopeConstraint::Owned,
                priority: 10,
            },
        ];

        for rule in defaults {
            self.repo.save_rule(&rule).await?;
        }
        self.refresh_cache().await
    }
}
