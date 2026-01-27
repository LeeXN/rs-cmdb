use crate::entity::hardware::Hardware;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn default_uuid() -> String {
    Uuid::new_v4().to_string()
}

fn default_now() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Client hardware info message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientHardwareInfo {
    /// Unique client identifier
    pub client_id: String,
    /// Hardware information
    pub hardware: Option<Hardware>,
    /// Timestamp of collection (ISO 8601 format)
    pub collected_at: String,
}

/// Server pull request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PullRequest {
    /// Request ID
    pub request_id: String,
    /// Target hardware components to collect
    pub components: Vec<String>,
    /// Request time
    pub requested_at: String,
}

/// Client response to pull request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PullResponse {
    /// Request ID (matches the original request)
    pub request_id: String,
    /// Hardware information
    pub hardware: Option<Hardware>,
    /// Status of the collection (success/error)
    pub status: String,
    /// Error message if status is error
    pub error: Option<String>,
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiResponse<T>
where
    T: PartialEq,
{
    /// Status code
    pub status: u16,
    /// Status message
    pub message: String,
    /// Response data
    pub data: Option<T>,
}

/// Client base model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Client {
    /// Unique client identifier
    pub id: String,
    /// Client hostname
    pub hostname: String,
    /// Client IP address
    pub ip_address: String,
    /// OS Type
    pub os: Option<String>,
    /// Kernel Version
    pub kernel_version: Option<String>,
    /// Machine Serial Number
    pub serial_number: Option<String>,
    /// System sys vendor  
    pub sys_vendor: Option<String>,
    /// System product name
    pub product_name: Option<String>,
    /// Last seen timestamp
    pub last_seen: Option<String>,
    /// Registration timestamp
    pub registered_at: Option<String>,
    /// Optional comment for the client
    pub comment: Option<String>,

    // --- New Fields for CMDB Enhancement ---
    /// Physical location (Data Center / Room)
    pub location: Option<String>,
    /// Rack identifier
    pub rack: Option<String>,
    /// Unit position in rack
    pub unit_position: Option<String>,
    /// Height in U (default 1)
    pub u_height: Option<u32>,

    /// Associated Project ID
    pub project_id: Option<String>,
    /// Responsible Person ID (Owner)
    pub owner_id: Option<String>,

    /// Operational Status
    pub status: Option<ClientStatus>,
    /// Deployment Environment
    pub environment: Option<Environment>,

    /// Asset Tag (Fixed Asset Number)
    pub asset_tag: Option<String>,
    /// Warranty Expiration Date
    pub warranty_expiration: Option<String>,
    /// Supplier / Vendor
    pub supplier: Option<String>,

    /// Power Consumption (Watts) - Manual setting
    pub power_consumption: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClientStatus {
    Active,
    Maintenance,
    InStock,
    Decommissioned,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
    Prod,
    Dev,
    Test,
    Staging,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Person {
    #[serde(default = "default_uuid")]
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub department: Option<String>,
    pub title: Option<String>,
    pub cost_center: Option<String>,
    #[serde(default = "default_now")]
    pub created_at: String,
    #[serde(default = "default_now")]
    pub updated_at: String,
}

impl Default for Person {
    fn default() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            name: String::new(),
            email: String::new(),
            phone: None,
            department: None,
            title: None,
            cost_center: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    #[serde(default = "default_uuid")]
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    pub department: Option<String>,
    pub cost_center: Option<String>,
    pub manager_id: Option<String>,
    #[serde(default = "default_now")]
    pub created_at: String,
    #[serde(default = "default_now")]
    pub updated_at: String,
}

impl Default for Project {
    fn default() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            name: String::new(),
            code: None,
            department: None,
            cost_center: None,
            manager_id: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: String::new(),
            hostname: String::new(),
            ip_address: String::new(),
            os: None,
            kernel_version: None,
            sys_vendor: None,
            product_name: None,
            serial_number: None,
            last_seen: Some(now.clone()),
            registered_at: Some(now),
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
}

impl Client {
    pub fn new(hostname: String, ip_address: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            hostname,
            ip_address,
            os: None,
            kernel_version: None,
            sys_vendor: None,
            product_name: None,
            serial_number: None,
            last_seen: Some(now.clone()),
            registered_at: Some(now),
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

    pub fn update_last_seen(&mut self) {
        self.last_seen = Some(chrono::Utc::now().to_rfc3339());
    }
}

/// 详细统计数据结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetailedStats {
    pub total_clients: usize,
    pub online_clients: usize,
    pub offline_clients: usize,
    pub cpu_stats: CpuStats,
    pub memory_stats: MemoryStats,
    pub gpu_stats: GpuStats,
    pub network_stats: NetworkStats,
    pub os_stats: OsStats,
    pub server_stats: ServerStats,
    pub storage_stats: StorageStats,
}

impl Default for DetailedStats {
    fn default() -> Self {
        Self {
            total_clients: 0,
            online_clients: 0,
            offline_clients: 0,
            cpu_stats: CpuStats::default(),
            memory_stats: MemoryStats::default(),
            gpu_stats: GpuStats::default(),
            network_stats: NetworkStats::default(),
            os_stats: OsStats::default(),
            server_stats: ServerStats::default(),
            storage_stats: StorageStats::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatItem {
    pub name: String,
    pub count: usize,
    pub percentage: f64,
    pub client_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CpuStats {
    pub by_vendor: Vec<StatItem>,
    pub by_model: Vec<StatItem>,
    pub by_cores: Vec<StatItem>,
    pub by_threads: Vec<StatItem>,
}

impl Default for CpuStats {
    fn default() -> Self {
        Self {
            by_vendor: Vec::new(),
            by_model: Vec::new(),
            by_cores: Vec::new(),
            by_threads: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryStats {
    pub by_capacity: Vec<StatItem>,
    pub by_vendor: Vec<StatItem>,
    pub by_type: Vec<StatItem>,
    pub by_speed: Vec<StatItem>,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            by_capacity: Vec::new(),
            by_vendor: Vec::new(),
            by_type: Vec::new(),
            by_speed: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GpuStats {
    pub by_vendor: Vec<StatItem>,
    pub by_model: Vec<StatItem>,
    pub by_model_with_count: Vec<StatItem>,
    pub by_driver_version: Vec<StatItem>,
}

impl Default for GpuStats {
    fn default() -> Self {
        Self {
            by_vendor: Vec::new(),
            by_model: Vec::new(),
            by_model_with_count: Vec::new(),
            by_driver_version: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkStats {
    pub by_type: Vec<StatItem>,
    pub by_vendor: Vec<StatItem>,
    pub by_speed: Vec<StatItem>,
    pub by_status: Vec<StatItem>,
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            by_type: Vec::new(),
            by_vendor: Vec::new(),
            by_speed: Vec::new(),
            by_status: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OsStats {
    pub by_name: Vec<StatItem>,
    pub by_version: Vec<StatItem>,
    pub by_kernel: Vec<StatItem>,
    pub by_architecture: Vec<StatItem>,
}

impl Default for OsStats {
    fn default() -> Self {
        Self {
            by_name: Vec::new(),
            by_version: Vec::new(),
            by_kernel: Vec::new(),
            by_architecture: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerStats {
    pub by_vendor: Vec<StatItem>,
    pub by_product_name: Vec<StatItem>,
    pub by_product_version: Vec<StatItem>,
}

impl Default for ServerStats {
    fn default() -> Self {
        Self {
            by_vendor: Vec::new(),
            by_product_name: Vec::new(),
            by_product_version: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorageStats {
    pub by_type: Vec<StatItem>,
    pub by_capacity: Vec<StatItem>,
    pub by_vendor: Vec<StatItem>,
}

impl Default for StorageStats {
    fn default() -> Self {
        Self {
            by_type: Vec::new(),
            by_capacity: Vec::new(),
            by_vendor: Vec::new(),
        }
    }
}

/// 筛选条件结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FilterCriteria {
    pub cpu_vendor: Option<String>,
    pub cpu_model: Option<String>,
    pub cpu_cores: Option<u32>,
    pub memory_capacity_min: Option<u32>,
    pub memory_capacity_max: Option<u32>,
    pub gpu_vendor: Option<String>,
    pub gpu_model: Option<String>,
    pub os_name: Option<String>,
    pub os_kernel: Option<String>,
    pub server_vendor: Option<String>,
    pub storage_type: Option<String>,
    pub network_type: Option<String>,
}

/// 筛选选项结构 - 从数据库实际数据中动态生成
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FilterOptions {
    pub cpu_vendors: Vec<String>,
    pub cpu_models: Vec<String>,
    pub gpu_vendors: Vec<String>,
    pub gpu_models: Vec<String>,
    pub os_names: Vec<String>,
    pub os_kernels: Vec<String>,
    pub server_vendors: Vec<String>,
    pub storage_types: Vec<String>,
    pub network_types: Vec<String>,
    pub network_models: Vec<String>,
}

impl Default for FilterCriteria {
    fn default() -> Self {
        Self {
            cpu_vendor: None,
            cpu_model: None,
            cpu_cores: None,
            memory_capacity_min: None,
            memory_capacity_max: None,
            gpu_vendor: None,
            gpu_model: None,
            os_name: None,
            os_kernel: None,
            server_vendor: None,
            storage_type: None,
            network_type: None,
        }
    }
}

/// 客户端硬件导出数据结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientHardwareExport {
    // 基本信息
    pub client_id: String,
    pub hostname: String,
    pub ip_address: String,
    pub os: String,
    pub kernel_version: String,
    pub sys_vendor: String,
    pub product_name: String,
    pub serial_number: String,
    pub last_seen: String,
    pub registered_at: String,

    // CPU信息
    pub cpu_vendor: String,
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub cpu_threads: u32,
    pub cpu_frequency: String,

    // 内存信息
    pub memory_total: String,
    pub memory_vendor: String,
    pub memory_speed: String,
    pub memory_modules: u32,

    // GPU信息
    pub gpu_count: u32,
    pub gpu_models: String,
    pub gpu_vendors: String,

    // 存储信息
    pub storage_count: u32,
    pub storage_total: String,
    pub storage_types: String,

    // 网络信息
    pub network_count: u32,
    pub network_types: String,
    pub network_speeds: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Component {
    #[serde(default = "default_uuid")]
    pub id: String,
    pub serial_number: String,
    pub model: String,
    pub vendor: Option<String>,
    pub component_type: ComponentType,
    pub status: ComponentStatus,

    // Associations
    pub client_id: Option<String>,       // If installed in a server
    pub client_hostname: Option<String>, // Hostname of the client
    pub location: Option<String>,        // If in stock

    // Financial/Asset
    pub purchase_date: Option<String>,
    pub warranty_expiration: Option<String>,

    // Flapping control
    pub missing_since: Option<String>,

    #[serde(default = "default_now")]
    pub created_at: String,
    #[serde(default = "default_now")]
    pub updated_at: String,
}

impl Default for Component {
    fn default() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            serial_number: String::new(),
            model: String::new(),
            vendor: None,
            component_type: ComponentType::Other,
            status: ComponentStatus::Unknown,
            client_id: None,
            client_hostname: None,
            location: None,
            purchase_date: None,
            warranty_expiration: None,
            missing_since: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComponentType {
    GPU,
    CPU,
    Memory,
    Disk,
    NetworkCard,
    Motherboard,
    PowerSupply,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComponentStatus {
    InStock,
    InUse,
    Faulty,
    Decommissioned,
    LentOut,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub status: Option<ComponentStatus>,
    pub component_type: Option<ComponentType>,
    pub search: Option<String>,
    pub client_id: Option<String>,
}

impl Default for ComponentQuery {
    fn default() -> Self {
        Self {
            page: None,
            page_size: None,
            status: None,
            component_type: None,
            search: None,
            client_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub search: Option<String>,
    pub os: Option<String>,
    pub status: Option<String>,
}

impl Default for ClientQuery {
    fn default() -> Self {
        Self {
            page: None,
            page_size: None,
            search: None,
            os: None,
            status: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RackQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub search: Option<String>,
    pub location: Option<String>,
}

impl Default for RackQuery {
    fn default() -> Self {
        Self {
            page: None,
            page_size: None,
            search: None,
            location: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub search: Option<String>,
    pub department: Option<String>,
}

impl Default for PersonQuery {
    fn default() -> Self {
        Self {
            page: None,
            page_size: None,
            search: None,
            department: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub search: Option<String>,
    pub department: Option<String>,
}

impl Default for ProjectQuery {
    fn default() -> Self {
        Self {
            page: None,
            page_size: None,
            search: None,
            department: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rack {
    #[serde(default = "default_uuid")]
    pub id: String,
    pub name: String,
    pub location: Option<String>, // Data Center / Room
    pub height_u: u32,            // Total U height (e.g. 42)
    pub power_limit: Option<u32>, // Power limit in Watts
    pub description: Option<String>,
    #[serde(default = "default_now")]
    pub created_at: String,
    #[serde(default = "default_now")]
    pub updated_at: String,
}

impl Default for Rack {
    fn default() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            name: String::new(),
            location: None,
            height_u: 42,
            power_limit: None,
            description: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}
