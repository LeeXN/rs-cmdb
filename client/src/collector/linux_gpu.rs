use common::entity::hardware::GPU;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::sync::OnceLock;

const DEVICE_ID_PATH: &str = "/sys/bus/pci/devices/";
// 定义已知的GPU厂商ID
const GPU_VENDOR_IDS: &[(u16, &str)] = &[
    (0x1002, "AMD"),
    (0x10de, "NVIDIA"),
    (0x1d22, "Baidu"),
    (0x1e3e, "Iluvatar"),
];
const BLACK_LIST_GPU_VENDOR_IDS: &[(u16, &str)] = &[
    (0x102b, "Matrox"),
    (0x1a03, "ASPEED"),
    (0x1db7, "Phytium"), // Phytium E2000s is bmc
    (0x1234, "qemu"),
];

const NVIDIA_DRIVER_VERSION_CMD: &str = "nvidia-smi";
const ILUVATAR_DRIVER_VERSION_CMD: &str = "ixsmi";
const KUNLUN_DRIVER_VERSION_CMD: &str = "xpu_smi";

const NVIDIA_SN_CMD: &str = "nvidia-smi --query-gpu=pci.bus_id,serial --format=csv";
const ILUVATAR_SN_CMD: &str = "ixsmi --query-gpu=pci.bus_id,serial --format=csv";
const KUNLUN_SN_CMD: &str = "xpu_smi -m";

const KUNLUN_GPU_DEVICE_IDS: &[(u16, &str)] = &[
    (0x3685, "Kunlun2 AI Accelerator [VF]"),
    (0x3684, "Kunlun AI Accelerator"),
    (0x3688, "P800"),
];

const ILUVATAR_GPU_DEVICE_IDS: &[(u16, &str)] = &[(0x0003, "BI-V150")];

// 用于缓存 SMI 命令输出的结构
struct SmiCache {
    nvidia_smi_output: Option<String>,
    iluvatar_smi_output: Option<String>,
    kunlun_smi_output: Option<String>,
    nvidia_driver_output: Option<String>,
    iluvatar_driver_output: Option<String>,
    kunlun_driver_output: Option<String>,
}

impl SmiCache {
    fn new() -> Self {
        SmiCache {
            nvidia_smi_output: None,
            iluvatar_smi_output: None,
            kunlun_smi_output: None,
            nvidia_driver_output: None,
            iluvatar_driver_output: None,
            kunlun_driver_output: None,
        }
    }

    fn get_smi_output(&mut self, gpu_vendor_name: &str) -> Option<&String> {
        match gpu_vendor_name {
            "NVIDIA" => {
                if self.nvidia_smi_output.is_none() {
                    self.nvidia_smi_output = run_command(NVIDIA_SN_CMD);
                }
                self.nvidia_smi_output.as_ref()
            }
            "Iluvatar" => {
                if self.iluvatar_smi_output.is_none() {
                    self.iluvatar_smi_output = run_command(ILUVATAR_SN_CMD);
                }
                self.iluvatar_smi_output.as_ref()
            }
            "Baidu" => {
                if self.kunlun_smi_output.is_none() {
                    self.kunlun_smi_output = run_command(KUNLUN_SN_CMD);
                }
                self.kunlun_smi_output.as_ref()
            }
            _ => None,
        }
    }

    fn get_driver_output(&mut self, gpu_vendor_name: &str) -> Option<&String> {
        match gpu_vendor_name {
            "NVIDIA" => {
                if self.nvidia_driver_output.is_none() {
                    let smi_output = run_command(NVIDIA_DRIVER_VERSION_CMD)?;
                    let driver_output = grep_driver_version(&smi_output)?;
                    self.nvidia_driver_output = Some(driver_output);
                }
                self.nvidia_driver_output.as_ref()
            }
            "Iluvatar" => {
                if self.iluvatar_driver_output.is_none() {
                    let smi_output = run_command(ILUVATAR_DRIVER_VERSION_CMD)?;
                    let driver_output = grep_driver_version(&smi_output)?;
                    self.iluvatar_driver_output = Some(driver_output);
                }
                self.iluvatar_driver_output.as_ref()
            }
            "Baidu" => {
                if self.kunlun_driver_output.is_none() {
                    let smi_output = run_command(KUNLUN_DRIVER_VERSION_CMD)?;
                    let driver_output = grep_driver_version(&smi_output)?;
                    self.kunlun_driver_output = Some(driver_output);
                }
                self.kunlun_driver_output.as_ref()
            }
            _ => None,
        }
    }
}

