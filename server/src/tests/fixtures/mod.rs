//! Common test fixtures for the rs-cmdb server
//!
//! This module provides reusable test fixtures including:
//! - Test database setup
//! - Authentication test users
//! - Test data helpers

use crate::db::{Database, redb_store::RedbStore};
use crate::repository::terminal_session_repository::TerminalSessionRepository;
use crate::service::auth_service::{AuthService, init_master_client_key};
use crate::service::terminal_session_service::TerminalSessionService;
use crate::service::web_terminal_service::WebTerminalService;
use chrono::Utc;
use common::entity::hardware::{CPU, Disk, GPU, Hardware, IpmiInfo, NIC, OS, RAM, SystemInfo};
use common::entity::user::{Role, User};
use common::models::{Client, Person, Project, PullResponse, Rack};

/// Test user structure with known credentials
pub struct TestUser {
    pub username: &'static str,
    pub password: &'static str,
    pub role: Role,
    pub id: String,
}

/// Returns a test admin user
///
/// # Example
/// ```
/// let admin = test_admin();
/// assert_eq!(admin.username, "test_admin");
/// ```
pub fn test_admin() -> TestUser {
    TestUser {
        username: "test_admin",
        password: "admin123",
        role: Role::Admin,
        id: "admin-test-001".to_string(),
    }
}

/// Returns a test regular user
///
/// # Example
/// ```
/// let user = test_user();
/// assert_eq!(user.username, "test_user");
/// ```
#[allow(dead_code)]
pub fn test_user() -> TestUser {
    TestUser {
        username: "test_user",
        password: "user123",
        role: Role::User,
        id: "user-test-001".to_string(),
    }
}

/// Returns a test viewer user
#[allow(dead_code)]
pub fn test_viewer() -> TestUser {
    TestUser {
        username: "test_viewer",
        password: "viewer123",
        role: Role::Viewer,
        id: "viewer-test-001".to_string(),
    }
}

/// Setup an in-memory test database
///
/// Creates a temporary in-memory RedbStore instance for testing.
/// The database is automatically cleaned up when dropped.
///
/// Note: ReDB doesn't support true in-memory databases with the `:memory:` path
/// when used across multiple connections. For testing, each test should create
/// its own database instance or use temporary files.
///
/// # Returns
///
/// A `RedbStore` instance for testing
///
/// # Example
/// ```no_run
/// use crate::tests::fixtures::setup_test_db;
///
/// let db = setup_test_db().unwrap();
/// // Use db for testing...
/// // Database is automatically cleaned up when dropped
/// ```
pub fn setup_test_db() -> Result<RedbStore, Box<dyn std::error::Error>> {
    // Use a temporary file-based database for ReDB
    // ReDB doesn't support true :memory: databases
    // Generate a unique filename using timestamp and random value
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let random: u64 = rand::random();
    let db_path = std::env::temp_dir().join(format!("test_rs_cmdb_{}_{}.redb", timestamp, random));
    let db = RedbStore::new(&db_path)?;

    // Initialize master client key for HMAC token generation (OnceLock-safe)
    init_master_client_key("test-master-client-key-for-deterministic-tokens".to_string());

    // Note: The temp file will remain on disk but in the temp directory
    // which gets cleaned up by the OS periodically
    Ok(db)
}

/// Seeds a test user into the database
///
/// Creates a user with hashed password and stores it in the database.
///
/// # Arguments
///
/// * `db` - The database to seed the user into
/// * `user` - The test user to create
///
/// # Example
/// ```no_run
/// use crate::tests::fixtures::{setup_test_db, seed_test_user, test_admin};
///
/// let db = setup_test_db().unwrap();
/// let admin = test_admin();
/// seed_test_user(&db, &admin).await.unwrap();
/// ```
pub async fn seed_test_user(
    db: &RedbStore,
    user: &TestUser,
) -> Result<(), Box<dyn std::error::Error>> {
    let auth_service = AuthService::new("test_secret".to_string());
    let password_hash = auth_service.hash_password(user.password)?;

    let user_entity = User {
        id: user.id.clone(),
        username: user.username.to_string(),
        password_hash,
        role: user.role.clone(),
        created_at: Utc::now().to_rfc3339(),
        last_login: None,
        is_active: true,
    };

    // Serialize and store the user
    let user_json = serde_json::to_vec(&user_entity)?;
    let key = format!("user:{}", user.username);
    db.set(&key, &user_json).await?;

    Ok(())
}

