use crate::types::Client;
use std::collections::HashMap;

// 统计相关工具函数
pub fn calculate_os_counts(clients: &[Client]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for client in clients {
        if let Some(os) = &client.os {
            *counts.entry(os.clone()).or_insert(0) += 1;
        }
    }
    counts
}

pub fn calculate_vendor_counts(clients: &[Client]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for client in clients {
        if let Some(vendor) = &client.sys_vendor {
            *counts.entry(vendor.clone()).or_insert(0) += 1;
        }
    }
    counts
}

// API 参数构建工具
pub fn build_api_filter_params(
    search_term: &str,
    filters: &super::state::FilterState,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<u32>,
    Option<u32>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    (
        if search_term.is_empty() { None } else { Some(search_term.to_string()) },
        if filters.os.is_empty() { None } else { Some(filters.os.clone()) },
        if filters.os_kernel.is_empty() { None } else { Some(filters.os_kernel.clone()) },
        if filters.cpu_vendor.is_empty() { None } else { Some(filters.cpu_vendor.clone()) },
        if filters.cpu_model.is_empty() { None } else { Some(filters.cpu_model.clone()) },
        if filters.gpu_vendor.is_empty() { None } else { Some(filters.gpu_vendor.clone()) },
        if filters.gpu_model.is_empty() { None } else { Some(filters.gpu_model.clone()) },
        if filters.memory_min.is_empty() { None } else { filters.memory_min.parse().ok() },
        if filters.memory_max.is_empty() { None } else { filters.memory_max.parse().ok() },
        if filters.server_vendor.is_empty() { None } else { Some(filters.server_vendor.clone()) },
        if filters.network_type.is_empty() { None } else { Some(filters.network_type.clone()) },
        if filters.network_model.is_empty() { None } else { Some(filters.network_model.clone()) },
        if filters.storage_type.is_empty() { None } else { Some(filters.storage_type.clone()) },
        if filters.status.is_empty() { None } else { Some(filters.status.clone()) },
        if filters.client_status.is_empty() { None } else { Some(filters.client_status.clone()) },
        if filters.environment.is_empty() { None } else { Some(filters.environment.clone()) },
        if filters.rack_id.is_empty() { None } else { Some(filters.rack_id.clone()) },
        if filters.project_id.is_empty() { None } else { Some(filters.project_id.clone()) },
        if filters.owner_id.is_empty() { None } else { Some(filters.owner_id.clone()) },
    )
}

// 分页计算工具
#[allow(dead_code)]
pub fn calculate_pagination(
    total_items: usize,
    current_page: usize,
    page_size: usize,
) -> (usize, usize, usize) {
    let total_pages = (total_items + page_size - 1) / page_size;
    let start_index = (current_page - 1) * page_size;
    let end_index = std::cmp::min(start_index + page_size, total_items);
    (total_pages, start_index, end_index)
} 