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

#[cfg(test)]
mod tests {
    use crate::tests::fixtures::{TestAppBuilder, auth_headers, create_client_hardware_info};
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
    async fn test_push_and_get_hardware() {
        let app = TestAppBuilder::new().build().await;
        let client_id = "test-c-001";

        let (status, body) = make_post(&app.router, "/api/v1/clients/register", None, json!({
            "id": client_id,
            "hostname": "test-client",
            "ip_address": "10.0.0.1",
        })).await;
        assert_eq!(status, StatusCode::OK, "register failed: {:?}", body);

        let agent_token = body["data"]["agent_token"].as_str().unwrap().to_string();
        let agent_auth = format!("{}:{}", client_id, agent_token);

        let push_body = serde_json::to_value(create_client_hardware_info(client_id)).unwrap();
        let (status, body) = make_post(&app.router, &format!("/api/v1/clients/{}/hardware", client_id), Some(&agent_auth), push_body).await;
        assert_eq!(status, StatusCode::OK, "push failed: {:?}", body);

        let (status, body) = make_get(&app.router, &format!("/api/v1/clients/{}/hardware", client_id), Some(&app.admin_token)).await;
        assert_eq!(status, StatusCode::OK, "get failed: {:?}", body);
        assert!(body["data"].is_object());
    }

    #[tokio::test]
    async fn test_get_hardware_not_found() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_get(&app.router, "/api/v1/clients/nonexistent/hardware", Some(&app.admin_token)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["status"], 404);
    }

    #[tokio::test]
    async fn test_get_hardware_history() {
        let app = TestAppBuilder::new().build().await;
        let client_id = "test-c-002";

        let (status, body) = make_post(&app.router, "/api/v1/clients/register", None, json!({
            "id": client_id,
            "hostname": "test-client-2",
            "ip_address": "10.0.0.2",
        })).await;
        assert_eq!(status, StatusCode::OK, "register failed: {:?}", body);

        let agent_token = body["data"]["agent_token"].as_str().unwrap().to_string();
        let agent_auth = format!("{}:{}", client_id, agent_token);

        let mut info1 = create_client_hardware_info(client_id);
        info1.collected_at = "2024-01-01T00:00:00Z".to_string();
        let (status, _) = make_post(&app.router, &format!("/api/v1/clients/{}/hardware", client_id), Some(&agent_auth), serde_json::to_value(&info1).unwrap()).await;
        assert_eq!(status, StatusCode::OK, "first push failed");

        let mut info2 = create_client_hardware_info(client_id);
        info2.collected_at = "2024-06-01T00:00:00Z".to_string();
        if let Some(ref mut hw) = info2.hardware {
            hw.cpu.cores = 16;
            hw.cpu.threads = 32;
            hw.cpu.model_name = "Intel(R) Xeon(R) Gold 6438M".to_string();
        }
        let (status, _) = make_post(&app.router, &format!("/api/v1/clients/{}/hardware", client_id), Some(&agent_auth), serde_json::to_value(&info2).unwrap()).await;
        assert_eq!(status, StatusCode::OK, "second push failed");

        let (status, body) = make_get(&app.router, &format!("/api/v1/clients/{}/hardware/history", client_id), Some(&app.admin_token)).await;
        assert_eq!(status, StatusCode::OK, "history failed: {:?}", body);
        assert!(body["data"].as_array().map(|a| a.len() >= 2).unwrap_or(false));
    }
}
