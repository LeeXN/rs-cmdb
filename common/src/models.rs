use crate::entity::execution::ExecutionType;
use crate::entity::hardware::{Disk, Hardware, GPU, NIC};
use crate::entity::permission::{ApprovalStatus, TerminalMode};
use crate::entity::user::Role;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

// ---- Remote Command Execution DTOs ----

/// 创建命令任务请求
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateCommandRequest {
    /// 目标 client_id
    pub client_id: String,
    /// 命令（主程序）
    pub command: String,
    /// 是否按 shell 脚本块执行（主要用于 batch 多行命令）
    #[serde(default)]
    pub shell_mode: bool,
    /// 命令参数列表（可选，向后兼容；为空时按空格从 command 分割）
    #[serde(default)]
    pub args: Option<Vec<String>>,
    /// 执行来源类型，用于区分单机终端和批量执行历史
    #[serde(default)]
    pub execution_type: Option<ExecutionType>,
    /// 超时秒数（可选，默认 300）
    pub timeout_secs: Option<u64>,
    /// 危险命令强制确认标志
    #[serde(default)]
    pub force: bool,
}

/// 命令创建请求最大长度（10KB）
pub const MAX_COMMAND_LENGTH: usize = 10 * 1024;

/// 创建命令任务响应
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateCommandResponse {
    /// 任务 ID（仅在成功创建时存在）
    pub task_id: Option<String>,
    /// 审批请求 ID（需要审批时存在）
    #[serde(default)]
    pub approval_id: Option<String>,
    /// 危险等级
    pub danger_level: String,
    /// 是否需要二次确认
    #[serde(default)]
    pub requires_confirmation: bool,
    /// 匹配的危险规则名称（warning/blocked 时）
    pub matched_rule: Option<String>,
    /// 风险说明或拒绝原因
    pub message: Option<String>,
}

/// 命令历史列表查询参数
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommandQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub search: Option<String>,
    pub client_id: Option<String>,
    pub user_id: Option<String>,
    pub submitted_by: Option<String>,
    pub status: Option<String>,
    pub execution_type: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandTaskListResponse {
    pub tasks: Vec<crate::command::CommandTask>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

/// 远程执行开关配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRemoteExecConfigRequest {
    pub enabled: bool,
}

/// 远程执行开关配置响应
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RemoteExecConfigResponse {
    pub enabled: bool,
}

/// 权限管理概览响应
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PermissionOverviewResponse {
    pub rules_count: usize,
    pub groups_count: usize,
    pub exec_policies_count: usize,
    pub web_terminal_policies_count: usize,
}

/// 审批运营统计响应
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovalSummaryResponse {
    pub total: usize,
    pub pending: usize,
    pub approved: usize,
    pub rejected: usize,
    pub expired: usize,
    pub executed: usize,
}

impl ApprovalSummaryResponse {
    pub fn record_status(&mut self, status: &ApprovalStatus, executed_task_id: Option<&String>) {
        self.total += 1;
        match status {
            ApprovalStatus::Pending => self.pending += 1,
            ApprovalStatus::Approved => self.approved += 1,
            ApprovalStatus::Rejected => self.rejected += 1,
            ApprovalStatus::Expired => self.expired += 1,
        }
        if executed_task_id.is_some() {
            self.executed += 1;
        }
    }
}

/// 终端运维统计响应
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TerminalOpsSummaryResponse {
    pub total_sessions: usize,
    pub pending_sessions: usize,
    pub active_sessions: usize,
    pub closed_sessions: usize,
    pub failed_sessions: usize,
    pub active_clients: usize,
    pub stale_session_threshold_secs: i64,
}

/// 远程执行运维总览响应
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RemoteExecOpsOverviewResponse {
    pub remote_exec_enabled: bool,
    pub exec_policies_count: usize,
    pub web_terminal_policies_count: usize,
    pub approval_summary: ApprovalSummaryResponse,
    pub terminal_summary: TerminalOpsSummaryResponse,
    pub cast_storage_dir: String,
}

/// Cast 历史清理请求
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CastCleanupRequest {
    pub retention_days: u64,
}

/// Cast 历史清理响应
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CastCleanupResponse {
    pub retention_days: u64,
    pub deleted_files: usize,
    pub cast_storage_dir: String,
}

/// Agent 注册成功响应（包含仅此一次下发的 token）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegisterClientResponse {
    /// The registered/updated client record (agent_token field omitted after this response)
    pub client: Client,
    /// Bearer token the agent must include in all subsequent requests.
    /// Only present in the registration response; not retrievable afterwards.
    pub agent_token: String,
}

/// Agent 上传日志请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLogRequest {
    pub lines: Vec<crate::command::CommandLogLine>,
}

