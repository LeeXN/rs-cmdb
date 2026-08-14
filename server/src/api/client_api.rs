//! Client API endpoints
//!
//! This module provides HTTP endpoints for client management.
//! Filtering and search logic has been moved to ClientFilterService.

use crate::middleware::permission::PermissionContext;
use crate::repository::{
    client_repository::ClientRepository, component_repository::ComponentRepository,
};
use crate::service::{
    auth_service::verify_token,
    client_filter_service::{ClientFilterService, HardwareFilterQuery, SearchQuery},
    client_service::ClientService,
    validation_service::ValidationService,
};
use crate::validation::validate_ip_address;
use axum::{
    extract::{Extension, Json, Path, Query},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use axum_extra::headers::{Authorization, authorization::Bearer};
use axum_macros::debug_handler;
use common::entity::permission::{PermissionAction, ResourceType, ScopeConstraint};
use common::entity::user::User;
use common::models::{
    ApiResponse, Client, ClientQuery, ExportFilterRequest, ExportFilterResponse, FilterOptions,
    PaginatedResult, RegisterClientResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{error, info, instrument, warn};

const AGENT_TOKEN_CAPABILITY_HEADER: &str = "x-rs-cmdb-agent-token-capable";

/// List all clients
#[debug_handler]
#[instrument(skip(client_repo))]
pub async fn list_clients(
    Query(query): Query<ClientQuery>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    if let Some(ref search) = query.search {
        if search.len() > crate::constants::MAX_FILTER_LENGTH {
            let response = ApiResponse::<PaginatedResult<Client>> {
                status: 400,
                message: format!(
                    "Search query exceeds maximum length of {} characters",
                    crate::constants::MAX_FILTER_LENGTH
                ),
                data: None,
            };
            return (StatusCode::BAD_REQUEST, Json(response));
        }
    }
    if let Some(ref os) = query.os {
        if os.len() > crate::constants::MAX_FILTER_LENGTH {
            let response = ApiResponse::<PaginatedResult<Client>> {
                status: 400,
                message: format!(
                    "OS filter exceeds maximum length of {} characters",
                    crate::constants::MAX_FILTER_LENGTH
                ),
                data: None,
            };
            return (StatusCode::BAD_REQUEST, Json(response));
        }
    }
    if let Some(ref status) = query.status {
        if status.len() > crate::constants::MAX_FILTER_LENGTH {
            let response = ApiResponse::<PaginatedResult<Client>> {
                status: 400,
                message: format!(
                    "Status filter exceeds maximum length of {} characters",
                    crate::constants::MAX_FILTER_LENGTH
                ),
                data: None,
            };
            return (StatusCode::BAD_REQUEST, Json(response));
        }
    }
    match client_repo.list_all().await {
        Ok(mut clients) => {
            clients = filter_clients_by_view_scope(clients, &perm_ctx);
            // Filter by search term
            if let Some(ref search) = query.search {
                let search_lower = search.to_lowercase();
                clients.retain(|c| {
                    c.hostname.to_lowercase().contains(&search_lower)
                        || c.ip_address.contains(&search_lower)
                        || c.primary_ip
                            .as_deref()
                            .is_some_and(|ip| ip.contains(&search_lower))
                        || c.os
                            .as_ref()
                            .is_some_and(|os| os.to_lowercase().contains(&search_lower))
                });
            }

            // Filter by OS
            if let Some(ref os) = query.os
                && os != "all"
            {
                clients.retain(|c| c.os.as_ref() == Some(os));
            }

            // Filter by status
            if let Some(ref status) = query.status
                && status != "all"
            {
                let now = chrono::Utc::now();
                clients.retain(|c| {
                    let is_online = c
                        .last_seen
                        .as_ref()
                        .and_then(|last_seen| chrono::DateTime::parse_from_rfc3339(last_seen).ok())
                        .map(|dt| {
                            let duration =
                                now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                            duration.num_minutes() <= 5
                        })
                        .unwrap_or(false);

                    match status.as_str() {
                        "online" => is_online,
                        "offline" => !is_online,
                        _ => true,
                    }
                });
            }

            // Sort by hostname
            clients.sort_by(|a, b| a.hostname.cmp(&b.hostname));

            // Pagination
            let total = clients.len();
            let page = query.page.unwrap_or(1);
            let page_size = query.page_size.unwrap_or(10);
            let total_pages = (total as f64 / page_size as f64).ceil() as usize;

            let start = (page - 1) * page_size;
            let end = std::cmp::min(start + page_size, total);

            let items = if start < total {
                clients[start..end].to_vec()
            } else {
                Vec::new()
            };

            info!(
                "Listed {} clients (page {}/{})",
                items.len(),
                page,
                total_pages
            );

            let result = PaginatedResult {
                items,
                total,
                page,
                page_size,
                total_pages,
            };

            let response = ApiResponse {
                status: 200,
                message: "Clients retrieved successfully".to_string(),
                data: Some(result),
            };

            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to list clients: {}", err);
            let response = ApiResponse::<PaginatedResult<Client>> {
                status: err.status_code(),
                message: err.log_and_user_message(),
                data: None,
            };

            (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

/// Get a client by ID
#[debug_handler]
#[instrument(skip(client_repo))]
pub async fn get_client(
    Path(client_id): Path<String>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    match client_repo.get(&client_id).await {
        Ok(Some(client)) => {
            if !perm_ctx.allows_resource_with_scope(
                &ResourceType::Client,
                &PermissionAction::View,
                client.created_by.as_deref(),
                client.project_id.as_deref(),
                &PermissionContext::client_tags(&client),
            ) {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::<Client> {
                        status: 403,
                        message: "Forbidden".into(),
                        data: None,
                    }),
                );
            }
            let response = ApiResponse {
                status: 200,
                message: "Client retrieved successfully".to_string(),
                data: Some(client),
            };

            (StatusCode::OK, Json(response))
        }
        Ok(None) => {
            let response = ApiResponse::<Client> {
                status: 404,
                message: format!("Client {} not found", client_id),
                data: None,
            };

            (StatusCode::NOT_FOUND, Json(response))
        }
        Err(err) => {
            error!("Failed to get client {}: {}", client_id, err);
            let response = ApiResponse::<Client> {
                status: err.status_code(),
                message: err.log_and_user_message(),
                data: None,
            };

            (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

/// Register a new client
#[debug_handler]
#[instrument(skip(client_service, validation_service, auth_header, headers, registration))]
pub async fn register_client(
    Extension(client_service): Extension<Arc<ClientService>>,
    Extension(validation_service): Extension<Arc<ValidationService>>,
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    headers: HeaderMap,
    Json(mut registration): Json<Client>,
) -> impl IntoResponse {
    info!("Registering client: {}", registration.hostname);

    registration.hostname = registration.hostname.trim().to_string();
    registration.id = registration.id.trim().to_string();
    let token_capable = headers
        .get(AGENT_TOKEN_CAPABILITY_HEADER)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == "1");
    if registration.hostname.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<RegisterClientResponse> {
                status: 400,
                message: "client hostname must not be empty".into(),
                data: None,
            }),
        );
    }

    // Registration is public only for a new identity. Updating an existing
    // client record must prove possession of that agent's current token;
    // otherwise anyone who guesses a client_id or serial number could
    // overwrite its metadata and mint a fresh credential.
    let requested_existing = if !registration.id.is_empty() {
        client_service.get_client(&registration.id).await
    } else {
        Ok(None)
    };
    let requested_existing = match requested_existing {
        Ok(existing) => existing,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<RegisterClientResponse> {
                    status: 500,
                    message: err.log_and_user_message(),
                    data: None,
                }),
            );
        }
    };

    // Always inspect the hardware identity too. A buggy upgrade may already
    // have created a duplicate record under a generated ID; in that case the
    // requested ID exists, but the persisted token still belongs to the
    // original record selected by serial number.
    let serial_existing = if let Some(serial) = registration
        .serial_number
        .as_deref()
        .map(str::trim)
        .filter(|serial| !serial.is_empty())
    {
        match client_service.get_by_serial(serial).await {
            Ok(existing) => existing,
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<RegisterClientResponse> {
                        status: 500,
                        message: err.log_and_user_message(),
                        data: None,
                    }),
                );
            }
        }
    } else {
        None
    };

    let token_existing = if let Some((_, token)) = auth_header
        .as_ref()
        .and_then(|header| header.token().split_once(':'))
        .filter(|(header_client_id, _)| *header_client_id == registration.id)
    {
        match client_service.get_by_issued_agent_token(token).await {
            Ok(existing) => existing,
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<RegisterClientResponse> {
                        status: 500,
                        message: err.log_and_user_message(),
                        data: None,
                    }),
                );
            }
        }
    } else {
        None
    };

    let mut candidates = Vec::new();
    if let Some(existing) = requested_existing {
        candidates.push(existing);
    }
    if let Some(existing) = serial_existing
        && !candidates
            .iter()
            .any(|candidate| candidate.id == existing.id)
    {
        candidates.push(existing);
    }
    if let Some(existing) = token_existing
        && !candidates
            .iter()
            .any(|candidate| candidate.id == existing.id)
    {
        warn!(
            "Recovered canonical client {} from a valid legacy token",
            existing.id
        );
        candidates.push(existing);
    }

    if !candidates.is_empty() {
        let authenticated_existing = auth_header.as_ref().and_then(|header| {
            let (client_id, token) = header.token().split_once(':')?;
            if client_id != registration.id
                && !candidates.iter().any(|candidate| candidate.id == client_id)
            {
                return None;
            }
            candidates
                .iter()
                .find(|candidate| {
                    candidate
                        .agent_token
                        .as_deref()
                        .and_then(|hash| verify_token(token, hash).ok())
                        .unwrap_or(false)
                })
                .cloned()
        });
        let legacy_existing = candidates
            .iter()
            .find(|candidate| {
                candidate.id == registration.id
                    && candidate.agent_token.is_none()
                    && matching_stable_serial(candidate, &registration)
            })
            .cloned();
        let existing = if let Some(existing) = authenticated_existing {
            existing
        } else if let Some(existing) = legacy_existing {
            if !token_capable {
                info!(
                    "Deferring token enrollment for legacy client {} until a token-capable agent registers",
                    existing.id
                );
                return (
                    StatusCode::OK,
                    Json(ApiResponse {
                        status: 200,
                        message: "Legacy client recognized; token enrollment deferred until agent upgrade"
                            .into(),
                        data: Some(RegisterClientResponse {
                            client: existing,
                            agent_token: String::new(),
                        }),
                    }),
                );
            }
            existing
        } else {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<RegisterClientResponse> {
                    status: 401,
                    message: "existing client registration requires its current agent token".into(),
                    data: None,
                }),
            );
        };
        if existing.agent_token.is_none() {
            warn!(
                "Allowing one-time token bootstrap for legacy tokenless client {} with matching hardware serial",
                existing.id
            );
        }
        // Reconcile a generated migration ID to the existing record selected
        // by hardware serial number. The response returns this canonical ID so
        // the agent can persist and use it for all authenticated endpoints.
        registration.id = existing.id;
    }

    // Validate references
    if let Some(rack_id) = &registration.rack
        && !rack_id.is_empty()
        && let Err(e) = validation_service.validate_rack_exists(rack_id).await
    {
        let response = ApiResponse::<RegisterClientResponse> {
            status: e.status_code(),
            message: e.log_and_user_message(),
            data: None,
        };
        return (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::BAD_REQUEST),
            Json(response),
        );
    }

    if let Some(project_id) = &registration.project_id
        && !project_id.is_empty()
        && let Err(e) = validation_service.validate_project_exists(project_id).await
    {
        let response = ApiResponse::<RegisterClientResponse> {
            status: e.status_code(),
            message: e.log_and_user_message(),
            data: None,
        };
        return (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::BAD_REQUEST),
            Json(response),
        );
    }

    if let Some(owner_id) = &registration.owner_id
        && !owner_id.is_empty()
        && let Err(e) = validation_service.validate_person_exists(owner_id).await
    {
        let response = ApiResponse::<RegisterClientResponse> {
            status: e.status_code(),
            message: e.log_and_user_message(),
            data: None,
        };
        return (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::BAD_REQUEST),
            Json(response),
        );
    }

    // Registration must be processed exactly once and synchronously because
    // the generated agent token is returned in this response. Queueing the
    // same request as well can rotate the stored token after the response and,
    // for a new agent with an empty ID, persist an invalid `client:` record.
    match client_service
        .register_client(
            &registration.hostname,
            &registration.ip_address,
            &registration.sys_vendor.unwrap_or_default(),
            &registration.product_name.unwrap_or_default(),
            &registration.serial_number.unwrap_or_default(),
            &registration.os.unwrap_or_default(),
            if registration.id.is_empty() {
                None
            } else {
                Some(registration.id.clone())
            },
            registration.primary_ip.clone(),
        )
        .await
    {
        Ok((client, agent_token)) => {
            let response = ApiResponse {
                status: 200,
                message: "Client registered successfully".to_string(),
                data: Some(RegisterClientResponse {
                    client,
                    agent_token,
                }),
            };

            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!(
                "Failed to register client {}: {}",
                registration.hostname, err
            );
            let response = ApiResponse::<RegisterClientResponse> {
                status: err.status_code(),
                message: err.log_and_user_message(),
                data: None,
            };

            (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

fn matching_stable_serial(existing: &Client, registration: &Client) -> bool {
    fn stable_serial(value: Option<&str>) -> Option<&str> {
        let value = value?.trim();
        if value.is_empty()
            || [
                "unknown",
                "n/a",
                "none",
                "not specified",
                "default string",
                "to be filled by o.e.m.",
            ]
            .iter()
            .any(|placeholder| value.eq_ignore_ascii_case(placeholder))
        {
            None
        } else {
            Some(value)
        }
    }

    stable_serial(existing.serial_number.as_deref())
        .zip(stable_serial(registration.serial_number.as_deref()))
        .is_some_and(|(existing, incoming)| existing.eq_ignore_ascii_case(incoming))
}

#[derive(Serialize, PartialEq)]
pub struct AgentTokenRecoveryResponse {
    client_id: String,
    agent_token: String,
}

/// Re-issue an Agent credential after the local token file was irrecoverably
/// lost. Routing restricts this endpoint to administrators.
#[debug_handler]
#[instrument(skip(client_service))]
pub async fn recover_agent_token(
    Path(client_id): Path<String>,
    Extension(client_service): Extension<Arc<ClientService>>,
) -> impl IntoResponse {
    match client_service.recover_agent_token(&client_id).await {
        Ok(agent_token) => (
            StatusCode::OK,
            Json(ApiResponse {
                status: 200,
                message: "Agent token recovered; deploy it as a root-only credential".into(),
                data: Some(AgentTokenRecoveryResponse {
                    client_id,
                    agent_token,
                }),
            }),
        ),
        Err(err) => (
            StatusCode::from_u16(err.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<AgentTokenRecoveryResponse> {
                status: err.status_code(),
                message: err.log_and_user_message(),
                data: None,
            }),
        ),
    }
}

/// Delete a client
#[debug_handler]
#[instrument(skip(client_service, component_repo))]
pub async fn delete_client(
    Path(client_id): Path<String>,
    Extension(client_service): Extension<Arc<ClientService>>,
    Extension(component_repo): Extension<Arc<ComponentRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    info!("Deleting client: {}", client_id);

    // Ownership check
    if let Ok(Some(client)) = client_service.get_client(&client_id).await {
        if !perm_ctx.allows_resource_with_scope(
            &ResourceType::Client,
            &PermissionAction::Delete,
            client.created_by.as_deref(),
            client.project_id.as_deref(),
            &PermissionContext::client_tags(&client),
        ) {
            let response = ApiResponse::<()> {
                status: 403,
                message: "Forbidden: you can only delete resources you created".to_string(),
                data: None,
            };
            return (StatusCode::FORBIDDEN, Json(response));
        }
    }

    // Cascade update: Release components
    if let Err(e) = component_repo
        .release_components_by_client(&client_id)
        .await
    {
        let response = ApiResponse::<()> {
            status: 500,
            message: format!("Failed to release components: {}", e),
            data: None,
        };
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
    }

    match client_service.delete_client(&client_id).await {
        Ok(_) => {
            let response = ApiResponse::<()> {
                status: 200,
                message: "Client deleted successfully".to_string(),
                data: None,
            };

            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to delete client {}: {}", client_id, err);
            let response = ApiResponse::<()> {
                status: err.status_code(),
                message: err.log_and_user_message(),
                data: None,
            };

            (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

/// Update a client
#[debug_handler]
#[instrument(skip(client_repo, validation_service))]
pub async fn update_client(
    Path(client_id): Path<String>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(validation_service): Extension<Arc<ValidationService>>,
    Extension(perm_ctx): Extension<PermissionContext>,
    Json(client): Json<Client>,
) -> impl IntoResponse {
    info!("Updating client: {}", client_id);

    // Validate references
    if let Some(rack_id) = &client.rack
        && !rack_id.is_empty()
        && let Err(e) = validation_service.validate_rack_exists(rack_id).await
    {
        let response = ApiResponse::<Client> {
            status: e.status_code(),
            message: e.log_and_user_message(),
            data: None,
        };
        return (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::BAD_REQUEST),
            Json(response),
        );
    }

    if let Some(project_id) = &client.project_id
        && !project_id.is_empty()
        && let Err(e) = validation_service.validate_project_exists(project_id).await
    {
        let response = ApiResponse::<Client> {
            status: e.status_code(),
            message: e.log_and_user_message(),
            data: None,
        };
        return (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::BAD_REQUEST),
            Json(response),
        );
    }

    if let Some(owner_id) = &client.owner_id
        && !owner_id.is_empty()
        && let Err(e) = validation_service.validate_person_exists(owner_id).await
    {
        let response = ApiResponse::<Client> {
            status: e.status_code(),
            message: e.log_and_user_message(),
            data: None,
        };
        return (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::BAD_REQUEST),
            Json(response),
        );
    }

    // Check if client exists
    match client_repo.get(&client_id).await {
        Ok(Some(existing_client)) => {
            // Ownership check
            if !perm_ctx.allows_resource_with_scope(
                &ResourceType::Client,
                &PermissionAction::Update,
                existing_client.created_by.as_deref(),
                existing_client.project_id.as_deref(),
                &PermissionContext::client_tags(&existing_client),
            ) || !perm_ctx.allows_action_with_scope(
                &ResourceType::Client,
                &PermissionAction::Update,
                client.project_id.as_deref(),
                &PermissionContext::client_tags(&client),
            ) {
                let response = ApiResponse::<Client> {
                    status: 403,
                    message: "Forbidden: you can only update resources you created".to_string(),
                    data: None,
                };
                return (StatusCode::FORBIDDEN, Json(response));
            }
            // Preserve created_by
            let created_by = existing_client.created_by.clone();

            // Ensure ID matches
            if client.id != client_id {
                let response = ApiResponse::<Client> {
                    status: 400,
                    message: "Client ID mismatch".to_string(),
                    data: None,
                };
                return (StatusCode::BAD_REQUEST, Json(response));
            }

            let mut updated_client = client;
            updated_client.created_by = created_by;

            match client_repo.save(&updated_client).await {
                Ok(_) => {
                    let response = ApiResponse {
                        status: 200,
                        message: "Client updated successfully".to_string(),
                        data: Some(updated_client),
                    };
                    (StatusCode::OK, Json(response))
                }
                Err(err) => {
                    error!("Failed to update client {}: {}", client_id, err);
                    let response = ApiResponse::<Client> {
                        status: err.status_code(),
                        message: err.log_and_user_message(),
                        data: None,
                    };
                    (
                        StatusCode::from_u16(err.status_code())
                            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        Json(response),
                    )
                }
            }
        }
        Ok(None) => {
            let response = ApiResponse::<Client> {
                status: 404,
                message: format!("Client {} not found", client_id),
                data: None,
            };
            (StatusCode::NOT_FOUND, Json(response))
        }
        Err(err) => {
            error!("Failed to check client {}: {}", client_id, err);
            let response = ApiResponse::<Client> {
                status: err.status_code(),
                message: err.log_and_user_message(),
                data: None,
            };
            (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

/// Search clients with advanced filtering
#[debug_handler]
#[instrument(skip(client_filter_service))]
pub async fn search_clients(
    Query(params): Query<SearchQuery>,
    Extension(client_filter_service): Extension<Arc<ClientFilterService>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    if let Err(msg) = validate_search_params(&params) {
        let response = ApiResponse::<Vec<Client>> {
            status: 400,
            message: msg,
            data: None,
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }
    match client_filter_service.search_clients(&params).await {
        Ok(clients) => {
            let clients = filter_clients_by_view_scope(clients, &perm_ctx);
            info!("Search returned {} clients", clients.len());
            let response = ApiResponse {
                status: 200,
                message: "Clients searched successfully".to_string(),
                data: Some(clients),
            };

            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to search clients: {}", err);
            let response = ApiResponse::<Vec<Client>> {
                status: 500,
                message: err,
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

/// Filter clients by hardware specifications
#[debug_handler]
#[instrument(skip(client_filter_service))]
pub async fn filter_clients_by_hardware(
    Query(params): Query<HardwareFilterQuery>,
    Extension(client_filter_service): Extension<Arc<ClientFilterService>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    match client_filter_service
        .filter_clients_by_hardware(&params)
        .await
    {
        Ok(clients) => {
            let clients = filter_clients_by_view_scope(clients, &perm_ctx);
            info!("Hardware filter returned {} clients", clients.len());
            let response = ApiResponse {
                status: 200,
                message: crate::constants::MSG_CLIENTS_FILTERED_SUCCESS.to_string(),
                data: Some(clients),
            };

            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to filter clients by hardware: {}", err);
            let response = ApiResponse::<Vec<Client>> {
                status: 500,
                message: err,
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

/// Get filter options by client IDs
#[debug_handler]
#[instrument(skip(client_filter_service, client_repo, perm_ctx))]
pub async fn get_filter_options_by_client_ids(
    Query(params): Query<HashMap<String, String>>,
    Extension(client_filter_service): Extension<Arc<ClientFilterService>>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    let client_ids_str = params.get("client_ids").unwrap_or(&String::new()).clone();

    if client_ids_str.is_empty() {
        let response = ApiResponse {
            status: 200,
            message: crate::constants::MSG_EMPTY_CLIENT_IDS.to_string(),
            data: Some(FilterOptions::default()),
        };
        return (StatusCode::OK, Json(response));
    }

    // Parse client_ids string (comma separated)
    let client_ids: Vec<String> = client_ids_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if client_ids.is_empty() {
        let response = ApiResponse {
            status: 200,
            message: crate::constants::MSG_NO_VALID_CLIENT_IDS.to_string(),
            data: Some(FilterOptions::default()),
        };
        return (StatusCode::OK, Json(response));
    }

    let authorized_ids = match authorized_client_ids(&client_repo, &perm_ctx).await {
        Ok(None) => client_ids,
        Ok(Some(allowed_ids)) => client_ids
            .into_iter()
            .filter(|id| allowed_ids.contains(id))
            .collect(),
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

    match client_filter_service
        .get_filter_options_by_client_ids(&authorized_ids)
        .await
    {
        Ok(options) => {
            let response = ApiResponse {
                status: 200,
                message: crate::constants::MSG_FILTER_OPTIONS_SUCCESS.to_string(),
                data: Some(options),
            };
            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to get filter options: {}", err);
            let response = ApiResponse::<FilterOptions> {
                status: 500,
                message: err,
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

/// Export clients to JSON
#[debug_handler]
#[instrument(skip(client_repo))]
pub async fn export_clients(
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    match client_repo.list_all().await {
        Ok(clients) => (
            StatusCode::OK,
            Json(filter_clients_by_view_scope(clients, &perm_ctx)),
        ),
        Err(err) => {
            error!("Failed to export clients: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PrimaryIpRequest {
    pub primary_ip: Option<String>,
}

/// Update client primary IP
#[debug_handler]
#[instrument(skip(client_repo))]
pub async fn update_client_primary_ip(
    Path(client_id): Path<String>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
    Json(request): Json<PrimaryIpRequest>,
) -> impl IntoResponse {
    info!("Updating primary IP for client: {}", client_id);

    match client_repo.get(&client_id).await {
        Ok(Some(client))
            if perm_ctx.allows_resource_with_scope(
                &ResourceType::Client,
                &PermissionAction::Update,
                client.created_by.as_deref(),
                client.project_id.as_deref(),
                &PermissionContext::client_tags(&client),
            ) => {}
        Ok(Some(_)) => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<Client> {
                    status: 403,
                    message: "Forbidden".into(),
                    data: None,
                }),
            );
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<Client> {
                    status: 404,
                    message: "Client not found".into(),
                    data: None,
                }),
            );
        }
        Err(err) => {
            return (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(ApiResponse::<Client> {
                    status: err.status_code(),
                    message: err.log_and_user_message(),
                    data: None,
                }),
            );
        }
    }

    // Validate IP if provided
    if let Some(ref ip) = request.primary_ip {
        if !ip.is_empty() {
            if let Err(e) = validate_ip_address(ip) {
                let response = ApiResponse::<Client> {
                    status: 400,
                    message: format!("Invalid IP address: {}", e),
                    data: None,
                };
                return (StatusCode::BAD_REQUEST, Json(response));
            }
        }
    }

    match client_repo
        .update_primary_ip(&client_id, request.primary_ip.as_deref().unwrap_or(""))
        .await
    {
        Ok(_) => {
            // Fetch and return updated client
            match client_repo.get(&client_id).await {
                Ok(Some(client)) => {
                    let response = ApiResponse {
                        status: 200,
                        message: "Primary IP updated successfully".to_string(),
                        data: Some(client),
                    };
                    (StatusCode::OK, Json(response))
                }
                Ok(None) => {
                    error!("Client {} not found after primary IP update", client_id);
                    let response = ApiResponse::<Client> {
                        status: 404,
                        message: "Client not found after update".to_string(),
                        data: None,
                    };
                    (StatusCode::NOT_FOUND, Json(response))
                }
                Err(err) => {
                    error!(
                        "Failed to fetch client {} after primary IP update: {}",
                        client_id, err
                    );
                    let response = ApiResponse::<Client> {
                        status: err.status_code(),
                        message: err.log_and_user_message(),
                        data: None,
                    };
                    (
                        StatusCode::from_u16(err.status_code())
                            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        Json(response),
                    )
                }
            }
        }
        Err(err) => {
            error!(
                "Failed to update primary IP for client {}: {}",
                client_id, err
            );
            let response = ApiResponse::<Client> {
                status: err.status_code(),
                message: err.log_and_user_message(),
                data: None,
            };
            (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

/// Import clients from JSON
#[debug_handler]
#[instrument(skip(client_service))]
pub async fn import_clients(
    Extension(client_service): Extension<Arc<ClientService>>,
    Extension(user): Extension<User>,
    Extension(perm_ctx): Extension<PermissionContext>,
    Json(mut clients): Json<Vec<Client>>,
) -> impl IntoResponse {
    let max_batch = crate::config::get_config().max_batch_size;
    if clients.len() > max_batch {
        let response = ApiResponse::<()> {
            status: 413,
            message: format!(
                "Batch size {} exceeds maximum of {}",
                clients.len(),
                max_batch
            ),
            data: None,
        };
        return (StatusCode::PAYLOAD_TOO_LARGE, Json(response));
    }
    let user_id = user.id.clone();
    for client in &mut clients {
        client.created_by = Some(user_id.clone());
    }
    if clients.iter().any(|client| {
        !perm_ctx.allows_action_with_scope(
            &ResourceType::Client,
            &PermissionAction::Create,
            client.project_id.as_deref(),
            &PermissionContext::client_tags(client),
        )
    }) {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()> {
                status: 403,
                message: "Forbidden: one or more clients are outside your permitted scope".into(),
                data: None,
            }),
        );
    }
    info!("Importing {} clients", clients.len());
    match client_service.import_clients(clients).await {
        Ok(count) => {
            let response = ApiResponse {
                status: 200,
                message: format!("Successfully imported {} clients", count),
                data: Some(()),
            };
            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to import clients: {}", err);
            let response = ApiResponse::<()> {
                status: err.status_code(),
                message: err.log_and_user_message(),
                data: None,
            };
            (
                StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(response),
            )
        }
    }
}

/// Export filtered clients with hardware data
#[debug_handler]
#[instrument(skip(client_filter_service))]
pub async fn export_filtered_clients(
    Extension(client_filter_service): Extension<Arc<ClientFilterService>>,
    Extension(perm_ctx): Extension<PermissionContext>,
    Json(request): Json<ExportFilterRequest>,
) -> impl IntoResponse {
    info!("Exporting filtered clients");

    const MAX_EXPORT_LIMIT: usize = 5000;

    let filter_query = HardwareFilterQuery {
        search_term: request.search_term,
        os_filter: request.os,
        os_kernel_filter: request.os_kernel,
        cpu_vendor_filter: request.cpu_vendor,
        cpu_model_filter: request.cpu_model,
        gpu_vendor_filter: request.gpu_vendor,
        gpu_model_filter: request.gpu_model,
        memory_min_filter: request.memory_min,
        memory_max_filter: request.memory_max,
        server_vendor_filter: request.server_vendor,
        server_model_filter: None,
        network_type_filter: request.network_type,
        network_model_filter: request.network_model,
        storage_type_filter: request.storage_type,
        status_filter: request.status,
        client_status_filter: request.client_status,
        environment_filter: request.environment,
        rack_id_filter: request.rack_id,
        project_id_filter: request.project_id,
        owner_id_filter: request.owner_id,
    };

    match client_filter_service
        .filter_clients_by_hardware(&filter_query)
        .await
    {
        Ok(clients) => {
            let clients = filter_clients_by_view_scope(clients, &perm_ctx);
            let total_count = clients.len();

            if total_count > MAX_EXPORT_LIMIT {
                let response = ApiResponse::<ExportFilterResponse> {
                    status: 400,
                    message: format!(
                        "Export limit exceeded. Found {} clients, maximum allowed is {}. Please apply more filters to reduce the result set.",
                        total_count, MAX_EXPORT_LIMIT
                    ),
                    data: None,
                };
                return (StatusCode::BAD_REQUEST, Json(response));
            }

            let mut hardware_data = Vec::new();
            for client in &clients {
                match client_filter_service
                    .get_hardware_export_data(&client.id)
                    .await
                {
                    Ok(Some(data)) => hardware_data.push(data),
                    Ok(None) => {
                        info!("No hardware data for client: {}", client.id);
                    }
                    Err(err) => {
                        error!("Failed to get hardware data for {}: {}", client.id, err);
                    }
                }
            }

            let response = ApiResponse {
                status: 200,
                message: format!("Successfully prepared {} clients for export", total_count),
                data: Some(ExportFilterResponse {
                    clients,
                    hardware_data,
                    total_count,
                }),
            };

            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            error!("Failed to filter clients for export: {}", err);
            let response = ApiResponse::<ExportFilterResponse> {
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

/// Validate search/filter parameter lengths
fn validate_search_params(params: &SearchQuery) -> Result<(), String> {
    if let Some(ref q) = params.q {
        if q.len() > crate::constants::MAX_FILTER_LENGTH {
            return Err(format!(
                "Search query exceeds maximum length of {} characters",
                crate::constants::MAX_FILTER_LENGTH
            ));
        }
    }
    if let Some(ref os) = params.os {
        if os.len() > crate::constants::MAX_FILTER_LENGTH {
            return Err(format!(
                "OS filter exceeds maximum length of {} characters",
                crate::constants::MAX_FILTER_LENGTH
            ));
        }
    }
    if let Some(ref status) = params.status {
        if status.len() > crate::constants::MAX_FILTER_LENGTH {
            return Err(format!(
                "Status filter exceeds maximum length of {} characters",
                crate::constants::MAX_FILTER_LENGTH
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::AGENT_TOKEN_CAPABILITY_HEADER;
    use crate::repository::client_repository::ClientRepository;
    use crate::tests::fixtures::{TestAppBuilder, auth_headers};
    use axum::{
        body::Body,
        extract::Request,
        http::{Method, StatusCode, header},
    };
    use common::models::Client;
    use serde_json::json;
    use tower::ServiceExt;

    async fn make_post_with_capability(
        app: &axum::Router,
        path: &str,
        token: Option<&str>,
        body: serde_json::Value,
        token_capable: bool,
    ) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder()
            .method(Method::POST)
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json");
        if token_capable {
            req = req.header(AGENT_TOKEN_CAPABILITY_HEADER, "1");
        }
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = if bytes.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap()
        };
        (status, body)
    }

    async fn make_post(
        app: &axum::Router,
        path: &str,
        token: Option<&str>,
        body: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        make_post_with_capability(app, path, token, body, true).await
    }

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
    async fn test_list_clients_empty() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_get(&app.router, "/api/v1/clients", Some(&app.admin_token)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["data"]["total"], 0);
    }

    #[tokio::test]
    async fn test_list_clients_requires_auth() {
        let app = TestAppBuilder::new().build().await;
        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/clients")
            .body(Body::empty())
            .unwrap();
        let resp = app.router.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_register_client() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_post(
            &app.router,
            "/api/v1/clients/register",
            None,
            json!({
                "id": "test-c-001",
                "hostname": "test.example.com",
                "ip_address": "10.0.0.1"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "body: {:?}", body);
        assert!(body["data"]["agent_token"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_register_client_rejects_empty_hostname() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_post(
            &app.router,
            "/api/v1/clients/register",
            None,
            json!({
                "id": "bad-empty-host",
                "hostname": "   ",
                "ip_address": "10.0.0.9"
            }),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["message"], "client hostname must not be empty");
    }

    #[tokio::test]
    async fn test_register_client_reconciles_migration_id_by_serial() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_post(
            &app.router,
            "/api/v1/clients/register",
            None,
            json!({
                "id": "canonical-client",
                "hostname": "agent-node.example.test",
                "ip_address": "192.0.2.29",
                "serial_number": "SERIAL-MIGRATION-1",
                "os": "AlmaLinux-9.4"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "body: {:?}", body);
        let token = body["data"]["agent_token"].as_str().unwrap();

        let migration_auth = format!("generated-migration-id:{}", token);
        let (status, body) = make_post(
            &app.router,
            "/api/v1/clients/register",
            Some(&migration_auth),
            json!({
                "id": "generated-migration-id",
                "hostname": "agent-node.example.test",
                "ip_address": "192.0.2.29",
                "serial_number": "SERIAL-MIGRATION-1",
                "os": "AlmaLinux-9.4"
            }),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "body: {:?}", body);
        assert_eq!(body["data"]["client"]["id"], "canonical-client");
    }

    #[tokio::test]
    async fn test_register_client_recovers_when_migration_id_already_exists() {
        let app = TestAppBuilder::new().build().await;
        let (status, canonical_body) = make_post(
            &app.router,
            "/api/v1/clients/register",
            None,
            json!({
                "id": "canonical-client",
                "hostname": "agent-node.example.test",
                "ip_address": "192.0.2.29",
                "serial_number": "SERIAL-DUPLICATE-RECOVERY"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "body: {:?}", canonical_body);
        let canonical_token = canonical_body["data"]["agent_token"].as_str().unwrap();

        // Simulate the bad upgrader having already registered a generated ID
        // without useful hardware identity.
        let (status, duplicate_body) = make_post(
            &app.router,
            "/api/v1/clients/register",
            None,
            json!({
                "id": "generated-duplicate",
                "hostname": "agent-node.example.test",
                "ip_address": "192.0.2.29"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "body: {:?}", duplicate_body);

        let migration_auth = format!("generated-duplicate:{}", canonical_token);
        let (status, recovered_body) = make_post(
            &app.router,
            "/api/v1/clients/register",
            Some(&migration_auth),
            json!({
                "id": "generated-duplicate",
                "hostname": "agent-node.example.test",
                "ip_address": "192.0.2.29",
                "serial_number": "SERIAL-DUPLICATE-RECOVERY"
            }),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "body: {:?}", recovered_body);
        assert_eq!(recovered_body["data"]["client"]["id"], "canonical-client");
    }

    #[tokio::test]
    async fn test_register_client_recovers_unknown_token_without_legacy_serial() {
        let app = TestAppBuilder::new().build().await;
        let (status, canonical_body) = make_post(
            &app.router,
            "/api/v1/clients/register",
            None,
            json!({
                "id": "server-returned-legacy-id",
                "hostname": "legacy-agent.example.test",
                "ip_address": "192.0.2.40"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "body: {:?}", canonical_body);
        let unknown_token = canonical_body["data"]["agent_token"].as_str().unwrap();

        let (status, duplicate_body) = make_post(
            &app.router,
            "/api/v1/clients/register",
            None,
            json!({
                "id": "locally-generated-upgrade-id",
                "hostname": "legacy-agent.example.test",
                "ip_address": "192.0.2.40"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "body: {:?}", duplicate_body);

        let migration_auth = format!("locally-generated-upgrade-id:{}", unknown_token);
        let (status, recovered_body) = make_post(
            &app.router,
            "/api/v1/clients/register",
            Some(&migration_auth),
            json!({
                "id": "locally-generated-upgrade-id",
                "hostname": "legacy-agent.example.test",
                "ip_address": "192.0.2.40",
                "serial_number": "SERIAL-COLLECTED-BY-NEW-AGENT"
            }),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "body: {:?}", recovered_body);
        assert_eq!(
            recovered_body["data"]["client"]["id"],
            "server-returned-legacy-id"
        );
    }

    #[tokio::test]
    async fn test_register_client_bootstraps_tokenless_legacy_record_once() {
        let app = TestAppBuilder::new().build().await;
        // Model a database created by a release that predated Agent Tokens.
        let repo = ClientRepository::new(app.db.clone());
        let mut legacy = Client::new(
            "legacy-agent.example.test".to_string(),
            "192.0.2.30".to_string(),
        );
        legacy.id = "legacy-tokenless-client".to_string();
        legacy.serial_number = Some("SERIAL-LEGACY-TOKENLESS".to_string());
        repo.save(&legacy).await.unwrap();

        let registration = json!({
            "id": "legacy-tokenless-client",
            "hostname": "legacy-agent.example.test",
            "ip_address": "192.0.2.30",
            "serial_number": "SERIAL-LEGACY-TOKENLESS"
        });
        let (status, _) = make_post(
            &app.router,
            "/api/v1/clients/register",
            None,
            json!({
                "id": "legacy-tokenless-client",
                "hostname": "attacker.example.test",
                "ip_address": "192.0.2.31",
                "serial_number": "SERIAL-DOES-NOT-MATCH"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);

        let (status, migrated) = make_post(
            &app.router,
            "/api/v1/clients/register",
            None,
            registration.clone(),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "body: {:?}", migrated);
        assert!(migrated["data"]["agent_token"].as_str().is_some());

        // The successful bootstrap stores a hash, closing the compatibility
        // path immediately for subsequent unauthenticated requests.
        let (status, _) =
            make_post(&app.router, "/api/v1/clients/register", None, registration).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_legacy_agent_does_not_consume_token_enrollment() {
        let app = TestAppBuilder::new().build().await;
        let repo = ClientRepository::new(app.db.clone());
        let mut legacy = Client::new(
            "legacy-before-upgrade.example.test".to_string(),
            "192.0.2.41".to_string(),
        );
        legacy.id = "legacy-before-upgrade".to_string();
        legacy.serial_number = Some("SERIAL-BEFORE-UPGRADE".to_string());
        repo.save(&legacy).await.unwrap();

        let registration = json!({
            "id": "legacy-before-upgrade",
            "hostname": "legacy-before-upgrade.example.test",
            "ip_address": "192.0.2.41",
            "serial_number": "SERIAL-BEFORE-UPGRADE"
        });
        let (status, legacy_response) = make_post_with_capability(
            &app.router,
            "/api/v1/clients/register",
            None,
            registration.clone(),
            false,
        )
        .await;

        assert_eq!(status, StatusCode::OK, "body: {:?}", legacy_response);
        assert_eq!(legacy_response["data"]["agent_token"], "");
        assert!(
            repo.get("legacy-before-upgrade")
                .await
                .unwrap()
                .unwrap()
                .agent_token
                .is_none()
        );

        let (status, upgraded_response) =
            make_post(&app.router, "/api/v1/clients/register", None, registration).await;
        assert_eq!(status, StatusCode::OK, "body: {:?}", upgraded_response);
        assert!(
            !upgraded_response["data"]["agent_token"]
                .as_str()
                .unwrap()
                .is_empty()
        );
        assert!(
            repo.get("legacy-before-upgrade")
                .await
                .unwrap()
                .unwrap()
                .agent_token
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_admin_can_recover_lost_agent_token() {
        let app = TestAppBuilder::new().build().await;
        let (status, _) = make_post(
            &app.router,
            "/api/v1/clients/register",
            None,
            json!({
                "id": "lost-token-client",
                "hostname": "lost-token-agent.example.test",
                "ip_address": "192.0.2.32",
                "serial_number": "SERIAL-LOST-TOKEN"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let recovery_path = "/api/v1/clients/lost-token-client/agent-token/recover";
        let (status, _) = make_post(&app.router, recovery_path, None, json!({})).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);

        let (status, recovered) = make_post(
            &app.router,
            recovery_path,
            Some(&app.admin_token),
            json!({}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "body: {:?}", recovered);
        let token = recovered["data"]["agent_token"].as_str().unwrap();
        assert!(!token.is_empty());

        let auth = format!("lost-token-client:{}", token);
        let (status, body) = make_post(
            &app.router,
            "/api/v1/clients/register",
            Some(&auth),
            json!({
                "id": "lost-token-client",
                "hostname": "lost-token-agent.example.test",
                "ip_address": "192.0.2.32",
                "serial_number": "SERIAL-LOST-TOKEN"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "body: {:?}", body);
    }

    #[tokio::test]
    async fn test_get_client_found() {
        let app = TestAppBuilder::new().build().await;
        let (_reg_status, _reg_body) = make_post(
            &app.router,
            "/api/v1/clients/register",
            None,
            json!({
                "id": "test-c-001",
                "hostname": "test.example.com",
                "ip_address": "10.0.0.1"
            }),
        )
        .await;
        let (status, body) = make_get(
            &app.router,
            "/api/v1/clients/test-c-001",
            Some(&app.admin_token),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["data"]["hostname"], "test.example.com");
    }

    #[tokio::test]
    async fn test_get_client_not_found() {
        let app = TestAppBuilder::new().build().await;
        let (status, _) = make_get(
            &app.router,
            "/api/v1/clients/nonexistent",
            Some(&app.admin_token),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_search_clients() {
        let app = TestAppBuilder::new().build().await;
        let (_reg_status, _reg_body) = make_post(
            &app.router,
            "/api/v1/clients/register",
            None,
            json!({
                "id": "test-c-002",
                "hostname": "search-me.example.com",
                "ip_address": "10.0.0.2"
            }),
        )
        .await;
        let (status, body) = make_get(
            &app.router,
            "/api/v1/clients/search?q=search-me",
            Some(&app.admin_token),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(body["data"].as_array().is_some_and(|arr| !arr.is_empty()));
    }

    #[tokio::test]
    async fn test_export_clients() {
        let app = TestAppBuilder::new().build().await;
        let (status, _) = make_get(
            &app.router,
            "/api/v1/clients/export",
            Some(&app.admin_token),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_import_clients_exceeds_batch_limit() {
        let app = TestAppBuilder::new().build().await;
        let clients: Vec<serde_json::Value> = (0..1001)
            .map(|i| {
                json!({
                    "id": format!("batch-test-{:04}", i),
                    "hostname": format!("host-{:04}.example.com", i),
                    "ip_address": format!("10.0.{}.{}", i / 256, i % 256),
                })
            })
            .collect();
        let (status, body) = make_post(
            &app.router,
            "/api/v1/clients/import",
            Some(&app.admin_token),
            json!(clients),
        )
        .await;
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "body: {:?}", body);
        assert!(
            body["message"]
                .as_str()
                .unwrap_or("")
                .contains("exceeds maximum")
        );
    }
}
