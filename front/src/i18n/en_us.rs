use std::collections::HashMap;

pub fn get_translations() -> HashMap<String, String> {
    let mut translations = HashMap::new();

    // 通用
    translations.insert("all".to_string(), "All".to_string());
    translations.insert("unknown".to_string(), "Unknown".to_string());
    translations.insert("online".to_string(), "Online".to_string());
    translations.insert("offline".to_string(), "Offline".to_string());
    translations.insert("none".to_string(), "None".to_string());
    translations.insert("never".to_string(), "Never".to_string());
    translations.insert("count".to_string(), "".to_string());

    // 硬件单位
    translations.insert("cores".to_string(), "Cores".to_string());
    translations.insert("threads".to_string(), "Threads".to_string());
    translations.insert("gb".to_string(), "GB".to_string());
    translations.insert("mhz".to_string(), "MHz".to_string());
    translations.insert("ghz".to_string(), "GHz".to_string());
    translations.insert("nics".to_string(), "NICs".to_string());

    // 硬件类别
    translations.insert("cpu_config".to_string(), "CPU Configuration".to_string());
    translations.insert(
        "memory_config".to_string(),
        "Memory Configuration".to_string(),
    );
    translations.insert("gpu_config".to_string(), "GPU Configuration".to_string());
    translations.insert(
        "storage_config".to_string(),
        "Storage Configuration".to_string(),
    );
    translations.insert(
        "network_config".to_string(),
        "Network Configuration".to_string(),
    );
    translations.insert(
        "operating_system".to_string(),
        "Operating System".to_string(),
    );
    translations.insert("server_model".to_string(), "Server Model".to_string());

    // Hardware Component Titles
    translations.insert("cpu.title".to_string(), "CPU".to_string());
    translations.insert("gpu.title".to_string(), "GPU".to_string());
    translations.insert("memory.title".to_string(), "Memory".to_string());
    translations.insert("network.title".to_string(), "Network".to_string());
    translations.insert("storage.title".to_string(), "Storage".to_string());

    // Hardware Labels
    translations.insert("label.vendor".to_string(), "Vendor".to_string());
    translations.insert("label.model".to_string(), "Model".to_string());
    translations.insert("label.frequency".to_string(), "Frequency".to_string());
    translations.insert("label.cores".to_string(), "Cores".to_string());
    translations.insert("label.threads".to_string(), "Threads".to_string());
    translations.insert("label.device_id".to_string(), "Device ID".to_string());
    translations.insert("label.driver_version".to_string(), "Driver Version".to_string());
    translations.insert("label.serial_number".to_string(), "Serial Number".to_string());
    translations.insert("label.capacity".to_string(), "Capacity".to_string());
    translations.insert("label.speed".to_string(), "Speed".to_string());
    translations.insert("label.firmware".to_string(), "Firmware".to_string());
    translations.insert("label.interface".to_string(), "Interface".to_string());
    translations.insert("label.size".to_string(), "Size".to_string());
    translations.insert("label.type".to_string(), "Type".to_string());
    translations.insert("label.manufacturer".to_string(), "Manufacturer".to_string());
    translations.insert("label.temperature".to_string(), "Temperature".to_string());
    translations.insert("label.voltage".to_string(), "Voltage".to_string());
    translations.insert("label.power".to_string(), "Power".to_string());
    translations.insert("label.utilization".to_string(), "Utilization".to_string());

    // Hardware History
    translations.insert("history.change".to_string(), "Change".to_string());
    translations.insert("history.change_type".to_string(), "Change Type".to_string());
    translations.insert("history.empty".to_string(), "No changes recorded".to_string());
    translations.insert("history.loading".to_string(), "Loading changes...".to_string());
    translations.insert("history.time".to_string(), "Time".to_string());
    translations.insert("history.title".to_string(), "Hardware History".to_string());
    translations.insert("history.view_details".to_string(), "View Details".to_string());

    // IPMI
    translations.insert("ipmi.access_denied".to_string(), "Access Denied".to_string());
    translations.insert("ipmi.not_available".to_string(), "Not Available".to_string());
    translations.insert("ipmi.status_online".to_string(), "Online".to_string());
    translations.insert("ipmi.status_offline".to_string(), "Offline".to_string());
    translations.insert("ipmi.status_unknown".to_string(), "Unknown".to_string());
    translations.insert("ipmi.privilege_admin".to_string(), "Administrator".to_string());
    translations.insert("ipmi.privilege_user".to_string(), "User".to_string());
    translations.insert("ipmi.privilege_operator".to_string(), "Operator".to_string());
    translations.insert("ipmi.privilege_callback".to_string(), "Callback".to_string());
    translations.insert("ipmi.users".to_string(), "Users".to_string());
    translations.insert("ipmi.username".to_string(), "Username".to_string());
    translations.insert("ipmi.password".to_string(), "Password".to_string());
    translations.insert("ipmi.channel".to_string(), "Channel".to_string());
    translations.insert("ipmi.ip_address".to_string(), "IP Address".to_string());
    translations.insert("ipmi.mac_address".to_string(), "MAC Address".to_string());
    translations.insert("ipmi.netmask".to_string(), "Netmask".to_string());
    translations.insert("ipmi.gateway".to_string(), "Gateway".to_string());

    // Status (with prefix to avoid conflicts)
    translations.insert("status.online".to_string(), "Online".to_string());
    translations.insert("status.offline".to_string(), "Offline".to_string());
    translations.insert("status.enabled".to_string(), "Enabled".to_string());
    translations.insert("status.disabled".to_string(), "Disabled".to_string());
    translations.insert("status.active".to_string(), "Active".to_string());
    translations.insert("status.inactive".to_string(), "Inactive".to_string());
    translations.insert("status.unknown".to_string(), "Unknown".to_string());
    translations.insert("status.available".to_string(), "Available".to_string());
    translations.insert("status.unavailable".to_string(), "Unavailable".to_string());

    // Change Types
    translations.insert("change.added".to_string(), "Added".to_string());
    translations.insert("change.removed".to_string(), "Removed".to_string());
    translations.insert("change.modified".to_string(), "Modified".to_string());
    translations.insert("change.upgraded".to_string(), "Upgraded".to_string());
    translations.insert("change.downgraded".to_string(), "Downgraded".to_string());
    translations.insert("change.replaced".to_string(), "Replaced".to_string());
    translations.insert("change.migrated".to_string(), "Migrated".to_string());

    // Network Configuration
    translations.insert("network.bonding_slaves".to_string(), "Bonding Slaves".to_string());
    translations.insert("network.config".to_string(), "Network Configuration".to_string());
    translations.insert("network.ipv4_config".to_string(), "IPv4 Configuration".to_string());
    translations.insert("network.ipv6_config".to_string(), "IPv6 Configuration".to_string());
    translations.insert("network.mac_address".to_string(), "MAC Address".to_string());
    translations.insert("network.ip_address".to_string(), "IP Address".to_string());
    translations.insert("network.subnet_mask".to_string(), "Subnet Mask".to_string());
    translations.insert("network.gateway".to_string(), "Gateway".to_string());
    translations.insert("network.dns_servers".to_string(), "DNS Servers".to_string());
    translations.insert("network.speed".to_string(), "Speed".to_string());
    translations.insert("network.duplex".to_string(), "Duplex".to_string());
    translations.insert("network.mtu".to_string(), "MTU".to_string());
    translations.insert("network.bond_mode".to_string(), "Bond Mode".to_string());
    translations.insert("network.vlan".to_string(), "VLAN".to_string());
    translations.insert("network.bridge".to_string(), "Bridge".to_string());

    // Storage
    translations.insert("storage.partitions".to_string(), "Partitions".to_string());
    translations.insert("storage.partition".to_string(), "Partition".to_string());
    translations.insert("storage.mount_point".to_string(), "Mount Point".to_string());
    translations.insert("storage.file_system".to_string(), "File System".to_string());
    translations.insert("storage.used".to_string(), "Used".to_string());
    translations.insert("storage.available".to_string(), "Available".to_string());
    translations.insert("storage.usage_percent".to_string(), "Usage %".to_string());
    translations.insert("storage.disk_type".to_string(), "Disk Type".to_string());
    translations.insert("storage.rotational_speed".to_string(), "Rotational Speed".to_string());
    translations.insert("storage.form_factor".to_string(), "Form Factor".to_string());
    translations.insert("storage.smart_status".to_string(), "SMART Status".to_string());

    // Memory
    translations.insert("memory.modules_detail".to_string(), "Memory Modules Detail".to_string());
    translations.insert("memory.module".to_string(), "Memory Module".to_string());
    translations.insert("memory.type".to_string(), "Type".to_string());
    translations.insert("memory.speed".to_string(), "Speed".to_string());
    translations.insert("memory.size".to_string(), "Size".to_string());
    translations.insert("memory.bank_label".to_string(), "Bank Label".to_string());
    translations.insert("memory.manufacturer".to_string(), "Manufacturer".to_string());
    translations.insert("memory.serial_number".to_string(), "Serial Number".to_string());
    translations.insert("memory.part_number".to_string(), "Part Number".to_string());
    translations.insert("memory.ecc".to_string(), "ECC".to_string());
    translations.insert("memory.voltage".to_string(), "Voltage".to_string());
    translations.insert("memory.frequency".to_string(), "Frequency".to_string());
    translations.insert("memory.bandwidth".to_string(), "Bandwidth".to_string());
    translations.insert("memory.channels".to_string(), "Channels".to_string());

    // 未知值
    translations.insert("no_discrete_gpu".to_string(), "No Discrete GPU".to_string());
    translations.insert("unknown_system".to_string(), "Unknown System".to_string());
    translations.insert("unknown_model".to_string(), "Unknown Model".to_string());
    translations.insert("unknown_vendor".to_string(), "Unknown Vendor".to_string());
    translations.insert("unknown_version".to_string(), "Unknown Version".to_string());
    translations.insert("unknown_kernel".to_string(), "Unknown Kernel".to_string());
    translations.insert(
        "unknown_architecture".to_string(),
        "Unknown Architecture".to_string(),
    );
    translations.insert("no_driver".to_string(), "No Driver".to_string());
    translations.insert(
        "no_storage_devices".to_string(),
        "No Storage Devices".to_string(),
    );

    // 存储类型
    translations.insert(
        "nvme_ssd_hdd_mixed".to_string(),
        "NVMe+SSD+HDD Mixed".to_string(),
    );
    translations.insert("nvme_ssd_mixed".to_string(), "NVMe+SSD Mixed".to_string());
    translations.insert("nvme_hdd_mixed".to_string(), "NVMe+HDD Mixed".to_string());
    translations.insert("ssd_hdd_mixed".to_string(), "SSD+HDD Mixed".to_string());
    translations.insert("pure_nvme".to_string(), "Pure NVMe".to_string());
    translations.insert("pure_ssd".to_string(), "Pure SSD".to_string());
    translations.insert("pure_hdd".to_string(), "Pure HDD".to_string());
    translations.insert(
        "unknown_storage_type".to_string(),
        "Unknown Storage Type".to_string(),
    );

    // API 成功消息
    translations.insert(
        "clients_filtered_successfully".to_string(),
        "Clients filtered successfully".to_string(),
    );
    translations.insert(
        "filter_options_retrieved_successfully".to_string(),
        "Filter options retrieved successfully".to_string(),
    );
    translations.insert(
        "client_registered_successfully".to_string(),
        "Client registered successfully".to_string(),
    );
    translations.insert(
        "client_updated_successfully".to_string(),
        "Client updated successfully".to_string(),
    );
    translations.insert(
        "client_deleted_successfully".to_string(),
        "Client deleted successfully".to_string(),
    );
    translations.insert(
        "clients_listed_successfully".to_string(),
        "Clients listed successfully".to_string(),
    );
    translations.insert(
        "client_retrieved_successfully".to_string(),
        "Client retrieved successfully".to_string(),
    );
    translations.insert(
        "hardware_retrieved_successfully".to_string(),
        "Hardware retrieved successfully".to_string(),
    );
    translations.insert(
        "stats_retrieved_successfully".to_string(),
        "Statistics retrieved successfully".to_string(),
    );

    // API 错误消息
    translations.insert(
        "empty_client_ids_provided".to_string(),
        "Empty client IDs provided".to_string(),
    );
    translations.insert(
        "no_valid_client_ids_provided".to_string(),
        "No valid client IDs provided".to_string(),
    );
    translations.insert(
        "no_clients_found_with_provided_ids".to_string(),
        "No clients found with provided IDs".to_string(),
    );

    // 错误码翻译
    translations.insert(
        "internal_server_error".to_string(),
        "Internal Server Error".to_string(),
    );
    translations.insert("invalid_request".to_string(), "Invalid Request".to_string());
    translations.insert(
        "validation_error".to_string(),
        "Validation Error".to_string(),
    );
    translations.insert("not_found".to_string(), "Not Found".to_string());
    translations.insert(
        "client_not_found".to_string(),
        "Client Not Found".to_string(),
    );
    translations.insert(
        "client_already_exists".to_string(),
        "Client Already Exists".to_string(),
    );
    translations.insert(
        "client_registration_failed".to_string(),
        "Client Registration Failed".to_string(),
    );
    translations.insert(
        "client_update_failed".to_string(),
        "Client Update Failed".to_string(),
    );
    translations.insert(
        "client_delete_failed".to_string(),
        "Client Delete Failed".to_string(),
    );
    translations.insert(
        "hardware_not_found".to_string(),
        "Hardware Not Found".to_string(),
    );
    translations.insert(
        "hardware_data_invalid".to_string(),
        "Hardware Data Invalid".to_string(),
    );
    translations.insert(
        "hardware_collection_failed".to_string(),
        "Hardware Collection Failed".to_string(),
    );
    translations.insert(
        "filter_options_error".to_string(),
        "Filter Options Error".to_string(),
    );
    translations.insert(
        "filter_query_invalid".to_string(),
        "Filter Query Invalid".to_string(),
    );
    translations.insert(
        "filter_execution_failed".to_string(),
        "Filter Execution Failed".to_string(),
    );
    translations.insert(
        "database_connection_error".to_string(),
        "Database Connection Error".to_string(),
    );
    translations.insert(
        "database_query_error".to_string(),
        "Database Query Error".to_string(),
    );
    translations.insert(
        "database_transaction_error".to_string(),
        "Database Transaction Error".to_string(),
    );
    translations.insert("network_error".to_string(), "Network Error".to_string());
    translations.insert(
        "connection_timeout".to_string(),
        "Connection Timeout".to_string(),
    );
    translations.insert("request_timeout".to_string(), "Request Timeout".to_string());

    // UI 文本
    translations.insert(
        "search_placeholder".to_string(),
        "Search clients...".to_string(),
    );
    translations.insert("filter_by_os".to_string(), "Filter by OS".to_string());
    translations.insert(
        "filter_by_vendor".to_string(),
        "Filter by Vendor".to_string(),
    );
    translations.insert("filter_by_model".to_string(), "Filter by Model".to_string());
    translations.insert("clear_filters".to_string(), "Clear Filters".to_string());
    translations.insert("apply_filters".to_string(), "Apply Filters".to_string());
    translations.insert("total_clients".to_string(), "Total Clients".to_string());
    translations.insert("online_clients".to_string(), "Online Clients".to_string());
    translations.insert("offline_clients".to_string(), "Offline Clients".to_string());
    translations.insert("loading".to_string(), "Loading...".to_string());

    // Menu
    translations.insert("menu.dashboard".to_string(), "Dashboard".to_string());
    translations.insert("menu.assets".to_string(), "Assets".to_string());
    translations.insert("menu.clients".to_string(), "Clients".to_string());
    translations.insert("menu.racks".to_string(), "Racks".to_string());
    translations.insert("racks.list_view".to_string(), "List View".to_string());
    translations.insert("racks.rack_view".to_string(), "Rack View".to_string());
    translations.insert("racks.grid_layout".to_string(), "Grid Layout".to_string());
    translations.insert(
        "racks.single_column_layout".to_string(),
        "Single Column".to_string(),
    );
    translations.insert("racks.capacity_status".to_string(), "Capacity".to_string());
    translations.insert("racks.power_status".to_string(), "Power".to_string());
    translations.insert("racks.remaining".to_string(), "Remaining".to_string());
    translations.insert(
        "racks.used_no_limit".to_string(),
        "Used: {val} W (No Limit)".to_string(),
    );
    translations.insert(
        "racks.confirm_delete".to_string(),
        "Confirm Delete".to_string(),
    );
    translations.insert(
        "racks.confirm_delete_msg".to_string(),
        "Are you sure you want to delete this rack? This action cannot be undone.".to_string(),
    );
    translations.insert("racks.rack_name".to_string(), "Rack Name".to_string());
    translations.insert("racks.location".to_string(), "Location".to_string());
    translations.insert("racks.height_u".to_string(), "Height (U)".to_string());
    translations.insert(
        "racks.power_limit_w".to_string(),
        "Power Limit (W)".to_string(),
    );
    translations.insert("racks.description".to_string(), "Description".to_string());
    translations.insert("racks.cancel".to_string(), "Cancel".to_string());
    translations.insert("racks.save".to_string(), "Save".to_string());
    translations.insert("racks.edit_rack".to_string(), "Edit Rack".to_string());
    translations.insert("racks.add_rack".to_string(), "Add Rack".to_string());
    translations.insert(
        "racks.rack_capacity".to_string(),
        "Rack Capacity".to_string(),
    );
    translations.insert("racks.used".to_string(), "Used".to_string());
    translations.insert("racks.free".to_string(), "Free".to_string());
    translations.insert("racks.power_usage".to_string(), "Power Usage".to_string());
    translations.insert("racks.total_units".to_string(), "Total Units".to_string());
    translations.insert("racks.power_limit".to_string(), "Power Limit".to_string());
    translations.insert("racks.devices".to_string(), "Devices".to_string());
    translations.insert("racks.status".to_string(), "Status".to_string());
    translations.insert("racks.status.active".to_string(), "Active".to_string());
    translations.insert("racks.status.maint".to_string(), "Maintenance".to_string());
    translations.insert("racks.status.stock".to_string(), "In Stock".to_string());
    translations.insert("racks.status.error".to_string(), "Error".to_string());
    translations.insert(
        "racks.delete_success".to_string(),
        "Deleted successfully".to_string(),
    );
    translations.insert(
        "racks.save_success".to_string(),
        "Saved successfully".to_string(),
    );
    translations.insert(
        "racks.save_failed".to_string(),
        "Save failed: {val}".to_string(),
    );
    translations.insert("racks.actions".to_string(), "Actions".to_string());

    translations.insert("menu.components".to_string(), "Components".to_string());
    translations.insert("menu.organization".to_string(), "Organization".to_string());
    translations.insert("menu.users".to_string(), "Users".to_string());
    translations.insert("menu.projects".to_string(), "Projects".to_string());
    translations.insert("menu.system".to_string(), "System".to_string());
    translations.insert("menu.analytics".to_string(), "Analytics".to_string());
    translations.insert("menu.setup_guide".to_string(), "Setup Guide".to_string());
    translations.insert("menu.base_data".to_string(), "Base Data".to_string());
    translations.insert("menu.accounts".to_string(), "Accounts".to_string());
    translations.insert("menu.source_code".to_string(), "Source Code".to_string());

    // Header
    translations.insert(
        "header.search_placeholder".to_string(),
        "Search hostname, IP...".to_string(),
    );
    translations.insert(
        "header.change_password".to_string(),
        "Change Password".to_string(),
    );
    translations.insert("header.logout".to_string(), "Logout".to_string());
    translations.insert(
        "header.switch_language".to_string(),
        "Switch Language".to_string(),
    );

    // Auth
    translations.insert("auth.login_title".to_string(), "Login to CMDB".to_string());
    translations.insert("auth.username".to_string(), "Username".to_string());
    translations.insert("auth.password".to_string(), "Password".to_string());
    translations.insert("auth.login_button".to_string(), "Sign In".to_string());
    translations.insert("auth.logging_in".to_string(), "Signing in...".to_string());

    // Change Password
    translations.insert(
        "password.change_title".to_string(),
        "Change Password".to_string(),
    );
    translations.insert(
        "password.current".to_string(),
        "Current Password".to_string(),
    );
    translations.insert("password.new".to_string(), "New Password".to_string());
    translations.insert(
        "password.confirm".to_string(),
        "Confirm New Password".to_string(),
    );
    translations.insert("password.submit".to_string(), "Change Password".to_string());
    translations.insert(
        "password.submitting".to_string(),
        "Submitting...".to_string(),
    );
    translations.insert(
        "password.success".to_string(),
        "Password changed successfully. Redirecting to login...".to_string(),
    );
    translations.insert(
        "password.mismatch".to_string(),
        "New passwords do not match".to_string(),
    );
    translations.insert(
        "password.too_short".to_string(),
        "Password must be at least 6 characters".to_string(),
    );

    // Dashboard
    translations.insert(
        "dashboard.loading".to_string(),
        "Loading dashboard data...".to_string(),
    );
    translations.insert(
        "dashboard.total_clients".to_string(),
        "Total Clients".to_string(),
    );
    translations.insert(
        "dashboard.registered_nodes".to_string(),
        "Registered Nodes".to_string(),
    );
    translations.insert(
        "dashboard.online_rate".to_string(),
        "Online Rate".to_string(),
    );
    translations.insert("dashboard.online".to_string(), "Online".to_string());
    translations.insert("dashboard.new_today".to_string(), "New Today".to_string());
    translations.insert(
        "dashboard.24h_registered".to_string(),
        "Registered in 24h".to_string(),
    );
    translations.insert(
        "dashboard.system_types".to_string(),
        "System Types".to_string(),
    );
    translations.insert("dashboard.diverse_os".to_string(), "Diverse OS".to_string());

    // Dashboard Sub-components
    translations.insert(
        "dashboard.os_dist_title".to_string(),
        "OS Distribution".to_string(),
    );
    translations.insert(
        "dashboard.os_dist_desc".to_string(),
        "By Registered Clients".to_string(),
    );
    translations.insert("dashboard.realtime".to_string(), "Realtime".to_string());
    translations.insert("dashboard.no_data".to_string(), "No Data".to_string());
    translations.insert(
        "analytics.unit_machine".to_string(),
        "{count} units".to_string(),
    );
    translations.insert(
        "analytics.gpu_model_distribution".to_string(),
        "GPU Model Distribution".to_string(),
    );
    translations.insert(
        "dashboard.realtime_refresh".to_string(),
        "Realtime Refresh".to_string(),
    );
    translations.insert(
        "dashboard.online_clients".to_string(),
        "Online Clients".to_string(),
    );
    translations.insert(
        "dashboard.offline_clients".to_string(),
        "Offline Clients".to_string(),
    );
    translations.insert(
        "dashboard.realtime_update".to_string(),
        " Realtime Update".to_string(),
    );

    translations.insert(
        "dashboard.recent_active_title".to_string(),
        "Recently Active".to_string(),
    );
    translations.insert(
        "dashboard.recent_active_desc".to_string(),
        "Last 10 Heartbeats".to_string(),
    );
    translations.insert(
        "dashboard.no_clients_registered".to_string(),
        "No Clients Registered".to_string(),
    );
    translations.insert("dashboard.offline".to_string(), "Offline".to_string());

    translations.insert(
        "dashboard.client_status_list".to_string(),
        "Client Status List".to_string(),
    );
    translations.insert("dashboard.total".to_string(), "Total".to_string());
    translations.insert(
        "dashboard.managed_nodes".to_string(),
        "Managed Nodes".to_string(),
    );
    translations.insert("dashboard.view_all".to_string(), "View All".to_string());
    translations.insert("dashboard.host".to_string(), "Host".to_string());
    translations.insert("dashboard.system".to_string(), "System".to_string());
    translations.insert("dashboard.config".to_string(), "Config".to_string());
    translations.insert("dashboard.status".to_string(), "Status".to_string());

    // Clients Page
    translations.insert(
        "clients.stats.total_devices".to_string(),
        "Total Devices".to_string(),
    );
    translations.insert(
        "clients.stats.filtered_results".to_string(),
        "Filtered Results".to_string(),
    );
    translations.insert("clients.stats.os_types".to_string(), "OS Types".to_string());
    translations.insert(
        "clients.stats.vendor_count".to_string(),
        "Vendor Count".to_string(),
    );

    translations.insert(
        "clients.search.title".to_string(),
        "Advanced Search & Filter".to_string(),
    );
    translations.insert(
        "clients.search.keyword_label".to_string(),
        "Keyword Search".to_string(),
    );
    translations.insert(
        "clients.search.placeholder".to_string(),
        "Search hostname, IP, OS, vendor, model or serial...".to_string(),
    );
    translations.insert(
        "clients.search.hint".to_string(),
        "Supports fuzzy search, type to filter instantly".to_string(),
    );
    translations.insert(
        "clients.search.export_csv".to_string(),
        "Export CSV".to_string(),
    );
    translations.insert(
        "clients.search.export_json".to_string(),
        "Export JSON".to_string(),
    );
    translations.insert(
        "clients.search.import".to_string(),
        "Import Data".to_string(),
    );
    translations.insert(
        "clients.search.apply".to_string(),
        "Apply Filters".to_string(),
    );
    translations.insert(
        "clients.search.clear".to_string(),
        "Clear Filters".to_string(),
    );
    translations.insert(
        "clients.filter.active_filters".to_string(),
        "Active Filters".to_string(),
    );

    translations.insert("clients.filter.status".to_string(), "Status".to_string());
    translations.insert(
        "clients.filter.environment".to_string(),
        "Environment".to_string(),
    );
    translations.insert("clients.filter.rack".to_string(), "Rack".to_string());
    translations.insert("clients.filter.project".to_string(), "Project".to_string());
    translations.insert("clients.filter.owner".to_string(), "Owner".to_string());
    translations.insert("clients.filter.os".to_string(), "OS".to_string());
    translations.insert("clients.filter.kernel".to_string(), "Kernel".to_string());
    translations.insert(
        "clients.filter.vendor".to_string(),
        "Server Vendor".to_string(),
    );
    translations.insert(
        "clients.filter.cpu_vendor".to_string(),
        "CPU Vendor".to_string(),
    );
    translations.insert(
        "clients.filter.cpu_model".to_string(),
        "CPU Model".to_string(),
    );
    translations.insert(
        "clients.filter.gpu_vendor".to_string(),
        "GPU Vendor".to_string(),
    );
    translations.insert(
        "clients.filter.gpu_model".to_string(),
        "GPU Model".to_string(),
    );
    translations.insert(
        "clients.filter.memory_min".to_string(),
        "Min Memory (GB)".to_string(),
    );
    translations.insert(
        "clients.filter.memory_max".to_string(),
        "Max Memory (GB)".to_string(),
    );
    translations.insert(
        "clients.filter.network_type".to_string(),
        "Network Type".to_string(),
    );
    translations.insert(
        "clients.filter.network_model".to_string(),
        "Network Model".to_string(),
    );
    translations.insert(
        "clients.filter.storage_type".to_string(),
        "Storage Type".to_string(),
    );

    translations.insert(
        "clients.table.no_data".to_string(),
        "No device data available".to_string(),
    );
    translations.insert("clients.table.hostname".to_string(), "Hostname".to_string());
    translations.insert("clients.table.ip".to_string(), "IP Address".to_string());
    translations.insert("clients.table.os".to_string(), "OS".to_string());
    translations.insert("clients.table.owner".to_string(), "Owner".to_string());
    translations.insert("clients.table.project".to_string(), "Project".to_string());
    translations.insert("clients.table.status".to_string(), "Status".to_string());
    translations.insert(
        "clients.table.environment".to_string(),
        "Environment".to_string(),
    );
    translations.insert("clients.table.actions".to_string(), "Actions".to_string());

    translations.insert("clients.status.active".to_string(), "Active".to_string());
    translations.insert(
        "clients.status.maintenance".to_string(),
        "Maintenance".to_string(),
    );
    translations.insert("clients.status.instock".to_string(), "In Stock".to_string());
    translations.insert(
        "clients.status.decommissioned".to_string(),
        "Decommissioned".to_string(),
    );

    translations.insert("clients.env.prod".to_string(), "Prod".to_string());
    translations.insert("clients.env.staging".to_string(), "Staging".to_string());
    translations.insert("clients.env.test".to_string(), "Test".to_string());
    translations.insert("clients.env.dev".to_string(), "Dev".to_string());

    translations.insert(
        "clients.actions.view".to_string(),
        "View Details".to_string(),
    );
    translations.insert("clients.actions.edit".to_string(), "Edit".to_string());
    translations.insert("clients.actions.delete".to_string(), "Delete".to_string());
    translations.insert(
        "clients.actions.confirm_delete".to_string(),
        "Are you sure you want to delete this device? This action cannot be undone.".to_string(),
    );

    translations.insert(
        "clients.import.success".to_string(),
        "Import Successful!".to_string(),
    );
    translations.insert(
        "clients.import.error_title".to_string(),
        "Import Failed".to_string(),
    );
    translations.insert(
        "clients.import.error_desc".to_string(),
        "Errors found, import cancelled. Please fix and retry.".to_string(),
    );
    translations.insert(
        "clients.import.progress".to_string(),
        "Importing...".to_string(),
    );
    translations.insert(
        "clients.selection.selected".to_string(),
        "Selected".to_string(),
    );
    translations.insert(
        "clients.selection.export_template".to_string(),
        "Export Template".to_string(),
    );

    translations.insert("common.close".to_string(), "Close".to_string());
    translations.insert("common.error_prefix".to_string(), "Error: ".to_string());

    translations.insert("error".to_string(), "Error".to_string());
    translations.insert("success".to_string(), "Success".to_string());
    translations.insert("warning".to_string(), "Warning".to_string());
    translations.insert("info".to_string(), "Info".to_string());

    // Notification
    translations.insert("notification.success".to_string(), "Success".to_string());
    translations.insert("notification.info".to_string(), "Information".to_string());
    translations.insert("notification.warning".to_string(), "Warning".to_string());
    translations.insert("notification.error".to_string(), "Error".to_string());

    // 表格列标题
    translations.insert("hostname".to_string(), "Hostname".to_string());
    translations.insert("ip_address".to_string(), "IP Address".to_string());
    translations.insert("os".to_string(), "Operating System".to_string());
    translations.insert("vendor".to_string(), "Vendor".to_string());
    translations.insert("model".to_string(), "Model".to_string());
    translations.insert("last_seen".to_string(), "Last Seen".to_string());
    translations.insert("status".to_string(), "Status".to_string());
    translations.insert("actions".to_string(), "Actions".to_string());

    // 分页
    translations.insert("previous_page".to_string(), "Previous".to_string());
    translations.insert("next_page".to_string(), "Next".to_string());
    translations.insert("page".to_string(), "Page".to_string());
    translations.insert("of".to_string(), "of".to_string());
    translations.insert("items_per_page".to_string(), "items per page".to_string());

    // Common
    translations.insert("common.cancel".to_string(), "Cancel".to_string());
    translations.insert("common.save".to_string(), "Save".to_string());
    translations.insert(
        "common.delete_success".to_string(),
        "Delete Successful".to_string(),
    );
    translations.insert(
        "common.save_success".to_string(),
        "Save Successful".to_string(),
    );
    translations.insert(
        "common.save_failed".to_string(),
        "Save Failed: {}".to_string(),
    );
    translations.insert("common.actions".to_string(), "Actions".to_string());
    translations.insert(
        "common.confirm_delete".to_string(),
        "Confirm Delete".to_string(),
    );
    translations.insert("common.delete".to_string(), "Delete".to_string());
    translations.insert("common.export".to_string(), "Export".to_string());
    translations.insert("common.reset".to_string(), "Reset".to_string());

    // Projects
    translations.insert(
        "projects.select_cost_center".to_string(),
        "Select Cost Center".to_string(),
    );
    translations.insert(
        "projects.select_manager".to_string(),
        "Select Manager".to_string(),
    );
    translations.insert(
        "projects.edit_project".to_string(),
        "Edit Project".to_string(),
    );
    translations.insert(
        "projects.add_project".to_string(),
        "Add Project".to_string(),
    );
    translations.insert("projects.name".to_string(), "Project Name".to_string());
    translations.insert("projects.code".to_string(), "Project Code".to_string());
    translations.insert("projects.department".to_string(), "Department".to_string());
    translations.insert(
        "projects.cost_center".to_string(),
        "Cost Center".to_string(),
    );
    translations.insert("projects.manager".to_string(), "Manager".to_string());
    translations.insert(
        "projects.new_project".to_string(),
        "New Project".to_string(),
    );
    translations.insert(
        "projects.confirm_delete_msg".to_string(),
        "Are you sure you want to delete this project? This action cannot be undone.".to_string(),
    );

    // Persons
    translations.insert(
        "persons.select_department".to_string(),
        "Select Department".to_string(),
    );
    translations.insert(
        "persons.select_title".to_string(),
        "Select Title".to_string(),
    );
    translations.insert("persons.edit_person".to_string(), "Edit Person".to_string());
    translations.insert("persons.add_person".to_string(), "Add Person".to_string());
    translations.insert("persons.name".to_string(), "Name".to_string());
    translations.insert("persons.email".to_string(), "Email".to_string());
    translations.insert("persons.department".to_string(), "Department".to_string());
    translations.insert("persons.phone".to_string(), "Phone".to_string());
    translations.insert("persons.title".to_string(), "Title".to_string());
    translations.insert(
        "persons.delete_success".to_string(),
        "Delete Successful".to_string(),
    );
    translations.insert(
        "persons.save_success".to_string(),
        "Save Successful".to_string(),
    );
    translations.insert(
        "persons.save_failed".to_string(),
        "Save Failed: {}".to_string(),
    );
    translations.insert("persons.new_person".to_string(), "New Person".to_string());
    translations.insert("persons.actions".to_string(), "Actions".to_string());
    translations.insert(
        "persons.confirm_delete".to_string(),
        "Confirm Delete".to_string(),
    );
    translations.insert(
        "persons.confirm_delete_msg".to_string(),
        "Are you sure you want to delete this person? This action cannot be undone.".to_string(),
    );

    // Components
    translations.insert("components.type_other".to_string(), "Other".to_string());
    translations.insert("components.type_gpu".to_string(), "GPU".to_string());
    translations.insert("components.type_cpu".to_string(), "CPU".to_string());
    translations.insert("components.type_memory".to_string(), "Memory".to_string());
    translations.insert("components.type_disk".to_string(), "Disk".to_string());
    translations.insert(
        "components.type_network_card".to_string(),
        "Network Card".to_string(),
    );
    translations.insert(
        "components.type_motherboard".to_string(),
        "Motherboard".to_string(),
    );
    translations.insert(
        "components.type_power_supply".to_string(),
        "Power Supply".to_string(),
    );
    translations.insert(
        "components.status_in_stock".to_string(),
        "In Stock".to_string(),
    );
    translations.insert("components.status_in_use".to_string(), "In Use".to_string());
    translations.insert(
        "components.status_lent_out".to_string(),
        "Lent Out".to_string(),
    );
    translations.insert("components.status_faulty".to_string(), "Faulty".to_string());
    translations.insert(
        "components.status_decommissioned".to_string(),
        "Decommissioned".to_string(),
    );
    translations.insert(
        "components.status_unknown".to_string(),
        "Unknown".to_string(),
    );
    translations.insert(
        "components.new_component".to_string(),
        "New Component".to_string(),
    );
    translations.insert(
        "components.edit_component".to_string(),
        "Edit Component: {model} ({sn})".to_string(),
    );
    translations.insert(
        "components.serial_number".to_string(),
        "Serial Number (SN)".to_string(),
    );
    translations.insert("components.model".to_string(), "Model".to_string());
    translations.insert("components.type".to_string(), "Type".to_string());
    translations.insert("components.vendor".to_string(), "Vendor".to_string());
    translations.insert("components.status".to_string(), "Status".to_string());
    translations.insert("components.location".to_string(), "Location".to_string());
    translations.insert(
        "components.purchase_date".to_string(),
        "Purchase Date".to_string(),
    );
    translations.insert(
        "components.warranty_expiration".to_string(),
        "Warranty Expiration".to_string(),
    );
    translations.insert(
        "components.batch_create_json".to_string(),
        "Batch Create (JSON)".to_string(),
    );
    translations.insert(
        "components.json_parse_error".to_string(),
        "JSON Parse Error: {error}".to_string(),
    );
    translations.insert(
        "components.json_input_hint".to_string(),
        "Please enter a JSON array containing component information. Example:".to_string(),
    );
    translations.insert(
        "components.batch_create".to_string(),
        "Batch Create".to_string(),
    );
    translations.insert(
        "components.select_components_first".to_string(),
        "Please select components first".to_string(),
    );
    translations.insert(
        "components.confirm_batch_status_update".to_string(),
        "Are you sure you want to update the status of {count} selected components to {status}?"
            .to_string(),
    );
    translations.insert(
        "components.batch_status_update_success".to_string(),
        "Batch status update successful".to_string(),
    );
    translations.insert(
        "components.batch_status_update_failed".to_string(),
        "Batch status update failed: {error}".to_string(),
    );
    translations.insert(
        "components.batch_edit_export".to_string(),
        "Batch Edit (Export)".to_string(),
    );
    translations.insert(
        "components.json_import".to_string(),
        "JSON Import".to_string(),
    );
    translations.insert(
        "components.excel_import".to_string(),
        "Excel Import".to_string(),
    );
    translations.insert(
        "components.quick_status_change".to_string(),
        "Quick Status Change...".to_string(),
    );
    translations.insert("components.search".to_string(), "Search".to_string());
    translations.insert("components.search_label".to_string(), "Search".to_string());
    translations.insert(
        "components.search_placeholder".to_string(),
        "Search SN or Model...".to_string(),
    );
    translations.insert(
        "components.component_info".to_string(),
        "Component Info".to_string(),
    );
    translations.insert(
        "components.type_vendor".to_string(),
        "Type/Vendor".to_string(),
    );
    translations.insert(
        "components.location_owner".to_string(),
        "Location/Owner".to_string(),
    );
    translations.insert("components.actions".to_string(), "Actions".to_string());
    translations.insert(
        "components.server_prefix".to_string(),
        "Server: ".to_string(),
    );
    translations.insert(
        "components.importing".to_string(),
        "Importing...".to_string(),
    );
    translations.insert(
        "components.batch_update_complete".to_string(),
        "Batch update complete: {success}/{total} successful".to_string(),
    );
    translations.insert(
        "components.batch_create_success".to_string(),
        "Successfully created {count} components".to_string(),
    );
    translations.insert(
        "components.batch_create_failed".to_string(),
        "Batch create failed: {error}".to_string(),
    );

    // Analytics
    translations.insert(
        "analytics.total_devices".to_string(),
        "Total Devices".to_string(),
    );
    translations.insert(
        "analytics.online_devices".to_string(),
        "Online Devices".to_string(),
    );
    translations.insert(
        "analytics.offline_devices".to_string(),
        "Offline Devices".to_string(),
    );
    translations.insert(
        "analytics.online_rate".to_string(),
        "Online Rate".to_string(),
    );
    translations.insert("analytics.retry".to_string(), "Retry".to_string());
    translations.insert(
        "analytics.gpu_vendor_distribution".to_string(),
        "GPU Vendor Distribution".to_string(),
    );
    translations.insert(
        "analytics.by_machine_count".to_string(),
        "By Machine Count".to_string(),
    );
    translations.insert(
        "analytics.detailed_stats".to_string(),
        "Detailed Stats".to_string(),
    );
    translations.insert("analytics.gpu_vendor".to_string(), "GPU Vendor".to_string());
    translations.insert("analytics.count".to_string(), "Count".to_string());
    translations.insert("analytics.percentage".to_string(), "Percentage".to_string());
    translations.insert("analytics.unit_machine".to_string(), "{} units".to_string());
    translations.insert(
        "analytics.gpu_model_distribution".to_string(),
        "GPU Model Distribution".to_string(),
    );
    translations.insert("analytics.gpu_model".to_string(), "GPU Model".to_string());
    translations.insert(
        "analytics.gpu_detailed_config".to_string(),
        "GPU Detailed Config".to_string(),
    );
    translations.insert(
        "analytics.by_model_and_count".to_string(),
        "By Model and Count".to_string(),
    );
    translations.insert("analytics.gpu_config".to_string(), "GPU Config".to_string());
    translations.insert(
        "analytics.cpu_model_distribution".to_string(),
        "CPU Model Distribution".to_string(),
    );
    translations.insert("analytics.cpu_model".to_string(), "CPU Model".to_string());
    translations.insert(
        "analytics.storage_type_distribution".to_string(),
        "Storage Type Distribution".to_string(),
    );
    translations.insert(
        "analytics.storage_type".to_string(),
        "Storage Type".to_string(),
    );
    translations.insert(
        "analytics.os_distribution".to_string(),
        "OS Distribution".to_string(),
    );
    translations.insert("analytics.os".to_string(), "OS".to_string());
    translations.insert(
        "analytics.memory_size_distribution".to_string(),
        "Memory Size Distribution".to_string(),
    );
    translations.insert(
        "analytics.memory_size".to_string(),
        "Memory Size".to_string(),
    );
    translations.insert(
        "analytics.network_type_distribution".to_string(),
        "Network Type Distribution".to_string(),
    );
    translations.insert(
        "analytics.network_type".to_string(),
        "Network Type".to_string(),
    );
    translations.insert(
        "analytics.server_model_distribution".to_string(),
        "Server Model Distribution".to_string(),
    );
    translations.insert(
        "analytics.server_model".to_string(),
        "Server Model".to_string(),
    );

    // Stats Filter
    translations.insert("stats.filter.cpu_vendor".to_string(), "CPU Vendor".to_string());
    translations.insert("stats.filter.gpu_vendor".to_string(), "GPU Vendor".to_string());
    translations.insert("stats.filter.memory_capacity".to_string(), "Memory Capacity (GB)".to_string());
    translations.insert("stats.filter.os".to_string(), "Operating System".to_string());

    // Client Setup
    translations.insert(
        "client_setup.parse_error".to_string(),
        "Failed to parse response: {error}".to_string(),
    );
    translations.insert(
        "client_setup.request_failed".to_string(),
        "Request failed: {status}".to_string(),
    );
    translations.insert(
        "client_setup.network_error".to_string(),
        "Network request failed: {error}".to_string(),
    );
    translations.insert(
        "client_setup.load_failed".to_string(),
        "Failed to load client info".to_string(),
    );
    translations.insert(
        "client_setup.guide_title".to_string(),
        "Client Installation Guide".to_string(),
    );
    translations.insert(
        "client_setup.guide_subtitle".to_string(),
        "Select platform and architecture for installation instructions".to_string(),
    );
    translations.insert(
        "client_setup.select_platform".to_string(),
        "Select Platform".to_string(),
    );
    translations.insert(
        "client_setup.select_arch".to_string(),
        "Select Architecture".to_string(),
    );
    translations.insert(
        "client_setup.step1_download".to_string(),
        "1. Download Client".to_string(),
    );
    translations.insert(
        "client_setup.download_url".to_string(),
        "Download URL: ".to_string(),
    );
    translations.insert(
        "client_setup.server_url".to_string(),
        "Server URL: ".to_string(),
    );
    translations.insert(
        "client_setup.step2_quick_install".to_string(),
        "2. Quick Install (Recommended)".to_string(),
    );
    translations.insert(
        "client_setup.copy_command".to_string(),
        "Copy and run the following command in terminal:".to_string(),
    );
    translations.insert(
        "client_setup.quick_install_desc".to_string(),
        "This command will automatically download, install and start the client service"
            .to_string(),
    );
    translations.insert(
        "client_setup.step3_manual_install".to_string(),
        "3. Manual Install".to_string(),
    );
    translations.insert(
        "client_setup.step2_install_script".to_string(),
        "2. Install Script".to_string(),
    );
    translations.insert(
        "client_setup.save_script".to_string(),
        "Save the following script to a file and run it:".to_string(),
    );
    translations.insert(
        "client_setup.step4_config".to_string(),
        "4. Configuration".to_string(),
    );
    translations.insert(
        "client_setup.step3_config".to_string(),
        "3. Configuration".to_string(),
    );
    translations.insert(
        "client_setup.config_template_desc".to_string(),
        "Config template (Default path: /etc/rs-cmdb/client.toml):".to_string(),
    );
    translations.insert(
        "client_setup.step5_systemd".to_string(),
        "5. Systemd Service".to_string(),
    );
    translations.insert(
        "client_setup.systemd_desc".to_string(),
        "Systemd service file (/etc/systemd/system/rs-cmdb-client.service):".to_string(),
    );
    translations.insert(
        "client_setup.step6_verify".to_string(),
        "6. Verify Installation".to_string(),
    );
    translations.insert(
        "client_setup.step4_verify".to_string(),
        "4. Verify Installation".to_string(),
    );
    translations.insert(
        "client_setup.check_status".to_string(),
        "Check Service Status".to_string(),
    );
    translations.insert(
        "client_setup.manual_run_check".to_string(),
        "Run client manually and check connection to server".to_string(),
    );
    translations.insert(
        "client_setup.check_logs".to_string(),
        "Check Logs".to_string(),
    );
    translations.insert(
        "client_setup.check_logs_dir".to_string(),
        "Check application log directory".to_string(),
    );
    translations.insert(
        "client_setup.install_complete_prefix".to_string(),
        "After installation, the client will automatically appear in ".to_string(),
    );
    translations.insert(
        "client_setup.client_list".to_string(),
        "Client List".to_string(),
    );
    translations.insert(
        "client_setup.install_complete_suffix".to_string(),
        ".".to_string(),
    );

    // Dictionaries
    translations.insert(
        "dictionaries.department".to_string(),
        "Department".to_string(),
    );
    translations.insert("dictionaries.title".to_string(), "Title".to_string());
    translations.insert(
        "dictionaries.cost_center".to_string(),
        "Cost Center".to_string(),
    );
    translations.insert(
        "dictionaries.dictionary_item".to_string(),
        "Dictionary Item".to_string(),
    );
    translations.insert(
        "dictionaries.create_prefix".to_string(),
        "Create ".to_string(),
    );
    translations.insert("dictionaries.edit_prefix".to_string(), "Edit ".to_string());
    translations.insert("dictionaries.key_label".to_string(), "Key:".to_string());
    translations.insert(
        "dictionaries.key_desc".to_string(),
        " Unique identifier used internally, usually in English or code (e.g., 'HR', 'DEV_01')."
            .to_string(),
    );
    translations.insert("dictionaries.value_label".to_string(), "Value:".to_string());
    translations.insert(
        "dictionaries.value_desc".to_string(),
        " Name displayed to users (e.g., 'Human Resources', 'Dev Team 1').".to_string(),
    );
    translations.insert("dictionaries.key".to_string(), "Key".to_string());
    translations.insert(
        "dictionaries.key_placeholder".to_string(),
        "e.g., HR".to_string(),
    );
    translations.insert("dictionaries.value".to_string(), "Value".to_string());
    translations.insert(
        "dictionaries.value_placeholder".to_string(),
        "e.g., Human Resources".to_string(),
    );
    translations.insert(
        "dictionaries.description".to_string(),
        "Description".to_string(),
    );
    translations.insert(
        "dictionaries.description_placeholder".to_string(),
        "Optional description".to_string(),
    );
    translations.insert("dictionaries.cancel".to_string(), "Cancel".to_string());
    translations.insert("dictionaries.save".to_string(), "Save".to_string());
    translations.insert(
        "dictionaries.delete_success".to_string(),
        "Deleted successfully".to_string(),
    );
    translations.insert(
        "dictionaries.save_success".to_string(),
        "Saved successfully".to_string(),
    );
    translations.insert(
        "dictionaries.save_failed".to_string(),
        "Save failed: {error}".to_string(),
    );
    translations.insert(
        "dictionaries.create_department".to_string(),
        "Create Department".to_string(),
    );
    translations.insert(
        "dictionaries.create_title".to_string(),
        "Create Title".to_string(),
    );
    translations.insert(
        "dictionaries.create_cost_center".to_string(),
        "Create Cost Center".to_string(),
    );
    translations.insert("dictionaries.create".to_string(), "Create".to_string());
    translations.insert("dictionaries.create_item".to_string(), "Create Item".to_string());
    translations.insert("dictionaries.actions".to_string(), "Actions".to_string());
    translations.insert(
        "dictionaries.confirm_delete_title".to_string(),
        "Confirm Delete".to_string(),
    );
    translations.insert(
        "dictionaries.confirm_delete_message".to_string(),
        "Are you sure you want to delete this item? This action cannot be undone.".to_string(),
    );

    // Users
    translations.insert(
        "users.update_success".to_string(),
        "User updated successfully".to_string(),
    );
    translations.insert(
        "users.create_success".to_string(),
        "User created successfully".to_string(),
    );
    translations.insert(
        "users.delete_confirm".to_string(),
        "Are you sure you want to delete this user?".to_string(),
    );
    translations.insert(
        "users.delete_success".to_string(),
        "User deleted successfully".to_string(),
    );
    translations.insert("users.create_user".to_string(), "Create User".to_string());
    translations.insert("users.username".to_string(), "Username".to_string());
    translations.insert("users.role".to_string(), "Role".to_string());
    translations.insert("users.status".to_string(), "Status".to_string());
    translations.insert("users.last_login".to_string(), "Last Login".to_string());
    translations.insert("users.actions".to_string(), "Actions".to_string());
    translations.insert("users.active".to_string(), "Active".to_string());
    translations.insert("users.inactive".to_string(), "Inactive".to_string());
    translations.insert("users.edit_user".to_string(), "Edit User".to_string());
    translations.insert(
        "users.password_placeholder_edit".to_string(),
        "Password (leave blank to keep unchanged)".to_string(),
    );
    translations.insert("users.password".to_string(), "Password".to_string());
    translations.insert(
        "users.username_placeholder".to_string(),
        "Enter username".to_string(),
    );
    translations.insert(
        "users.password_placeholder".to_string(),
        "Enter password".to_string(),
    );
    translations.insert(
        "users.enable_account".to_string(),
        "Enable Account".to_string(),
    );
    translations.insert("users.cancel".to_string(), "Cancel".to_string());
    translations.insert("users.save".to_string(), "Save".to_string());
    translations.insert("users.role_viewer".to_string(), "Viewer".to_string());
    translations.insert("users.role_user".to_string(), "User".to_string());
    translations.insert("users.role_admin".to_string(), "Admin".to_string());

    translations.insert(
        "analytics.unit_machine".to_string(),
        "{count} units".to_string(),
    );

    // Dashboard
    translations.insert("dashboard.unit_machines".to_string(), " units".to_string());
    translations.insert(
        "dashboard.system_status_title".to_string(),
        "System Status Overview".to_string(),
    );

    // Pagination
    translations.insert(
        "pagination.total_items".to_string(),
        "Total {count} items".to_string(),
    );
    translations.insert(
        "pagination.items_per_page".to_string(),
        "items/page".to_string(),
    );
    translations.insert("pagination.per_page".to_string(), "Per page".to_string());
    translations.insert("pagination.unit".to_string(), "items".to_string());
    translations.insert("pagination.jump_to".to_string(), "Go to".to_string());
    translations.insert("pagination.go".to_string(), "GO".to_string());

    // Client Detail
    translations.insert(
        "client_detail.title".to_string(),
        "Client Details".to_string(),
    );
    translations.insert(
        "client_detail.basic_info".to_string(),
        "Basic Information".to_string(),
    );
    translations.insert("client_detail.edit".to_string(), "Edit".to_string());
    translations.insert("client_detail.refresh".to_string(), "Refresh".to_string());
    translations.insert("client_detail.id".to_string(), "ID".to_string());
    translations.insert("client_detail.hostname".to_string(), "Hostname".to_string());
    translations.insert("client_detail.ip".to_string(), "IP Address".to_string());
    translations.insert("client_detail.os".to_string(), "OS".to_string());
    translations.insert("client_detail.kernel".to_string(), "Kernel".to_string());
    translations.insert("client_detail.location".to_string(), "Location".to_string());
    translations.insert("client_detail.rack".to_string(), "Rack".to_string());
    translations.insert(
        "client_detail.unit_position".to_string(),
        "Unit Position".to_string(),
    );
    translations.insert(
        "client_detail.u_height".to_string(),
        "Height (U)".to_string(),
    );
    translations.insert("client_detail.power".to_string(), "Power (W)".to_string());
    translations.insert("client_detail.owner".to_string(), "Owner".to_string());
    translations.insert("client_detail.project".to_string(), "Project".to_string());
    translations.insert(
        "client_detail.serial".to_string(),
        "Serial Number".to_string(),
    );
    translations.insert(
        "client_detail.asset_tag".to_string(),
        "Asset Tag".to_string(),
    );
    translations.insert(
        "client_detail.warranty".to_string(),
        "Warranty Expiration".to_string(),
    );
    translations.insert("client_detail.supplier".to_string(), "Supplier".to_string());
    translations.insert(
        "client_detail.registered".to_string(),
        "Registered At".to_string(),
    );
    translations.insert(
        "client_detail.last_seen".to_string(),
        "Last Seen".to_string(),
    );
    translations.insert("client_detail.status".to_string(), "Status".to_string());
    translations.insert(
        "client_detail.environment".to_string(),
        "Environment".to_string(),
    );
    translations.insert("client_detail.comment".to_string(), "Comment".to_string());
    translations.insert(
        "client_detail.tab_overview".to_string(),
        "Overview".to_string(),
    );
    translations.insert(
        "client_detail.tab_hardware".to_string(),
        "Hardware".to_string(),
    );
    translations.insert(
        "client_detail.tab_history".to_string(),
        "History".to_string(),
    );
    translations.insert(
        "client_detail.loading_overview".to_string(),
        "Loading overview...".to_string(),
    );
    translations.insert(
        "client_detail.loading_hardware".to_string(),
        "Loading hardware...".to_string(),
    );
    translations.insert(
        "client_detail.loading_client".to_string(),
        "Loading client info...".to_string(),
    );
    translations.insert(
        "client_detail.no_hardware".to_string(),
        "No hardware info".to_string(),
    );
    translations.insert(
        "client_detail.update_success".to_string(),
        "Update successful".to_string(),
    );
    translations.insert(
        "client_detail.update_failed".to_string(),
        "Update failed: {}".to_string(),
    );

    // Client Status
    translations.insert("client_status.active".to_string(), "Active".to_string());
    translations.insert(
        "client_status.maintenance".to_string(),
        "Maintenance".to_string(),
    );
    translations.insert("client_status.in_stock".to_string(), "In Stock".to_string());
    translations.insert(
        "client_status.decommissioned".to_string(),
        "Decommissioned".to_string(),
    );

    // Environment
    translations.insert("environment.prod".to_string(), "Production".to_string());
    translations.insert("environment.staging".to_string(), "Staging".to_string());
    translations.insert("environment.test".to_string(), "Test".to_string());
    translations.insert("environment.dev".to_string(), "Development".to_string());

    translations.insert(
        "client_edit.title".to_string(),
        "Edit Client Info".to_string(),
    );
    translations.insert(
        "client_edit.description".to_string(),
        "Modify client details including location, associations, status, etc.".to_string(),
    );
    translations.insert(
        "client_edit.unassigned".to_string(),
        "Unassigned".to_string(),
    );
    translations.insert("client_edit.save".to_string(), "Save".to_string());
    translations.insert("client_edit.cancel".to_string(), "Cancel".to_string());

    translations
}
