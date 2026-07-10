//! Client API endpoints
//!
//! This module provides HTTP endpoints for client management.
//! Filtering and search logic has been moved to ClientFilterService.

use crate::middleware::permission::PermissionContext;
use crate::queue::{Message, MessageQueue};
use crate::repository::{
    client_repository::ClientRepository, component_repository::ComponentRepository,
};
use crate::service::{
    client_filter_service::{ClientFilterService, HardwareFilterQuery, SearchQuery},
    client_service::ClientService,
    validation_service::ValidationService,
};
use crate::validation::validate_ip_address;
use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use common::entity::permission::{PermissionAction, ResourceType, ScopeConstraint};
use axum_macros::debug_handler;
use common::entity::user::{Role, User};
use common::models::{
    ApiResponse, Client, ClientQuery, ExportFilterRequest, ExportFilterResponse, FilterOptions,
    PaginatedResult, RegisterClientResponse,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, instrument};

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
                message: format!("Search query exceeds maximum length of {} characters", crate::constants::MAX_FILTER_LENGTH),
                data: None,
            };
            return (StatusCode::BAD_REQUEST, Json(response));
        }
    }
    if let Some(ref os) = query.os {
        if os.len() > crate::constants::MAX_FILTER_LENGTH {
            let response = ApiResponse::<PaginatedResult<Client>> {
                status: 400,
                message: format!("OS filter exceeds maximum length of {} characters", crate::constants::MAX_FILTER_LENGTH),
                data: None,
            };
            return (StatusCode::BAD_REQUEST, Json(response));
        }
    }
    if let Some(ref status) = query.status {
        if status.len() > crate::constants::MAX_FILTER_LENGTH {
            let response = ApiResponse::<PaginatedResult<Client>> {
                status: 400,
                message: format!("Status filter exceeds maximum length of {} characters", crate::constants::MAX_FILTER_LENGTH),
                data: None,
            };
            return (StatusCode::BAD_REQUEST, Json(response));
        }
    }
    match client_repo.list_all().await {
        Ok(mut clients) => {
            let scope = perm_ctx
                .evaluate(&ResourceType::Client, &PermissionAction::View)
                .unwrap_or(ScopeConstraint::None);
            clients = PermissionContext::filter_by_scope(
                clients,
                &scope,
                &perm_ctx.user_id,
                |c| c.created_by.as_deref(),
            );
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
) -> impl IntoResponse {
    match client_repo.get(&client_id).await {
        Ok(Some(client)) => {
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
#[instrument(skip(client_service, message_queue, validation_service, registration))]
pub async fn register_client(
    Extension(client_service): Extension<Arc<ClientService>>,
    Extension(message_queue): Extension<Arc<dyn MessageQueue>>,
    Extension(validation_service): Extension<Arc<ValidationService>>,
    Json(registration): Json<Client>,
) -> impl IntoResponse {
    info!("Registering client: {}", registration.hostname);

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

    // Queue the registration message
    if let Err(err) = message_queue.send_message(Message::ClientRegistration(registration.clone()))
    {
        error!(
            "Failed to queue registration for {}: {}",
            registration.hostname, err
        );
        let response = ApiResponse::<RegisterClientResponse> {
            status: err.status_code(),
            message: format!("Failed to queue registration: {}", err),
            data: None,
        };

        return (
            StatusCode::from_u16(err.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(response),
        );
    }

    // Process registration synchronously
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
                data: Some(RegisterClientResponse { client, agent_token }),
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

/// Delete a client
#[debug_handler]
#[instrument(skip(client_service, component_repo))]
pub async fn delete_client(
    Path(client_id): Path<String>,
    Extension(client_service): Extension<Arc<ClientService>>,
    Extension(component_repo): Extension<Arc<ComponentRepository>>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    info!("Deleting client: {}", client_id);

    // Ownership check
    if let Ok(Some(client)) = client_service.get_client(&client_id).await {
        if user.role != Role::Admin
            && client.created_by.as_deref() != Some(&user.id)
        {
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
    Extension(user): Extension<User>,
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
            if user.role != Role::Admin
                && existing_client.created_by.as_deref() != Some(&user.id)
            {
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
) -> impl IntoResponse {
    match client_filter_service
        .filter_clients_by_hardware(&params)
        .await
    {
        Ok(clients) => {
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
#[instrument(skip(client_filter_service))]
pub async fn get_filter_options_by_client_ids(
    Query(params): Query<HashMap<String, String>>,
    Extension(client_filter_service): Extension<Arc<ClientFilterService>>,
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

    match client_filter_service
        .get_filter_options_by_client_ids(&client_ids)
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
) -> impl IntoResponse {
    match client_repo.list_all().await {
        Ok(clients) => (StatusCode::OK, Json(clients)),
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
    Json(request): Json<PrimaryIpRequest>,
) -> impl IntoResponse {
    info!("Updating primary IP for client: {}", client_id);

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
    Json(mut clients): Json<Vec<Client>>,
) -> impl IntoResponse {
    let max_batch = crate::config::get_config().max_batch_size;
    if clients.len() > max_batch {
        let response = ApiResponse::<()> {
            status: 413,
            message: format!("Batch size {} exceeds maximum of {}", clients.len(), max_batch),
            data: None,
        };
        return (StatusCode::PAYLOAD_TOO_LARGE, Json(response));
    }
    let user_id = user.id.clone();
    for client in &mut clients {
        client.created_by = Some(user_id.clone());
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
    use crate::tests::fixtures::{TestAppBuilder, auth_headers};
    use axum::{
        body::Body,
        extract::Request,
        http::{Method, StatusCode, header},
    };
    use serde_json::json;
    use tower::ServiceExt;

    async fn make_post(app: &axum::Router, path: &str, token: Option<&str>, body: serde_json::Value) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder()
            .method(Method::POST)
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req.body(Body::from(serde_json::to_vec(&body).unwrap())).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
        ).unwrap();
        (status, body)
    }

    async fn make_get(app: &axum::Router, path: &str, token: Option<&str>) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder()
            .method(Method::GET)
            .uri(path);
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req.body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
        ).unwrap();
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
            .body(Body::empty()).unwrap();
        let resp = app.router.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_register_client() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_post(&app.router, "/api/v1/clients/register", None, json!({
            "id": "test-c-001",
            "hostname": "test.example.com",
            "ip_address": "10.0.0.1"
        })).await;
        assert_eq!(status, StatusCode::OK, "body: {:?}", body);
        assert!(body["data"]["agent_token"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_get_client_found() {
        let app = TestAppBuilder::new().build().await;
        let (_reg_status, _reg_body) = make_post(&app.router, "/api/v1/clients/register", None, json!({
            "id": "test-c-001",
            "hostname": "test.example.com",
            "ip_address": "10.0.0.1"
        })).await;
        let (status, body) = make_get(&app.router, "/api/v1/clients/test-c-001", Some(&app.admin_token)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["data"]["hostname"], "test.example.com");
    }

    #[tokio::test]
    async fn test_get_client_not_found() {
        let app = TestAppBuilder::new().build().await;
        let (status, _) = make_get(&app.router, "/api/v1/clients/nonexistent", Some(&app.admin_token)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_search_clients() {
        let app = TestAppBuilder::new().build().await;
        let (_reg_status, _reg_body) = make_post(&app.router, "/api/v1/clients/register", None, json!({
            "id": "test-c-002",
            "hostname": "search-me.example.com",
            "ip_address": "10.0.0.2"
        })).await;
        let (status, body) = make_get(&app.router, "/api/v1/clients/search?q=search-me", Some(&app.admin_token)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body["data"].as_array().map_or(false, |arr| arr.len() > 0));
    }

    #[tokio::test]
    async fn test_export_clients() {
        let app = TestAppBuilder::new().build().await;
        let (status, _) = make_get(&app.router, "/api/v1/clients/export", Some(&app.admin_token)).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_import_clients_exceeds_batch_limit() {
        let app = TestAppBuilder::new().build().await;
        let clients: Vec<serde_json::Value> = (0..1001)
            .map(|i| json!({
                "id": format!("batch-test-{:04}", i),
                "hostname": format!("host-{:04}.example.com", i),
                "ip_address": format!("10.0.{}.{}", i / 256, i % 256),
            }))
            .collect();
        let (status, body) = make_post(&app.router, "/api/v1/clients/import", Some(&app.admin_token), json!(clients)).await;
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "body: {:?}", body);
        assert!(body["message"].as_str().unwrap_or("").contains("exceeds maximum"));
    }
}
