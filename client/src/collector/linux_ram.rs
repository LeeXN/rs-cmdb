use common::entity::hardware::{RAMModule, RAM};
use std::collections::HashSet;
use std::fs;
use std::io::{self, Error};
use tracing::{debug, error};

// 以小端序从字节数组中读取 u16
fn word(data: &[u8], index: usize) -> u16 {
    u16::from_le_bytes([data[index], data[index + 1]])
}

// 以小端序从字节数组中读取 u32
fn dword(data: &[u8], index: usize) -> u32 {
    u32::from_le_bytes([
        data[index],
        data[index + 1],
        data[index + 2],
        data[index + 3],
    ])
}

// 以小端序从字节数组中读取 u64
fn qword(data: &[u8], index: usize) -> u64 {
    u64::from_le_bytes([
        data[index],
        data[index + 1],
        data[index + 2],
        data[index + 3],
        data[index + 4],
        data[index + 5],
        data[index + 6],
        data[index + 7],
    ])
}

// 从字符串数组中提取对应索引的字符串
fn get_string_value(dmi: &[u8], base_offset: usize, string_index: u8) -> Option<String> {
    if string_index == 0 {
        return None;
    }

    // 找到字符串表的开始位置
    let rec_len = dmi[base_offset + 1] as usize;
    let mut offset = base_offset + rec_len;

    // 跳过前面的字符串，找到第 string_index 个字符串
    let mut current_index = 1;
    while current_index < string_index && offset < dmi.len() {
        if dmi[offset] == 0 {
            current_index += 1;
            offset += 1;
            if current_index == string_index {
                break;
            }
        } else {
            offset += 1;
        }
    }

    // 提取字符串，直到遇到结束符（0）
    // 如果起始位置就是 0，说明遇到了双重 null 结尾，即字符串列表结束，该索引不存在
    if offset >= dmi.len() || dmi[offset] == 0 {
        return None;
    }

    let mut result = Vec::new();
    while offset < dmi.len() && dmi[offset] != 0 {
        result.push(dmi[offset]);
        offset += 1;
    }

    String::from_utf8(result).ok()
}

// 解析 /proc/meminfo 中的内存信息
fn get_meminfo_total() -> Option<u32> {
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(size_str) = line.split_whitespace().nth(1) {
                    if let Ok(size) = size_str.parse::<u32>() {
                        // MemTotal 单位是 kB，转换为 MB
                        return Some(size / 1024);
                    }
                }
            }
        }
    }
    None
}

// 获取常见内存厂商标识与全名映射
fn get_vendor_mapping() -> Vec<(&'static str, &'static str)> {
    vec![
        ("SMI", "Samsung"),
        ("HYN", "SK Hynix"),
        ("HYX", "SK Hynix"),
        ("HDT", "SK Hynix"),
        ("MIC", "Micron"),
        ("KIN", "Kingston"),
        ("KHX", "Kingston HyperX"),
        ("COR", "Corsair"),
        ("CRU", "Crucial"),
        ("KST", "Kingston"),
        ("ADA", "ADATA"),
        ("TEC", "Tectop"),
        ("GLW", "Geil"),
        ("PAT", "Patriot"),
        ("TRA", "Transcend"),
        ("VCX", "V-Color"),
        ("GSK", "G.Skill"),
        ("NEX", "Nanya"),
        ("WIN", "Winbond"),
        ("ESS", "Elpida"),
        ("KMG", "Kingston"),
    ]
}

// 获取内存形态映射
fn get_form_factor_mapping() -> Vec<(u8, &'static str)> {
    vec![
        (0x01, "Other"),
        (0x02, "Unknown"),
        (0x03, "SIMM"),
        (0x04, "SIP"),
        (0x05, "Chip"),
        (0x06, "DIP"),
        (0x07, "ZIP"),
        (0x08, "Proprietary Card"),
        (0x09, "DIMM"),
        (0x0A, "TSOP"),
        (0x0B, "Row of chips"),
        (0x0C, "RIMM"),
        (0x0D, "SODIMM"),
        (0x0E, "SRIMM"),
        (0x0F, "FB-DIMM"),
        (0x10, "Die"),
    ]
}

