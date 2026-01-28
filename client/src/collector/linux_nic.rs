use std::fs;
use std::io::{self, BufRead};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};
use std::process::Command;

use common::entity::hardware::{NICStatus, NICType, NIC};

const SYS_CLASS_NET: &str = "/sys/class/net";
const PROC_NET_DEV: &str = "/proc/net/dev";
const PROC_NET_ROUTE: &str = "/proc/net/route";
const PROC_NET_IPV6_ROUTE: &str = "/proc/net/ipv6_route";
const PROC_NET_IF_INET6: &str = "/proc/net/if_inet6";
const RESOLV_CONF: &str = "/etc/resolv.conf";
const IB_SYSFS_PATH: &str = "/sys/class/infiniband";
const IGNORE_IFACES: [&str; 11] = [
    "cali",
    "kube-ipvs",
    "nodelocaldns",
    "tunl0",
    "tap0",
    "veth",
    "docker",
    "virbr",
    "br-",
    "vxlan",
    "lo",
];

// 系统信息缓存结构
#[derive(Debug, Default)]
struct SystemCache {
    ipv4_routes: Vec<RouteInfo>,
    ipv6_routes: Vec<RouteInfo>,
    ipv6_addresses: Vec<IPv6AddrInfo>,
    dns_servers: Vec<String>,
    pci_ids_cache: Option<String>,
    infiniband_devices: Vec<(String, PathBuf)>,
}

#[derive(Debug, Clone)]
struct RouteInfo {
    interface: String,
    #[allow(dead_code)]
    destination: String,
    gateway: String,
    is_default: bool,
}

#[derive(Debug, Clone)]
struct IPv6AddrInfo {
    interface: String,
    address: String,
    prefix_len: String,
}

// 接口设备信息结构
#[derive(Debug, Default)]
struct InterfaceDeviceInfo {
    vendor_id: String,
    device_id: String,
    vendor_name: String,
    device_name: String,
    driver_name: String,
    pci_slot: String,
}

// InfiniBand 设备信息结构
#[derive(Debug, Default)]
struct InfiniBandInfo {
    is_ib: bool,
    link_layer: String,
    node_type: String,
    roce_type: String,
    driver: String,
    firmware_version: String,
}

impl SystemCache {
    fn new() -> io::Result<Self> {
        let mut cache = SystemCache::default();

        // 一次性读取所有路由信息
        cache.ipv4_routes = Self::parse_ipv4_routes()?;
        cache.ipv6_routes = Self::parse_ipv6_routes()?;
        cache.ipv6_addresses = Self::parse_ipv6_addresses()?;
        cache.dns_servers = Self::parse_dns_servers();
        cache.pci_ids_cache = Self::load_pci_ids();
        cache.infiniband_devices = Self::list_infiniband_devices();

        Ok(cache)
    }

