//! API Integration Tests
//!
//! Comprehensive integration tests for API endpoints using the actual router
//! with in-memory test databases.

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, Method, StatusCode, Uri, header},
};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

use common::entity::execution::{ExecutionType, SessionStatus};
use common::entity::user::{Role, User};
// use common::entity::dictionary::Dictionary; // Removed unused import

use crate::service::cast_recorder::CastRecorderInner;

use crate::api::create_router;
use crate::cache::{CacheConfigs, CachedClientRepository};
use crate::config::{DatabaseConfig, QueueConfig, ServerConfig};
use crate::dao::{ClientDao, RackDao};
use crate::db::redb_store::RedbStore;
use crate::queue::message_queue::MessageQueueFactory;
use crate::repository::{
    approval_repository::ApprovalRepository, client_repository::ClientRepository,
    command_repository::CommandRepository, component_repository::ComponentRepository,
    dictionary_repository::DictionaryRepository, exec_policy_repository::ExecPolicyRepository,
    execution_session_repository::ExecutionSessionRepository,
    hardware_repository::HardwareRepository, permission_repository::PermissionRepository,
    person_repository::PersonRepository, project_repository::ProjectRepository,
    rack_repository::RackRepository, terminal_session_repository::TerminalSessionRepository,
    user_repository::UserRepository, web_terminal_policy_repository::WebTerminalPolicyRepository,
};
use crate::service::{
    approval_service::ApprovalService, auth_service::AuthService,
    client_filter_service::ClientFilterService, client_service::ClientService,
    command_service::CommandService, component_service::ComponentService,
    danger_detection::DangerDetectionService, execution_session_service::ExecutionSessionService,
    export_service::ExportService, hardware_service::HardwareService,
    permission_service::PermissionService, sse_hub::SseHub, stats_service::StatsService,
    terminal_session_service::TerminalSessionService, validation_service::ValidationService,
    web_terminal_service::WebTerminalService,
};
use chrono::Utc;

/// Test application state
pub struct TestApp {
    pub db: Arc<RedbStore>,
    pub router: axum::Router,
    pub auth_token: String,
    pub admin_token: String,
    pub test_user: User,
    #[allow(dead_code)]
    pub test_admin: User,
}

/// Sets up a test application with a fresh database
async fn setup_test_app() -> TestApp {
    // Create in-memory database
    let db = Arc::new(RedbStore::new("file:///tmp/test_db_").unwrap_or_else(|_| {
        // Fallback to temp file creation
        let db_path = format!("/tmp/cmdb_test_{}.db", Uuid::new_v4());
        RedbStore::new(&db_path).expect("Failed to create test database")
    }));

    // Initialize repositories
    let client_repo_inner = Arc::new(ClientRepository::new(db.clone()));
    let cache_configs = CacheConfigs::default();
    let client_repo = Arc::new(CachedClientRepository::new(
        client_repo_inner.clone(),
        &cache_configs,
    ));
    let hardware_repo = Arc::new(HardwareRepository::new(db.clone()));
    let user_repo = Arc::new(UserRepository::new(db.clone()));
    let person_repo = Arc::new(PersonRepository::new(db.clone()));
    let project_repo = Arc::new(ProjectRepository::new(db.clone()));
    let component_repo = Arc::new(ComponentRepository::new(db.clone()));
    let dictionary_repo = Arc::new(DictionaryRepository::new(db.clone()));
    let rack_repo = Arc::new(RackRepository::new(db.clone()));

    // Initialize services
    let auth_service = Arc::new(AuthService::new(
        "test_secret_key_for_integration_tests_min_32_chars".to_string(),
    ));

    let _client_dao = Arc::new(ClientDao::new(client_repo.clone(), hardware_repo.clone()));
    let _rack_dao = Arc::new(RackDao::new(rack_repo.clone(), client_repo.clone()));
    let client_service = Arc::new(ClientService::from_repositories(
        client_repo.clone(),
        hardware_repo.clone(),
        rack_repo.clone(),
    ));

    let component_service = Arc::new(ComponentService::new(component_repo.clone()));
    let _hardware_service = Arc::new(HardwareService::new(
        client_repo.clone(),
        hardware_repo.clone(),
        component_service.clone(),
        MessageQueueFactory::create_flume_queue(),
        None,
    ));

    let validation_service = Arc::new(ValidationService::new(
        client_repo_inner.clone(),
        project_repo.clone(),
        rack_repo.clone(),
        person_repo.clone(),
    ));

    let stats_service = Arc::new(StatsService::new(
        client_repo_inner.clone(),
        hardware_repo.clone(),
    ));

    let client_filter_service = Arc::new(ClientFilterService::new(
        client_repo_inner.clone(),
        hardware_repo.clone(),
    ));

    let export_service = Arc::new(ExportService::new(
        client_repo_inner.clone(),
        hardware_repo.clone(),
    ));

    let message_queue = MessageQueueFactory::create_flume_queue();

    // Create test users
    let test_admin = User {
        id: Uuid::new_v4().to_string(),
        username: "test_admin".to_string(),
        password_hash: auth_service.hash_password("admin123").unwrap(),
        role: Role::Admin,
        created_at: Utc::now().to_rfc3339(),
        last_login: None,
        is_active: true,
    };

    let test_user = User {
        id: Uuid::new_v4().to_string(),
        username: "test_user".to_string(),
        password_hash: auth_service.hash_password("user123").unwrap(),
        role: Role::User,
        created_at: Utc::now().to_rfc3339(),
        last_login: None,
        is_active: true,
    };

    user_repo.save(&test_admin).await.unwrap();
    user_repo.save(&test_user).await.unwrap();

    let admin_token = auth_service.generate_token(&test_admin).unwrap();
    let user_token = auth_service.generate_token(&test_user).unwrap();

    let config = Arc::new(ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        log_level: "info".to_string(),
        database: DatabaseConfig {
            db_type: "redb".to_string(),
            path: format!("/tmp/test_cmdb_{}.db", Uuid::new_v4()),
        },
        jwt_secret: "test_secret_key_for_integration_tests_min_32_chars".to_string(),
        poll_interval: 300,
        ssh_known_hosts_file: None,
        queue: QueueConfig {
            queue_type: "flume".to_string(),
            capacity: 1000,
        },
        client_timeout: 3600,
        enable_tls: false,
        tls_cert: None,
        tls_key: None,
        component_missing_grace_period_hours: 24,
        primary_ip: None,
        cors_allowed_origins: vec!["http://localhost:8080".to_string()],
        max_batch_size: 1000,
        expose_version: true,
    });

    // Create router
    let command_repo = Arc::new(CommandRepository::new(db.clone()));
    let danger_svc = Arc::new(DangerDetectionService::new());
    let sse_hub = SseHub::new();
    let command_service = Arc::new(
        CommandService::new(command_repo.clone(), danger_svc, sse_hub.clone())
            .await
            .expect("Failed to create command service"),
    );

    let approval_repo = Arc::new(ApprovalRepository::new(db.clone()));
    let approval_svc = Arc::new(ApprovalService::new(approval_repo.clone(), command_repo));

    let perm_repo = Arc::new(PermissionRepository::new(db.clone()));
    let perm_svc = Arc::new(PermissionService::new(perm_repo.clone()));
    let _ = perm_svc.ensure_default_rules().await;
    let _ = perm_svc.refresh_cache().await;

    let exec_policy_repo = Arc::new(ExecPolicyRepository::new(db.clone()));
    let web_terminal_policy_repo = Arc::new(WebTerminalPolicyRepository::new(db.clone()));
    let web_terminal_service = Arc::new(WebTerminalService::new(web_terminal_policy_repo.clone()));

    let execution_session_repo = Arc::new(ExecutionSessionRepository::new(db.clone()));
    let terminal_session_repo = Arc::new(TerminalSessionRepository::new(db.clone()));
    let session_svc = Arc::new(ExecutionSessionService::new(
        execution_session_repo,
        client_repo_inner.clone(),
    ));
    let terminal_svc = TerminalSessionService::new(terminal_session_repo, web_terminal_service);
    terminal_svc.configure_history(session_svc.clone());

    let router = create_router(
        client_repo_inner,
        hardware_repo,
        user_repo,
        person_repo,
        project_repo,
        component_repo,
        dictionary_repo,
        rack_repo,
        message_queue,
        client_service,
        auth_service,
        validation_service,
        stats_service,
        client_filter_service,
        export_service,
        config,
        command_service,
        sse_hub,
        perm_svc,
        perm_repo,
        exec_policy_repo,
        web_terminal_policy_repo,
        approval_repo,
        approval_svc,
        session_svc,
        terminal_svc,
    );

    TestApp {
        db,
        router,
        auth_token: user_token,
        admin_token: admin_token,
        test_user,
        test_admin,
    }
}