/// Agent 完成上报请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCompleteRequest {
    pub exit_code: i32,
    /// "success" | "failed" | "timeout"
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TerminalSessionState {
    #[default]
    Pending,
    Active,
    Closed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalSessionSummary {
    pub session_id: String,
    pub client_id: String,
    pub user_id: String,
    pub username: String,
    #[serde(default)]
    pub role: Role,
    /// Groups used when the terminal policy was evaluated. Persisting this
    /// snapshot keeps group-scoped policies effective for the whole session.
    #[serde(default)]
    pub group_ids: Vec<String>,
    pub mode: TerminalMode,
    pub state: TerminalSessionState,
    pub shell: String,
    pub cols: u16,
    pub rows: u16,
    pub created_at: String,
    pub activated_at: Option<String>,
    pub closed_at: Option<String>,
    pub last_activity_at: Option<String>,
    #[serde(default)]
    pub last_heartbeat_at: Option<String>,
    #[serde(default)]
    pub lease_id: Option<String>,
    #[serde(default)]
    pub lease_expires_at: Option<String>,
    pub close_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateTerminalSessionRequest {
    pub client_id: String,
    pub shell: Option<String>,
    pub cols: Option<u16>,
    pub rows: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateTerminalSessionResponse {
    pub session: TerminalSessionSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalInputRequest {
    pub input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResizeTerminalRequest {
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TerminalOutputChunk {
    pub seq: u64,
    pub data: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentTerminalPollResponse {
    pub session: TerminalSessionSummary,
    pub pending_input: Vec<TerminalInputChunk>,
    pub resize: Option<TerminalResizeInstruction>,
    pub close_requested: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentTerminalStreamServerMessage {
    Sync { work: AgentTerminalPollResponse },
    Heartbeat,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentTerminalStreamClientMessage {
    Output {
        session_id: String,
        payload: AgentTerminalOutputRequest,
    },
    State {
        session_id: String,
        payload: AgentTerminalStateRequest,
    },
    Heartbeat {
        session_ids: Vec<String>,
        claim_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalInputChunk {
    pub seq: u64,
    pub data: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalResizeInstruction {
    pub cols: u16,
    pub rows: u16,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentTerminalOutputRequest {
    #[serde(default)]
    pub claim_id: Option<String>,
    pub chunks: Vec<TerminalOutputChunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentTerminalStateRequest {
    #[serde(default)]
    pub claim_id: Option<String>,
    pub state: TerminalSessionState,
    pub message: Option<String>,
}

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
    /// Additional free-form tags used by scoped permissions and policies.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Warranty Expiration Date
    pub warranty_expiration: Option<String>,
    /// Supplier / Vendor
    pub supplier: Option<String>,

    /// Power Consumption (Watts) - Manual setting
    pub power_consumption: Option<u32>,

    /// Agent authentication token (generated on first registration, stored hashed).
    /// Only returned once at registration time; thereafter used for verifying agent requests.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub agent_token: Option<String>,
    /// User ID of the creator
    #[serde(default)]
    pub created_by: Option<String>,
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
    pub created_by: Option<String>,
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
            created_by: None,
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
    pub created_by: Option<String>,
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
            created_by: None,
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
            tags: Vec::new(),
            warranty_expiration: None,
            supplier: None,
            power_consumption: None,
            agent_token: None,
            created_by: None,
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
            tags: Vec::new(),
            warranty_expiration: None,
            supplier: None,
            power_consumption: None,
            agent_token: None,
            created_by: None,
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

    pub created_by: Option<String>,

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
            created_by: None,
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
    pub created_by: Option<String>,
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
            created_by: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_round_trip() {
        let c = Client::new("host1.example.com".into(), "192.168.1.1".into());
        let json = serde_json::to_string(&c).unwrap();
        let deserialized: Client = serde_json::from_str(&json).unwrap();
        assert_eq!(c.id, deserialized.id);
        assert_eq!(c.hostname, deserialized.hostname);
        assert_eq!(c.ip_address, deserialized.ip_address);
        assert!(deserialized.agent_token.is_none());
    }

    #[test]
    fn test_client_default_round_trip() {
        let c = Client::default();
        let json = serde_json::to_string(&c).unwrap();
        let deserialized: Client = serde_json::from_str(&json).unwrap();
        assert_eq!(c.id, deserialized.id);
    }

    #[test]
    fn test_component_round_trip() {
        let c = Component::default();
        let json = serde_json::to_string(&c).unwrap();
        let deserialized: Component = serde_json::from_str(&json).unwrap();
        assert_eq!(c.id, deserialized.id);
        assert_eq!(c.serial_number, deserialized.serial_number);
    }

    #[test]
    fn test_component_with_fields() {
        let c = Component {
            serial_number: "SN-001".into(),
            model: "Tesla T4".into(),
            vendor: Some("NVIDIA".into()),
            component_type: ComponentType::GPU,
            status: ComponentStatus::InUse,
            ..Default::default()
        };
        let json = serde_json::to_string(&c).unwrap();
        let deserialized: Component = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.serial_number, "SN-001");
        assert_eq!(deserialized.vendor, Some("NVIDIA".into()));
    }

    #[test]
    fn test_person_round_trip() {
        let p = Person::default();
        let json = serde_json::to_string(&p).unwrap();
        let deserialized: Person = serde_json::from_str(&json).unwrap();
        assert_eq!(p.id, deserialized.id);
    }

    #[test]
    fn test_project_round_trip() {
        let p = Project::default();
        let json = serde_json::to_string(&p).unwrap();
        let deserialized: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(p.id, deserialized.id);
    }

    #[test]
    fn test_rack_round_trip() {
        let r = Rack::default();
        let json = serde_json::to_string(&r).unwrap();
        let deserialized: Rack = serde_json::from_str(&json).unwrap();
        assert_eq!(r.id, deserialized.id);
        assert_eq!(r.height_u, deserialized.height_u);
    }

    #[test]
    fn test_apiresponse_round_trip() {
        let resp: ApiResponse<String> = ApiResponse {
            status: 200,
            message: "ok".into(),
            data: Some("hello".into()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: ApiResponse<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.status, 200);
        assert_eq!(deserialized.data, Some("hello".into()));
    }

    #[test]
    fn test_client_new() {
        let c = Client::new("host-a".into(), "10.0.0.5".into());
        assert_eq!(c.hostname, "host-a");
        assert_eq!(c.ip_address, "10.0.0.5");
        assert!(c.last_seen.is_some());
        assert_eq!(c.u_height, Some(1));
    }

    #[test]
    fn test_client_update_last_seen() {
        let mut c = Client::default();
        let before = c.last_seen.clone();
        std::thread::sleep(std::time::Duration::from_millis(10));
        c.update_last_seen();
        assert_ne!(c.last_seen, before);
    }
}
