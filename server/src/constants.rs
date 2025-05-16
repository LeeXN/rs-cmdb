/// 标准化的统计类别名称（英文）
pub const CATEGORY_CPU_CONFIG: &str = "cpu_config";
pub const CATEGORY_MEMORY_CONFIG: &str = "memory_config";
pub const CATEGORY_GPU_CONFIG: &str = "gpu_config";
pub const CATEGORY_STORAGE_CONFIG: &str = "storage_config";
pub const CATEGORY_NETWORK_CONFIG: &str = "network_config";
pub const CATEGORY_OS: &str = "operating_system";
pub const CATEGORY_SERVER_MODEL: &str = "server_model";

/// 标准化的未知值标识符（英文）
pub const UNKNOWN_GPU: &str = "no_discrete_gpu";
pub const UNKNOWN_SYSTEM: &str = "unknown_system";
pub const UNKNOWN_MODEL: &str = "unknown_model";
pub const UNKNOWN_VENDOR: &str = "unknown_vendor";
pub const UNKNOWN_VERSION: &str = "unknown_version";
pub const UNKNOWN_KERNEL: &str = "unknown_kernel";
pub const UNKNOWN_ARCH: &str = "unknown_architecture";
pub const UNKNOWN_DRIVER: &str = "no_driver";
pub const UNKNOWN_VALUE: &str = "unknown";
pub const NEVER_SEEN: &str = "never";
pub const NO_STORAGE: &str = "no_storage_devices";

/// 存储类型组合标识符（英文）
pub const STORAGE_NVME_SSD_HDD: &str = "nvme_ssd_hdd_mixed";
pub const STORAGE_NVME_SSD: &str = "nvme_ssd_mixed";
pub const STORAGE_NVME_HDD: &str = "nvme_hdd_mixed";
pub const STORAGE_SSD_HDD: &str = "ssd_hdd_mixed";
pub const STORAGE_PURE_NVME: &str = "pure_nvme";
pub const STORAGE_PURE_SSD: &str = "pure_ssd";
pub const STORAGE_PURE_HDD: &str = "pure_hdd";
pub const STORAGE_UNKNOWN_TYPE: &str = "unknown_storage_type";

/// API 成功消息（英文）
pub const MSG_CLIENTS_FILTERED_SUCCESS: &str = "clients_filtered_successfully";
pub const MSG_FILTER_OPTIONS_SUCCESS: &str = "filter_options_retrieved_successfully";

/// API 错误消息（英文）
pub const MSG_EMPTY_CLIENT_IDS: &str = "empty_client_ids_provided";
pub const MSG_NO_VALID_CLIENT_IDS: &str = "no_valid_client_ids_provided";
pub const MSG_NO_CLIENTS_FOUND: &str = "no_clients_found_with_provided_ids";

/// CPU 和其他硬件单位标识符（英文）
pub const UNIT_CORES: &str = "cores";
pub const UNIT_THREADS: &str = "threads";
pub const UNIT_GB: &str = "gb";
pub const UNIT_MHZ: &str = "mhz";
pub const UNIT_GHZ: &str = "ghz";

/// 硬件数量标识符
pub const COUNT_NICS: &str = "nics";
pub const COUNT_NONE: &str = "none";

/// 筛选默认值（替换中文"全部"）
pub const FILTER_ALL: &str = "all"; 