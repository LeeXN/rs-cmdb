use super::state::FilterState;
use crate::types::Client;

#[allow(dead_code)]
pub fn filter_clients_local(
    all_clients: &[Client],
    search_term: &str,
    filters: &FilterState,
) -> Vec<Client> {
    all_clients
        .iter()
        .filter(|client| {
            // 搜索词筛选
            if !search_term.is_empty() {
                let search_lower = search_term.to_lowercase();
                let matches_search = client.hostname.to_lowercase().contains(&search_lower)
                    || client.ip_address.contains(search_term)
                    || client
                        .os
                        .as_ref()
                        .is_some_and(|os| os.to_lowercase().contains(&search_lower))
                    || client
                        .kernel_version
                        .as_ref()
                        .is_some_and(|kernel| kernel.to_lowercase().contains(&search_lower))
                    || client
                        .sys_vendor
                        .as_ref()
                        .is_some_and(|vendor| vendor.to_lowercase().contains(&search_lower))
                    || client
                        .product_name
                        .as_ref()
                        .is_some_and(|product| product.to_lowercase().contains(&search_lower))
                    || client
                        .serial_number
                        .as_ref()
                        .is_some_and(|serial| serial.to_lowercase().contains(&search_lower));

                if !matches_search {
                    return false;
                }
            }

            // 操作系统筛选
            if !filters.os.is_empty() {
                if let Some(os) = &client.os {
                    if !os.contains(&filters.os) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // 内核版本筛选
            if !filters.os_kernel.is_empty() {
                if let Some(kernel) = &client.kernel_version {
                    if !kernel.contains(&filters.os_kernel) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // 服务器厂商筛选
            if !filters.server_vendor.is_empty() {
                if let Some(vendor) = &client.sys_vendor {
                    if !vendor.contains(&filters.server_vendor) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // 状态筛选 (在线/离线)
            if !filters.status.is_empty() {
                let is_online = client
                    .last_seen
                    .as_ref()
                    .and_then(|last_seen| chrono::DateTime::parse_from_rfc3339(last_seen).ok())
                    .map(|dt| {
                        let now = chrono::Utc::now();
                        let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                        duration.num_minutes() <= 5
                    })
                    .unwrap_or(false);

                if filters.status == "online" && !is_online {
                    return false;
                }
                if filters.status == "offline" && is_online {
                    return false;
                }
            }

            // 设备状态筛选 (Active, Maintenance, etc.)
            if !filters.client_status.is_empty() {
                let status_str = client
                    .status
                    .as_ref()
                    .map(|s| format!("{:?}", s))
                    .unwrap_or_default();
                if status_str != filters.client_status {
                    return false;
                }
            }

            // 环境筛选
            if !filters.environment.is_empty() {
                let env_str = client
                    .environment
                    .as_ref()
                    .map(|e| format!("{:?}", e))
                    .unwrap_or_default();
                if env_str != filters.environment {
                    return false;
                }
            }

            // 机柜筛选
            if !filters.rack_id.is_empty() {
                if let Some(rack) = &client.rack {
                    if rack != &filters.rack_id {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // 项目筛选
            if !filters.project_id.is_empty() {
                if let Some(project) = &client.project_id {
                    if project != &filters.project_id {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // 负责人筛选
            if !filters.owner_id.is_empty() {
                if let Some(owner) = &client.owner_id {
                    if owner != &filters.owner_id {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // 对于硬件筛选，如果有设置就表示需要API筛选
            // 这里只做基础验证，实际筛选由API完成

            true
        })
        .cloned()
        .collect()
}

pub fn has_active_filters(search_term: &str, filters: &FilterState) -> bool {
    !search_term.is_empty() || !filters.is_empty()
}
