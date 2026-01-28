use common::entity::hardware::{IpmiInfo, IpmiStatus, IpmiUser};
use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::str;

// IPMI 设备结构
struct IpmiDevice {
    file: std::fs::File,
}

impl IpmiDevice {
    fn open() -> Result<Self, Box<dyn std::error::Error>> {
        let possible_devices = [
            "/dev/ipmi0",
            "/dev/ipmi/0",
            "/dev/ipmidev/0",
            "/dev/ipmi1",
            "/dev/ipmi/1",
            "/dev/ipmidev/1",
        ];

        for device_path in &possible_devices {
            if Path::new(device_path).exists() {
                match OpenOptions::new().read(true).write(true).open(device_path) {
                    Ok(file) => {
                        return Ok(IpmiDevice { file });
                    }
                    Err(_) => continue,
                }
            }
        }

        Err("No accessible IPMI device found".into())
    }

    // 简单的设备检测
    fn test_device(&mut self) -> bool {
        // 尝试读取一些数据来测试设备是否可用
        let mut buffer = [0u8; 1];
        self.file.read(&mut buffer).is_ok()
    }
}

pub fn collect_ipmi_info() -> Result<Option<IpmiInfo>, Box<dyn std::error::Error>> {
    // 检查是否有IPMI设备文件
    if !has_ipmi_device() {
        return Ok(None);
    }

    // 首先尝试直接设备访问
    if let Ok(mut device) = IpmiDevice::open() {
        if device.test_device() {
            // 设备可访问，使用优化的ipmitool获取详细信息
            return collect_via_ipmitool_optimized();
        }
    }

    // 如果直接访问失败，尝试ipmitool
    collect_via_ipmitool_optimized()
}

fn collect_via_ipmitool_optimized() -> Result<Option<IpmiInfo>, Box<dyn std::error::Error>> {
    // 检查ipmitool是否可用
    if !is_ipmitool_available() {
        return Ok(Some(IpmiInfo {
            ip_address: None,
            mac_address: None,
            subnet_mask: None,
            gateway: None,
            channel: 1,
            device_id: None,
            firmware_version: None,
            manufacturer_id: None,
            users: Vec::new(),
            status: IpmiStatus::Error("ipmitool not available".to_string()),
        }));
    }

    let mut ipmi_info = IpmiInfo {
        ip_address: None,
        mac_address: None,
        subnet_mask: None,
        gateway: None,
        channel: 1,
        device_id: None,
        firmware_version: None,
        manufacturer_id: None,
        users: Vec::new(),
        status: IpmiStatus::NotAvailable,
    };

    // 优化：只尝试通道1，因为大多数系统都使用通道1
    // 如果通道1失败，再尝试其他通道
    let mut found_channel = false;

    // 首先尝试通道1
    if let Ok(lan_config) = get_lan_config_via_ipmitool(1) {
        if lan_config.ip_address != "0.0.0.0" && !lan_config.ip_address.is_empty() {
            ipmi_info.ip_address = Some(lan_config.ip_address);
            ipmi_info.mac_address = Some(lan_config.mac_address);
            ipmi_info.subnet_mask = Some(lan_config.subnet_mask);
            ipmi_info.gateway = Some(lan_config.gateway);
            ipmi_info.channel = 1;
            ipmi_info.status = IpmiStatus::Available;
            found_channel = true;
        }
    }

    // 如果通道1失败，快速尝试其他常用通道
    if !found_channel {
        for &channel in &[2u8, 8u8] {
            if let Ok(lan_config) = get_lan_config_via_ipmitool(channel) {
                if lan_config.ip_address != "0.0.0.0" && !lan_config.ip_address.is_empty() {
                    ipmi_info.ip_address = Some(lan_config.ip_address);
                    ipmi_info.mac_address = Some(lan_config.mac_address);
                    ipmi_info.subnet_mask = Some(lan_config.subnet_mask);
                    ipmi_info.gateway = Some(lan_config.gateway);
                    ipmi_info.channel = channel;
                    ipmi_info.status = IpmiStatus::Available;
                    break;
                }
            }
        }
    }

    // 获取设备信息
    if let Ok(device_info) = get_device_info_via_ipmitool() {
        ipmi_info.device_id = device_info.device_id;
        ipmi_info.firmware_version = device_info.firmware_version;
        ipmi_info.manufacturer_id = device_info.manufacturer_id;

        // 如果获取到设备信息，至少标记为已配置
        if ipmi_info.status == IpmiStatus::NotAvailable {
            ipmi_info.status = IpmiStatus::NotConfigured;
        }
    }

    // 获取用户信息 - 只在找到有效通道时获取
    if ipmi_info.status == IpmiStatus::Available {
        ipmi_info.users = get_users_via_ipmitool(ipmi_info.channel);
    }

    Ok(Some(ipmi_info))
}

