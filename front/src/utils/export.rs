use crate::types::ClientHardwareExport;
use wasm_bindgen::prelude::*;
use web_sys::{window, Blob, BlobPropertyBag, Url, HtmlAnchorElement, HtmlElement};

/// 将数据导出为CSV格式并下载
pub fn export_to_csv(data: &[ClientHardwareExport], filename: &str) -> Result<(), JsValue> {
    // CSV头部
    let headers = vec![
        "客户端ID", "主机名", "IP地址", "操作系统", "系统厂商", "产品型号", "序列号", "最后在线", "注册时间",
        "CPU厂商", "CPU型号", "CPU核心数", "CPU线程数", "CPU频率",
        "内存总量", "内存厂商", "内存速度", "内存模块数",
        "GPU数量", "GPU型号", "GPU厂商",
        "存储设备数", "存储总量", "存储类型",
        "网卡数量", "网络类型", "网络速度"
    ];
    
    // 构建CSV内容
    let mut csv_content = String::new();
    
    // 添加BOM以支持中文
    csv_content.push_str("\u{FEFF}");
    
    // 添加头部
    csv_content.push_str(&headers.join(","));
    csv_content.push('\n');
    
    // 添加数据行
    for item in data {
        let row = vec![
            escape_csv_field(&item.client_id),
            escape_csv_field(&item.hostname),
            escape_csv_field(&item.ip_address),
            escape_csv_field(&item.os),
            escape_csv_field(&item.sys_vendor),
            escape_csv_field(&item.product_name),
            escape_csv_field(&item.serial_number),
            escape_csv_field(&item.last_seen),
            escape_csv_field(&item.registered_at),
            escape_csv_field(&item.cpu_vendor),
            escape_csv_field(&item.cpu_model),
            item.cpu_cores.to_string(),
            item.cpu_threads.to_string(),
            escape_csv_field(&item.cpu_frequency),
            escape_csv_field(&item.memory_total),
            escape_csv_field(&item.memory_vendor),
            escape_csv_field(&item.memory_speed),
            item.memory_modules.to_string(),
            item.gpu_count.to_string(),
            escape_csv_field(&item.gpu_models),
            escape_csv_field(&item.gpu_vendors),
            item.storage_count.to_string(),
            escape_csv_field(&item.storage_total),
            escape_csv_field(&item.storage_types),
            item.network_count.to_string(),
            escape_csv_field(&item.network_types),
            escape_csv_field(&item.network_speeds),
        ];
        csv_content.push_str(&row.join(","));
        csv_content.push('\n');
    }
    
    // 创建Blob
    let blob_options = BlobPropertyBag::new();
    blob_options.set_type("text/csv;charset=utf-8");
    
    let blob = Blob::new_with_str_sequence_and_options(
        &js_sys::Array::of1(&JsValue::from_str(&csv_content)),
        &blob_options,
    )?;
    
    // 创建下载链接
    let window = window().ok_or("无法获取window对象")?;
    let document = window.document().ok_or("无法获取document对象")?;
    
    let url = Url::create_object_url_with_blob(&blob)?;
    let anchor = document.create_element("a")?.dyn_into::<HtmlAnchorElement>()?;
    
    anchor.set_href(&url);
    anchor.set_download(filename);
    
    // 将anchor转换为HtmlElement以访问style方法
    let anchor_element: HtmlElement = anchor.clone().dyn_into()?;
    anchor_element.style().set_property("display", "none")?;
    
    document.body().ok_or("无法获取body")?.append_child(&anchor)?;
    anchor.click();
    document.body().ok_or("无法获取body")?.remove_child(&anchor)?;
    
    Url::revoke_object_url(&url)?;
    
    Ok(())
}

/// 转义CSV字段
fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// 将数据导出为JSON格式并下载
pub fn export_to_json(data: &[ClientHardwareExport], filename: &str) -> Result<(), JsValue> {
    let json_content = serde_json::to_string_pretty(data)
        .map_err(|e| JsValue::from_str(&format!("JSON序列化失败: {}", e)))?;
    
    // 创建Blob
    let blob_options = BlobPropertyBag::new();
    blob_options.set_type("application/json;charset=utf-8");
    
    let blob = Blob::new_with_str_sequence_and_options(
        &js_sys::Array::of1(&JsValue::from_str(&json_content)),
        &blob_options,
    )?;
    
    // 创建下载链接
    let window = window().ok_or("无法获取window对象")?;
    let document = window.document().ok_or("无法获取document对象")?;
    
    let url = Url::create_object_url_with_blob(&blob)?;
    let anchor = document.create_element("a")?.dyn_into::<HtmlAnchorElement>()?;
    
    anchor.set_href(&url);
    anchor.set_download(filename);
    
    // 将anchor转换为HtmlElement以访问style方法
    let anchor_element: HtmlElement = anchor.clone().dyn_into()?;
    anchor_element.style().set_property("display", "none")?;
    
    document.body().ok_or("无法获取body")?.append_child(&anchor)?;
    anchor.click();
    document.body().ok_or("无法获取body")?.remove_child(&anchor)?;
    
    Url::revoke_object_url(&url)?;
    
    Ok(())
} 