// 定义一个内部使用的 MemoryDevice 结构体，用于存储从 DMI 表中解析的信息
struct MemoryDevice {
    module_size: u32,         // 单个内存条大小（MB）
    total_size: u32,          // 总内存大小（MB）
    memory_type: String,      // 内存类型，如 DDR4
    speed: u32,               // 内存速度（MHz）
    number: u32,              // 内存条数量
    modules: Vec<ModuleInfo>, // 各个内存模块的详细信息
    part_number: String,      // 部件号
    form_factor: String,      // 内存形态
    serial_number: String,    // 序列号
}

// 内存模块详细信息
struct ModuleInfo {
    size: u32,             // 模块大小（MB）
    vendor: String,        // 制造商
    part_number: String,   // 部件号
    serial_number: String, // 序列号
    speed: u32,            // 频率（MHz）
    memory_type: String,   // 内存类型
    form_factor: String,   // 内存形态
    locator: String,       // 内存插槽位置
}

// 从 DMI 表中获取内存信息
fn get_memory_info() -> io::Result<MemoryDevice> {
    let mut memory_device = MemoryDevice {
        module_size: 0,
        total_size: 0,
        memory_type: String::new(),
        speed: 0,
        number: 0,
        modules: Vec::new(),
        part_number: String::new(),
        form_factor: String::new(),
        serial_number: String::new(),
    };

    // 读取 DMI 表数据
    let dmi = match fs::read("/sys/firmware/dmi/tables/DMI") {
        Ok(data) => data,
        Err(err) => {
            error!("Error reading DMI table: {}", err);

            // 尝试从 /proc/meminfo 获取总内存信息作为备选
            if let Some(total_mb) = get_meminfo_total() {
                memory_device.total_size = total_mb;
                memory_device.module_size = total_mb; // 无法确定单条大小，默认就是总大小
                memory_device.memory_type = "DRAM".to_string();
                memory_device.number = 1; // 无法确定内存条数量，默认为1
                return Ok(memory_device);
            }

            return Err(Error::other(format!("Error reading DMI table: {}", err)));
        }
    };

    let mut mem_size_alt: u32 = 0;
    let mut p = 0;
    let vendor_map = get_vendor_mapping();
    let form_factor_map = get_form_factor_mapping();

    // 解析 DMI 表
    while p < dmi.len() - 1 {
        let rec_type = dmi[p];
        let rec_len = dmi[p + 1] as usize;

        if rec_type == 127 {
            // End-of-Table，跳出解析
            break;
        }

        match rec_type {
            17 => {
                // Memory Device 结构类型
                if p + 0x0c + 1 >= dmi.len() {
                    break;
                }

                let size = word(&dmi, p + 0x0c) as u32;

                // 如果值为 0，表示插槽中没有安装内存设备
                // 如果值为 0xFFFF，表示大小未知
                if size == 0 || size == 0xffff {
                    p += rec_len;
                    // 跳过结束符
                    while p < dmi.len() - 1 {
                        if dmi[p] == 0 && dmi[p + 1] == 0 {
                            p += 2;
                            break;
                        }
                        p += 1;
                    }
                    continue;
                }

                let mut actual_size = size;
                if size == 0x7FFF && rec_len >= 0x20 && p + 0x1c + 3 < dmi.len() {
                    actual_size = dword(&dmi, p + 0x1c); // Extended size
                }

                let mut module_sizes: HashSet<u32> = HashSet::new();
                module_sizes.insert(actual_size);

                // 如果大小大于等于 32GB - 1MB，则字段值为 0x7FFF，
                // 实际大小存储在 Extended Size 字段中
                if size == 0x7fff && rec_len >= 0x20 && p + 0x1c + 3 < dmi.len() {
                    actual_size = dword(&dmi, p + 0x1c);
                }

                memory_device.module_size = if module_sizes.len() == 1 {
                    // Safe: we checked len == 1, so next() will return Some
                    *module_sizes.iter().next().expect("module_sizes has exactly one element")
                } else {
                    0
                };

                memory_device.total_size += actual_size;
                memory_device.number += 1;

                // 读取内存类型
                let memory_type = if p + 0x12 < dmi.len() {
                    // SMBIOS Reference Specification Version 3.2.0
                    // let mem_types = [
                    //     "Other", "Unknown", "DRAM", "EDRAM", "VRAM", "SRAM", "RAM", "ROM", "FLASH",
                    //     "EEPROM", "FEPROM", "EPROM", "CDRAM", "3DRAM", "SDRAM", "SGRAM", "RDRAM",
                    //     "DDR", "DDR2", "DDR2 FB-DIMM", "Reserved", "Reserved", "Reserved", "DDR3",
                    //     "FBD2", "DDR4", "LPDDR", "LPDDR2", "LPDDR3", "LPDDR4", "LPDDR5", "DDR5",
                    // ];

                    let mem_types = [
                        "Other",
                        "Unknown",
                        "DRAM",
                        "EDRAM",
                        "VRAM",
                        "SRAM",
                        "RAM",
                        "ROM",
                        "FLASH",
                        "EEPROM",
                        "FEPROM",
                        "EPROM",
                        "CDRAM",
                        "3DRAM",
                        "SDRAM",
                        "SGRAM",
                        "RDRAM",
                        "DDR",
                        "DDR2",
                        "DDR2 FB-DIMM",
                        "Reserved",
                        "Reserved",
                        "Reserved",
                        "DDR3",
                        "FBD2",
                        "DDR4",
                        "LPDDR",
                        "LPDDR2",
                        "LPDDR3",
                        "LPDDR4",
                        "Logical non-volatile device",
                        "HBM",
                        "HBM2",
                        "DDR5",
                        "LPDDR5",
                        "HBM3",
                    ];

                    let index = dmi[p + 0x12] as usize;
                    if index >= 1 && index <= mem_types.len() {
                        mem_types[index - 1].to_string()
                    } else {
                        "Unknown".to_string()
                    }
                } else {
                    "Unknown".to_string()
                };

                if memory_device.memory_type.is_empty() {
                    memory_device.memory_type = memory_type.clone();
                }

                // 读取内存速度
                let speed = if rec_len >= 0x17 && p + 0x15 + 1 < dmi.len() {
                    let spd = word(&dmi, p + 0x15) as u32;
                    if memory_device.speed == 0 && spd != 0 {
                        memory_device.speed = spd;
                    }
                    spd
                } else {
                    0
                };

                // 读取内存形态
                let form_factor = if p + 0x0e < dmi.len() {
                    let form_id = dmi[p + 0x0e];
                    let form_name = form_factor_map
                        .iter()
                        .find(|&&(id, _)| id == form_id)
                        .map(|&(_, name)| name)
                        .unwrap_or("Unknown");

                    if memory_device.form_factor.is_empty() {
                        memory_device.form_factor = form_name.to_string();
                    }
                    form_name.to_string()
                } else {
                    "Unknown".to_string()
                };

                // 读取制造商
                let vendor = if p + 0x17 < dmi.len() {
                    // 0x17 字节是制造商字符串的索引
                    let vendor_idx = dmi[p + 0x17];
                    get_string_value(&dmi, p, vendor_idx).unwrap_or_else(|| "Unknown".to_string())
                } else {
                    "Unknown".to_string()
                };

                // 读取序列号
                let serial_number = if p + 0x18 < dmi.len() {
                    // 0x18 字节是序列号字符串的索引
                    let serial_idx = dmi[p + 0x18];
                    let sn = get_string_value(&dmi, p, serial_idx)
                        .unwrap_or_else(|| "Unknown".to_string());
                    if memory_device.serial_number.is_empty() && is_valid_string(&sn) {
                        memory_device.serial_number = sn.clone();
                    }
                    sn
                } else {
                    "Unknown".to_string()
                };

                // 读取部件号
                let part_number = if p + 0x1a < dmi.len() {
                    // 0x1A 字节是部件号字符串的索引
                    let part_idx = dmi[p + 0x1a];
                    let pn = get_string_value(&dmi, p, part_idx)
                        .unwrap_or_else(|| "Unknown".to_string());
                    if memory_device.part_number.is_empty() && is_valid_string(&pn) {
                        memory_device.part_number = pn.clone();
                    }
                    pn
                } else {
                    "Unknown".to_string()
                };

                // 读取内存位置
                let locator = if p + 0x10 < dmi.len() {
                    // 0x10 字节是内存位置字符串的索引
                    let locator_idx = dmi[p + 0x10];
                    get_string_value(&dmi, p, locator_idx).unwrap_or_else(|| "Unknown".to_string())
                } else {
                    "Unknown".to_string()
                };

                // 读取内存条插槽标识
                let device_locator = if p + 0x11 < dmi.len() {
                    // 0x11 字节是设备位置字符串的索引
                    let device_locator_idx = dmi[p + 0x11];
                    get_string_value(&dmi, p, device_locator_idx)
                        .unwrap_or_else(|| "Unknown".to_string())
                } else {
                    "Unknown".to_string()
                };

                // 添加到模块列表
                memory_device.modules.push(ModuleInfo {
                    size: actual_size,
                    vendor,
                    part_number,
                    serial_number,
                    speed,
                    memory_type,
                    form_factor,
                    locator: if device_locator != "Unknown" {
                        device_locator
                    } else {
                        locator
                    },
                });
            }
            19 => {
                // Memory Array Mapped Address 结构类型
                if p + 0x08 + 3 >= dmi.len() {
                    break;
                }

                let start = dword(&dmi, p + 0x04) as u32; // 起始地址(以千字节为单位)
                let end = dword(&dmi, p + 0x08) as u32; // 结束地址(以千字节为单位)

                if start == 0xffffffff && end == 0xffffffff {
                    // 0xffffffff 表示内存大小存储在 0x0f-0x16 和 0x17-0x1e
                    if rec_len >= 0x1f && p + 0x17 + 7 < dmi.len() {
                        let start64 = qword(&dmi, p + 0x0f); // 扩展起始地址(以字节为单位)
                        let end64 = qword(&dmi, p + 0x17); // 扩展结束地址(以字节为单位)
                        mem_size_alt += ((end64 - start64 + 1) / 1024 / 1024) as u32;
                        // 转换为 MB
                    }
                } else {
                    mem_size_alt += end - start + 1
                }
            }
            _ => {}
        }

        // 跳过当前记录，继续解析下一个记录
        p += rec_len;

        // 跳过结束符
        while p < dmi.len() - 1 {
            if dmi[p] == 0 && dmi[p + 1] == 0 {
                p += 2;
                break;
            }
            p += 1;
        }
    }

    // 如果 DMI type 17 没有信息，我们可以回退到 DMI type 19，至少获取到内存大小
    if memory_device.module_size == 0 && mem_size_alt > 0 {
        memory_device.memory_type = "DRAM".to_string();
        memory_device.module_size = mem_size_alt;
        memory_device.total_size = mem_size_alt;
        memory_device.number = 1; // 无法确定内存条数量，默认为1
    }

    // 如果上述方法都失败，尝试从 /proc/meminfo 获取总内存
    if memory_device.total_size == 0 {
        if let Some(total_mb) = get_meminfo_total() {
            memory_device.total_size = total_mb;
            memory_device.module_size = total_mb; // 无法确定单条大小，默认就是总大小
            memory_device.memory_type = "DRAM".to_string();
            memory_device.number = 1; // 无法确定内存条数量，默认为1
        }
    }

    // 标准化内存厂商名称
    for module in &mut memory_device.modules {
        let trimmed_vendor = module.vendor.trim();

        // 尝试匹配已知的厂商代码
        for (code, full_name) in &vendor_map {
            if trimmed_vendor.starts_with(code)
                || trimmed_vendor.contains(code)
                || module.part_number.starts_with(code)
            {
                module.vendor = full_name.to_string();
                break;
            }
        }
    }

    debug!(
        "Memory info: Type={}, Size={}MB, Speed={}MHz, Count={}",
        memory_device.memory_type,
        memory_device.total_size,
        memory_device.speed,
        memory_device.number
    );

    Ok(memory_device)
}

