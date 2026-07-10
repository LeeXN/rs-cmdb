use crate::service::permission_service::PermissionService;
use axum::{
    extract::{Extension, Request},
    middleware::Next,
    response::Response,
};
use common::entity::permission::{PermissionAction, ResourceType, ScopeConstraint};
use common::entity::user::{Role, User};
use common::error::CmdbResult;
use std::sync::Arc;

/// Context injected by PermissionMiddleware for use in handlers and services
#[derive(Debug, Clone)]
pub struct PermissionContext {
    pub user_id: String,
    pub role: Role,
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
            &[],
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
            &[],
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
            ScopeConstraint::None
            | ScopeConstraint::Project(_)
            | ScopeConstraint::Tag(_) => vec![],
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
            &[],
            resource_type,
            action,
            resource_owner_id,
        )
    }
}

/// Middleware that injects a PermissionContext extension for each authenticated request
pub async fn permission_middleware(
    Extension(user): Extension<User>,
    Extension(perm_svc): Extension<Arc<PermissionService>>,
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let ctx = PermissionContext {
        user_id: user.id.clone(),
        role: user.role.clone(),
        permission_service: perm_svc,
    };

    let mut request = request;
    request.extensions_mut().insert(ctx);
    Ok(next.run(request).await)
}