    fn parse_ipv4_routes() -> io::Result<Vec<RouteInfo>> {
        let mut routes = Vec::new();
        let file = fs::File::open(PROC_NET_ROUTE)?;
        let reader = io::BufReader::new(file);

        for line in reader.lines().skip(1) {
            let line = line?;
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let interface = parts[0].trim().to_string();
                let destination = parts[1].trim();
                let gateway = parts[2].trim();
                let is_default = destination == "00000000";

                routes.push(RouteInfo {
                    interface,
                    destination: destination.to_string(),
                    gateway: gateway.to_string(),
                    is_default,
                });
            }
        }
        Ok(routes)
    }

    fn parse_ipv6_routes() -> io::Result<Vec<RouteInfo>> {
        let mut routes = Vec::new();
        let file = fs::File::open(PROC_NET_IPV6_ROUTE)?;
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                let destination = parts[0];
                let prefix_len = parts[1];
                let gateway = parts[4];
                let interface = parts[9].to_string();
                let is_default =
                    destination == "00000000000000000000000000000000" && prefix_len == "00";

                routes.push(RouteInfo {
                    interface,
                    destination: destination.to_string(),
                    gateway: gateway.to_string(),
                    is_default,
                });
            }
        }
        Ok(routes)
    }

    fn parse_ipv6_addresses() -> io::Result<Vec<IPv6AddrInfo>> {
        let mut addresses = Vec::new();
        let file = fs::File::open(PROC_NET_IF_INET6)?;
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let addr_hex = parts[0];
                let prefix_len = parts[1];
                let interface = parts[5].to_string();

                if addr_hex.len() == 32 {
                    let mut addr_bytes = [0u8; 16];
                    for i in 0..16 {
                        if let Ok(byte) = u8::from_str_radix(&addr_hex[i * 2..i * 2 + 2], 16) {
                            addr_bytes[i] = byte;
                        }
                    }

                    let ipv6_addr = Ipv6Addr::from(addr_bytes);
                    let prefix_decimal = u8::from_str_radix(prefix_len, 16).unwrap_or(0);

                    addresses.push(IPv6AddrInfo {
                        interface,
                        address: ipv6_addr.to_string(),
                        prefix_len: prefix_decimal.to_string(),
                    });
                }
            }
        }
        Ok(addresses)
    }

    fn parse_dns_servers() -> Vec<String> {
        let mut dns_servers = Vec::new();
        if let Ok(file) = fs::File::open(RESOLV_CONF) {
            let reader = io::BufReader::new(file);
            for line in reader.lines().flatten() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2
                    && parts[0] == "nameserver"
                    && parts[1].parse::<IpAddr>().is_ok()
                {
                    dns_servers.push(parts[1].to_string());
                }
            }
        }
        dns_servers
    }

    fn load_pci_ids() -> Option<String> {
        let pci_ids_paths = ["/usr/share/hwdata/pci.ids", "/usr/share/misc/pci.ids"];

        for path in &pci_ids_paths {
            if let Ok(content) = fs::read_to_string(path) {
                return Some(content);
            }
        }
        None
    }

    fn list_infiniband_devices() -> Vec<(String, PathBuf)> {
        let mut devices = Vec::new();
        let infiniband_path = Path::new(IB_SYSFS_PATH);

        if let Ok(entries) = fs::read_dir(infiniband_path) {
            for entry in entries.filter_map(Result::ok) {
                let hca = entry.file_name().to_string_lossy().into_owned();
                let device_link = infiniband_path.join(&hca).join("device");
                if let Ok(device_path) = fs::canonicalize(&device_link) {
                    devices.push((hca, device_path));
                }
            }
        }
        devices
    }

    fn get_ipv4_gateway(&self, interface: &str) -> Option<String> {
        self.ipv4_routes
            .iter()
            .find(|route| {
                route.interface == interface && route.is_default && route.gateway != "00000000"
            })
            .and_then(|route| {
                if let Ok(gw_val) = u32::from_str_radix(&route.gateway, 16) {
                    let ip_bytes = gw_val.to_le_bytes();
                    let gateway_ip =
                        Ipv4Addr::new(ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]);
                    Some(gateway_ip.to_string())
                } else {
                    None
                }
            })
    }

    fn get_ipv6_gateway(&self, interface: &str) -> Option<String> {
        self.ipv6_routes
            .iter()
            .find(|route| {
                route.interface == interface
                    && route.is_default
                    && route.gateway != "00000000000000000000000000000000"
            })
            .map(|route| {
                let mut addr_bytes = [0u8; 16];
                for i in 0..16 {
                    if let Ok(byte) = u8::from_str_radix(&route.gateway[i * 2..i * 2 + 2], 16) {
                        addr_bytes[i] = byte;
                    }
                }
                let ipv6_addr = Ipv6Addr::from(addr_bytes);
                ipv6_addr.to_string()
            })
    }

    fn get_ipv6_info(&self, interface: &str) -> (String, String) {
        // 优先选择非链路本地地址
        let mut link_local = None;
        for addr_info in &self.ipv6_addresses {
            if addr_info.interface == interface {
                if !addr_info.address.starts_with("fe80") {
                    return (
                        addr_info.address.clone(),
                        format!("/{}", addr_info.prefix_len),
                    );
                } else if link_local.is_none() {
                    link_local = Some((
                        addr_info.address.clone(),
                        format!("/{}", addr_info.prefix_len),
                    ));
                }
            }
        }
        link_local.unwrap_or_default()
    }

    fn parse_pci_ids(&self, vendor_id: &str, device_id: &str) -> Option<(String, String)> {
        let content = self.pci_ids_cache.as_ref()?;
        let mut current_vendor = String::new();
        let mut vendor_name = String::new();
        let vendor_id = vendor_id.trim_start_matches("0x").to_lowercase();
        let device_id = device_id.trim_start_matches("0x").to_lowercase();

        for line in content.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }

            if !line.starts_with('\t') {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() >= 2 {
                    let vid = parts[0].trim_start_matches("0x").to_lowercase();
                    if vid == vendor_id {
                        current_vendor = vid;
                        vendor_name = parts[1].trim().to_string();
                    }
                }
            } else if line.starts_with('\t')
                && !line.starts_with("\t\t")
                && current_vendor == vendor_id
            {
                let line = line.trim();
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() >= 2 {
                    let did = parts[0].trim_start_matches("0x").to_lowercase();
                    if did == device_id {
                        let device_name = parts[1].trim().to_string();
                        return Some((vendor_name, device_name));
                    }
                }
            }
        }

        if !vendor_name.is_empty() {
            Some((vendor_name, format!("Unknown Device {}", device_id)))
        } else {
            None
        }
    }
}

