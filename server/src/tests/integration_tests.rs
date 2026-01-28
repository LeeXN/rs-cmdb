//! API Integration Tests
//!
//! Comprehensive integration tests for API endpoints using the actual router
//! with in-memory test databases.

use std::sync::Arc;
use axum::{
    body::Body,
    extract::Request,
    http::{
        header,
        HeaderValue, Method, StatusCode, Uri,
    },
};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use common::entity::user::{Role, User};
use common::entity::dictionary::Dictionary;

use crate::api::create_router;
use crate::config::{ServerConfig, DatabaseConfig, QueueConfig};
use crate::db::redb_store::RedbStore;
use crate::queue::message_queue::MessageQueueFactory;
use crate::repository::{
    client_repository::ClientRepository, component_repository::ComponentRepository,
    dictionary_repository::DictionaryRepository, hardware_repository::HardwareRepository,
    person_repository::PersonRepository, project_repository::ProjectRepository,
    rack_repository::RackRepository, user_repository::UserRepository,
};
use crate::service::{
    auth_service::AuthService, client_service::ClientService,
    client_filter_service::ClientFilterService, component_service::ComponentService,
    hardware_service::HardwareService, stats_service::StatsService,
    validation_service::ValidationService,
};
use crate::dao::{ClientDao, RackDao};
use chrono::Utc;

/// Test application state
pub struct TestApp {
    pub db: Arc<RedbStore>,
    pub router: axum::Router,
    pub auth_token: String,
    pub admin_token: String,
    pub test_user: User,
    pub test_admin: User,
}

/// Sets up a test application with a fresh database
async fn setup_test_app() -> TestApp {
    // Create in-memory database
    let db = Arc::new(
        RedbStore::new("file:///tmp/test_db_").unwrap_or_else(|_| {
            // Fallback to temp file creation
            let db_path = format!("/tmp/cmdb_test_{}.db", Uuid::new_v4());
            RedbStore::new(&db_path).expect("Failed to create test database")
        }),
    );

    // Initialize repositories
    let client_repo = Arc::new(ClientRepository::new(db.clone()));
    let hardware_repo = Arc::new(HardwareRepository::new(db.clone()));
    let user_repo = Arc::new(UserRepository::new(db.clone()));
    let person_repo = Arc::new(PersonRepository::new(db.clone()));
    let project_repo = Arc::new(ProjectRepository::new(db.clone()));
    let component_repo = Arc::new(ComponentRepository::new(db.clone()));
    let dictionary_repo = Arc::new(DictionaryRepository::new(db.clone()));
    let rack_repo = Arc::new(RackRepository::new(db.clone()));

    // Initialize services
    let auth_service = Arc::new(AuthService::new("test_secret_key_for_integration_tests_min_32_chars".to_string()));

    let client_dao = Arc::new(ClientDao::new(client_repo.clone(), hardware_repo.clone()));
    let rack_dao = Arc::new(RackDao::new(rack_repo.clone(), client_repo.clone()));
    let client_service = Arc::new(ClientService::from_repositories(
        client_repo.clone(),
        hardware_repo.clone(),
        rack_repo.clone(),
    ));

    let component_service = Arc::new(ComponentService::new(component_repo.clone()));
    let hardware_service = Arc::new(HardwareService::new(
        client_repo.clone(),
        hardware_repo.clone(),
        component_service.clone(),
        MessageQueueFactory::create_flume_queue(),
    ));

    let validation_service = Arc::new(ValidationService::new(
        client_repo.clone(),
        project_repo.clone(),
        rack_repo.clone(),
        person_repo.clone(),
    ));

    let stats_service = Arc::new(StatsService::new(
        client_repo.clone(),
        hardware_repo.clone(),
    ));

    let client_filter_service = Arc::new(ClientFilterService::new(
        client_repo.clone(),
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
    });

    // Create router
    let router = create_router(
        client_repo,
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
        config,
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

    let (status, body) = make_request(&app.router, Method::GET, "/api/v1/version", None, None).await;

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

    let (status, body) = make_request(&app.router, Method::POST, "/api/v1/auth/login", None, Some(creds)).await;

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

    let (status, _) = make_request(&app.router, Method::POST, "/api/v1/auth/login", None, Some(creds)).await;

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
    assert_eq!(body["data"]["hostname"], "test-server");
    assert!(body["data"]["id"].is_string());
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

    let client_id = create_response["data"]["id"].as_str().unwrap();

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

    let client_id = create_response["data"]["id"].as_str().unwrap();

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

    let client_id = create_response["data"]["id"].as_str().unwrap();

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

    let client_id = create_response["data"]["id"].as_str().unwrap();

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

    let (status, body) = make_request(
        &app.router,
        Method::POST,
        &format!("/api/v1/clients/{}/hardware", client_id),
        None,
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

    let client_id = create_response["data"]["id"].as_str().unwrap();

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
        None,
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
        .header(header::CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .header(header::AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", app.admin_token)).unwrap())
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
