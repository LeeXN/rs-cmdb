use crate::i18n::{I18n, Language};
use std::rc::Rc;
use std::cell::RefCell;

thread_local! {
    /// 线程本地的国际化实例
    static GLOBAL_I18N: RefCell<Option<Rc<I18n>>> = RefCell::new(None);
}

/// 初始化全局国际化实例
pub fn init_i18n(language: Language) {
    GLOBAL_I18N.with(|i18n| {
        *i18n.borrow_mut() = Some(Rc::new(I18n::new(language)));
    });
}

/// 获取全局国际化实例
#[allow(dead_code)]
pub fn get_i18n() -> Rc<I18n> {
    GLOBAL_I18N.with(|i18n| {
        i18n.borrow()
            .clone()
            .unwrap_or_else(|| {
                let default_i18n = Rc::new(I18n::default());
                // 设置默认实例
                drop(i18n.borrow()); // 释放借用
                GLOBAL_I18N.with(|i18n_inner| {
                    *i18n_inner.borrow_mut() = Some(default_i18n.clone());
                });
                default_i18n
            })
    })
}

/// 翻译函数的简化版本
#[allow(dead_code)]
pub fn t(key: &str) -> String {
    get_i18n().t(key)
}

/// 翻译API响应消息
#[allow(dead_code)]
pub fn translate_api_message(message: &str) -> String {
    // 如果消息是英文的错误码，则翻译它
    if message.chars().all(|c| c.is_ascii()) && message.contains('_') {
        t(message)
    } else {
        // 如果已经是中文或其他语言，直接返回
        message.to_string()
    }
}

/// 翻译硬件相关的值
#[allow(dead_code)]
pub fn translate_hardware_value(value: &str) -> String {
    match value {
        "all" => t("all"),
        "unknown" => t("unknown"),
        "none" => t("none"),
        "never" => t("never"),
        "no_discrete_gpu" => t("no_discrete_gpu"),
        "unknown_system" => t("unknown_system"),
        "unknown_model" => t("unknown_model"),
        "unknown_vendor" => t("unknown_vendor"),
        "unknown_version" => t("unknown_version"),
        "unknown_kernel" => t("unknown_kernel"),
        "unknown_architecture" => t("unknown_architecture"),
        "no_driver" => t("no_driver"),
        "no_storage_devices" => t("no_storage_devices"),
        // 存储类型
        "nvme_ssd_hdd_mixed" => t("nvme_ssd_hdd_mixed"),
        "nvme_ssd_mixed" => t("nvme_ssd_mixed"),
        "nvme_hdd_mixed" => t("nvme_hdd_mixed"),
        "ssd_hdd_mixed" => t("ssd_hdd_mixed"),
        "pure_nvme" => t("pure_nvme"),
        "pure_ssd" => t("pure_ssd"),
        "pure_hdd" => t("pure_hdd"),
        "unknown_storage_type" => t("unknown_storage_type"),
        // 硬件类别
        "cpu_config" => t("cpu_config"),
        "memory_config" => t("memory_config"),
        "gpu_config" => t("gpu_config"),
        "storage_config" => t("storage_config"),
        "network_config" => t("network_config"),
        "operating_system" => t("operating_system"),
        "server_model" => t("server_model"),
        _ => value.to_string(),
    }
}

/// 翻译单位
#[allow(dead_code)]
pub fn translate_unit(value: &str, unit: &str) -> String {
    let translated_unit = match unit {
        "cores" => t("cores"),
        "threads" => t("threads"),
        "gb" => t("gb"),
        "mhz" => t("mhz"),
        "ghz" => t("ghz"),
        "nics" => t("nics"),
        _ => unit.to_string(),
    };
    
    if translated_unit.is_empty() {
        format!("{}{}", value, translated_unit)
    } else {
        format!("{} {}", value, translated_unit)
    }
} 