/// Helper to make authenticated requests
async fn make_request(
    app: &axum::Router,
    method: Method,
    path: &str,
    token: Option<&str>,
    body: Option<serde_json::Value>,
) -> (StatusCode, serde_json::Value) {
    let path_owned = path.to_owned();
    let mut request_builder = Request::builder()
        .method(method)
        .uri(Uri::from_maybe_shared(path_owned).unwrap());

    // Add Content-Type header
    request_builder = request_builder.header(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );

    if let Some(token) = token {
        request_builder = request_builder.header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );
    }

    let request = if let Some(body) = body {
        request_builder
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap()
    } else {
        request_builder.body(Body::empty()).unwrap()
    };

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();

    let body_bytes = axum::body::to_bytes(response.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap_or_default();

    let body: serde_json::Value = if body_bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&body_bytes).unwrap_or(json!({"error": "Failed to parse response"}))
    };

    (status, body)
}

/// Convenience wrapper for GET requests
async fn make_get(
    app: &axum::Router,
    path: &str,
    token: Option<&str>,
) -> (StatusCode, serde_json::Value) {
    make_request(app, Method::GET, path, token, None).await
}

/// Convenience wrapper for POST requests with JSON body
async fn make_post(
    app: &axum::Router,
    path: &str,
    token: Option<&str>,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    make_request(app, Method::POST, path, token, Some(body)).await
}

/// Convenience wrapper for PUT requests with JSON body
async fn make_put(
    app: &axum::Router,
    path: &str,
    token: Option<&str>,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    make_request(app, Method::PUT, path, token, Some(body)).await
}

// ============================================================================
// Health API Tests
// ============================================================================

#[tokio::test]
async fn test_health_check_returns_ok() {
    let app = setup_test_app().await;

    let (status, body) = make_request(&app.router, Method::GET, "/api/v1/health", None, None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "UP");
}

#[tokio::test]
async fn test_version_endpoint_returns_version() {
    let app = setup_test_app().await;

    let (status, body) =
        make_request(&app.router, Method::GET, "/api/v1/version", None, None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["version"].is_string());
}

// ============================================================================
// Auth API Tests
// ============================================================================

