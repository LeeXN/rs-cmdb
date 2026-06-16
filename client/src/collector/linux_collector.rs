use crate::collector::linux_cpu::get_cpu_info;
use crate::collector::linux_disk::collect_disks;
use crate::collector::linux_gpu::collect_gpus;
use crate::collector::linux_ipmi;
use crate::collector::linux_nic::{collect_nics, get_dns_servers, get_ip_address};
use crate::collector::linux_ram::collect_ram;
use common::entity::hardware::{Disk, Hardware, SystemInfo, CPU, GPU, NIC, OS, RAM};
use std::fs;
use std::io;
use sysinfo::System;
use tracing::{info, warn};

const DMI_PATH: &str = "/sys/class/dmi/id/";

// 收集操作系统信息
pub fn collect_os_info() -> OS {
    let dns_servers = get_dns_servers();
    let ip_address = get_ip_address();
    OS {
        name: System::name().unwrap_or_else(|| "Unknown".to_string()),
        version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
        kernel: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
        architecture: System::cpu_arch(),
        hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
        dns: dns_servers.join(", "),
        ip_address,
    }
}

pub fn collect_system_info() -> SystemInfo {
    let (sys_vendor, product_name, product_version, serial_number) = get_dmi_info();
    SystemInfo {
        sys_vendor,
        product_name,
        product_version,
        serial_number,
    }
}
fn read_dmi_file(file: &str) -> String {
    let path = format!("{}/{}", DMI_PATH, file);
    let content = fs::read_to_string(path)
        .unwrap_or_else(|_| "Unknown".to_string())
        .trim()
        .to_string();
    content
}

fn get_dmi_info() -> (String, String, String, String) {
    let sys_vendor = read_dmi_file("sys_vendor");
    let product_name = read_dmi_file("product_name");
    let product_version = read_dmi_file("product_version");
    let serial_number = read_dmi_file("product_serial");
    (sys_vendor, product_name, product_version, serial_number)
}

// 收集CPU信息
pub fn collect_cpu_info() -> CPU {
    get_cpu_info()
}

// 收集磁盘信息
pub fn collect_disk_info() -> Vec<Disk> {
    collect_disks()
}

// 收集网卡信息
pub fn collect_nic_info() -> io::Result<Vec<NIC>> {
    collect_nics()
}

// 收集所有硬件信息
pub fn collect_hardware() -> Hardware {
    collect_hardware_info(&[])
}

// 获取线程数/物理核心数比例 (超线程比例)
pub fn get_threads_per_core() -> f32 {
    let cpu = collect_cpu_info();
    if cpu.cores > 0 {
        cpu.threads as f32 / cpu.cores as f32
    } else {
        0.0
    }
}

// 获取每个CPU的核心数
pub fn get_cores_per_cpu() -> u32 {
    let cpu = collect_cpu_info();
    cpu.cores.checked_div(cpu.cpus).unwrap_or(0)
}

// 收集GPU信息
pub fn collect_gpu_info() -> Vec<GPU> {
    collect_gpus()
}

// 收集内存信息
pub fn collect_ram_info() -> RAM {
    collect_ram()
}

/// 收集指定组件的硬件信息
pub fn collect_hardware_info(components: &[String]) -> common::entity::hardware::Hardware {
    // 将字符串组件列表转换为小写以便匹配
    let components: Vec<String> = components.iter().map(|c| c.to_lowercase()).collect();

    // 根据请求的组件收集信息
    let cpu = if components.is_empty() || components.contains(&"cpu".to_string()) {
        collect_cpu_info()
    } else {
        // 返回空CPU信息
        common::entity::hardware::CPU {
            vendor_id: String::new(),
            model_name: String::new(),
            cores: 0,
            threads: 0,
            cpus: 0,
            flags: Vec::new(),
            speed: 0,
        }
    };

    let gpus = if components.is_empty() || components.contains(&"gpu".to_string()) {
        collect_gpu_info()
    } else {
        Vec::new()
    };

    let ram = if components.is_empty() || components.contains(&"ram".to_string()) {
        collect_ram_info()
    } else {
        common::entity::hardware::RAM {
            vendor: String::new(),
            model: String::new(),
            size: 0,
            speed: 0,
            total_size: 0,
            count: 0,
            form_factor: String::new(),
            modules: Vec::new(),
        }
    };

    let disks = if components.is_empty() || components.contains(&"disk".to_string()) {
        collect_disk_info()
    } else {
        Vec::new()
    };

    // 网卡信息需要处理可能出现的错误
    let nics = if components.is_empty() || components.contains(&"nic".to_string()) {
        collect_nic_info().unwrap_or_default()
    } else {
        Vec::new()
    };

    // IPMI信息采集
    let ipmi = if components.is_empty() || components.contains(&"ipmi".to_string()) {
        match linux_ipmi::collect_ipmi_info() {
            Ok(info) => {
                if let Some(ref ipmi) = info {
                    info!("IPMI信息采集完成，状态: {:?}", ipmi.status);
                    if let Some(ref ip) = ipmi.ip_address {
                        info!("IPMI IP地址: {}", ip);
                    }
                    info!("IPMI用户数量: {}", ipmi.users.len());
                    for user in &ipmi.users {
                        info!(
                            "  用户: {} (ID: {}, 启用: {}, 权限: {})",
                            user.username, user.user_id, user.enabled, user.privilege_level
                        );
                    }
                } else {
                    info!("未检测到IPMI设备");
                }
                info
            }
            Err(e) => {
                warn!("IPMI信息采集失败: {}", e);
                None
            }
        }
    } else {
        None
    };

    // OS信息采集
    let os = if components.is_empty() || components.contains(&"os".to_string()) {
        collect_os_info()
    } else {
        common::entity::hardware::OS::default()
    };

    // System信息采集
    let system = if components.is_empty() || components.contains(&"system".to_string()) {
        Some(collect_system_info())
    } else {
        None
    };

    common::entity::hardware::Hardware {
        system,
        os,
        cpu,
        gpus,
        ram,
        disks,
        nics,
        ipmi,
    }
}