// 统一的文件系统访问接口
struct SysfsInterface {
    base_path: PathBuf,
}

impl SysfsInterface {
    fn new(interface_name: &str) -> Self {
        Self {
            base_path: Path::new(SYS_CLASS_NET).join(interface_name),
        }
    }

    fn read_value(&self, file: &str) -> io::Result<String> {
        fs::read_to_string(self.base_path.join(file)).map(|s| s.trim().to_string())
    }

    fn read_optional_value(&self, file: &str) -> Option<String> {
        self.read_value(file).ok()
    }

    fn path_exists(&self, path: &str) -> bool {
        self.base_path.join(path).exists()
    }

    fn get_device_path(&self) -> Option<PathBuf> {
        fs::canonicalize(self.base_path.join("device")).ok()
    }
}

// 获取接口基本信息
fn get_interface_basic_info(
    sysfs: &SysfsInterface,
) -> io::Result<(String, NICStatus, u32, Vec<String>)> {
    let mac_address = sysfs
        .read_value("address")
        .unwrap_or_else(|_| "Unknown".to_string());
    let operstate = sysfs
        .read_value("operstate")
        .unwrap_or_else(|_| "unknown".to_string());
    let speed_str = sysfs
        .read_optional_value("speed")
        .unwrap_or_else(|| "0".to_string());
    let speed: u32 = speed_str.parse().unwrap_or(0);

    let mut bonding_slaves = Vec::new();
    if sysfs.path_exists("bonding/slaves") {
        if let Ok(slaves) = sysfs.read_value("bonding/slaves") {
            bonding_slaves.push(slaves);
        }
    }

    let status = match operstate.as_str() {
        "up" => NICStatus::Up,
        "down" => NICStatus::Down,
        _ => NICStatus::Unknown,
    };

    Ok((mac_address, status, speed, bonding_slaves))
}