/// Generates a valid JWT token for a test user
///
/// # Arguments
///
/// * `user` - The test user to generate a token for
///
/// # Returns
///
/// A JWT token string
///
/// # Example
/// ```no_run
/// use crate::tests::fixtures::{test_admin, generate_test_token};
///
/// let admin = test_admin();
/// let token = generate_test_token(&admin);
/// assert!(!token.is_empty());
/// ```
pub fn generate_test_token(user: &TestUser) -> String {
    let auth_service = AuthService::new("test_secret".to_string());
    let user_entity = User {
        id: user.id.clone(),
        username: user.username.to_string(),
        password_hash: "hash".to_string(),
        role: user.role.clone(),
        created_at: Utc::now().to_rfc3339(),
        last_login: None,
        is_active: true,
    };

    auth_service
        .generate_token(&user_entity)
        .expect("Failed to generate test token")
}

/// Helper to create authenticated request headers
///
/// # Arguments
///
/// * `token` - The JWT token to include in the Authorization header
///
/// # Returns
///
/// A tuple of ("authorization", "Bearer <token>")
///
/// # Example
/// ```no_run
/// use crate::tests::fixtures::{test_admin, generate_test_token, auth_headers};
///
/// let admin = test_admin();
/// let token = generate_test_token(&admin);
/// let headers = auth_headers(&token);
/// ```
pub fn auth_headers(token: &str) -> (&'static str, String) {
    ("authorization", format!("Bearer {}", token))
}

pub fn create_test_client(id: &str) -> Client {
    Client {
        id: id.to_string(),
        hostname: format!("client-{}", id),
        ip_address: format!("192.168.1.{}", id.split('-').next_back().unwrap_or("1")),
        primary_ip: None,
        os: Some("Linux".to_string()),
        kernel_version: None,
        serial_number: Some(format!("SN-{}", id)),
        sys_vendor: Some("Dell".to_string()),
        product_name: Some("PowerEdge R740".to_string()),
        last_seen: Some(Utc::now().to_rfc3339()),
        registered_at: Some(Utc::now().to_rfc3339()),
        comment: None,
        location: None,
        rack: None,
        unit_position: None,
        u_height: Some(1),
        project_id: None,
        owner_id: None,
        status: None,
        environment: None,
        asset_tag: None,
        tags: Vec::new(),
        warranty_expiration: None,
        supplier: None,
        power_consumption: None,
        agent_token: None,
        created_by: None,
    }
}

