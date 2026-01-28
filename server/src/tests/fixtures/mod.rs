//! Common test fixtures for the rs-cmdb server
//!
//! This module provides reusable test fixtures including:
//! - Test database setup
//! - Authentication test users
//! - Test data helpers

use crate::db::{Database, redb_store::RedbStore};
use crate::service::auth_service::AuthService;
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
pub fn test_user() -> TestUser {
    TestUser {
        username: "test_user",
        password: "user123",
        role: Role::User,
        id: "user-test-001".to_string(),
    }
}

/// Returns a test viewer user
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
        warranty_expiration: None,
        supplier: None,
        power_consumption: None,
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
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
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