// 获取设备信息（vendor, device等）
fn get_device_info(sysfs: &SysfsInterface, cache: &SystemCache) -> InterfaceDeviceInfo {
    let mut info = InterfaceDeviceInfo::default();

    if let Some(device_path) = sysfs.get_device_path() {
        // 读取uevent文件获取设备信息
        if let Ok(uevent_content) = fs::read_to_string(device_path.join("uevent")) {
            for line in uevent_content.lines() {
                if line.starts_with("DRIVER=") {
                    info.driver_name = line["DRIVER=".len()..].to_string();
                } else if let Some(pci_id) = line.strip_prefix("PCI_ID=") {
                    if let Some(pos) = pci_id.find(':') {
                        info.vendor_id = pci_id[..pos].to_lowercase();
                        info.device_id = pci_id[pos + 1..].to_lowercase();
                    }
                } else if line.starts_with("PCI_SLOT_NAME=") {
                    info.pci_slot = line["PCI_SLOT_NAME=".len()..].to_string();
                }
            }
        }

        // 直接从设备目录读取vendor和device文件
        if info.vendor_id.is_empty() {
            info.vendor_id = sysfs
                .read_optional_value("device/vendor")
                .unwrap_or_else(|| "Unknown".to_string());
        }
        if info.device_id.is_empty() {
            info.device_id = sysfs
                .read_optional_value("device/device")
                .unwrap_or_else(|| "Unknown".to_string());
        }

        // 获取友好名称
        if let Some((vendor_name, device_name)) =
            cache.parse_pci_ids(&info.vendor_id, &info.device_id)
        {
            info.vendor_name = format!(
                "{}(0x{})",
                vendor_name,
                info.vendor_id.trim_start_matches("0x")
            );
            info.device_name = format!(
                "{}(0x{})",
                device_name,
                info.device_id.trim_start_matches("0x")
            );
        } else {
            // 使用预定义的厂商名称
            info.vendor_name = match info.vendor_id.trim_start_matches("0x") {
                "8086" => format!("Intel(0x{})", info.vendor_id.trim_start_matches("0x")),
                "15b3" => format!("Mellanox(0x{})", info.vendor_id.trim_start_matches("0x")),
                "1077" => format!("QLogic(0x{})", info.vendor_id.trim_start_matches("0x")),
                "14e4" => format!("Broadcom(0x{})", info.vendor_id.trim_start_matches("0x")),
                "10ec" => format!("Realtek(0x{})", info.vendor_id.trim_start_matches("0x")),
                _ => format!(
                    "Unknown Vendor(0x{})",
                    info.vendor_id.trim_start_matches("0x")
                ),
            };
            info.device_name = format!(
                "Unknown Device(0x{})",
                info.device_id.trim_start_matches("0x")
            );
        }
    } else {
        info.vendor_name = "Unknown".to_string();
        info.device_name = "Unknown".to_string();
        info.driver_name = "Unknown".to_string();
    }

    info
}

// 获取InfiniBand设备信息
fn get_infiniband_info(sysfs: &SysfsInterface, cache: &SystemCache) -> InfiniBandInfo {
    let mut ib_info = InfiniBandInfo::default();

    if let Some(interface_device_path) = sysfs.get_device_path() {
        for (hca, ib_device_path) in &cache.infiniband_devices {
            if interface_device_path == *ib_device_path {
                ib_info.is_ib = true;

                let ib_base = Path::new(IB_SYSFS_PATH).join(hca);

                // 获取链路层类型
                if let Ok(link_layer) = fs::read_to_string(ib_base.join("ports/1/link_layer")) {
                    ib_info.link_layer = link_layer.trim().to_string();
                }

                // 获取节点类型
                if let Ok(node_type) = fs::read_to_string(ib_base.join("node_type")) {
                    ib_info.node_type = node_type.trim().to_string();
                }

                // 获取RoCE类型
                if let Ok(roce_type) = fs::read_to_string(ib_base.join("ports/1/gid_attrs/types/0"))
                {
                    ib_info.roce_type = roce_type.trim().to_string();
                }

                // 获取固件版本
                if let Ok(fw_ver) = fs::read_to_string(ib_base.join("fw_ver")) {
                    ib_info.firmware_version = fw_ver.trim().to_string();
                }

                // 获取驱动名称
                if let Ok(uevent_content) = fs::read_to_string(ib_base.join("device/uevent")) {
                    for line in uevent_content.lines() {
                        if line.starts_with("DRIVER=") {
                            ib_info.driver = line["DRIVER=".len()..].to_string();
                            break;
                        }
                    }
                }

                break;
            }
        }
    }

    ib_info
}

