use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorageType {
    SSD,
    HDD,
    NVMe,
    Unknown,
}

impl fmt::Display for StorageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageType::SSD => write!(f, "SSD"),
            StorageType::HDD => write!(f, "HDD"),
            StorageType::NVMe => write!(f, "NVMe"),
            StorageType::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NICStatus {
    Up,
    Down,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NICType {
    Ethernet,
    WiFi,
    Bonding,
    BondingSlave,
    InfiniBand,
    IbRoCEv1,
    RoCEv2,
    IWarp,
    VLAN,
    Unknown,
}

impl fmt::Display for NICType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NICType::Ethernet => write!(f, "Ethernet"),
            NICType::WiFi => write!(f, "Wifi"),
            NICType::Bonding => write!(f, "Bond"),
            NICType::BondingSlave => write!(f, "BondingSlave"),
            NICType::InfiniBand => write!(f, "InfiniBand"),
            NICType::IbRoCEv1 => write!(f, "IB/RoCEv1"),
            NICType::RoCEv2 => write!(f, "RoCEv2"),
            NICType::IWarp => write!(f, "iWARP"),
            NICType::VLAN => write!(f, "VLAN"),
            NICType::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemInfo {
    pub sys_vendor: String,
    pub product_name: String,
    pub product_version: String,
    pub serial_number: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct OS {
    pub name: String,         // Linux, Windows, etc.
    pub version: String,      // 5.10.0-11-amd64, 10.0.19041, etc.
    pub kernel: String,       // 5.10.0-11-amd64, 10.0.19041, etc.
    pub architecture: String, // x86_64, aarch64, etc.
    pub hostname: String,     // hostname
    pub dns: String,
    pub ip_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CPU {
    pub vendor_id: String,  //GenuineIntel
    pub model_name: String, // Intel(R) Xeon(R) Gold 5218 CPU @ 2.30GHz"
    pub cores: u32,         // Number of cores
    pub threads: u32,       // Number of threads
    pub cpus: u32,          // Number of CPUs
    pub flags: Vec<String>,
    pub speed: u32, // Speed in MHz
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GPU {
    pub vendor: String,         // NVIDIA, AMD, etc.
    pub model: String,          // GTX 1080, RX 5700 XT, etc.
    pub device_id: String,      // Device ID
    pub serial_number: String,  // Serial number
    pub driver_version: String, // Driver version
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RAM {
    pub vendor: String,          // 制造商
    pub model: String,           // 型号
    pub size: u32,               // 单根内存大小，单位为 GB
    pub speed: u32,              // 内存频率，单位为 MHz
    pub total_size: u32,         // 总内存大小，单位为 GB
    pub count: u32,              // 内存条数量
    pub form_factor: String,     // 内存形态
    pub modules: Vec<RAMModule>, // 内存模块列表
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RAMModule {
    pub slot: String,          // 插槽标识
    pub vendor: String,        // 制造商
    pub part_number: String,   // 部件号
    pub serial_number: String, // 序列号
    pub size: u32,             // 大小 (GB)
    pub speed: u32,            // 速度 (MHz)
    pub form_factor: String,   // 内存形态
    pub memory_type: String,   // 内存类型，如 DDR4
    pub locator: String,       // 内存位置，如 "DIMM_A1"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Partition {
    pub name: String,      // /dev/sda1, /dev/sda2, etc.
    pub size: String,      // Formatted size string (e.g., "120.5", "800.0")
    pub size_unit: String, // Unit (e.g., "GB", "MB", "KB", "B")
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Disk {
    pub vendor: String,            // Samsung, Seagate, WD, etc.
    pub size: String,              // Formatted size string (e.g., "480.0", "1000.5")
    pub size_unit: String,         // Unit (e.g., "GB", "MB", "KB", "B")
    pub model: String,             // 型号INTEL SSDSCKKB480GZ
    pub storage_type: StorageType, // SSD, HDD, NVMe, etc.
    pub firmware_version: String,  // Firmware version
    pub serial_number: String,     // Serial number
    pub parted: bool,              // True if the disk is partitioned
    pub partitions: Vec<Partition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NIC {
    pub name: String,                // eth0, wlan0, etc.
    pub vendor: String,              // Intel, Broadcom, etc.
    pub model: String,               // Ethernet, WiFi, etc.
    pub speed: u32,                  // Speed in MB/s
    pub mac_address: String,         // MAC address
    pub ipv4_address: String,        // IP address
    pub ipv4_subnet_mask: String,    // Subnet mask
    pub ipv4_gateway: String,        // Gateway
    pub ipv6_address: String,        // IP address
    pub ipv6_subnet_mask: String,    // Subnet mask
    pub ipv6_gateway: String,        // Gateway
    pub dhcp: bool,                  // DHCP enabled
    pub bonding_slaves: Vec<String>, // Bonding slaves
    pub nic_type: NICType,
    pub status: NICStatus,
    pub pci_slot: Option<String>, // PCI ID
    pub firmware_version: String, // Firmware version
    pub ib_node_type: String,     // RDMA node type
    pub driver: String,           // Driver
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Hardware {
    pub system: Option<SystemInfo>,
    #[serde(default)]
    pub os: OS,
    pub cpu: CPU,
    pub gpus: Vec<GPU>,
    pub ram: RAM,
    pub disks: Vec<Disk>,
    pub nics: Vec<NIC>,
    pub ipmi: Option<IpmiInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IpmiInfo {
    pub ip_address: Option<String>,
    pub mac_address: Option<String>,
    pub subnet_mask: Option<String>,
    pub gateway: Option<String>,
    pub channel: u8,
    pub device_id: Option<String>,
    pub firmware_version: Option<String>,
    pub manufacturer_id: Option<u32>,
    pub users: Vec<IpmiUser>,
    pub status: IpmiStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IpmiUser {
    pub user_id: u8,
    pub username: String,
    pub enabled: bool,
    pub privilege_level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IpmiStatus {
    Available,
    NotConfigured,
    NotAvailable,
    AccessDenied,
    Error(String),
}

impl Hardware {
    /// Compare two hardware configurations for semantic equality
    /// This ignores volatile fields like CPU speed that might fluctuate
    pub fn semantically_eq(&self, other: &Hardware) -> bool {
        // Compare CPU (ignoring speed)
        if self.cpu.vendor_id != other.cpu.vendor_id
            || self.cpu.model_name != other.cpu.model_name
            || self.cpu.cores != other.cpu.cores
            || self.cpu.threads != other.cpu.threads
            || self.cpu.cpus != other.cpu.cpus
            || self.cpu.flags != other.cpu.flags
        {
            return false;
        }

        // Compare other fields directly as they are less likely to fluctuate
        // or their fluctuation is significant (e.g. RAM size change)
        if self.system != other.system
            || self.os != other.os
            || self.gpus != other.gpus
            || self.ram != other.ram
            || self.disks != other.disks
            || self.nics != other.nics
            || self.ipmi != other.ipmi
        {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_type_display() {
        assert_eq!(format!("{}", StorageType::SSD), "SSD");
        assert_eq!(format!("{}", StorageType::HDD), "HDD");
        assert_eq!(format!("{}", StorageType::NVMe), "NVMe");
        assert_eq!(format!("{}", StorageType::Unknown), "Unknown");
    }

    #[test]
    fn test_nic_type_display() {
        assert_eq!(format!("{}", NICType::Ethernet), "Ethernet");
        assert_eq!(format!("{}", NICType::WiFi), "Wifi");
        assert_eq!(format!("{}", NICType::Bonding), "Bond");
    }
}
