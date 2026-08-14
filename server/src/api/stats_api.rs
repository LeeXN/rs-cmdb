//! Statistics API endpoints
//!
//! This module provides HTTP endpoints for hardware and client statistics.
//! Business logic has been moved to the StatsService.

use crate::middleware::permission::PermissionContext;
use crate::repository::client_repository::ClientRepository;
use crate::service::{
    client_filter_service::{ClientFilterService, HardwareFilterQuery},
    export_service::ExportService,
    stats_service::{
        CategoryStats as ServiceCategoryStats, OverallStats as ServiceOverallStats, StatsService,
    },
};
use axum::{
    Json,
    extract::{Extension, Query},
    http::StatusCode,
    response::IntoResponse,
};
use axum_macros::debug_handler;
use common::entity::permission::{PermissionAction, ResourceType, ScopeConstraint};
use common::models::{
    ApiResponse, Client, ClientHardwareExport, DetailedStats, FilterCriteria, FilterOptions,
    StatItem,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{error, info, instrument};

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsQuery {
    pub category: Option<String>, // cpu, memory, gpu, disk, nic, os, kernel, server_model
}

// Re-export types from service for API compatibility
#[derive(Debug, Serialize, PartialEq)]
pub struct CategoryStats {
    pub category: String,
    pub total_clients: usize,
    pub items: Vec<StatItem>,
}

impl From<ServiceCategoryStats> for CategoryStats {
    fn from(inner: ServiceCategoryStats) -> Self {
        Self {
            category: inner.category,
            total_clients: inner.total_clients,
            items: inner.items,
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct OverallStats {
    pub total_clients: usize,
    pub online_clients: usize,
    pub offline_clients: usize,
    pub categories: Vec<CategoryStats>,
}

impl From<ServiceOverallStats> for OverallStats {
    fn from(inner: ServiceOverallStats) -> Self {
        Self {
            total_clients: inner.total_clients,
            online_clients: inner.online_clients,
            offline_clients: inner.offline_clients,
            categories: inner
                .categories
                .into_iter()
                .map(CategoryStats::from)
                .collect(),
        }
    }
}

/// Get hardware statistics
#[debug_handler]
#[instrument(skip(stats_service, client_repo, perm_ctx))]
pub async fn get_hardware_stats(
    Query(params): Query<StatsQuery>,
    Extension(stats_service): Extension<Arc<StatsService>>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    let allowed_client_ids = match authorized_client_ids(&client_repo, &perm_ctx).await {
        Ok(ids) => ids,
        Err(err) => {
            error!("Failed to resolve client permissions: {}", err);
            let response = ApiResponse::<OverallStats> {
                status: 500,
                message: "Failed to resolve client permissions".to_string(),
                data: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
        }
    };
    match stats_service
        .get_overall_stats_for_client_ids(params.category.as_deref(), allowed_client_ids.as_ref())
        .await
    {
        Ok(stats) => {
            let api_stats = OverallStats::from(stats);
            info!(
                "Generated hardware stats for {} clients",
                api_stats.total_clients
            );

            let response = ApiResponse {
                status: 200,
                message: "Hardware statistics retrieved successfully".to_string(),
                data: Some(api_stats),
            };

            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to generate hardware stats: {}", err);
            let response = ApiResponse::<OverallStats> {
                status: 500,
                message: err,
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

/// Get detailed hardware statistics
#[debug_handler]
#[instrument(skip(stats_service, client_repo, perm_ctx))]
pub async fn get_detailed_stats(
    Extension(stats_service): Extension<Arc<StatsService>>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    let allowed_client_ids = match authorized_client_ids(&client_repo, &perm_ctx).await {
        Ok(ids) => ids,
        Err(err) => {
            error!("Failed to resolve client permissions: {}", err);
            let response = ApiResponse::<DetailedStats> {
                status: 500,
                message: "Failed to resolve client permissions".to_string(),
                data: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
        }
    };
    match stats_service
        .get_detailed_stats_for_client_ids(allowed_client_ids.as_ref())
        .await
    {
        Ok(stats) => {
            info!(
                "Generated detailed stats for {} clients",
                stats.total_clients
            );

            let response = ApiResponse {
                status: 200,
                message: "Detailed statistics retrieved successfully".to_string(),
                data: Some(stats),
            };

            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to generate detailed stats: {}", err);
            let response = ApiResponse::<DetailedStats> {
                status: 500,
                message: err,
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

/// Get filter options from actual database data
#[debug_handler]
#[instrument(skip(stats_service, client_repo, perm_ctx))]
pub async fn get_filter_options(
    Extension(stats_service): Extension<Arc<StatsService>>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    let allowed_client_ids = match authorized_client_ids(&client_repo, &perm_ctx).await {
        Ok(ids) => ids,
        Err(err) => {
            error!("Failed to resolve client permissions: {}", err);
            let response = ApiResponse::<FilterOptions> {
                status: 500,
                message: "Failed to resolve client permissions".to_string(),
                data: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
        }
    };
    match stats_service
        .get_filter_options_for_client_ids(allowed_client_ids.as_ref())
        .await
    {
        Ok(options) => {
            info!("Generated filter options");

            let response = ApiResponse {
                status: 200,
                message: "Filter options retrieved successfully".to_string(),
                data: Some(options),
            };

            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to generate filter options: {}", err);
            let response = ApiResponse::<FilterOptions> {
                status: 500,
                message: err,
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

/// Get clients by specific hardware criteria
#[debug_handler]
#[instrument(skip(client_repo))]
pub async fn get_clients_by_criteria(
    Query(params): Query<HashMap<String, String>>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    let clients = match client_repo.list_all().await {
        Ok(clients) => clients,
        Err(err) => {
            error!("Failed to list clients for criteria: {}", err);
            let response = ApiResponse::<Vec<Client>> {
                status: err.status_code(),
                message: err.log_and_user_message(),
                data: None,
            };
            return (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            );
        }
    };

    // Filter clients based on criteria
    let filtered_clients = if let Some(ids_str) = params.get("ids") {
        let ids: Vec<&str> = ids_str.split(',').collect();
        clients
            .into_iter()
            .filter(|c| ids.contains(&c.id.as_str()))
            .collect()
    } else {
        clients
    };
    let filtered_clients = filter_clients_by_view_scope(filtered_clients, &perm_ctx);

    info!("Retrieved {} clients by criteria", filtered_clients.len());

    let response = ApiResponse {
        status: 200,
        message: "Clients retrieved successfully".to_string(),
        data: Some(filtered_clients),
    };

    (StatusCode::OK, Json(response))
}

/// Filter clients by criteria
#[debug_handler]
#[instrument(skip(client_filter_service))]
pub async fn filter_clients(
    Extension(client_filter_service): Extension<Arc<ClientFilterService>>,
    Extension(perm_ctx): Extension<PermissionContext>,
    Json(filter): Json<FilterCriteria>,
) -> impl IntoResponse {
    // Convert FilterCriteria to HardwareFilterQuery
    // This mapping handles the difference between the API model and the Service model
    let query = HardwareFilterQuery {
        search_term: None, // FilterCriteria doesn't have a global search term
        os_filter: filter.os_name,
        os_kernel_filter: filter.os_kernel,
        cpu_vendor_filter: filter.cpu_vendor,
        cpu_model_filter: filter.cpu_model,
        gpu_vendor_filter: filter.gpu_vendor,
        gpu_model_filter: filter.gpu_model,
        memory_min_filter: filter.memory_capacity_min,
        memory_max_filter: filter.memory_capacity_max,
        server_vendor_filter: filter.server_vendor,
        server_model_filter: None, // FilterCriteria doesn't have server model
        network_type_filter: filter.network_type,
        network_model_filter: None,
        storage_type_filter: filter.storage_type,
        status_filter: None,
        client_status_filter: None,
        environment_filter: None,
        rack_id_filter: None,
        project_id_filter: None,
        owner_id_filter: None,
    };

    match client_filter_service
        .filter_clients_by_hardware(&query)
        .await
    {
        Ok(filtered_clients) => {
            let filtered_clients = filter_clients_by_view_scope(filtered_clients, &perm_ctx);
            info!("Filtered clients: {} matches found", filtered_clients.len());
            let response = ApiResponse {
                status: 200,
                message: "Clients filtered successfully".to_string(),
                data: Some(filtered_clients),
            };
            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to filter clients: {}", err);
            let response = ApiResponse::<Vec<Client>> {
                status: 500, // ClientFilterService currently returns String error, mapping to 500
                message: err,
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

/// Export detailed client and hardware data
#[debug_handler]
#[instrument(skip(export_service, client_repo, perm_ctx))]
pub async fn export_client_hardware_data(
    Extension(export_service): Extension<Arc<ExportService>>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    let allowed_client_ids = match authorized_client_ids(&client_repo, &perm_ctx).await {
        Ok(ids) => ids,
        Err(err) => {
            error!("Failed to resolve client permissions: {}", err);
            let response = ApiResponse::<Vec<ClientHardwareExport>> {
                status: 500,
                message: "Failed to resolve client permissions".to_string(),
                data: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
        }
    };
    match export_service
        .export_client_hardware_data_for_client_ids(allowed_client_ids.as_ref())
        .await
    {
        Ok(export_data) => {
            let response = ApiResponse {
                status: 200,
                message: "Export data retrieved successfully".to_string(),
                data: Some(export_data),
            };
            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to export data: {}", err);
            let response = ApiResponse::<Vec<ClientHardwareExport>> {
                status: 500,
                message: err,
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

fn filter_clients_by_view_scope(clients: Vec<Client>, perm_ctx: &PermissionContext) -> Vec<Client> {
    if perm_ctx.is_admin() {
        return clients;
    }
    let scope = perm_ctx
        .evaluate(&ResourceType::Client, &PermissionAction::View)
        .unwrap_or(ScopeConstraint::None);
    clients
        .into_iter()
        .filter(|client| {
            PermissionContext::matches_scope(
                &scope,
                &perm_ctx.user_id,
                client.created_by.as_deref(),
                client.project_id.as_deref(),
                &PermissionContext::client_tags(client),
            )
        })
        .collect()
}

async fn authorized_client_ids(
    client_repo: &ClientRepository,
    perm_ctx: &PermissionContext,
) -> Result<Option<HashSet<String>>, String> {
    if perm_ctx.is_admin() {
        return Ok(None);
    }
    let scope = perm_ctx
        .evaluate(&ResourceType::Client, &PermissionAction::View)
        .unwrap_or(ScopeConstraint::None);
    if matches!(scope, ScopeConstraint::All) {
        return Ok(None);
    }
    let clients = client_repo
        .list_all()
        .await
        .map_err(|err| err.to_string())?;
    Ok(Some(
        filter_clients_by_view_scope(clients, perm_ctx)
            .into_iter()
            .map(|client| client.id)
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use crate::tests::fixtures::{TestAppBuilder, auth_headers};
    use axum::{
        body::Body,
        extract::Request,
        http::{Method, StatusCode},
    };
    use tower::ServiceExt;

    async fn make_get(
        app: &axum::Router,
        path: &str,
        token: Option<&str>,
    ) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder().method(Method::GET).uri(path);
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req.body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        (status, body)
    }

    #[tokio::test]
    async fn test_hardware_stats() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_get(
            &app.router,
            "/api/v1/stats/hardware",
            Some(&app.admin_token),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(body["data"].is_object());
    }

    #[tokio::test]
    async fn test_detailed_stats() {
        let app = TestAppBuilder::new().build().await;
        let (status, _body) = make_get(
            &app.router,
            "/api/v1/stats/detailed",
            Some(&app.admin_token),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_filter_options() {
        let app = TestAppBuilder::new().build().await;
        let (status, _body) = make_get(
            &app.router,
            "/api/v1/filter_options",
            Some(&app.admin_token),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }
}