// 获取IP地址信息
fn get_ip_info(
    interface: &str,
    cache: &SystemCache,
) -> (String, String, String, String, String, String) {
    let mut ipv4_address = String::new();
    let mut ipv4_subnet_mask = String::new();
    let mut ipv6_address = String::new();
    let mut ipv6_subnet_mask = String::new();

    // 获取IPv4信息
    if let Ok(output) = Command::new("ip")
        .args(["-4", "addr", "show", interface])
        .output()
    {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.trim().starts_with("inet ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let inet_parts: Vec<&str> = parts[1].split('/').collect();
                        if inet_parts.len() == 2 {
                            ipv4_address = inet_parts[0].to_string();
                            if let Ok(prefix_len) = inet_parts[1].parse::<u8>() {
                                let mask_bits: u32 = (!0u32) << (32 - prefix_len);
                                ipv4_subnet_mask = format!(
                                    "{}.{}.{}.{}",
                                    (mask_bits >> 24) & 0xFF,
                                    (mask_bits >> 16) & 0xFF,
                                    (mask_bits >> 8) & 0xFF,
                                    mask_bits & 0xFF
                                );
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    // 获取IPv6信息，优先使用ip命令
    if let Ok(output) = Command::new("ip")
        .args(["-6", "addr", "show", interface])
        .output()
    {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.trim().starts_with("inet6 ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let inet_parts: Vec<&str> = parts[1].split('/').collect();
                        if inet_parts.len() == 2
                            && (!inet_parts[0].starts_with("fe80") || ipv6_address.is_empty())
                        {
                            ipv6_address = inet_parts[0].to_string();
                            ipv6_subnet_mask = format!("/{}", inet_parts[1]);
                            if !inet_parts[0].starts_with("fe80") {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    // 如果ip命令失败，回退到缓存的IPv6信息
    if ipv6_address.is_empty() {
        let (addr, prefix) = cache.get_ipv6_info(interface);
        ipv6_address = addr;
        ipv6_subnet_mask = prefix;
    }

    let ipv4_gateway = cache.get_ipv4_gateway(interface).unwrap_or_default();
    let ipv6_gateway = cache.get_ipv6_gateway(interface).unwrap_or_default();

    (
        ipv4_address,
        ipv4_subnet_mask,
        ipv4_gateway,
        ipv6_address,
        ipv6_subnet_mask,
        ipv6_gateway,
    )
}

// 列出所有网络接口
fn list_interfaces() -> io::Result<Vec<String>> {
    let file = fs::File::open(PROC_NET_DEV)?;
    let reader = io::BufReader::new(file);
    let mut interfaces = Vec::new();

    for line in reader.lines().skip(2) {
        let line = line?;
        if let Some(iface_part) = line.split(':').next() {
            let iface_part = iface_part.trim();
            // 过滤掉不需要的接口
            if !IGNORE_IFACES
                .iter()
                .any(|prefix| iface_part.starts_with(prefix))
            {
                interfaces.push(iface_part.to_string());
            }
        }
    }
    Ok(interfaces)
}

// 判断接口类型
fn determine_nic_type(sysfs: &SysfsInterface, ib_info: &InfiniBandInfo) -> NICType {
    if sysfs.path_exists("bonding") {
        NICType::Bonding
    } else if ib_info.is_ib {
        if ib_info.link_layer == "InfiniBand" {
            NICType::InfiniBand
        } else if ib_info.link_layer == "Ethernet" {
            match ib_info.roce_type.as_str() {
                "IB/RoCE v1" => NICType::IbRoCEv1,
                "RoCE v2" => NICType::RoCEv2,
                _ => NICType::Unknown,
            }
        } else {
            NICType::Unknown
        }
    } else if is_vlan_interface(sysfs) {
        NICType::VLAN
    } else if sysfs.path_exists("bonding/slaves") {
        NICType::BondingSlave
    } else if sysfs.path_exists("device") {
        NICType::Ethernet
    } else {
        NICType::Unknown
    }
}

fn is_vlan_interface(sysfs: &SysfsInterface) -> bool {
    if let Ok(uevent_content) = sysfs.read_value("uevent") {
        uevent_content.lines().any(|line| line == "DEVTYPE=vlan")
    } else {
        false
    }
}

// 主要的收集函数
pub fn collect_nics() -> io::Result<Vec<NIC>> {
    let mut nics = Vec::new();
    let interfaces = list_interfaces()?;
    let cache = SystemCache::new()?;

    for interface in interfaces {
        let sysfs = SysfsInterface::new(&interface);

        // 检查是否应该包含此接口
        let is_physical_ish = sysfs.path_exists("device");
        let is_bond = sysfs.path_exists("bonding");
        let is_bridge = sysfs.path_exists("bridge");
        let is_vlan = is_vlan_interface(&sysfs);
        let ib_info = get_infiniband_info(&sysfs, &cache);

        // 过滤纯虚拟接口
        if !is_physical_ish && !is_bond && !is_bridge && !is_vlan && !ib_info.is_ib {
            continue;
        }

        // 获取基本信息
        let (mac_address, status, speed, bonding_slaves) = get_interface_basic_info(&sysfs)?;

        // 获取设备信息
        let device_info = if is_physical_ish || ib_info.is_ib {
            get_device_info(&sysfs, &cache)
        } else {
            InterfaceDeviceInfo::default()
        };

        // 获取IP信息
        let (
            ipv4_address,
            ipv4_subnet_mask,
            ipv4_gateway,
            ipv6_address,
            ipv6_subnet_mask,
            ipv6_gateway,
        ) = get_ip_info(&interface, &cache);

        // 确定接口类型
        let nic_type = determine_nic_type(&sysfs, &ib_info);

        nics.push(NIC {
            name: interface,
            vendor: if device_info.vendor_name.is_empty() {
                "Unknown".to_string()
            } else {
                device_info.vendor_name
            },
            model: if device_info.device_name.is_empty() {
                "Unknown".to_string()
            } else {
                device_info.device_name
            },
            speed,
            mac_address,
            ipv4_address,
            ipv4_subnet_mask,
            ipv4_gateway,
            ipv6_address,
            ipv6_subnet_mask,
            ipv6_gateway,
            dhcp: false, // 默认为false，实际检测较复杂
            nic_type,
            bonding_slaves,
            status,
            pci_slot: if device_info.pci_slot.is_empty() {
                None
            } else {
                Some(device_info.pci_slot)
            },
            // 目前只有rdma能想办法拿到fw_ver，普通网卡只能调用ioctl
            firmware_version: ib_info.firmware_version,
            ib_node_type: ib_info.node_type,
            driver: device_info.driver_name,
        });
    }

    Ok(nics)
}

// 保留的向后兼容函数
pub fn get_dns_servers() -> Vec<String> {
    SystemCache::parse_dns_servers()
}

pub fn get_ip_address() -> String {
    let output = Command::new("hostname").arg("-I").output().unwrap();
    String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string()
}

pub fn get_pci_slot(iface: &str) -> String {
    let sysfs = SysfsInterface::new(iface);
    if let Some(device_path) = sysfs.get_device_path() {
        if let Ok(uevent_content) = fs::read_to_string(device_path.join("uevent")) {
            for line in uevent_content.lines() {
                if line.starts_with("PCI_SLOT_NAME=") {
                    return line["PCI_SLOT_NAME=".len()..].to_string();
                }
            }
        }
    }
    "Unknown".to_string()
}

pub fn is_vlan(iface: &str) -> bool {
    let sysfs = SysfsInterface::new(iface);
    is_vlan_interface(&sysfs)
}