// 收集内存信息的公共函数
pub fn collect_ram() -> RAM {
    match get_memory_info() {
        Ok(memory) => {
            // 确定厂商信息
            let mut vendor = "Unknown".to_string();

            // 如果有内存模块信息，使用第一个模块的厂商信息
            if !memory.modules.is_empty() {
                vendor = memory.modules[0].vendor.clone();

                // 如果多个模块的厂商相同，则直接使用该厂商
                // 如果不同，则使用逗号分隔列出不同的厂商
                let mut all_same = true;
                for module in &memory.modules[1..] {
                    if module.vendor != vendor {
                        all_same = false;
                        break;
                    }
                }

                if !all_same {
                    // 收集所有不同的厂商
                    let mut vendors = std::collections::HashSet::new();
                    for module in &memory.modules {
                        vendors.insert(module.vendor.clone());
                    }

                    // 将不同的厂商用逗号分隔
                    vendor = vendors.into_iter().collect::<Vec<String>>().join(", ");
                }
            }

            // 将 MB 转换为 GB
            let size_gb = (memory.module_size + 512) / 1024; // 四舍五入转换为 GB
            let total_size_gb = (memory.total_size + 512) / 1024; // 四舍五入转换为 GB

            // 转换 ModuleInfo 为 RAMModule
            let modules = memory
                .modules
                .iter()
                .map(|m| {
                    RAMModule {
                        slot: m.locator.clone(),
                        vendor: m.vendor.clone(),
                        part_number: m.part_number.clone(),
                        serial_number: m.serial_number.clone(),
                        size: (m.size + 512) / 1024, // 转换为 GB
                        speed: m.speed,
                        form_factor: m.form_factor.clone(),
                        memory_type: m.memory_type.clone(),
                        locator: m.locator.clone(),
                    }
                })
                .collect();

            RAM {
                vendor,
                model: memory.memory_type,
                size: size_gb, // 单根内存大小，单位为 GB
                speed: memory.speed,
                total_size: total_size_gb, // 总内存大小，单位为 GB
                count: memory.number,      // 内存条数量
                form_factor: memory.form_factor,
                modules,
            }
        }
        Err(err) => {
            error!("Failed to get memory info: {}", err);

            // 尝试从 /proc/meminfo 获取总内存作为备选
            if let Some(total_mb) = get_meminfo_total() {
                let total_size_gb = (total_mb + 512) / 1024; // 四舍五入转换为 GB

                return RAM {
                    vendor: "Unknown".to_string(),
                    model: "DRAM".to_string(),
                    size: total_size_gb, // 无法确定单条大小，默认等于总大小
                    speed: 0,
                    total_size: total_size_gb,
                    count: 1, // 无法确定内存条数量，默认为1
                    form_factor: "Unknown".to_string(),
                    modules: Vec::new(),
                };
            }

            // 返回默认值
            RAM {
                vendor: "Unknown".to_string(),
                model: "Unknown".to_string(),
                size: 0,
                speed: 0,
                total_size: 0,
                count: 0,
                form_factor: "Unknown".to_string(),
                modules: Vec::new(),
            }
        }
    }
}