#[tokio::test]
async fn test_login_with_valid_credentials_returns_token() {
    let app = setup_test_app().await;

    let creds = json!({
        "username": "test_admin",
        "password": "admin123",
    });

    let (status, body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/auth/login",
        None,
        Some(creds),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["data"]["token"].is_string());
    assert!(!body["data"]["token"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_login_with_invalid_credentials_returns_401() {
    let app = setup_test_app().await;

    let creds = json!({
        "username": "test_admin",
        "password": "wrong_password",
    });

    let (status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/auth/login",
        None,
        Some(creds),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_me_endpoint_returns_user_info() {
    let app = setup_test_app().await;

    let (status, body) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/auth/me",
        Some(&app.admin_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["username"], "test_admin");
    assert_eq!(body["data"]["role"], "Admin");
}

#[tokio::test]
async fn test_me_without_token_returns_401() {
    let app = setup_test_app().await;

    let (status, _) = make_request(&app.router, Method::GET, "/api/v1/auth/me", None, None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Client API Tests
// ============================================================================

#[tokio::test]
async fn test_list_clients_requires_auth() {
    let app = setup_test_app().await;

    let (status, _) = make_request(&app.router, Method::GET, "/api/v1/clients", None, None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_clients_with_auth_returns_empty_list() {
    let app = setup_test_app().await;

    let (status, body) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/clients",
        Some(&app.admin_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["data"]["items"].is_array());
    assert_eq!(body["data"]["items"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_register_client_creates_new_client() {
    let app = setup_test_app().await;

    let client_data = json!({
        "hostname": "test-server",
        "serial_number": "SN123456",
        "ip_address": "192.168.1.100",
    });

    let (status, body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/register",
        None,
        Some(client_data),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["client"]["hostname"], "test-server");
    assert!(body["data"]["client"]["id"].is_string());
}

#[tokio::test]
async fn test_get_client_by_id() {
    let app = setup_test_app().await;

    // First create a client
    let client_data = json!({
        "hostname": "test-server-2",
        "serial_number": "SN789012",
        "ip_address": "192.168.1.101",
    });

    let (_, create_response) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/register",
        None,
        Some(client_data),
    )
    .await;

    let client_id = create_response["data"]["client"]["id"].as_str().unwrap();

    // Now get the client
    let (status, body) = make_request(
        &app.router,
        Method::GET,
        &format!("/api/v1/clients/{}", client_id),
        Some(&app.admin_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["id"], client_id);
    assert_eq!(body["data"]["hostname"], "test-server-2");
}

#[tokio::test]
async fn test_update_client() {
    let app = setup_test_app().await;

    // Create a client
    let client_data = json!({
        "hostname": "test-server-3",
        "serial_number": "SN111111",
        "ip_address": "192.168.1.102",
    });

    let (_, create_response) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/register",
        None,
        Some(client_data),
    )
    .await;

    let client_id = create_response["data"]["client"]["id"].as_str().unwrap();

    // Update the client
    let update_data = json!({
        "id": client_id,
        "hostname": "updated-server",
        "ip_address": "192.168.1.103",
    });

    let (status, body) = make_request(
        &app.router,
        Method::PUT,
        &format!("/api/v1/clients/{}", client_id),
        Some(&app.admin_token),
        Some(update_data),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["hostname"], "updated-server");
}

#[tokio::test]
async fn test_delete_client() {
    let app = setup_test_app().await;

    // Create a client
    let client_data = json!({
        "hostname": "test-server-4",
        "serial_number": "SN222222",
        "ip_address": "192.168.1.104",
    });

    let (_, create_response) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/register",
        None,
        Some(client_data),
    )
    .await;

    let client_id = create_response["data"]["client"]["id"].as_str().unwrap();

    // Delete the client
    let (status, _) = make_request(
        &app.router,
        Method::DELETE,
        &format!("/api/v1/clients/{}", client_id),
        Some(&app.admin_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    // Verify client is deleted
    let (status, _) = make_request(
        &app.router,
        Method::GET,
        &format!("/api/v1/clients/{}", client_id),
        Some(&app.admin_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ============================================================================
// Hardware API Tests
// ============================================================================

#[tokio::test]
async fn test_update_hardware_for_client() {
    let app = setup_test_app().await;

    // Create a client first
    let client_data = json!({
        "hostname": "hw-test-server",
        "serial_number": "SN333333",
        "ip_address": "192.168.1.105",
    });

    let (_, create_response) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/register",
        None,
        Some(client_data),
    )
    .await;

    let client_id = create_response["data"]["client"]["id"].as_str().unwrap();
    let agent_token = create_response["data"]["agent_token"]
        .as_str()
        .unwrap()
        .to_string();
    let agent_auth = format!("{}:{}", client_id, agent_token);

    // Update hardware
    let hardware_data = json!({
        "client_id": client_id,
        "collected_at": Utc::now().to_rfc3339(),
        "hardware": {
            "cpu": {
                "model_name": "Intel Xeon",
                "cores": 8,
                "speed": 3000
            },
            "ram": {
                "total_size": 16,
                "count": 4
            }
        }
    });

    let (status, _body) = make_request(
        &app.router,
        Method::POST,
        &format!("/api/v1/clients/{}/hardware", client_id),
        Some(&agent_auth),
        Some(hardware_data),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    // Note: update_hardware returns empty data (status 200), not the updated hardware info.
    // It returns ApiResponse<()>
}

#[tokio::test]
async fn test_get_hardware_for_client() {
    let app = setup_test_app().await;

    // Create a client
    let client_data = json!({
        "hostname": "hw-get-server",
        "serial_number": "SN444444",
        "ip_address": "192.168.1.106",
    });

    let (_, create_response) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/register",
        None,
        Some(client_data),
    )
    .await;

    let client_id = create_response["data"]["client"]["id"].as_str().unwrap();
    let agent_token = create_response["data"]["agent_token"]
        .as_str()
        .unwrap()
        .to_string();
    let agent_auth = format!("{}:{}", client_id, agent_token);

    // Add hardware
    let hardware_data = json!({
        "client_id": client_id,
        "collected_at": Utc::now().to_rfc3339(),
        "hardware": {
            "cpu": {"model_name": "AMD EPYC", "cores": 16, "speed": 2500}
        }
    });

    make_request(
        &app.router,
        Method::POST,
        &format!("/api/v1/clients/{}/hardware", client_id),
        Some(&agent_auth),
        Some(hardware_data),
    )
    .await;

    // Get hardware
    let (status, body) = make_request(
        &app.router,
        Method::GET,
        &format!("/api/v1/clients/{}/hardware", client_id),
        Some(&app.admin_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["cpu"]["model_name"], "AMD EPYC");
}

// ============================================================================
// Component API Tests
// ============================================================================

#[tokio::test]
async fn test_create_component() {
    let app = setup_test_app().await;

    let component_data = json!({
        // client_id removed as it would fail validation if random UUID doesn't exist
        "component_type": "NetworkCard",
        "serial_number": "SN-NIC-001",
        "model": "Intel I350",
        "status": "InUse"
    });

    let (status, body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/components",
        Some(&app.admin_token),
        Some(component_data),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["data"]["component_type"], "NetworkCard");
    assert!(body["data"]["id"].is_string());
}

#[tokio::test]
async fn test_list_components() {
    let app = setup_test_app().await;

    let (status, body) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/components",
        Some(&app.admin_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["data"]["items"].is_array());
}

// ============================================================================
// Dictionary API Tests
// ============================================================================

#[tokio::test]
async fn test_create_dictionary() {
    let app = setup_test_app().await;

    let dict_data = json!({
        "category": "Department",
        "key": "test_dict_key",
        "value": "test_value",
        "description": "Test dictionary entry"
    });

    let (status, body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/dictionaries",
        Some(&app.admin_token),
        Some(dict_data),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["data"]["key"], "test_dict_key");
}

#[tokio::test]
async fn test_list_dictionaries() {
    let app = setup_test_app().await;

    let (status, body) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/dictionaries",
        Some(&app.admin_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["data"].is_array());
}

// ============================================================================
// Person API Tests
// ============================================================================

#[tokio::test]
async fn test_create_person() {
    let app = setup_test_app().await;

    let person_data = json!({
        "name": "John Doe",
        "email": "john.doe@example.com",
        "phone": "123-456-7890"
    });

    let (status, body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/users",
        Some(&app.admin_token),
        Some(person_data),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["data"]["name"], "John Doe");
}

#[tokio::test]
async fn test_list_persons() {
    let app = setup_test_app().await;

    let (status, body) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/users",
        Some(&app.admin_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["data"]["items"].is_array());
}

// ============================================================================
// Project API Tests
// ============================================================================

#[tokio::test]
async fn test_create_project() {
    let app = setup_test_app().await;

    let project_data = json!({
        "name": "Test Project",
        "description": "A test project"
    });

    let (status, body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/projects",
        Some(&app.admin_token),
        Some(project_data),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["data"]["name"], "Test Project");
}

// ============================================================================
// Rack API Tests
// ============================================================================

#[tokio::test]
async fn test_create_rack() {
    let app = setup_test_app().await;

    let rack_data = json!({
        "name": "Rack-01",
        "location": "Data Center A",
        "height_u": 42
    });

    let (status, body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/racks",
        Some(&app.admin_token),
        Some(rack_data),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["data"]["name"], "Rack-01");
}

// ============================================================================
// Stats API Tests
// ============================================================================

#[tokio::test]
async fn test_get_hardware_stats() {
    let app = setup_test_app().await;

    let (status, body) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/stats/hardware",
        Some(&app.admin_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["data"]["total_clients"].is_number());
}

#[tokio::test]
async fn test_get_filter_options() {
    let app = setup_test_app().await;

    let (status, body) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/filter_options",
        Some(&app.admin_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["data"].is_object());
}

// ============================================================================
// RBAC Tests
// ============================================================================

#[tokio::test]
async fn test_admin_can_access_admin_endpoints() {
    let app = setup_test_app().await;

    let (status, _) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/accounts",
        Some(&app.admin_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_user_cannot_access_admin_endpoints() {
    let app = setup_test_app().await;

    let (status, _) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/accounts",
        Some(&app.auth_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_get_nonexistent_client_returns_404() {
    let app = setup_test_app().await;

    let fake_id = Uuid::new_v4().to_string();
    let (status, _) = make_request(
        &app.router,
        Method::GET,
        &format!("/api/v1/clients/{}", fake_id),
        Some(&app.admin_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_invalid_json_returns_400() {
    let app = setup_test_app().await;

    let request = Request::builder()
        .method(Method::POST)
        .uri(Uri::from_static("/api/v1/clients/register"))
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )
        .header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", app.admin_token)).unwrap(),
        )
        .body(Body::from("{invalid json}"))
        .unwrap();

    let response = app.router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_malformed_token_returns_401() {
    let app = setup_test_app().await;

    let (status, _) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/clients",
        Some("invalid_token"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Pagination and Filtering Tests
// ============================================================================

#[tokio::test]
async fn test_client_search() {
    let app = setup_test_app().await;

    // Create multiple clients
    for i in 0..3 {
        let client_data = json!({
            "hostname": format!("search-server-{}", i),
            "serial_number": format!("SN{}", i),
            "ip_address": format!("192.168.1.{}", 100 + i),
        });

        make_request(
            &app.router,
            Method::POST,
            "/api/v1/clients/register",
            None,
            Some(client_data),
        )
        .await;
    }

    // Search for clients
    let (status, body) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/clients/search?hostname=search-server-1",
        Some(&app.admin_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["data"].is_array());
    // Should find at least one matching client
}

// ============================================================================
// Primary IP API Tests
// ============================================================================

#[tokio::test]
async fn test_override_primary_ip() {
    let app = setup_test_app().await;

    let client_data = json!({
        "hostname": "primary-ip-test",
        "serial_number": "SN-PRIMARY",
        "ip_address": "192.168.1.100",
    });

    let (_, create_response) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/register",
        None,
        Some(client_data),
    )
    .await;

    let client_id = create_response["data"]["client"]["id"].as_str().unwrap();

    let (status, body) = make_request(
        &app.router,
        Method::PUT,
        &format!("/api/v1/clients/{}/primary-ip", client_id),
        Some(&app.admin_token),
        Some(json!({"primary_ip": "10.0.0.50"})),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["primary_ip"], "10.0.0.50");
}

#[tokio::test]
async fn test_override_primary_ip_clear() {
    let app = setup_test_app().await;

    let client_data = json!({
        "hostname": "primary-ip-clear",
        "serial_number": "SN-PRIMARY2",
        "ip_address": "192.168.1.101",
        "primary_ip": "10.0.0.50",
    });

    let (_, create_response) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/register",
        None,
        Some(client_data),
    )
    .await;

    let client_id = create_response["data"]["client"]["id"].as_str().unwrap();

    let (status, body) = make_request(
        &app.router,
        Method::PUT,
        &format!("/api/v1/clients/{}/primary-ip", client_id),
        Some(&app.admin_token),
        Some(json!({"primary_ip": null})),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["data"]["primary_ip"].is_null());
}

#[tokio::test]
async fn test_override_primary_ip_invalid_format() {
    let app = setup_test_app().await;

    let client_data = json!({
        "hostname": "primary-ip-invalid",
        "serial_number": "SN-PRIMARY3",
        "ip_address": "192.168.1.102",
    });

    let (_, create_response) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/register",
        None,
        Some(client_data),
    )
    .await;

    let client_id = create_response["data"]["client"]["id"].as_str().unwrap();

    let (status, _) = make_request(
        &app.router,
        Method::PUT,
        &format!("/api/v1/clients/{}/primary-ip", client_id),
        Some(&app.admin_token),
        Some(json!({"primary_ip": "not-an-ip"})),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_search_by_primary_ip() {
    let app = setup_test_app().await;

    let client_data = json!({
        "hostname": "search-primary",
        "serial_number": "SN-SRCH1",
        "ip_address": "192.168.1.200",
        "primary_ip": "10.0.0.100",
    });

    let (_, create_response) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/register",
        None,
        Some(client_data),
    )
    .await;

    let client_id = create_response["data"]["client"]["id"].as_str().unwrap();

    let (status, body) = make_request(
        &app.router,
        Method::GET,
        &format!("/api/v1/clients/search?hostname={}", "search-primary"),
        Some(&app.admin_token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let clients = body["data"].as_array().unwrap();
    assert!(
        clients
            .iter()
            .any(|c| c["id"] == client_id && c["primary_ip"] == "10.0.0.100")
    );
}

// ============================================================================
// Concurrent Request Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_client_creation() {
    let app = setup_test_app().await;

    let tasks = (0..5).map(|i| {
        let router = app.router.clone();
        let token = app.admin_token.clone();
        tokio::spawn(async move {
            let client_data = json!({
                "hostname": format!("concurrent-server-{}", i),
                "serial_number": format!("SN{}", i),
                "ip_address": format!("192.168.2.{}", i),
            });

            make_request(
                &router,
                Method::POST,
                "/api/v1/clients/register",
                Some(&token),
                Some(client_data),
            )
            .await
        })
    });

    let results = futures::future::join_all(tasks).await;

    for result in results {
        let task_result = result.expect("Task panicked");
        let (status, _) = task_result;
        assert_eq!(status, StatusCode::OK);
    }
}

// ============================================================================
// Data-Level RBAC Tests (M3)
// ============================================================================

#[tokio::test]
async fn test_user_can_update_own_person() {
    let app = setup_test_app().await;

    // Create a person as regular user
    let (create_status, create_body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/users",
        Some(&app.auth_token),
        Some(json!({
            "name": "User Owned",
            "email": "owned@test.com"
        })),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);
    let person_id = create_body["data"]["id"].as_str().unwrap().to_string();

    // Same user updates their own person
    let (update_status, _) = make_request(
        &app.router,
        Method::PUT,
        &format!("/api/v1/users/{}", person_id),
        Some(&app.auth_token),
        Some(json!({
            "name": "Updated By Owner",
            "email": "owned@test.com"
        })),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK);
}

#[tokio::test]
async fn test_admin_can_update_any_person() {
    let app = setup_test_app().await;

    // Create a person as regular user
    let (create_status, create_body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/users",
        Some(&app.auth_token),
        Some(json!({
            "name": "Admin Edit Target",
            "email": "admin_edit@test.com"
        })),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);
    let person_id = create_body["data"]["id"].as_str().unwrap().to_string();

    // Admin updates the person
    let (update_status, _) = make_request(
        &app.router,
        Method::PUT,
        &format!("/api/v1/users/{}", person_id),
        Some(&app.admin_token),
        Some(json!({
            "name": "Updated By Admin",
            "email": "admin_edit@test.com"
        })),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK);
}

#[tokio::test]
async fn test_user_cannot_update_another_users_person() {
    let app = setup_test_app().await;

    // Create a second user (User B)
    let second_user = User {
        id: Uuid::new_v4().to_string(),
        username: "second_user".to_string(),
        password_hash: "fake_hash".to_string(),
        role: Role::User,
        created_at: Utc::now().to_rfc3339(),
        last_login: None,
        is_active: true,
    };
    let user_repo = UserRepository::new(app.db.clone());
    user_repo.save(&second_user).await.unwrap();
    let auth_service =
        AuthService::new("test_secret_key_for_integration_tests_min_32_chars".to_string());
    let second_token = auth_service.generate_token(&second_user).unwrap();

    // Create a person as User A (the test_user from TestApp)
    let (create_status, create_body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/users",
        Some(&app.auth_token),
        Some(json!({
            "name": "Protected Person",
            "email": "protected@test.com"
        })),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);
    let person_id = create_body["data"]["id"].as_str().unwrap().to_string();

    // User B tries to update User A's person -> 403
    let (update_status, update_body) = make_request(
        &app.router,
        Method::PUT,
        &format!("/api/v1/users/{}", person_id),
        Some(&second_token),
        Some(json!({
            "name": "Hacker Attempt",
            "email": "hacker@test.com"
        })),
    )
    .await;
    assert_eq!(
        update_status,
        StatusCode::FORBIDDEN,
        "body: {:?}",
        update_body
    );
}

#[tokio::test]
async fn test_user_cannot_delete_another_users_person() {
    let app = setup_test_app().await;

    // Create second user
    let second_user = User {
        id: Uuid::new_v4().to_string(),
        username: "second_user_del".to_string(),
        password_hash: "fake_hash".to_string(),
        role: Role::User,
        created_at: Utc::now().to_rfc3339(),
        last_login: None,
        is_active: true,
    };
    let user_repo = UserRepository::new(app.db.clone());
    user_repo.save(&second_user).await.unwrap();
    let auth_service =
        AuthService::new("test_secret_key_for_integration_tests_min_32_chars".to_string());
    let second_token = auth_service.generate_token(&second_user).unwrap();

    // Create a person as User A
    let (create_status, create_body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/users",
        Some(&app.auth_token),
        Some(json!({
            "name": "Protected Deletion",
            "email": "protect_del@test.com"
        })),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);
    let person_id = create_body["data"]["id"].as_str().unwrap().to_string();

    // User B tries to delete -> 403
    let (delete_status, delete_body) = make_request(
        &app.router,
        Method::DELETE,
        &format!("/api/v1/users/{}", person_id),
        Some(&second_token),
        None,
    )
    .await;
    assert_eq!(
        delete_status,
        StatusCode::FORBIDDEN,
        "body: {:?}",
        delete_body
    );
}

#[tokio::test]
async fn test_user_ownership_project_crud() {
    let app = setup_test_app().await;

    // Create a project as regular user
    let (create_status, create_body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/projects",
        Some(&app.auth_token),
        Some(json!({
            "name": "User Project",
            "code": "UP-001"
        })),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);
    let project_id = create_body["data"]["id"].as_str().unwrap().to_string();

    // Same user updates
    let (update_status, _) = make_request(
        &app.router,
        Method::PUT,
        &format!("/api/v1/projects/{}", project_id),
        Some(&app.auth_token),
        Some(json!({
            "name": "Updated User Project",
            "code": "UP-001"
        })),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK);

    // Same user deletes
    let (delete_status, _) = make_request(
        &app.router,
        Method::DELETE,
        &format!("/api/v1/projects/{}", project_id),
        Some(&app.auth_token),
        None,
    )
    .await;
    assert_eq!(delete_status, StatusCode::OK);
}

#[tokio::test]
async fn test_admin_ownership_bypass_project() {
    let app = setup_test_app().await;

    // Create a project as regular user
    let (create_status, create_body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/projects",
        Some(&app.auth_token),
        Some(json!({
            "name": "Admin Bypass Project",
            "code": "ABP-001"
        })),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);
    let project_id = create_body["data"]["id"].as_str().unwrap().to_string();

    // Admin can delete it
    let (delete_status, _) = make_request(
        &app.router,
        Method::DELETE,
        &format!("/api/v1/projects/{}", project_id),
        Some(&app.admin_token),
        None,
    )
    .await;
    assert_eq!(delete_status, StatusCode::OK);
}

// ============================================================================
// Helper: register a second user via admin and return their token
// ============================================================================

async fn register_second_user_and_login(
    app: &TestApp,
    username: &str,
    password: &str,
    role: &str,
) -> String {
    // Register via admin
    let (reg_status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/auth/register",
        Some(&app.admin_token),
        Some(json!({
            "username": username,
            "password": password,
            "role": role,
        })),
    )
    .await;
    assert_eq!(
        reg_status,
        StatusCode::CREATED,
        "register user {username} failed"
    );

    // Login to get token
    let (login_status, login_body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/auth/login",
        None,
        Some(json!({
            "username": username,
            "password": password,
        })),
    )
    .await;
    assert_eq!(login_status, StatusCode::OK, "login user {username} failed");
    login_body["data"]["token"].as_str().unwrap().to_string()
}

// ============================================================================
// 8.1 Ownership Model Tests
// ============================================================================

#[tokio::test]
async fn test_8_1_client_import_sets_created_by() {
    let app = setup_test_app().await;

    // Import a client as the regular user
    let import_data = json!([{
        "id": Uuid::new_v4().to_string(),
        "hostname": "import-host-1",
        "ip_address": "10.1.1.1",
    }]);

    let (import_status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/import",
        Some(&app.auth_token),
        Some(import_data),
    )
    .await;
    assert_eq!(import_status, StatusCode::OK);

    // List clients as admin — should see the client
    let (list_status, list_body) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/clients",
        Some(&app.admin_token),
        None,
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    let items = list_body["data"]["items"].as_array().unwrap();
    assert!(
        !items.is_empty(),
        "imported client should be visible to admin"
    );

    // created_by should be set to the user who imported
    let imported = items
        .iter()
        .find(|c| c["hostname"] == "import-host-1")
        .unwrap();
    assert!(
        imported["created_by"].is_string(),
        "created_by should be set after import"
    );
    assert_eq!(
        imported["created_by"].as_str().unwrap(),
        app.test_user.id,
        "created_by should equal the importing user's id"
    );
}

#[tokio::test]
async fn test_8_1_client_update_by_non_owner_returns_403() {
    let app = setup_test_app().await;

    // Register a second user
    let second_token =
        register_second_user_and_login(&app, "second_user_upd", "Password123!", "User").await;

    // Import a client as user A (app.auth_token)
    let client_id = Uuid::new_v4().to_string();
    let import_data = json!([{
        "id": client_id,
        "hostname": "owner-client",
        "ip_address": "10.2.2.2",
    }]);
    let (imp_status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/import",
        Some(&app.auth_token),
        Some(import_data),
    )
    .await;
    assert_eq!(imp_status, StatusCode::OK);

    // User B tries to update the client
    let (update_status, update_body) = make_request(
        &app.router,
        Method::PUT,
        &format!("/api/v1/clients/{}", client_id),
        Some(&second_token),
        Some(json!({
            "id": client_id,
            "hostname": "hacked-client",
            "ip_address": "10.2.2.2",
        })),
    )
    .await;
    assert_eq!(
        update_status,
        StatusCode::FORBIDDEN,
        "non-owner update should be 403, body: {:?}",
        update_body
    );
}

#[tokio::test]
async fn test_8_1_client_delete_by_non_owner_returns_403() {
    let app = setup_test_app().await;

    // Register a second user
    let second_token =
        register_second_user_and_login(&app, "second_user_del2", "Password123!", "User").await;

    // Import a client as user A
    let client_id = Uuid::new_v4().to_string();
    let import_data = json!([{
        "id": client_id,
        "hostname": "owner-client-del",
        "ip_address": "10.3.3.3",
    }]);
    let (imp_status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/import",
        Some(&app.auth_token),
        Some(import_data),
    )
    .await;
    assert_eq!(imp_status, StatusCode::OK);

    // User B tries to delete the client
    let (delete_status, delete_body) = make_request(
        &app.router,
        Method::DELETE,
        &format!("/api/v1/clients/{}", client_id),
        Some(&second_token),
        None,
    )
    .await;
    assert_eq!(
        delete_status,
        StatusCode::FORBIDDEN,
        "non-owner delete should be 403, body: {:?}",
        delete_body
    );
}

#[tokio::test]
async fn test_8_1_component_batch_update_by_non_owner_returns_207() {
    let app = setup_test_app().await;

    // Register a second user
    let second_token =
        register_second_user_and_login(&app, "second_user_comp", "Password123!", "User").await;

    // Create a component as user A (app.auth_token)
    let (create_status, create_body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/components",
        Some(&app.auth_token),
        Some(json!({
            "component_type": "GPU",
            "serial_number": "SN-GPU-OWNED",
            "model": "RTX 4090",
            "status": "InStock",
        })),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);
    let comp_id = create_body["data"]["id"].as_str().unwrap().to_string();

    // User B tries to batch-update the component
    let (batch_status, _batch_body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/components/batch/update",
        Some(&second_token),
        Some(json!({
            "ids": [comp_id],
            "status": "InUse",
        })),
    )
    .await;
    // 207 Multi-Status when some ownership checks fail
    assert_eq!(
        batch_status,
        StatusCode::MULTI_STATUS,
        "batch update of non-owned component should return 207"
    );
}

#[tokio::test]
async fn test_8_1_dictionary_create_sets_created_by() {
    let app = setup_test_app().await;

    // Create dictionary as the regular user
    let (create_status, create_body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/dictionaries",
        Some(&app.auth_token),
        Some(json!({
            "category": "TestCat",
            "key": "test_key_ownership",
            "value": "test_val",
        })),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);

    let created_by = create_body["data"]["created_by"].as_str();
    assert!(
        created_by.is_some(),
        "created_by should be set on dictionary create"
    );
    assert_eq!(
        created_by.unwrap(),
        app.test_user.id,
        "created_by should match the creating user id"
    );
}

#[tokio::test]
async fn test_8_1_dictionary_update_by_non_owner_returns_403() {
    let app = setup_test_app().await;

    // Register second user
    let second_token =
        register_second_user_and_login(&app, "second_user_dict_upd", "Password123!", "User").await;

    // Create dictionary as user A
    let (create_status, create_body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/dictionaries",
        Some(&app.auth_token),
        Some(json!({
            "category": "TestCat",
            "key": "dict_owner_key",
            "value": "original",
        })),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);
    let dict_id = create_body["data"]["id"].as_str().unwrap().to_string();

    // User B tries to update
    let (update_status, update_body) = make_request(
        &app.router,
        Method::PUT,
        &format!("/api/v1/dictionaries/{}", dict_id),
        Some(&second_token),
        Some(json!({
            "category": "TestCat",
            "key": "dict_owner_key",
            "value": "hacked",
        })),
    )
    .await;
    assert_eq!(
        update_status,
        StatusCode::FORBIDDEN,
        "non-owner dict update should be 403, body: {:?}",
        update_body
    );
}

#[tokio::test]
async fn test_8_1_dictionary_delete_by_non_owner_returns_403() {
    let app = setup_test_app().await;

    // Register second user
    let second_token =
        register_second_user_and_login(&app, "second_user_dict_del", "Password123!", "User").await;

    // Create dictionary as user A
    let (create_status, create_body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/dictionaries",
        Some(&app.auth_token),
        Some(json!({
            "category": "TestCat",
            "key": "dict_del_key",
            "value": "to_delete",
        })),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);
    let dict_id = create_body["data"]["id"].as_str().unwrap().to_string();

    // User B tries to delete
    let (delete_status, delete_body) = make_request(
        &app.router,
        Method::DELETE,
        &format!("/api/v1/dictionaries/{}", dict_id),
        Some(&second_token),
        None,
    )
    .await;
    assert_eq!(
        delete_status,
        StatusCode::FORBIDDEN,
        "non-owner dict delete should be 403, body: {:?}",
        delete_body
    );
}

// ============================================================================
// 8.2 Permission Engine Tests
// ============================================================================

#[tokio::test]
async fn test_8_2_admin_can_list_exec_policies() {
    let app = setup_test_app().await;

    let (status, body) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/permissions/exec-policies",
        Some(&app.admin_token),
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "admin should list exec policies, body: {:?}",
        body
    );
    assert!(body["data"].is_array(), "data should be an array");
}

#[tokio::test]
async fn test_8_2_user_cannot_manage_exec_policies() {
    let app = setup_test_app().await;

    // User (non-admin) cannot access admin exec-policy routes
    let (get_status, _) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/permissions/exec-policies",
        Some(&app.auth_token),
        None,
    )
    .await;
    assert_eq!(
        get_status,
        StatusCode::FORBIDDEN,
        "User should not access exec-policies list"
    );

    // POST also forbidden
    let (post_status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/permissions/exec-policies",
        Some(&app.auth_token),
        Some(json!({
            "id": "ep-test",
            "name": "Test Policy",
            "description": "",
            "subject_type": "Role",
            "subject_id": "Admin",
            "target_scope": "All",
            "command_rules": {"default_action": "Allow", "overrides": []},
            "require_approval": false,
            "priority": 100
        })),
    )
    .await;
    assert_eq!(
        post_status,
        StatusCode::FORBIDDEN,
        "User should not create exec-policies"
    );
}

#[tokio::test]
async fn test_8_2_admin_can_create_and_delete_exec_policy() {
    let app = setup_test_app().await;

    let policy_id = Uuid::new_v4().to_string();

    // Create
    let (create_status, create_body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/permissions/exec-policies",
        Some(&app.admin_token),
        Some(json!({
            "id": policy_id,
            "name": "Test Exec Policy",
            "description": "test",
            "subject_type": "Role",
            "subject_id": "Admin",
            "target_scope": {"All": null},
            "command_rules": {
                "default_action": "Allow",
                "overrides": []
            },
            "require_approval": false,
            "priority": 100
        })),
    )
    .await;
    assert_eq!(
        create_status,
        StatusCode::CREATED,
        "admin should create exec policy, body: {:?}",
        create_body
    );

    // List — should contain our new policy
    let (list_status, list_body) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/permissions/exec-policies",
        Some(&app.admin_token),
        None,
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    let policies = list_body["data"].as_array().unwrap();
    assert!(
        policies.iter().any(|p| p["id"] == policy_id),
        "newly created policy should appear in list"
    );

    // Delete
    let (delete_status, _) = make_request(
        &app.router,
        Method::DELETE,
        &format!("/api/v1/permissions/exec-policies/{}", policy_id),
        Some(&app.admin_token),
        None,
    )
    .await;
    assert_eq!(
        delete_status,
        StatusCode::OK,
        "admin should delete exec policy"
    );
}

// ============================================================================
// 8.3 Permission Middleware Tests
// ============================================================================

#[tokio::test]
async fn test_8_3_admin_gets_all_scope_for_clients() {
    let app = setup_test_app().await;

    // Import a client as user A (app.auth_token)
    let (imp_status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/import",
        Some(&app.auth_token),
        Some(json!([{
            "id": Uuid::new_v4().to_string(),
            "hostname": "perm-client-1",
            "ip_address": "10.10.10.1",
        }])),
    )
    .await;
    assert_eq!(imp_status, StatusCode::OK);

    // Admin can see all clients (AllScope)
    let (list_status, list_body) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/clients",
        Some(&app.admin_token),
        None,
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    let items = list_body["data"]["items"].as_array().unwrap();
    assert!(
        items.iter().any(|c| c["hostname"] == "perm-client-1"),
        "admin should see all clients regardless of owner"
    );
}

#[tokio::test]
async fn test_8_3_user_gets_owned_scope_for_clients() {
    let app = setup_test_app().await;

    // Register second user
    let second_token =
        register_second_user_and_login(&app, "scope_user_b", "Password123!", "User").await;

    // User A imports a client
    let (imp_status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/import",
        Some(&app.auth_token),
        Some(json!([{
            "id": Uuid::new_v4().to_string(),
            "hostname": "scope-client-a",
            "ip_address": "10.20.20.1",
        }])),
    )
    .await;
    assert_eq!(imp_status, StatusCode::OK);

    // User B lists clients — should NOT see user A's client (OwnedScope)
    let (list_status, list_body) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/clients",
        Some(&second_token),
        None,
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    let items = list_body["data"]["items"].as_array().unwrap();
    assert!(
        !items.iter().any(|c| c["hostname"] == "scope-client-a"),
        "user B should not see user A's client under OwnedScope"
    );
}

#[tokio::test]
async fn test_8_3_viewer_cannot_write() {
    let app = setup_test_app().await;

    // Create viewer
    let viewer_token =
        register_second_user_and_login(&app, "viewer_perm_3", "Password123!", "Viewer").await;

    // Viewer cannot create a dictionary (write operation)
    let (create_status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/dictionaries",
        Some(&viewer_token),
        Some(json!({
            "category": "TestCat",
            "key": "viewer_write_attempt",
            "value": "nope",
        })),
    )
    .await;
    // Viewer has no Create permission (only View), so the rbac middleware should deny
    // with 403 Forbidden
    assert_eq!(
        create_status,
        StatusCode::FORBIDDEN,
        "viewer should not be able to create dictionaries"
    );
}

// ============================================================================
// 8.4 Data Filtering Tests
// ============================================================================

#[tokio::test]
async fn test_8_4_admin_sees_all_resources() {
    let app = setup_test_app().await;

    // Register two users and have each import a client
    let user_b_token =
        register_second_user_and_login(&app, "filter_user_b", "Password123!", "User").await;

    let (imp_a, _) = make_request(
        &app.router, Method::POST, "/api/v1/clients/import",
        Some(&app.auth_token),
        Some(json!([{ "id": Uuid::new_v4().to_string(), "hostname": "filter-client-a", "ip_address": "10.30.1.1" }])),
    ).await;
    assert_eq!(imp_a, StatusCode::OK);

    let (imp_b, _) = make_request(
        &app.router, Method::POST, "/api/v1/clients/import",
        Some(&user_b_token),
        Some(json!([{ "id": Uuid::new_v4().to_string(), "hostname": "filter-client-b", "ip_address": "10.30.1.2" }])),
    ).await;
    assert_eq!(imp_b, StatusCode::OK);

    // Admin should see both
    let (list_status, list_body) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/clients",
        Some(&app.admin_token),
        None,
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    let items = list_body["data"]["items"].as_array().unwrap();
    assert!(
        items.iter().any(|c| c["hostname"] == "filter-client-a"),
        "admin should see client-a"
    );
    assert!(
        items.iter().any(|c| c["hostname"] == "filter-client-b"),
        "admin should see client-b"
    );
}

#[tokio::test]
async fn test_8_4_user_sees_only_own_resources() {
    let app = setup_test_app().await;

    // Register second user
    let user_b_token =
        register_second_user_and_login(&app, "filter_user_b2", "Password123!", "User").await;

    // User A creates a dictionary
    let (ca_status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/dictionaries",
        Some(&app.auth_token),
        Some(json!({ "category": "OwnedTest", "key": "user_a_dict", "value": "a_val" })),
    )
    .await;
    assert_eq!(ca_status, StatusCode::CREATED);

    // User B creates a dictionary
    let (cb_status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/dictionaries",
        Some(&user_b_token),
        Some(json!({ "category": "OwnedTest", "key": "user_b_dict", "value": "b_val" })),
    )
    .await;
    assert_eq!(cb_status, StatusCode::CREATED);

    // User A lists dictionaries — should only see own (key = user_a_dict)
    let (list_status, list_body) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/dictionaries",
        Some(&app.auth_token),
        None,
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    let items = list_body["data"].as_array().unwrap();
    assert!(
        items.iter().any(|d| d["key"] == "user_a_dict"),
        "user A should see own dict"
    );
    assert!(
        !items.iter().any(|d| d["key"] == "user_b_dict"),
        "user A should NOT see user B's dict"
    );
}

#[tokio::test]
async fn test_8_4_viewer_sees_only_own_resources() {
    let app = setup_test_app().await;

    // Create a viewer
    let viewer_token =
        register_second_user_and_login(&app, "filter_viewer_4", "Password123!", "Viewer").await;

    // Admin creates a dictionary (admin-owned)
    let (ca_status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/dictionaries",
        Some(&app.admin_token),
        Some(json!({ "category": "ViewerTest", "key": "admin_dict_v", "value": "admin_val" })),
    )
    .await;
    assert_eq!(ca_status, StatusCode::CREATED);

    // Viewer lists dictionaries — default viewer rule is OwnedScope,
    // so viewer sees only dicts they created (none in this test)
    let (list_status, list_body) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/dictionaries",
        Some(&viewer_token),
        None,
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    let items = list_body["data"].as_array().unwrap();
    assert!(
        !items.iter().any(|d| d["key"] == "admin_dict_v"),
        "viewer should not see admin's dictionary under OwnedScope"
    );
}

// ============================================================================
// 8.5 Exec Policy Tests
// ============================================================================

#[tokio::test]
async fn test_8_5_admin_can_create_exec_policy() {
    let app = setup_test_app().await;

    let policy_id = Uuid::new_v4().to_string();

    let (status, body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/permissions/exec-policies",
        Some(&app.admin_token),
        Some(json!({
            "id": policy_id,
            "name": "Allow Admin Exec",
            "description": "Allow admins to run commands",
            "subject_type": "Role",
            "subject_id": "Admin",
            "target_scope": {"All": null},
            "command_rules": {
                "default_action": "Allow",
                "overrides": []
            },
            "require_approval": false,
            "priority": 100
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {:?}", body);
    assert_eq!(body["data"]["id"], policy_id);
}

#[tokio::test]
async fn test_8_5_no_exec_policy_command_creation_denied() {
    let app = setup_test_app().await;
    // Ensure MASTER_CLIENT_KEY is initialized (needed if no other TestAppBuilder test ran first)
    crate::service::auth_service::init_master_client_key(
        "test-master-client-key-for-deterministic-tokens".to_string(),
    );

    // Import a client as admin to get a valid client_id (import doesn't need HMAC token)
    let client_id = Uuid::new_v4().to_string();
    let (imp_status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/import",
        Some(&app.admin_token),
        Some(json!([{
            "id": client_id,
            "hostname": "exec-target-import",
            "ip_address": "10.50.50.2",
        }])),
    )
    .await;
    assert_eq!(imp_status, StatusCode::OK);

    // Remote exec is disabled by default — command creation should fail
    let (cmd_status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/remote-exec/commands",
        Some(&app.admin_token),
        Some(json!({
            "client_id": client_id,
            "command": "echo hello",
            "force": false,
        })),
    )
    .await;
    // Remote exec disabled → 403 Forbidden
    assert!(
        cmd_status.as_u16() >= 400,
        "command creation with remote exec disabled should be denied, got: {}",
        cmd_status
    );
}

// ============================================================================
// 8.6 Approval Workflow Tests
// ============================================================================

#[tokio::test]
async fn test_8_6_admin_can_list_pending_approvals() {
    let app = setup_test_app().await;

    let (status, body) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/permissions/pending-approvals",
        Some(&app.admin_token),
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "admin can list pending approvals, body: {:?}",
        body
    );
    assert!(body["data"].is_array(), "data should be an array");
}

#[tokio::test]
async fn test_8_6_user_can_see_own_approvals() {
    let app = setup_test_app().await;

    // Regular user can call GET /api/v1/permissions/my-approvals
    let (status, body) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/permissions/my-approvals",
        Some(&app.auth_token),
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "user should be able to see own approvals, body: {:?}",
        body
    );
    assert!(body["data"].is_array(), "data should be an array");
}

#[tokio::test]
async fn test_8_6_non_admin_cannot_manage_pending_approvals() {
    let app = setup_test_app().await;

    // Regular user cannot list pending approvals (admin-only)
    let (status, _) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/permissions/pending-approvals",
        Some(&app.auth_token),
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "non-admin should not access pending-approvals"
    );
}

#[tokio::test]
async fn test_8_6_viewer_can_see_own_approvals() {
    let app = setup_test_app().await;

    // Create viewer
    let viewer_token =
        register_second_user_and_login(&app, "viewer_approval_6", "Password123!", "Viewer").await;

    // Viewer (any authenticated user) can see their own approvals
    let (status, body) = make_request(
        &app.router,
        Method::GET,
        "/api/v1/permissions/my-approvals",
        Some(&viewer_token),
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "viewer should be able to see own approvals, body: {:?}",
        body
    );
}

// ============================================================================
// 8.7 Integration / Role Isolation Tests
// ============================================================================

#[tokio::test]
async fn test_8_7_viewer_cannot_create_resource() {
    let app = setup_test_app().await;

    // Create viewer
    let viewer_token =
        register_second_user_and_login(&app, "viewer_iso_7a", "Password123!", "Viewer").await;

    // Viewer cannot create a project (write)
    let (create_status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/projects",
        Some(&viewer_token),
        Some(json!({
            "name": "Viewer Project Attempt",
            "code": "VP-001"
        })),
    )
    .await;
    assert_eq!(
        create_status,
        StatusCode::FORBIDDEN,
        "viewer should not be able to create projects"
    );
}

#[tokio::test]
async fn test_8_7_viewer_cannot_delete_resource() {
    let app = setup_test_app().await;

    // Admin creates a project
    let (create_status, create_body) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/projects",
        Some(&app.admin_token),
        Some(json!({
            "name": "Admin Project For Viewer Delete Test",
            "code": "VP-DEL",
        })),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);
    let project_id = create_body["data"]["id"].as_str().unwrap().to_string();

    // Create viewer
    let viewer_token =
        register_second_user_and_login(&app, "viewer_iso_7b", "Password123!", "Viewer").await;

    // Viewer cannot delete
    let (delete_status, _) = make_request(
        &app.router,
        Method::DELETE,
        &format!("/api/v1/projects/{}", project_id),
        Some(&viewer_token),
        None,
    )
    .await;
    assert_eq!(
        delete_status,
        StatusCode::FORBIDDEN,
        "viewer should not be able to delete projects"
    );
}

#[tokio::test]
async fn test_8_7_admin_can_bypass_ownership_and_update_any_client() {
    let app = setup_test_app().await;

    // User A imports a client
    let client_id = Uuid::new_v4().to_string();
    let (imp_status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/import",
        Some(&app.auth_token),
        Some(json!([{
            "id": client_id,
            "hostname": "bypass-client",
            "ip_address": "10.60.60.1",
        }])),
    )
    .await;
    assert_eq!(imp_status, StatusCode::OK);

    // Admin updates the client (not their resource — AllScope bypasses ownership)
    let (update_status, update_body) = make_request(
        &app.router,
        Method::PUT,
        &format!("/api/v1/clients/{}", client_id),
        Some(&app.admin_token),
        Some(json!({
            "id": client_id,
            "hostname": "admin-updated-client",
            "ip_address": "10.60.60.2",
        })),
    )
    .await;
    assert_eq!(
        update_status,
        StatusCode::OK,
        "admin should bypass ownership and update any client, body: {:?}",
        update_body
    );
    assert_eq!(update_body["data"]["hostname"], "admin-updated-client");
}

#[tokio::test]
async fn test_8_7_admin_can_bypass_ownership_and_delete_any_client() {
    let app = setup_test_app().await;

    // User A imports a client
    let client_id = Uuid::new_v4().to_string();
    let (imp_status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/import",
        Some(&app.auth_token),
        Some(json!([{
            "id": client_id,
            "hostname": "bypass-delete-client",
            "ip_address": "10.70.70.1",
        }])),
    )
    .await;
    assert_eq!(imp_status, StatusCode::OK);

    // Admin deletes the client
    let (delete_status, delete_body) = make_request(
        &app.router,
        Method::DELETE,
        &format!("/api/v1/clients/{}", client_id),
        Some(&app.admin_token),
        None,
    )
    .await;
    assert_eq!(
        delete_status,
        StatusCode::OK,
        "admin should bypass ownership and delete any client, body: {:?}",
        delete_body
    );
}

#[tokio::test]
async fn test_8_7_user_can_update_own_client() {
    let app = setup_test_app().await;

    // User imports a client
    let client_id = Uuid::new_v4().to_string();
    let (imp_status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/import",
        Some(&app.auth_token),
        Some(json!([{
            "id": client_id,
            "hostname": "user-own-client",
            "ip_address": "10.80.80.1",
        }])),
    )
    .await;
    assert_eq!(imp_status, StatusCode::OK);

    // Same user updates their own client
    let (update_status, _) = make_request(
        &app.router,
        Method::PUT,
        &format!("/api/v1/clients/{}", client_id),
        Some(&app.auth_token),
        Some(json!({
            "id": client_id,
            "hostname": "user-updated-own-client",
            "ip_address": "10.80.80.2",
        })),
    )
    .await;
    assert_eq!(
        update_status,
        StatusCode::OK,
        "user should be able to update their own client"
    );
}

#[tokio::test]
async fn test_8_7_user_cannot_update_other_users_client() {
    let app = setup_test_app().await;

    // Register second user
    let second_token =
        register_second_user_and_login(&app, "isolation_user_b", "Password123!", "User").await;

    // User A imports a client
    let client_id = Uuid::new_v4().to_string();
    let (imp_status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/import",
        Some(&app.auth_token),
        Some(json!([{
            "id": client_id,
            "hostname": "isolation-client-a",
            "ip_address": "10.90.90.1",
        }])),
    )
    .await;
    assert_eq!(imp_status, StatusCode::OK);

    // User B cannot update user A's client
    let (update_status, update_body) = make_request(
        &app.router,
        Method::PUT,
        &format!("/api/v1/clients/{}", client_id),
        Some(&second_token),
        Some(json!({
            "id": client_id,
            "hostname": "isolation-hacked",
            "ip_address": "10.90.90.99",
        })),
    )
    .await;
    assert_eq!(
        update_status,
        StatusCode::FORBIDDEN,
        "user B should not update user A's client, body: {:?}",
        update_body
    );
}

#[tokio::test]
async fn test_8_7_user_cannot_delete_other_users_client() {
    let app = setup_test_app().await;

    // Register second user
    let second_token =
        register_second_user_and_login(&app, "isolation_user_c", "Password123!", "User").await;

    // User A imports a client
    let client_id = Uuid::new_v4().to_string();
    let (imp_status, _) = make_request(
        &app.router,
        Method::POST,
        "/api/v1/clients/import",
        Some(&app.auth_token),
        Some(json!([{
            "id": client_id,
            "hostname": "isolation-delete-a",
            "ip_address": "10.91.91.1",
        }])),
    )
    .await;
    assert_eq!(imp_status, StatusCode::OK);

    // User B cannot delete user A's client
    let (delete_status, delete_body) = make_request(
        &app.router,
        Method::DELETE,
        &format!("/api/v1/clients/{}", client_id),
        Some(&second_token),
        None,
    )
    .await;
    assert_eq!(
        delete_status,
        StatusCode::FORBIDDEN,
        "user B should not delete user A's client, body: {:?}",
        delete_body
    );
}

// ============================================================================
// 11. Terminal & Execution Refactor Tests
// ============================================================================

#[tokio::test]
async fn test_11_1_execution_session_repository_crud() {
    let app = setup_test_app().await;

    let repo = Arc::new(ExecutionSessionRepository::new(app.db.clone()));
    let client_repo = Arc::new(ClientRepository::new(app.db.clone()));
    let svc = ExecutionSessionService::new(repo, client_repo);

    // Create session
    let session = svc
        .create_session(
            &app.test_user.id,
            &app.test_user.username,
            vec!["client-1".to_string()],
            "echo hello",
            ExecutionType::Terminal,
        )
        .await
        .expect("create_session should succeed");
    assert_eq!(session.status, SessionStatus::Pending);

    // List user sessions
    let sessions = svc
        .list_user_sessions(
            &app.test_user.id,
            &common::models::CommandQuery {
                page: Some(1),
                page_size: Some(10),
                ..Default::default()
            },
        )
        .await
        .expect("list_user_sessions should succeed");
    assert_eq!(sessions.items.len(), 1, "should find the created session");
    assert_eq!(sessions.items[0].session_id, session.session_id);

    // Complete session
    svc.complete_session(&session.session_id, SessionStatus::Success)
        .await
        .expect("complete_session should succeed");
    let updated = svc
        .get_session(&session.session_id)
        .await
        .expect("get_session should succeed")
        .expect("session should exist after completion");
    assert_eq!(updated.status, SessionStatus::Success);
    assert!(updated.end_time.is_some(), "end_time should be set");
    assert!(
        updated.duration_secs.is_some(),
        "duration_secs should be set"
    );
}

#[tokio::test]
async fn test_11_2_cast_recorder_create_write_finalize() {
    let tmp_dir = std::env::temp_dir().join(format!("cast_test_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&tmp_dir).expect("create temp dir");
    // SAFETY: test-only; no other threads race on CMDB_CAST_DIR
    unsafe { std::env::set_var("CMDB_CAST_DIR", tmp_dir.to_str().unwrap()) };

    let session_id = Uuid::new_v4().to_string();

    CastRecorderInner::init_cast(&session_id, 80, 24).expect("init_cast");
    assert!(CastRecorderInner::cast_file_exists(&session_id));

    CastRecorderInner::append_frame(&session_id, 0.5, "hello\r\n").expect("append_frame");
    CastRecorderInner::append_frame(&session_id, 1.0, "world\r\n").expect("append_frame");

    let contents = CastRecorderInner::read_cast_file(&session_id).expect("read_cast_file");
    let text = String::from_utf8_lossy(&contents);
    assert!(
        text.contains("\"version\":2"),
        "should have asciinema header"
    );
    assert!(text.contains("hello"), "should contain first frame data");
    assert!(text.contains("world"), "should contain second frame data");

    let _ = std::fs::remove_dir_all(&tmp_dir);
}

#[tokio::test]
async fn test_11_3_session_api_endpoints() {
    let app = setup_test_app().await;

    // 401 without auth
    let (status, _) = make_get(&app.router, "/api/v1/sessions", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Admin can list sessions (empty list as admin)
    let (status, body) = make_get(&app.router, "/api/v1/sessions", Some(&app.admin_token)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body["data"]["items"].is_array(),
        "sessions list should be paginated"
    );
}

#[tokio::test]
async fn test_11_4_remote_exec_config() {
    let app = setup_test_app().await;

    // GET returns current config
    let (status, body) = make_get(
        &app.router,
        "/api/v1/remote-exec/config",
        Some(&app.admin_token),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "GET config should succeed, body: {:?}",
        body
    );

    // PUT update config as Admin
    let (status, _) = make_put(
        &app.router,
        "/api/v1/remote-exec/config",
        Some(&app.admin_token),
        json!({ "enabled": false }),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "PUT config should succeed for Admin"
    );

    // Verify the change
    let (status, body) = make_get(
        &app.router,
        "/api/v1/remote-exec/config",
        Some(&app.admin_token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["enabled"], false);
}

#[tokio::test]
async fn test_11_5_batch_execution_integration() {
    let app = setup_test_app().await;
    // Ensure MASTER_CLIENT_KEY is initialized
    crate::service::auth_service::init_master_client_key(
        "test-master-client-key-for-deterministic-tokens".to_string(),
    );

    // Import a client
    let client_id = Uuid::new_v4().to_string();
    let (imp_status, _) = make_post(
        &app.router,
        "/api/v1/clients/import",
        Some(&app.admin_token),
        json!([{
            "id": client_id,
            "hostname": "batch-exec-host",
            "ip_address": "10.100.1.1",
        }]),
    )
    .await;
    assert_eq!(imp_status, StatusCode::OK, "import client should succeed");

    // Enable remote exec
    let (cfg_status, _) = make_put(
        &app.router,
        "/api/v1/remote-exec/config",
        Some(&app.admin_token),
        json!({ "enabled": true }),
    )
    .await;
    assert_eq!(
        cfg_status,
        StatusCode::OK,
        "enable remote exec should succeed"
    );

    // Create a command
    let (cmd_status, cmd_body) = make_post(
        &app.router,
        "/api/v1/remote-exec/commands",
        Some(&app.admin_token),
        json!({
            "client_id": client_id,
            "command": "echo hello",
            "force": false,
        }),
    )
    .await;
    assert_eq!(
        cmd_status,
        StatusCode::OK,
        "create command should succeed, body: {:?}",
        cmd_body
    );
    assert!(
        cmd_body["data"]["task_id"].is_string(),
        "should return task_id"
    );
}

#[tokio::test]
async fn test_11_6_cast_cleanup() {
    let tmp_dir = std::env::temp_dir().join(format!("cast_cleanup_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&tmp_dir).expect("create temp dir");
    // SAFETY: test-only; no other threads race on CMDB_CAST_DIR
    unsafe { std::env::set_var("CMDB_CAST_DIR", tmp_dir.to_str().unwrap()) };

    // Create a cast file
    let session_id = Uuid::new_v4().to_string();
    CastRecorderInner::init_cast(&session_id, 80, 24).expect("init_cast");
    assert!(CastRecorderInner::cast_file_exists(&session_id));

    // Delete old casts with retention 0 (deletes everything)
    let deleted = CastRecorderInner::delete_old_casts(0).expect("delete_old_casts");
    assert!(
        deleted >= 1,
        "should have deleted at least 1 cast file, got {}",
        deleted
    );
    assert!(
        !CastRecorderInner::cast_file_exists(&session_id),
        "cast file should no longer exist"
    );

    let _ = std::fs::remove_dir_all(&tmp_dir);
}