pub fn create_test_hardware_info(client_id: &str) -> Hardware {
    Hardware {
        system: Some(SystemInfo {
            sys_vendor: "Dell".to_string(),
            product_name: "PowerEdge R740".to_string(),
            product_version: "".to_string(),
            serial_number: format!("SN-{}", client_id),
        }),
        os: OS {
            name: "Linux".to_string(),
            version: "8.8".to_string(),
            kernel: "4.18.0-477.10.1.el8_8.x86_64".to_string(),
            architecture: "x86_64".to_string(),
            hostname: format!("host-{}", client_id),
            dns: "8.8.8.8".to_string(),
            ip_address: format!(
                "192.168.1.{}",
                client_id.split('-').next_back().unwrap_or("1")
            ),
        },
        cpu: CPU {
            vendor_id: "GenuineIntel".to_string(),
            model_name: "Intel(R) Xeon(R) Silver 4410Y".to_string(),
            cores: 24,
            threads: 48,
            cpus: 2,
            flags: vec![],
            speed: 3900,
        },
        ram: RAM {
            vendor: "Samsung".to_string(),
            model: "DDR5".to_string(),
            size: 256,
            speed: 4800,
            total_size: 256,
            count: 8,
            form_factor: "DIMM".to_string(),
            modules: vec![
                common::entity::hardware::RAMModule {
                    slot: "DIMM_A1".to_string(),
                    vendor: "Samsung".to_string(),
                    part_number: "M321R8GA0BB0".to_string(),
                    serial_number: format!("RAM-{}-1", client_id),
                    size: 32,
                    speed: 4800,
                    form_factor: "DIMM".to_string(),
                    memory_type: "DDR5".to_string(),
                    locator: "DIMM_A1".to_string(),
                },
                common::entity::hardware::RAMModule {
                    slot: "DIMM_A2".to_string(),
                    vendor: "Samsung".to_string(),
                    part_number: "M321R8GA0BB0".to_string(),
                    serial_number: format!("RAM-{}-2", client_id),
                    size: 32,
                    speed: 4800,
                    form_factor: "DIMM".to_string(),
                    memory_type: "DDR5".to_string(),
                    locator: "DIMM_A2".to_string(),
                },
            ],
        },
        gpus: vec![GPU {
            vendor: "NVIDIA".to_string(),
            model: "AD102GL [L20]".to_string(),
            device_id: "0000:63:00.0".to_string(),
            serial_number: format!("GPU-{}-1", client_id),
            driver_version: "550.163.01".to_string(),
        }],
        disks: vec![Disk {
            vendor: "INTEL".to_string(),
            model: "SSDSC2KB96".to_string(),
            size: "894".to_string(),
            size_unit: "GB".to_string(),
            serial_number: format!("DISK-{}-1", client_id),
            storage_type: common::entity::hardware::StorageType::SSD,
            firmware_version: "0120".to_string(),
            parted: false,
            partitions: vec![],
        }],
        nics: vec![NIC {
            name: "eth0".to_string(),
            vendor: "Intel Corporation".to_string(),
            model: "I350 Gigabit Network Connection".to_string(),
            speed: 1000,
            mac_address: format!(
                "00:11:22:33:44:{}",
                client_id.split('-').next_back().unwrap_or("55")
            ),
            ipv4_address: format!(
                "192.168.1.{}",
                client_id.split('-').next_back().unwrap_or("1")
            ),
            ipv4_subnet_mask: "255.255.255.0".to_string(),
            ipv4_gateway: "192.168.1.1".to_string(),
            ipv6_address: "".to_string(),
            ipv6_subnet_mask: "".to_string(),
            ipv6_gateway: "".to_string(),
            dhcp: true,
            bonding_slaves: vec![],
            nic_type: common::entity::hardware::NICType::Ethernet,
            status: common::entity::hardware::NICStatus::Up,
            pci_slot: Some("0000:99:00.0".to_string()),
            firmware_version: "".to_string(),
            ib_node_type: "".to_string(),
            driver: "igb".to_string(),
        }],
        ipmi: Some(IpmiInfo {
            ip_address: Some("10.0.0.10".to_string()),
            mac_address: Some("b0:31:a6:4f:d6:57".to_string()),
            subnet_mask: Some("255.255.254.0".to_string()),
            gateway: Some("10.0.0.254".to_string()),
            channel: 1,
            device_id: Some("32".to_string()),
            firmware_version: Some("6.76".to_string()),
            manufacturer_id: Some(0x019046),
            users: vec![],
            status: common::entity::hardware::IpmiStatus::Available,
        }),
    }
}

pub fn create_client_hardware_info(client_id: &str) -> common::models::ClientHardwareInfo {
    common::models::ClientHardwareInfo {
        client_id: client_id.to_string(),
        collected_at: Utc::now().to_rfc3339(),
        hardware: Some(create_test_hardware_info(client_id)),
    }
}

