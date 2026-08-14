use crate::service::permission_service::PermissionService;
use axum::{
    extract::{Extension, Request},
    middleware::Next,
    response::Response,
};
use common::entity::permission::{PermissionAction, ResourceType, ScopeConstraint};
use common::entity::user::{Role, User};
use common::error::CmdbResult;
use common::models::Client;
use std::sync::Arc;

/// Context injected by PermissionMiddleware for use in handlers and services
#[derive(Debug, Clone)]
pub struct PermissionContext {
    pub user_id: String,
    pub role: Role,
    pub group_ids: Vec<String>,
    pub permission_service: Arc<PermissionService>,
}

impl PermissionContext {
    /// Evaluate the effective scope constraint for a resource+action combination
    pub fn evaluate(
        &self,
        resource_type: &ResourceType,
        action: &PermissionAction,
    ) -> CmdbResult<ScopeConstraint> {
        self.permission_service.evaluate(
            &self.user_id,
            &self.role,
            &self.group_ids,
            resource_type,
            action,
        )
    }

    /// Check if the user has any permission for a given resource+action
    #[allow(dead_code)]
    pub fn has_permission(
        &self,
        resource_type: &ResourceType,
        action: &PermissionAction,
    ) -> CmdbResult<bool> {
        self.permission_service.has_permission(
            &self.user_id,
            &self.role,
            &self.group_ids,
            resource_type,
            action,
        )
    }

    /// Filter a collection based on the scope constraint for a resource+action
    pub fn filter_by_scope<T>(
        items: Vec<T>,
        constraint: &ScopeConstraint,
        user_id: &str,
        owner_id: impl Fn(&T) -> Option<&str>,
    ) -> Vec<T> {
        match constraint {
            ScopeConstraint::All => items,
            ScopeConstraint::Owned => items
                .into_iter()
                .filter(|i| owner_id(i) == Some(user_id))
                .collect(),
            ScopeConstraint::None | ScopeConstraint::Project(_) | ScopeConstraint::Tag(_) => vec![],
        }
    }

    /// Check if the current user is an Admin (bypasses ownership checks)
    pub fn is_admin(&self) -> bool {
        self.role == Role::Admin
    }

    /// Check if user can access a specific resource based on ownership
    #[allow(dead_code)]
    pub fn can_access_resource(
        &self,
        resource_type: &ResourceType,
        action: &PermissionAction,
        resource_owner_id: &Option<String>,
    ) -> CmdbResult<bool> {
        self.permission_service.can_access_resource(
            &self.user_id,
            &self.role,
            &self.group_ids,
            resource_type,
            action,
            resource_owner_id,
        )
    }

    /// Authorize one already-loaded resource using its owner metadata. Project
    /// and tag constraints are intentionally fail-closed here because callers
    /// must provide the corresponding resource attributes explicitly.
    pub fn allows_resource(
        &self,
        resource_type: &ResourceType,
        action: &PermissionAction,
        owner_id: Option<&str>,
    ) -> bool {
        self.allows_resource_with_scope(resource_type, action, owner_id, None, &[])
    }

    /// Authorize a resource when its project and tag relationships are known.
    /// Tags are matched against both the explicit `tags` collection and the
    /// legacy single-valued `asset_tag` field on clients.
    pub fn allows_resource_with_scope(
        &self,
        resource_type: &ResourceType,
        action: &PermissionAction,
        owner_id: Option<&str>,
        project_id: Option<&str>,
        tags: &[String],
    ) -> bool {
        if self.is_admin() {
            return true;
        }
        Self::matches_scope(
            &self
                .evaluate(resource_type, action)
                .unwrap_or(ScopeConstraint::None),
            &self.user_id,
            owner_id,
            project_id,
            tags,
        )
    }

    /// Check a permission for a new resource before it has owner metadata.
    pub fn allows_action(&self, resource_type: &ResourceType, action: &PermissionAction) -> bool {
        self.allows_action_with_scope(resource_type, action, None, &[])
    }

    /// Check a permission for a new resource using the attributes supplied by
    /// the request. Owned creates are allowed because handlers assign the
    /// authenticated user as the owner before persistence.
    pub fn allows_action_with_scope(
        &self,
        resource_type: &ResourceType,
        action: &PermissionAction,
        project_id: Option<&str>,
        tags: &[String],
    ) -> bool {
        if self.is_admin() {
            return true;
        }
        match self
            .evaluate(resource_type, action)
            .unwrap_or(ScopeConstraint::None)
        {
            ScopeConstraint::All | ScopeConstraint::Owned => true,
            ScopeConstraint::Project(_) | ScopeConstraint::Tag(_) => Self::matches_scope(
                &self
                    .evaluate(resource_type, action)
                    .unwrap_or(ScopeConstraint::None),
                &self.user_id,
                None,
                project_id,
                tags,
            ),
            ScopeConstraint::None => false,
        }
    }

    /// Match a concrete resource against an already-evaluated scope.
    pub fn matches_scope(
        constraint: &ScopeConstraint,
        user_id: &str,
        owner_id: Option<&str>,
        project_id: Option<&str>,
        tags: &[String],
    ) -> bool {
        match constraint {
            ScopeConstraint::All => true,
            ScopeConstraint::Owned => owner_id == Some(user_id),
            ScopeConstraint::Project(project_ids) => {
                project_id.is_some_and(|project_id| project_ids.iter().any(|id| id == project_id))
            }
            ScopeConstraint::Tag(required_tags) => tags
                .iter()
                .any(|tag| required_tags.iter().any(|required| required == tag)),
            ScopeConstraint::None => false,
        }
    }

    /// Return all tags associated with a client. `asset_tag` is retained as a
    /// compatibility alias for deployments created before multi-tag support.
    pub fn client_tags(client: &Client) -> Vec<String> {
        let mut tags = client
            .tags
            .iter()
            .chain(client.asset_tag.iter())
            .map(|tag| tag.trim().to_string())
            .filter(|tag| !tag.is_empty())
            .collect::<Vec<_>>();
        tags.sort();
        tags.dedup();
        tags
    }
}

/// Middleware that injects a PermissionContext extension for each authenticated request
pub async fn permission_middleware(
    Extension(user): Extension<User>,
    Extension(perm_svc): Extension<Arc<PermissionService>>,
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let group_ids = perm_svc
        .group_ids_for_user(&user.id)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    let ctx = PermissionContext {
        user_id: user.id.clone(),
        role: user.role.clone(),
        group_ids,
        permission_service: perm_svc,
    };

    let mut request = request;
    request.extensions_mut().insert(ctx);
    Ok(next.run(request).await)
}