fn has_ipmi_device() -> bool {
    let possible_devices = [
        "/dev/ipmi0",
        "/dev/ipmi/0",
        "/dev/ipmidev/0",
        "/dev/ipmi1",
        "/dev/ipmi/1",
        "/dev/ipmidev/1",
    ];

    for device in &possible_devices {
        if Path::new(device).exists() {
            return true;
        }
    }

    false
}

fn is_ipmitool_available() -> bool {
    // 使用更快的版本检查
    Command::new("ipmitool")
        .arg("-V")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[derive(Debug)]
struct LanConfiguration {
    ip_address: String,
    mac_address: String,
    subnet_mask: String,
    gateway: String,
}

fn get_lan_config_via_ipmitool(
    channel: u8,
) -> Result<LanConfiguration, Box<dyn std::error::Error>> {
    let output = Command::new("ipmitool")
        .args(["lan", "print", &channel.to_string()])
        .output()?;

    if !output.status.success() {
        return Err(format!("ipmitool failed for channel {}", channel).into());
    }

    let output_str = str::from_utf8(&output.stdout)?;

    let mut config = LanConfiguration {
        ip_address: "0.0.0.0".to_string(),
        mac_address: "00:00:00:00:00:00".to_string(),
        subnet_mask: "0.0.0.0".to_string(),
        gateway: "0.0.0.0".to_string(),
    };

    // 优化解析：只查找需要的字段，找到后立即返回
    let mut found_fields = 0;
    for line in output_str.lines() {
        let line = line.trim();
        // 修复IP地址解析 - 确保匹配正确的行
        if line.starts_with("IP Address") && line.contains(":") && config.ip_address == "0.0.0.0" {
            // 跳过"IP Address Source"行，只处理"IP Address"行
            if !line.starts_with("IP Address Source") {
                if let Some(ip) = line.split(':').nth(1) {
                    let ip_addr = ip.trim().to_string();
                    // 确保不是源类型描述
                    if !ip_addr.contains("Static")
                        && !ip_addr.contains("DHCP")
                        && !ip_addr.contains("BIOS")
                    {
                        config.ip_address = ip_addr;
                        found_fields += 1;
                    }
                }
            }
        } else if line.starts_with("MAC Address")
            && line.contains(":")
            && config.mac_address == "00:00:00:00:00:00"
        {
            // MAC地址格式: "MAC Address             : 38:68:dd:23:26:1d"
            if let Some(mac_part) = line
                .split(':')
                .skip(1)
                .collect::<Vec<_>>()
                .join(":")
                .split_whitespace()
                .next()
            {
                config.mac_address = mac_part.to_string();
                found_fields += 1;
            }
        } else if line.starts_with("Subnet Mask")
            && line.contains(":")
            && config.subnet_mask == "0.0.0.0"
        {
            if let Some(subnet) = line.split(':').nth(1) {
                config.subnet_mask = subnet.trim().to_string();
                found_fields += 1;
            }
        } else if line.starts_with("Default Gateway IP")
            && line.contains(":")
            && config.gateway == "0.0.0.0"
        {
            if let Some(gateway) = line.split(':').nth(1) {
                config.gateway = gateway.trim().to_string();
                found_fields += 1;
            }
        }

        // 如果找到所有字段，提前退出
        if found_fields >= 4 {
            break;
        }
    }

    Ok(config)
}

#[derive(Debug)]
struct DeviceInfo {
    device_id: Option<String>,
    firmware_version: Option<String>,
    manufacturer_id: Option<u32>,
}

fn get_device_info_via_ipmitool() -> Result<DeviceInfo, Box<dyn std::error::Error>> {
    let output = Command::new("ipmitool").args(["mc", "info"]).output()?;

    if !output.status.success() {
        return Err("ipmitool mc info failed".into());
    }

    let output_str = str::from_utf8(&output.stdout)?;

    let mut device_info = DeviceInfo {
        device_id: None,
        firmware_version: None,
        manufacturer_id: None,
    };

    // 优化解析：只查找需要的字段
    let mut found_fields = 0;
    for line in output_str.lines() {
        let line = line.trim();
        if line.starts_with("Device ID") && line.contains(":") && device_info.device_id.is_none() {
            if let Some(id) = line.split(':').nth(1) {
                device_info.device_id = Some(id.trim().to_string());
                found_fields += 1;
            }
        } else if line.starts_with("Firmware Revision")
            && line.contains(":")
            && device_info.firmware_version.is_none()
        {
            if let Some(fw) = line.split(':').nth(1) {
                device_info.firmware_version = Some(fw.trim().to_string());
                found_fields += 1;
            }
        } else if line.starts_with("Manufacturer ID")
            && line.contains(":")
            && device_info.manufacturer_id.is_none()
        {
            if let Some(mfg) = line.split(':').nth(1) {
                let mfg_str = mfg.trim();
                // 尝试解析十六进制或十进制
                if let Ok(mfg_id) = u32::from_str_radix(mfg_str.trim_start_matches("0x"), 16) {
                    device_info.manufacturer_id = Some(mfg_id);
                    found_fields += 1;
                } else if let Ok(mfg_id) = mfg_str.parse::<u32>() {
                    device_info.manufacturer_id = Some(mfg_id);
                    found_fields += 1;
                }
            }
        }

        // 如果找到所有字段，提前退出
        if found_fields >= 3 {
            break;
        }
    }

    Ok(device_info)
}

fn get_users_via_ipmitool(channel: u8) -> Vec<IpmiUser> {
    let mut users = Vec::new();

    // 使用一次调用获取所有用户信息
    if let Ok(output) = Command::new("ipmitool")
        .args(["user", "list", &channel.to_string()])
        .output()
    {
        if output.status.success() {
            if let Ok(output_str) = str::from_utf8(&output.stdout) {
                for line in output_str.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with("ID") {
                        continue; // 跳过标题行和空行
                    }

                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(user_id) = parts[0].parse::<u8>() {
                            let username = parts[1].to_string();

                            // 跳过空用户名或默认值
                            if username.is_empty() || username == "true" || username == "false" {
                                continue;
                            }

                            // 解析启用状态 - 通常在第3列
                            let enabled = if parts.len() > 2 {
                                parts[2] == "true" || parts[2] == "yes" || parts[2] == "enabled"
                            } else {
                                true // 默认启用
                            };

                            // 解析权限级别 - 通常在第4列
                            let privilege_level = if parts.len() > 3 {
                                match parts[3].to_uppercase().as_str() {
                                    "ADMINISTRATOR" | "ADMIN" => 4,
                                    "OPERATOR" | "OPR" => 3,
                                    "USER" => 2,
                                    "CALLBACK" => 1,
                                    "NO ACCESS" => 15,
                                    _ => 2, // 默认为用户级别
                                }
                            } else {
                                2 // 默认为用户级别
                            };

                            users.push(IpmiUser {
                                user_id,
                                username,
                                enabled,
                                privilege_level,
                            });
                        }
                    }
                }
            }
        }
    }

    users
}