pub fn create_minimal_hardware_info(client_id: &str) -> Hardware {
    Hardware {
        system: Some(SystemInfo {
            sys_vendor: "Dell".to_string(),
            product_name: "PowerEdge R740".to_string(),
            product_version: "".to_string(),
            serial_number: format!("SN-{}", client_id),
        }),
        os: OS {
            name: "Linux".to_string(),
            version: "8.8".to_string(),
            kernel: "4.18.0-477.10.1.el8_8.x86_64".to_string(),
            architecture: "x86_64".to_string(),
            hostname: format!("host-{}", client_id),
            dns: "8.8.8.8".to_string(),
            ip_address: format!(
                "192.168.1.{}",
                client_id.split('-').next_back().unwrap_or("1")
            ),
        },
        cpu: CPU {
            vendor_id: "GenuineIntel".to_string(),
            model_name: "Intel(R) Xeon(R) Silver 4410Y".to_string(),
            cores: 24,
            threads: 48,
            cpus: 2,
            flags: vec![],
            speed: 3900,
        },
        ram: RAM {
            vendor: "".to_string(),
            model: "".to_string(),
            size: 0,
            speed: 0,
            total_size: 0,
            count: 0,
            form_factor: "".to_string(),
            modules: vec![],
        },
        gpus: vec![],
        disks: vec![],
        nics: vec![],
        ipmi: None,
    }
}

pub fn create_empty_hardware_info() -> Hardware {
    Hardware {
        system: None,
        os: OS::default(),
        cpu: CPU {
            vendor_id: "".to_string(),
            model_name: "".to_string(),
            cores: 0,
            threads: 0,
            cpus: 0,
            flags: vec![],
            speed: 0,
        },
        ram: RAM {
            vendor: "".to_string(),
            model: "".to_string(),
            size: 0,
            speed: 0,
            total_size: 0,
            count: 0,
            form_factor: "".to_string(),
            modules: vec![],
        },
        gpus: vec![],
        disks: vec![],
        nics: vec![],
        ipmi: None,
    }
}

pub fn create_pull_response(client_id: &str, status: &str) -> PullResponse {
    PullResponse {
        request_id: format!("{}:req-123", client_id),
        status: status.to_string(),
        hardware: if status == "success" {
            Some(create_test_hardware_info(client_id))
        } else {
            None
        },
        error: if status == "error" {
            Some("Error message".to_string())
        } else {
            None
        },
    }
}

pub fn create_test_rack(id: &str) -> Rack {
    Rack {
        id: id.to_string(),
        name: format!("Rack {}", id),
        location: Some(format!("Room A, Rack {}", id)),
        height_u: 42,
        power_limit: None,
        description: None,
        created_by: None,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    }
}

pub fn create_test_project(id: &str) -> Project {
    Project {
        id: id.to_string(),
        name: format!("Project {}", id),
        code: None,
        department: None,
        cost_center: None,
        manager_id: Some(format!("manager-{}", id)),
        created_by: None,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    }
}

pub fn create_test_person(id: &str) -> Person {
    Person {
        id: id.to_string(),
        name: format!("Person {}", id),
        email: format!("person{}@example.com", id),
        phone: Some(format!(
            "123-456-7{}0",
            id.split('-').next_back().unwrap_or("0")
        )),
        department: Some("IT".to_string()),
        title: Some("Engineer".to_string()),
        cost_center: None,
        created_by: None,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    }
}

// ── TestApp Builder ──────────────────────────────────────────────────────────

use axum::Router;
use std::sync::Arc;
use uuid::Uuid;

use crate::api::create_router;
use crate::cache::{CacheConfigs, CachedClientRepository};
use crate::config::{DatabaseConfig, QueueConfig, ServerConfig};
use crate::dao::{ClientDao, RackDao};
use crate::queue::message_queue::MessageQueueFactory;
use crate::repository::{
    approval_repository::ApprovalRepository, client_repository::ClientRepository,
    command_repository::CommandRepository, component_repository::ComponentRepository,
    dictionary_repository::DictionaryRepository, exec_policy_repository::ExecPolicyRepository,
    execution_session_repository::ExecutionSessionRepository,
    hardware_repository::HardwareRepository, permission_repository::PermissionRepository,
    person_repository::PersonRepository, project_repository::ProjectRepository,
    rack_repository::RackRepository, user_repository::UserRepository,
    web_terminal_policy_repository::WebTerminalPolicyRepository,
};
use crate::service::{
    approval_service::ApprovalService, client_filter_service::ClientFilterService,
    client_service::ClientService, command_service::CommandService,
    component_service::ComponentService, danger_detection::DangerDetectionService,
    execution_session_service::ExecutionSessionService, export_service::ExportService,
    hardware_service::HardwareService, permission_service::PermissionService, sse_hub::SseHub,
    stats_service::StatsService, validation_service::ValidationService,
};

