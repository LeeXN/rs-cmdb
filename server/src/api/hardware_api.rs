use crate::queue::{Message, MessageQueue};
use crate::repository::{
    client_repository::ClientRepository, hardware_repository::HardwareRepository,
};
use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use axum_macros::debug_handler;
use common::entity::hardware::Hardware;
use common::models::{ApiResponse, ClientHardwareInfo, HardwareHistoryEntry, PullRequest};
use std::sync::Arc;
use tracing::{error, info, instrument};
use uuid::Uuid;

/// Get hardware information for a client
#[debug_handler]
#[instrument(skip(client_repo, hardware_repo))]
pub async fn get_hardware(
    Path(client_id): Path<String>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(hardware_repo): Extension<Arc<HardwareRepository>>,
) -> impl IntoResponse {
    // Check if client exists
    match client_repo.exists(&client_id).await {
        Ok(true) => {
            // Client exists, get hardware
            match hardware_repo.get_hardware(&client_id).await {
                Ok(Some(hardware)) => {
                    let response = ApiResponse {
                        status: 200,
                        message: "Hardware information retrieved successfully".to_string(),
                        data: Some(hardware),
                    };

                    (StatusCode::OK, Json(response))
                }
                Ok(None) => {
                    let response = ApiResponse::<Hardware> {
                        status: 404,
                        message: format!("No hardware information found for client {}", client_id),
                        data: None,
                    };

                    (StatusCode::NOT_FOUND, Json(response))
                }
                Err(err) => {
                    error!("Failed to get hardware for client {}: {}", client_id, err);
                    let response = ApiResponse::<Hardware> {
                        status: err.status_code(),
                        message: err.to_string(),
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
        Ok(false) => {
            let response = ApiResponse::<Hardware> {
                status: 404,
                message: format!("Client {} not found", client_id),
                data: None,
            };

            (StatusCode::NOT_FOUND, Json(response))
        }
        Err(err) => {
            error!("Failed to check client existence {}: {}", client_id, err);
            let response = ApiResponse::<Hardware> {
                status: err.status_code(),
                message: err.to_string(),
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

/// Get hardware history for a client
#[debug_handler]
#[instrument(skip(client_repo, hardware_repo))]
pub async fn get_hardware_history(
    Path(client_id): Path<String>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(hardware_repo): Extension<Arc<HardwareRepository>>,
) -> impl IntoResponse {
    // Check if client exists
    match client_repo.exists(&client_id).await {
        Ok(true) => {
            // Client exists, get hardware history
            match hardware_repo.get_hardware_history(&client_id).await {
                Ok(history) => {
                    let response = ApiResponse {
                        status: 200,
                        message: "Hardware history retrieved successfully".to_string(),
                        data: Some(history),
                    };

                    (StatusCode::OK, Json(response))
                }
                Err(err) => {
                    error!(
                        "Failed to get hardware history for client {}: {}",
                        client_id, err
                    );
                    let response = ApiResponse::<Vec<HardwareHistoryEntry>> {
                        status: err.status_code(),
                        message: err.to_string(),
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
        Ok(false) => {
            let response = ApiResponse::<Vec<HardwareHistoryEntry>> {
                status: 404,
                message: format!("Client {} not found", client_id),
                data: None,
            };

            (StatusCode::NOT_FOUND, Json(response))
        }
        Err(err) => {
            error!("Failed to check client existence {}: {}", client_id, err);
            let response = ApiResponse::<Vec<HardwareHistoryEntry>> {
                status: err.status_code(),
                message: err.to_string(),
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

/// Update hardware information for a client (PUSH)
#[debug_handler]
#[instrument(skip(client_repo, hardware_repo, message_queue, hardware_info))]
pub async fn update_hardware(
    Path(client_id): Path<String>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(hardware_repo): Extension<Arc<HardwareRepository>>,
    Extension(message_queue): Extension<Arc<dyn MessageQueue>>,
    Json(hardware_info): Json<ClientHardwareInfo>,
) -> impl IntoResponse {
    info!("Updating hardware info for client: {}", client_id);
    // Check if client exists
    match client_repo.exists(&client_id).await {
        Ok(true) => {
            // Update last seen timestamp
            if let Err(err) = client_repo.update_last_seen(&client_id).await {
                error!("Failed to update client last seen {}: {}", client_id, err);
                let response = ApiResponse::<()> {
                    status: err.status_code(),
                    message: format!("Failed to update client last seen: {}", err),
                    data: None,
                };

                return (
                    StatusCode::from_u16(err.status_code())
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                    Json(response),
                );
            }

            // Queue hardware info for processing
            if let Err(err) =
                message_queue.send_message(Message::ClientHardwareInfo(hardware_info.clone()))
            {
                error!("Failed to queue hardware info for {}: {}", client_id, err);
                let response = ApiResponse::<()> {
                    status: err.status_code(),
                    message: format!("Failed to queue hardware info: {}", err),
                    data: None,
                };

                return (
                    StatusCode::from_u16(err.status_code())
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                    Json(response),
                );
            }

            // Process hardware info immediately if available
            if let Some(hardware) = &hardware_info.hardware {
                if let Err(err) = hardware_repo
                    .save_hardware_with_timestamp(
                        &client_id,
                        hardware,
                        true,
                        Some(&hardware_info.collected_at),
                    )
                    .await
                {
                    error!("Failed to save hardware info for {}: {}", client_id, err);
                    let response = ApiResponse::<()> {
                        status: err.status_code(),
                        message: format!("Failed to save hardware info: {}", err),
                        data: None,
                    };

                    return (
                        StatusCode::from_u16(err.status_code())
                            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        Json(response),
                    );
                }

                // Update client kernel version if available
                if !hardware.os.kernel.is_empty()
                    && let Ok(Some(mut client)) = client_repo.get(&client_id).await
                {
                    let mut changed = false;

                    if client.kernel_version.as_deref() != Some(&hardware.os.kernel) {
                        client.kernel_version = Some(hardware.os.kernel.clone());
                        changed = true;
                    }

                    if client.os.as_deref() != Some(&hardware.os.version) {
                        client.os = Some(hardware.os.version.clone());
                        changed = true;
                    }

                    if let Some(system) = &hardware.system {
                        if client.sys_vendor.as_deref() != Some(&system.sys_vendor) {
                            client.sys_vendor = Some(system.sys_vendor.clone());
                            changed = true;
                        }
                        if client.product_name.as_deref() != Some(&system.product_name) {
                            client.product_name = Some(system.product_name.clone());
                            changed = true;
                        }
                        if client.serial_number.as_deref() != Some(&system.serial_number) {
                            client.serial_number = Some(system.serial_number.clone());
                            changed = true;
                        }
                    }

                    if changed && let Err(e) = client_repo.save(&client).await {
                        error!("Failed to update client info {}: {}", client_id, e);
                    }
                }
            }

            let response = ApiResponse::<()> {
                status: 200,
                message: "Hardware information updated successfully".to_string(),
                data: None,
            };

            (StatusCode::OK, Json(response))
        }
        Ok(false) => {
            let response = ApiResponse::<()> {
                status: 404,
                message: format!("Client {} not found", client_id),
                data: None,
            };

            (StatusCode::NOT_FOUND, Json(response))
        }
        Err(err) => {
            error!("Failed to check client existence {}: {}", client_id, err);
            let response = ApiResponse::<()> {
                status: err.status_code(),
                message: err.to_string(),
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

/// Pull hardware information from a client (PULL)
#[debug_handler]
#[instrument(skip(client_repo, message_queue))]
pub async fn pull_hardware(
    Path(client_id): Path<String>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(message_queue): Extension<Arc<dyn MessageQueue>>,
    Json(components): Json<Vec<String>>, // List of components to pull
) -> impl IntoResponse {
    info!("Initiating pull hardware for client: {}", client_id);
    // Check if client exists
    match client_repo.exists(&client_id).await {
        Ok(true) => {
            // Create pull request
            let pull_request = PullRequest {
                request_id: Uuid::new_v4().to_string(),
                components,
                requested_at: chrono::Utc::now().to_rfc3339(),
            };

            // Queue pull request
            if let Err(err) = message_queue.send_message(Message::PullRequest(
                pull_request.clone(),
                client_id.clone(),
            )) {
                error!("Failed to queue pull request for {}: {}", client_id, err);
                let response = ApiResponse::<PullRequest> {
                    status: err.status_code(),
                    message: format!("Failed to queue pull request: {}", err),
                    data: None,
                };

                return (
                    StatusCode::from_u16(err.status_code())
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                    Json(response),
                );
            }

            let response = ApiResponse {
                status: 202,
                message: "Pull request initiated".to_string(),
                data: Some(pull_request),
            };

            (StatusCode::ACCEPTED, Json(response))
        }
        Ok(false) => {
            let response = ApiResponse::<PullRequest> {
                status: 404,
                message: format!("Client {} not found", client_id),
                data: None,
            };

            (StatusCode::NOT_FOUND, Json(response))
        }
        Err(err) => {
            error!("Failed to check client existence {}: {}", client_id, err);
            let response = ApiResponse::<PullRequest> {
                status: err.status_code(),
                message: err.to_string(),
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