fn is_valid_string(s: &str) -> bool {
    let s = s.trim();
    !(s.is_empty()
        || s == "0"
        || s == "00000000"
        || s.eq_ignore_ascii_case("unknown")
        || s.eq_ignore_ascii_case("not specified"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word() {
        let data = [0x01, 0x02];
        assert_eq!(word(&data, 0), 0x0201);
    }

    #[test]
    fn test_dword() {
        let data = [0x01, 0x02, 0x03, 0x04];
        assert_eq!(dword(&data, 0), 0x04030201);
    }

    #[test]
    fn test_qword() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        assert_eq!(qword(&data, 0), 0x0807060504030201);
    }

    #[test]
    fn test_get_string_value() {
        // DMI structure header: type(1), length(1), handle(2)
        // Followed by formatted section
        // Followed by strings section (null terminated)
        // Ends with double null

        // Let's simulate a simple structure
        // Header: Type 1, Length 4, Handle 0x0001
        // Strings: "String1\0String2\0\0"
        let mut dmi_data = vec![0x01, 0x04, 0x01, 0x00]; // Header
        dmi_data.extend_from_slice(b"String1\0");
        dmi_data.extend_from_slice(b"String2\0");
        dmi_data.push(0); // End of strings

        assert_eq!(
            get_string_value(&dmi_data, 0, 1),
            Some("String1".to_string())
        );
        assert_eq!(
            get_string_value(&dmi_data, 0, 2),
            Some("String2".to_string())
        );
        assert_eq!(get_string_value(&dmi_data, 0, 3), None); // Out of bounds
        assert_eq!(get_string_value(&dmi_data, 0, 0), None); // Invalid index
    }

    #[test]
    fn test_is_valid_string() {
        assert!(is_valid_string("Samsung"));
        assert!(is_valid_string("  Samsung  "));
        assert!(!is_valid_string(""));
        assert!(!is_valid_string("   "));
        assert!(!is_valid_string("0"));
        assert!(!is_valid_string("00000000"));
        assert!(!is_valid_string("Unknown"));
        assert!(!is_valid_string("unknown"));
        assert!(!is_valid_string("Not Specified"));
    }
}