// 全局缓存实例
static SMI_CACHE: OnceLock<Mutex<SmiCache>> = OnceLock::new();

fn get_cache() -> &'static Mutex<SmiCache> {
    SMI_CACHE.get_or_init(|| Mutex::new(SmiCache::new()))
}

// 执行命令并返回输出
fn run_command(cmd: &str) -> Option<String> {
    Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
}

// 从输出中提取驱动版本信息
fn grep_driver_version(output: &str) -> Option<String> {
    let mut result = String::new();
    for line in output.lines() {
        if line.contains("Driver Version:") {
            result.push_str(line);
            result.push('\n');
        }
    }
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn get_special_gpu_name(gpu_vendor_name: &str, device_id: u16) -> Option<String> {
    match gpu_vendor_name {
        "NVIDIA" => KUNLUN_GPU_DEVICE_IDS
            .iter()
            .find(|&&(id, _)| id == device_id)
            .map(|&(_, name)| name.to_string()),
        "Iluvatar" => ILUVATAR_GPU_DEVICE_IDS
            .iter()
            .find(|&&(id, _)| id == device_id)
            .map(|&(_, name)| name.to_string()),
        "Baidu" => KUNLUN_GPU_DEVICE_IDS
            .iter()
            .find(|&&(id, _)| id == device_id)
            .map(|&(_, name)| name.to_string()),
        _ => None,
    }
}

// 从pci.ids文件中获取设备名称
fn get_device_name_from_pci_ids(vendor_id: u16, device_id: u16) -> Option<String> {
    let pci_ids_paths = [
        "/usr/share/hwdata/pci.ids",
        "/usr/share/misc/pci.ids",
        "/usr/local/share/pci.ids",
    ];

    for path in pci_ids_paths {
        if let Ok(content) = fs::read_to_string(path) {
            let mut current_vendor = None;
            for line in content.lines() {
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                if !line.starts_with('\t') {
                    // Vendor line
                    if let Some((id_str, name)) = line.split_once(' ') {
                        if let Ok(id) = u16::from_str_radix(id_str, 16) {
                            if id == vendor_id {
                                current_vendor = Some(name.trim());
                            } else {
                                current_vendor = None;
                            }
                        }
                    }
                } else if line.starts_with('\t') && !line.starts_with("\t\t") {
                    // Device line
                    if current_vendor.is_some() {
                        let line = line.trim();
                        if let Some((id_str, name)) = line.split_once(' ') {
                            if let Ok(id) = u16::from_str_radix(id_str, 16) {
                                if id == device_id {
                                    return Some(name.trim().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

// 获取PCI设备的sysfs路径
fn get_pci_sysfs_paths() -> Vec<PathBuf> {
    let pci_devices_path = Path::new(DEVICE_ID_PATH);
    if let Ok(entries) = fs::read_dir(pci_devices_path) {
        entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect()
    } else {
        Vec::new()
    }
}

// 读取sysfs文件的值
fn read_sysfs_value(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

// 检查是否为显示控制器
fn is_display_controller(path: &Path) -> bool {
    // 先检查是否在黑名单中
    if let Some(vendor_id_str) = read_sysfs_value(&path.join("vendor")) {
        if let Ok(vendor_id) = u16::from_str_radix(&vendor_id_str[2..], 16) {
            if BLACK_LIST_GPU_VENDOR_IDS
                .iter()
                .any(|(id, _)| *id == vendor_id)
            {
                return false;
            }
        }
    }

    let vendor_id = if let Some(vendor_id_str) = read_sysfs_value(&path.join("vendor")) {
        u16::from_str_radix(&vendor_id_str[2..], 16).unwrap()
    } else {
        0
    };
    // 然后检查 class 是否匹配已知的 GPU 类型
    if let Some(class) = read_sysfs_value(&path.join("class")) {
        // 0x03xxxx - 标准显示控制器
        // 0x078000 - 昆仑芯 P800 GPU
        // 0x120000 - 其他处理器
        class.starts_with("0x03")
            || class == "0x120000"
            || (class == "0x078000" && vendor_id == 0x1d22)
    } else {
        false
    }
}

// 修改 get_gpu_sn 函数使用缓存
fn get_gpu_sn(gpu_vendor_name: &str, original_pci_bus_id: &str) -> Option<String> {
    let cache = get_cache();
    let output = match cache.lock().unwrap().get_smi_output(gpu_vendor_name) {
        Some(output) => output.clone(),
        None => return None,
    };

    let lines: Vec<&str> = output.lines().collect();
    if !lines.is_empty() {
        for line in lines.iter() {
            if line.trim().is_empty() {
                continue;
            }

            // 提取 PCI ID 和序列号
            let pci_id = if gpu_vendor_name == "Baidu" {
                line.split(" ").next().unwrap_or("")
            } else {
                line.split(",").next().unwrap_or("")
            };

            if pci_id.is_empty() {
                continue;
            }

            let pci_id_without_prefix = pci_id.split(":").collect::<Vec<&str>>()[1..].join(":");
            let original_pci_bus_id_without_prefix =
                original_pci_bus_id.split(":").collect::<Vec<&str>>()[1..].join(":");

            // 获取序列号
            let sn = if gpu_vendor_name == "Baidu" {
                line.split(" ").nth(3).unwrap_or("").trim()
            } else {
                line.split(",").nth(1).unwrap_or("").trim()
            };

            if original_pci_bus_id_without_prefix.to_uppercase()
                == pci_id_without_prefix.to_uppercase()
            {
                return Some(sn.to_string());
            }
        }
    }
    None
}

// 修改 get_driver_version 函数使用缓存
fn get_driver_version(gpu_vendor_name: &str) -> Option<String> {
    let cache = get_cache();
    let output = match cache.lock().unwrap().get_driver_output(gpu_vendor_name) {
        Some(output) => output.clone(),
        None => return None,
    };

    let lines: Vec<&str> = output.lines().collect();
    if !lines.is_empty() {
        let line = lines[0].trim();
        // 提取驱动版本号，寻找"Driver Version:"后面的内容
        if let Some(pos) = line.find("Driver Version:") {
            // 从"Driver Version:"后开始
            let version_part = &line[pos + "Driver Version:".len()..];
            // 提取版本号直到下一个空格序列
            if let Some(ver) = version_part.split_whitespace().next() {
                return Some(ver.to_string());
            }
        }
    }
    None
}

pub fn collect_gpus() -> Vec<GPU> {
    let mut gpus = Vec::new();
    for path in get_pci_sysfs_paths() {
        if is_display_controller(&path) {
            if let Some(vendor_id_str) = read_sysfs_value(&path.join("vendor")) {
                if let Some(device_id_str) = read_sysfs_value(&path.join("device")) {
                    if let (Ok(vendor_id), Ok(device_id)) = (
                        u16::from_str_radix(&vendor_id_str[2..], 16),
                        u16::from_str_radix(&device_id_str[2..], 16),
                    ) {
                        if BLACK_LIST_GPU_VENDOR_IDS
                            .iter()
                            .any(|(id, _)| *id == vendor_id)
                        {
                            continue;
                        }
                        let vendor_name = GPU_VENDOR_IDS
                            .iter()
                            .find(|&&(id, _)| id == vendor_id)
                            .map(|&(_, name)| name.to_string())
                            .unwrap_or_else(|| format!("Unknown Vendor ({:04x})", vendor_id));

                        let mut model = get_device_name_from_pci_ids(vendor_id, device_id)
                            .unwrap_or_else(|| format!("Unknown Model ({:04x})", device_id));
                        if model.starts_with("Unknown Model") {
                            let special_gpu_name = get_special_gpu_name(&vendor_name, device_id);
                            if let Some(special_gpu_name) = special_gpu_name {
                                model = special_gpu_name;
                            }
                        }
                        let pci_bus_id = path
                            .to_string_lossy()
                            .split("/")
                            .nth(5)
                            .unwrap()
                            .to_string();
                        let sn = get_gpu_sn(&vendor_name, &pci_bus_id);
                        let driver_version = get_driver_version(&vendor_name).unwrap_or_default();
                        gpus.push(GPU {
                            vendor: vendor_name,
                            model,
                            device_id: pci_bus_id.to_string(),
                            serial_number: sn.unwrap_or_default(),
                            driver_version,
                        });
                    }
                }
            }
        }
    }
    gpus
}