/// Test application state
pub struct TestApp {
    pub db: Arc<RedbStore>,
    pub router: Router,
    pub auth_token: String,
    pub admin_token: String,
    pub test_user: User,
    #[allow(dead_code)]
    pub test_admin: User,
}

/// Builder for test application instances with override support
pub struct TestAppBuilder {
    db: Option<Arc<RedbStore>>,
    auth_service: Option<Arc<AuthService>>,
    jwt_secret: String,
}

impl Default for TestAppBuilder {
    fn default() -> Self {
        Self {
            db: None,
            auth_service: None,
            jwt_secret: "test_secret_key_for_integration_tests_min_32_chars".to_string(),
        }
    }
}

impl TestAppBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn with_db(mut self, db: Arc<RedbStore>) -> Self {
        self.db = Some(db);
        self
    }

    #[allow(dead_code)]
    pub fn with_auth_service(mut self, svc: Arc<AuthService>) -> Self {
        self.auth_service = Some(svc);
        self
    }

    #[allow(dead_code)]
    pub fn with_jwt_secret(mut self, secret: &str) -> Self {
        self.jwt_secret = secret.to_string();
        self
    }

    pub async fn build(self) -> TestApp {
        // Initialize master client key for HMAC token generation (OnceLock-safe)
        init_master_client_key("test-master-client-key-for-deterministic-tokens".to_string());

        let db = self.db.unwrap_or_else(|| {
            Arc::new(RedbStore::new("file:///tmp/test_db_").unwrap_or_else(|_| {
                let db_path = format!("/tmp/cmdb_test_{}.db", Uuid::new_v4());
                RedbStore::new(&db_path).expect("Failed to create test database")
            }))
        });

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

        let auth_service = self
            .auth_service
            .unwrap_or_else(|| Arc::new(AuthService::new(self.jwt_secret.clone())));

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
            jwt_secret: self.jwt_secret,
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
        let web_terminal_service =
            Arc::new(WebTerminalService::new(web_terminal_policy_repo.clone()));

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
            admin_token,
            test_user,
            test_admin,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setup_test_db() {
        let db = setup_test_db().unwrap();
        // Test that we can write and read
        db.set("test_key", b"test_value").await.unwrap();
        let value: Option<Vec<u8>> = db.get("test_key").await.unwrap();
        assert_eq!(value, Some(b"test_value".to_vec()));
    }

    #[tokio::test]
    async fn test_seed_test_user() {
        let db = setup_test_db().unwrap();
        let admin = test_admin();

        seed_test_user(&db, &admin).await.unwrap();

        // Verify user was created
        let key = format!("user:{}", admin.username);
        let user_data: Option<Vec<u8>> = db.get(&key).await.unwrap();
        assert!(user_data.is_some());

        let user: User = serde_json::from_slice(&user_data.unwrap()).unwrap();
        assert_eq!(user.username, admin.username);
        assert_eq!(user.role, admin.role);
        assert_ne!(user.password_hash, admin.password); // Password should be hashed
    }

    #[test]
    fn test_generate_test_token() {
        let admin = test_admin();
        let token = generate_test_token(&admin);

        assert!(!token.is_empty());

        // Verify token can be decoded
        let auth_service = AuthService::new("test_secret".to_string());
        let claims = auth_service.verify_token(&token).unwrap();
        assert_eq!(claims.sub, admin.id);
        assert_eq!(claims.username, admin.username);
        assert_eq!(claims.role, admin.role);
    }

    #[test]
    fn test_auth_headers() {
        let token = "test_token_123";
        let (key, value) = auth_headers(token);
        assert_eq!(key, "authorization");
        assert_eq!(value, "Bearer test_token_123");
    }
}
