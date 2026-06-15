use crate::entity::hardware::{Disk, Hardware, GPU, NIC};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
#[serde(default)]
pub struct Client {
    /// Unique client identifier
    pub id: String,
    /// Client hostname
    pub hostname: String,
    /// Client IP address
    pub ip_address: String,
    /// Primary IP address (management network, auto-detected or manually set)
    pub primary_ip: Option<String>,
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
            primary_ip: None,
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
            primary_ip: None,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatItem {
    pub name: String,
    pub count: usize,
    pub percentage: f64,
    pub client_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CpuStats {
    pub by_vendor: Vec<StatItem>,
    pub by_model: Vec<StatItem>,
    pub by_cores: Vec<StatItem>,
    pub by_threads: Vec<StatItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MemoryStats {
    pub by_capacity: Vec<StatItem>,
    pub by_vendor: Vec<StatItem>,
    pub by_type: Vec<StatItem>,
    pub by_speed: Vec<StatItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct GpuStats {
    pub by_vendor: Vec<StatItem>,
    pub by_model: Vec<StatItem>,
    pub by_model_with_count: Vec<StatItem>,
    pub by_driver_version: Vec<StatItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NetworkStats {
    pub by_type: Vec<StatItem>,
    pub by_vendor: Vec<StatItem>,
    pub by_speed: Vec<StatItem>,
    pub by_status: Vec<StatItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct OsStats {
    pub by_name: Vec<StatItem>,
    pub by_version: Vec<StatItem>,
    pub by_kernel: Vec<StatItem>,
    pub by_architecture: Vec<StatItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ServerStats {
    pub by_vendor: Vec<StatItem>,
    pub by_product_name: Vec<StatItem>,
    pub by_product_version: Vec<StatItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StorageStats {
    pub by_type: Vec<StatItem>,
    pub by_capacity: Vec<StatItem>,
    pub by_vendor: Vec<StatItem>,
}

/// 筛选条件结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
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

/// 客户端硬件导出数据结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientHardwareExport {
    // 基本信息
    pub client_id: String,
    pub hostname: String,
    pub ip_address: String,
    pub primary_ip: Option<String>,
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
#[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComponentQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub status: Option<ComponentStatus>,
    pub component_type: Option<ComponentType>,
    pub search: Option<String>,
    pub client_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub search: Option<String>,
    pub os: Option<String>,
    pub status: Option<String>,
}

/// Export filter request for filtered client export
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportFilterRequest {
    pub search_term: Option<String>,
    pub status: Option<String>,
    pub client_status: Option<String>,
    pub environment: Option<String>,
    pub rack_id: Option<String>,
    pub project_id: Option<String>,
    pub owner_id: Option<String>,
    pub os: Option<String>,
    pub os_kernel: Option<String>,
    pub server_vendor: Option<String>,
    pub cpu_vendor: Option<String>,
    pub cpu_model: Option<String>,
    pub gpu_vendor: Option<String>,
    pub gpu_model: Option<String>,
    pub memory_min: Option<u32>,
    pub memory_max: Option<u32>,
    pub network_type: Option<String>,
    pub network_model: Option<String>,
    pub storage_type: Option<String>,
}

/// Export filter response containing filtered clients with hardware data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportFilterResponse {
    pub clients: Vec<Client>,
    pub hardware_data: Vec<ClientHardwareExport>,
    pub total_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RackQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub search: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersonQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub search: Option<String>,
    pub department: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub search: Option<String>,
    pub department: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HardwareHistoryChangeType {
    Added,
    Removed,
    Modified,
    Upgraded,
    Downgraded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwareHistoryChange {
    pub component: String,
    pub change_type: HardwareHistoryChangeType,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwareHistoryEntry {
    pub timestamp: String,
    pub changes: Vec<HardwareHistoryChange>,
    pub snapshot: Option<Hardware>,
}

pub fn build_hardware_history_entries(
    snapshots: &[(String, Hardware)],
) -> Vec<HardwareHistoryEntry> {
    snapshots
        .iter()
        .enumerate()
        .map(|(index, (timestamp, hardware))| {
            let changes = if index < snapshots.len() - 1 {
                calculate_hardware_history_changes(&snapshots[index + 1].1, hardware)
            } else {
                vec![HardwareHistoryChange {
                    component: "初始版本".to_string(),
                    change_type: HardwareHistoryChangeType::Added,
                    old_value: String::new(),
                    new_value: "首次记录".to_string(),
                }]
            };

            HardwareHistoryEntry {
                timestamp: timestamp.clone(),
                changes,
                snapshot: Some(hardware.clone()),
            }
        })
        .collect()
}

pub fn calculate_hardware_history_changes(
    old: &Hardware,
    new: &Hardware,
) -> Vec<HardwareHistoryChange> {
    let mut changes = Vec::new();

    push_system_changes(&mut changes, old, new);
    push_os_changes(&mut changes, old, new);

    if old.cpu.model_name != new.cpu.model_name {
        changes.push(HardwareHistoryChange {
            component: "CPU".to_string(),
            change_type: HardwareHistoryChangeType::Modified,
            old_value: old.cpu.model_name.clone(),
            new_value: new.cpu.model_name.clone(),
        });
    }

    if old.cpu.vendor_id != new.cpu.vendor_id {
        changes.push(HardwareHistoryChange {
            component: "CPU 厂商".to_string(),
            change_type: HardwareHistoryChangeType::Modified,
            old_value: old.cpu.vendor_id.clone(),
            new_value: new.cpu.vendor_id.clone(),
        });
    }

    if old.cpu.cores != new.cpu.cores || old.cpu.threads != new.cpu.threads {
        changes.push(HardwareHistoryChange {
            component: "CPU 核心/线程".to_string(),
            change_type: if new.cpu.cores >= old.cpu.cores && new.cpu.threads >= old.cpu.threads {
                HardwareHistoryChangeType::Upgraded
            } else {
                HardwareHistoryChangeType::Downgraded
            },
            old_value: format!("{}核{}线程", old.cpu.cores, old.cpu.threads),
            new_value: format!("{}核{}线程", new.cpu.cores, new.cpu.threads),
        });
    }

    if old.ram.total_size != new.ram.total_size {
        changes.push(HardwareHistoryChange {
            component: "内存容量".to_string(),
            change_type: if new.ram.total_size > old.ram.total_size {
                HardwareHistoryChangeType::Upgraded
            } else {
                HardwareHistoryChangeType::Downgraded
            },
            old_value: format!("{} GB", old.ram.total_size),
            new_value: format!("{} GB", new.ram.total_size),
        });
    }

    if old.ram.count != new.ram.count {
        changes.push(HardwareHistoryChange {
            component: "内存条数量".to_string(),
            change_type: if new.ram.count > old.ram.count {
                HardwareHistoryChangeType::Added
            } else {
                HardwareHistoryChangeType::Removed
            },
            old_value: format!("{} 根", old.ram.count),
            new_value: format!("{} 根", new.ram.count),
        });
    }

    if old.ram.speed != new.ram.speed {
        changes.push(HardwareHistoryChange {
            component: "内存频率".to_string(),
            change_type: if new.ram.speed >= old.ram.speed {
                HardwareHistoryChangeType::Upgraded
            } else {
                HardwareHistoryChangeType::Downgraded
            },
            old_value: format!("{} MHz", old.ram.speed),
            new_value: format!("{} MHz", new.ram.speed),
        });
    }

    if old.disks.len() != new.disks.len() {
        changes.push(HardwareHistoryChange {
            component: "硬盘数量".to_string(),
            change_type: if new.disks.len() > old.disks.len() {
                HardwareHistoryChangeType::Added
            } else {
                HardwareHistoryChangeType::Removed
            },
            old_value: format!("{} 个", old.disks.len()),
            new_value: format!("{} 个", new.disks.len()),
        });
    }

    if old.gpus.len() != new.gpus.len() {
        changes.push(HardwareHistoryChange {
            component: "显卡数量".to_string(),
            change_type: if new.gpus.len() > old.gpus.len() {
                HardwareHistoryChangeType::Added
            } else {
                HardwareHistoryChangeType::Removed
            },
            old_value: format!("{} 个", old.gpus.len()),
            new_value: format!("{} 个", new.gpus.len()),
        });
    }

    push_disk_detail_changes(&mut changes, old, new);
    push_gpu_detail_changes(&mut changes, old, new);
    push_nic_changes(&mut changes, old, new);
    push_ipmi_changes(&mut changes, old, new);

    changes
}

fn push_system_changes(changes: &mut Vec<HardwareHistoryChange>, old: &Hardware, new: &Hardware) {
    let old_system = old.system.as_ref();
    let new_system = new.system.as_ref();

    let old_vendor = old_system.map(|s| s.sys_vendor.as_str()).unwrap_or("");
    let new_vendor = new_system.map(|s| s.sys_vendor.as_str()).unwrap_or("");
    if old_vendor != new_vendor {
        changes.push(HardwareHistoryChange {
            component: "系统厂商".to_string(),
            change_type: HardwareHistoryChangeType::Modified,
            old_value: old_vendor.to_string(),
            new_value: new_vendor.to_string(),
        });
    }

    let old_product = old_system.map(|s| s.product_name.as_str()).unwrap_or("");
    let new_product = new_system.map(|s| s.product_name.as_str()).unwrap_or("");
    if old_product != new_product {
        changes.push(HardwareHistoryChange {
            component: "产品型号".to_string(),
            change_type: HardwareHistoryChangeType::Modified,
            old_value: old_product.to_string(),
            new_value: new_product.to_string(),
        });
    }

    let old_serial = old_system.map(|s| s.serial_number.as_str()).unwrap_or("");
    let new_serial = new_system.map(|s| s.serial_number.as_str()).unwrap_or("");
    if old_serial != new_serial {
        changes.push(HardwareHistoryChange {
            component: "序列号".to_string(),
            change_type: HardwareHistoryChangeType::Modified,
            old_value: old_serial.to_string(),
            new_value: new_serial.to_string(),
        });
    }
}

fn push_os_changes(changes: &mut Vec<HardwareHistoryChange>, old: &Hardware, new: &Hardware) {
    let fields = [
        ("操作系统", old.os.name.as_str(), new.os.name.as_str()),
        ("系统版本", old.os.version.as_str(), new.os.version.as_str()),
        ("内核版本", old.os.kernel.as_str(), new.os.kernel.as_str()),
        ("主机名", old.os.hostname.as_str(), new.os.hostname.as_str()),
        (
            "系统主 IP",
            old.os.ip_address.as_str(),
            new.os.ip_address.as_str(),
        ),
        ("DNS", old.os.dns.as_str(), new.os.dns.as_str()),
    ];

    for (component, old_value, new_value) in fields {
        if old_value != new_value {
            changes.push(HardwareHistoryChange {
                component: component.to_string(),
                change_type: HardwareHistoryChangeType::Modified,
                old_value: old_value.to_string(),
                new_value: new_value.to_string(),
            });
        }
    }
}

fn push_disk_detail_changes(
    changes: &mut Vec<HardwareHistoryChange>,
    old: &Hardware,
    new: &Hardware,
) {
    let old_map: BTreeMap<String, _> = old
        .disks
        .iter()
        .map(|disk| (disk_key(disk), disk))
        .collect();
    let new_map: BTreeMap<String, _> = new
        .disks
        .iter()
        .map(|disk| (disk_key(disk), disk))
        .collect();

    for key in old_map.keys() {
        if !new_map.contains_key(key) {
            changes.push(HardwareHistoryChange {
                component: format!("磁盘 {}", key),
                change_type: HardwareHistoryChangeType::Removed,
                old_value: key.clone(),
                new_value: String::new(),
            });
        }
    }

    for key in new_map.keys() {
        if !old_map.contains_key(key) {
            changes.push(HardwareHistoryChange {
                component: format!("磁盘 {}", key),
                change_type: HardwareHistoryChangeType::Added,
                old_value: String::new(),
                new_value: key.clone(),
            });
        }
    }
}

fn push_gpu_detail_changes(
    changes: &mut Vec<HardwareHistoryChange>,
    old: &Hardware,
    new: &Hardware,
) {
    let old_map: BTreeMap<String, _> = old.gpus.iter().map(|gpu| (gpu_key(gpu), gpu)).collect();
    let new_map: BTreeMap<String, _> = new.gpus.iter().map(|gpu| (gpu_key(gpu), gpu)).collect();

    for key in old_map.keys() {
        if !new_map.contains_key(key) {
            changes.push(HardwareHistoryChange {
                component: format!("GPU {}", key),
                change_type: HardwareHistoryChangeType::Removed,
                old_value: key.clone(),
                new_value: String::new(),
            });
        }
    }

    for key in new_map.keys() {
        if !old_map.contains_key(key) {
            changes.push(HardwareHistoryChange {
                component: format!("GPU {}", key),
                change_type: HardwareHistoryChangeType::Added,
                old_value: String::new(),
                new_value: key.clone(),
            });
        }
    }
}

fn push_nic_changes(changes: &mut Vec<HardwareHistoryChange>, old: &Hardware, new: &Hardware) {
    let old_map: BTreeMap<String, _> = old.nics.iter().map(|nic| (nic_key(nic), nic)).collect();
    let new_map: BTreeMap<String, _> = new.nics.iter().map(|nic| (nic_key(nic), nic)).collect();

    for key in old_map.keys() {
        if !new_map.contains_key(key) {
            changes.push(HardwareHistoryChange {
                component: format!("网卡 {}", key),
                change_type: HardwareHistoryChangeType::Removed,
                old_value: key.clone(),
                new_value: String::new(),
            });
        }
    }

    for key in new_map.keys() {
        if !old_map.contains_key(key) {
            changes.push(HardwareHistoryChange {
                component: format!("网卡 {}", key),
                change_type: HardwareHistoryChangeType::Added,
                old_value: String::new(),
                new_value: key.clone(),
            });
        }
    }

    for (key, old_nic) in &old_map {
        if let Some(new_nic) = new_map.get(key) {
            let fields = [
                (
                    "IPv4",
                    old_nic.ipv4_address.as_str(),
                    new_nic.ipv4_address.as_str(),
                ),
                (
                    "IPv6",
                    old_nic.ipv6_address.as_str(),
                    new_nic.ipv6_address.as_str(),
                ),
                (
                    "状态",
                    &format!("{:?}", old_nic.status),
                    &format!("{:?}", new_nic.status),
                ),
                (
                    "速率",
                    &format!("{} Mbps", old_nic.speed),
                    &format!("{} Mbps", new_nic.speed),
                ),
                ("驱动", old_nic.driver.as_str(), new_nic.driver.as_str()),
                ("型号", old_nic.model.as_str(), new_nic.model.as_str()),
            ];

            for (field, old_value, new_value) in fields {
                if old_value != new_value {
                    changes.push(HardwareHistoryChange {
                        component: format!("网卡 {} {}", key, field),
                        change_type: HardwareHistoryChangeType::Modified,
                        old_value: old_value.to_string(),
                        new_value: new_value.to_string(),
                    });
                }
            }
        }
    }
}

fn push_ipmi_changes(changes: &mut Vec<HardwareHistoryChange>, old: &Hardware, new: &Hardware) {
    let old_ipmi = old.ipmi.as_ref();
    let new_ipmi = new.ipmi.as_ref();

    match (old_ipmi, new_ipmi) {
        (None, Some(_)) => changes.push(HardwareHistoryChange {
            component: "IPMI".to_string(),
            change_type: HardwareHistoryChangeType::Added,
            old_value: String::new(),
            new_value: "已配置".to_string(),
        }),
        (Some(_), None) => changes.push(HardwareHistoryChange {
            component: "IPMI".to_string(),
            change_type: HardwareHistoryChangeType::Removed,
            old_value: "已配置".to_string(),
            new_value: String::new(),
        }),
        (Some(old_ipmi), Some(new_ipmi)) => {
            let fields = [
                (
                    "IPMI IP",
                    old_ipmi.ip_address.clone().unwrap_or_default(),
                    new_ipmi.ip_address.clone().unwrap_or_default(),
                ),
                (
                    "IPMI 网关",
                    old_ipmi.gateway.clone().unwrap_or_default(),
                    new_ipmi.gateway.clone().unwrap_or_default(),
                ),
                (
                    "IPMI 固件",
                    old_ipmi.firmware_version.clone().unwrap_or_default(),
                    new_ipmi.firmware_version.clone().unwrap_or_default(),
                ),
                (
                    "IPMI 厂商 ID",
                    old_ipmi
                        .manufacturer_id
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                    new_ipmi
                        .manufacturer_id
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                ),
            ];

            for (component, old_value, new_value) in fields {
                if old_value != new_value {
                    changes.push(HardwareHistoryChange {
                        component: component.to_string(),
                        change_type: HardwareHistoryChangeType::Modified,
                        old_value,
                        new_value,
                    });
                }
            }
        }
        (None, None) => {}
    }
}

fn disk_key(disk: &Disk) -> String {
    if !disk.serial_number.is_empty() {
        disk.serial_number.clone()
    } else {
        format!(
            "{}-{}-{}{}",
            disk.vendor, disk.model, disk.size, disk.size_unit
        )
    }
}

fn gpu_key(gpu: &GPU) -> String {
    if !gpu.serial_number.is_empty() {
        gpu.serial_number.clone()
    } else if !gpu.device_id.is_empty() {
        gpu.device_id.clone()
    } else {
        format!("{}-{}", gpu.vendor, gpu.model)
    }
}

fn nic_key(nic: &NIC) -> String {
    if !nic.name.is_empty() {
        nic.name.clone()
    } else {
        nic.mac_address.clone()
    }
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
