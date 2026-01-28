use crate::collector::linux_collector::{
    collect_cpu_info, collect_disk_info, collect_gpu_info, collect_nic_info, collect_os_info,
    collect_ram_info, collect_system_info, get_cores_per_cpu, get_threads_per_core,
};
use crate::collector::linux_ipmi;
use std::collections::HashSet;

/// 支持的硬件类型
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum HardwareType {
    OS,
    CPU,
    Disk,
    GPU,
    NIC,
    RAM,
    SYS,
    IPMI,
    All,
}

/// 命令行选项
pub struct DisplayOptions {
    pub hardware_types: HashSet<HardwareType>,
    pub show_detail: bool,
}

/// 打印硬件信息
pub fn print_hardware_info(options: &DisplayOptions) {
    // 打印系统信息
    if options.hardware_types.contains(&HardwareType::All)
        || options.hardware_types.contains(&HardwareType::SYS)
    {
        print_system_info();
    }
    // 打印操作系统信息
    if options.hardware_types.contains(&HardwareType::All)
        || options.hardware_types.contains(&HardwareType::OS)
    {
        print_os_info();
    }

    // 打印CPU信息
    if options.hardware_types.contains(&HardwareType::All)
        || options.hardware_types.contains(&HardwareType::CPU)
    {
        print_cpu_info();
    }

    // 打印磁盘信息
    if options.hardware_types.contains(&HardwareType::All)
        || options.hardware_types.contains(&HardwareType::Disk)
    {
        print_disk_info();
    }

    // 打印网卡信息
    if options.hardware_types.contains(&HardwareType::All)
        || options.hardware_types.contains(&HardwareType::NIC)
    {
        print_nic_info();
    }

    // 打印GPU信息
    if options.hardware_types.contains(&HardwareType::All)
        || options.hardware_types.contains(&HardwareType::GPU)
    {
        print_gpu_info();
    }

    // 打印内存信息
    if options.hardware_types.contains(&HardwareType::All)
        || options.hardware_types.contains(&HardwareType::RAM)
    {
        print_ram_info(options.show_detail);
    }

    // 打印IPMI信息
    if options.hardware_types.contains(&HardwareType::All)
        || options.hardware_types.contains(&HardwareType::IPMI)
    {
        print_ipmi_info();
    }
}

/// 打印操作系统信息
fn print_os_info() {
    let os = collect_os_info();

    println!("\n{}", "=".repeat(80));
    println!("{:^80}", "Operating System Information");
    println!("{}", "=".repeat(80));
    println!("System Name:    {}", os.name);
    println!("System Version: {}", os.version);
    println!("Kernel Version: {}", os.kernel);
    println!("Architecture:   {}", os.architecture);
    println!("Hostname:       {}", os.hostname);
    println!("IP Address:     {}", os.ip_address);
    println!("DNS Servers:    {}", os.dns);
}

fn print_system_info() {
    let system = collect_system_info();
    println!("\n{}", "=".repeat(80));
    println!("{:^80}", "System Information");
    println!("{}", "=".repeat(80));
    println!("System Vendor:          {}", system.sys_vendor);
    println!("System Product Name:    {}", system.product_name);
    println!("System Serial Number:   {}", system.serial_number);
    println!("System Product Version: {}", system.product_version);
}

/// 打印CPU信息
fn print_cpu_info() {
    let cpu = collect_cpu_info();

    if cpu.threads == 0 {
        println!("\n{}", "=".repeat(80));
        println!("{:^80}", "CPU Information");
        println!("{}", "=".repeat(80));
        println!("No CPU information detected");
        return;
    }

    println!("\n{}", "=".repeat(80));
    println!("{:^80}", "CPU Information");
    println!("{}", "=".repeat(80));
    println!("Vendor ID:      {}", cpu.vendor_id);
    println!("Model Name:     {}", cpu.model_name);
    println!("CPU Count:      {}", cpu.cpus);
    println!("Core Count:     {}", cpu.cores);
    println!("Thread Count:   {}", cpu.threads);
    println!("CPU Speed:      {} MHz", cpu.speed);

    if !cpu.flags.is_empty() {
        println!("CPU Flags:      {}", cpu.flags.join(" "));
    }

    println!("\nThreads per Core: {}", get_threads_per_core());
    println!("Cores per CPU:    {}", get_cores_per_cpu());
}

/// 打印磁盘信息
fn print_disk_info() {
    let disks = collect_disk_info();

    println!("\n{}", "=".repeat(80));
    println!("{:^80}", "Disk Information");
    println!("{}", "=".repeat(80));

    if disks.is_empty() {
        println!("No disk information detected");
        return;
    }

    println!(
        "{:<15} {:<15} {:<12} {:<20} {:<15}",
        "Vendor", "Model", "Capacity", "Type", "Firmware"
    );
    println!("{}", "-".repeat(80));

    for disk in &disks {
        let disk_type = match disk.storage_type {
            common::entity::hardware::StorageType::SSD => "SSD",
            common::entity::hardware::StorageType::HDD => "HDD",
            common::entity::hardware::StorageType::NVMe => "NVMe",
            common::entity::hardware::StorageType::Unknown => "Unknown",
        };

        println!(
            "{:<15} {:<15} {:<10} {:<2} {:<20} {:<15}",
            disk.vendor, disk.model, disk.size, disk.size_unit, disk_type, disk.firmware_version
        );

        // 打印分区信息
        if disk.parted && !disk.partitions.is_empty() {
            println!("  Partition Information:");
            println!("  {:<15} {:<10} {:<5}", "Name", "Size", "Unit");
            for partition in &disk.partitions {
                println!(
                    "  {:<15} {:<10} {:<5}",
                    partition.name, partition.size, partition.size_unit
                );
            }
        }
    }
}

/// 打印网卡信息
fn print_nic_info() {
    if let Ok(nics) = collect_nic_info() {
        println!("\n{}", "=".repeat(80));
        println!("{:^80}", "Network Interface Information");
        println!("{}", "=".repeat(80));

        if nics.is_empty() {
            println!("No network interface information detected");
            return;
        }

        for nic in &nics {
            // 网卡基本信息
            println!("Interface Name:   {}", nic.name);
            if nic.nic_type != common::entity::hardware::NICType::Bonding
                && nic.nic_type != common::entity::hardware::NICType::VLAN
            {
                println!("Vendor:           {}", nic.vendor);
                println!("Model:            {}", nic.model);
            }
            println!("MAC Address:      {}", nic.mac_address);
            if let Some(ref pci_slot) = nic.pci_slot {
                if !pci_slot.is_empty() {
                    println!("PCI Slot:         {}", pci_slot);
                }
            }
            if !nic.driver.is_empty() {
                println!("Driver:           {}", nic.driver);
            }
            if !nic.firmware_version.is_empty() {
                println!("Firmware Version: {}", nic.firmware_version);
            }

            let status = match nic.status {
                common::entity::hardware::NICStatus::Up => "Up",
                common::entity::hardware::NICStatus::Down => "Down",
                common::entity::hardware::NICStatus::Unknown => "Unknown",
            };

            println!("NIC Type:         {}", nic.nic_type);
            if nic.ib_node_type != "Unknown" && !nic.ib_node_type.is_empty() {
                println!("IB Node Type:     {}", nic.ib_node_type);
            }
            println!("Link Status:      {}", status);
            println!("Link Speed:       {} Mbps", nic.speed);

            // IPv4信息
            if !nic.ipv4_address.is_empty() {
                println!("IPv4 Address:     {}", nic.ipv4_address);
                if !nic.ipv4_subnet_mask.is_empty() {
                    println!("IPv4 Subnet:      {}", nic.ipv4_subnet_mask);
                }
                if !nic.ipv4_gateway.is_empty() {
                    println!("IPv4 Gateway:     {}", nic.ipv4_gateway);
                }
            }

            // IPv6信息
            if !nic.ipv6_address.is_empty() {
                println!("IPv6 Address:     {}", nic.ipv6_address);
                if !nic.ipv6_subnet_mask.is_empty() {
                    println!("IPv6 Subnet:      {}", nic.ipv6_subnet_mask);
                }
                if !nic.ipv6_gateway.is_empty() {
                    println!("IPv6 Gateway:     {}", nic.ipv6_gateway);
                }
            }

            // 绑定信息
            if !nic.bonding_slaves.is_empty() {
                println!("Bonded Devices:   {}", nic.bonding_slaves.join(" "));
            }

            println!("{}", "-".repeat(80));
        }
    } else {
        println!("\n{}", "=".repeat(80));
        println!("{:^80}", "Network Interface Information");
        println!("{}", "=".repeat(80));
        println!("Failed to retrieve network interface information");
    }
}

/// 打印GPU信息
fn print_gpu_info() {
    let gpus = collect_gpu_info();
    println!("\n{}", "=".repeat(80));
    println!("{:^80}", "GPU Information");
    println!("{}", "=".repeat(80));
    if gpus.is_empty() {
        println!("No GPU information detected");
        return;
    }
    for gpu in &gpus {
        println!("Vendor:         {}", gpu.vendor);
        println!("Model:          {}", gpu.model);
        println!("Device ID:      {}", gpu.device_id);
        println!("Serial Number:  {}", gpu.serial_number);
        println!("Driver Version: {}", gpu.driver_version);
        println!("{}", "-".repeat(80));
    }
}

/// 打印内存信息
fn print_ram_info(show_detail: bool) {
    let ram = collect_ram_info();
    println!("\n{}", "=".repeat(80));
    println!("{:^80}", "RAM Information");
    println!("{}", "=".repeat(80));

    // 检测是否为虚拟机环境
    let is_vm = is_virtual_machine();

    // 总体内存信息
    println!(
        "Total Memory:    {} GB ({} modules)",
        ram.total_size, ram.count
    );

    if !ram.model.is_empty() && ram.model != "Unknown" {
        println!("Memory Type:     {}", ram.model);
    }

    if !ram.vendor.is_empty() && ram.vendor != "Unknown" {
        println!("Vendor:          {}", ram.vendor);
    }

    // 只有在非虚拟机环境或有效值时才显示速度
    if !is_vm && ram.speed > 0 {
        println!("Speed:           {} MHz", ram.speed);
    }

    if show_detail {
        if !ram.form_factor.is_empty() && ram.form_factor != "Unknown" {
            println!("Form Factor:     {}", ram.form_factor);
        }

        // 如果有内存模块，显示详细列表（即使只有一个模块）
        if !ram.modules.is_empty() {
            println!("\n{:^80}", "Memory Modules Detail");
            println!("{}", "-".repeat(80));

            // 打印表头
            println!(
                "{:<12} {:<8} {:<6} {:<24} {:<20}",
                "Slot", "Size(GB)", "Type", "Part Number", "Serial Number"
            );
            println!("{}", "-".repeat(80));

            // 打印每个模块的信息
            for module in &ram.modules {
                // 确保字符串被适当裁剪，以防止超出宽度影响对齐
                let part_number = if module.part_number.len() > 24 {
                    &module.part_number[0..24]
                } else {
                    &module.part_number
                };

                let serial_number = if module.serial_number.len() > 20 {
                    &module.serial_number[0..20]
                } else {
                    &module.serial_number
                };

                println!(
                    "{:<12} {:<8} {:<6} {:<24} {:<20}",
                    module.slot, module.size, module.memory_type, part_number, serial_number
                );
            }
        }
    }

    println!("{}", "=".repeat(80));
}

/// 检测是否为虚拟机环境
fn is_virtual_machine() -> bool {
    // 检查常见的虚拟机标志
    if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
        if content.contains("QEMU") || content.contains("KVM") || content.contains("VMware") {
            return true;
        }
    }

    // 检查 DMI 信息中的制造商
    if let Ok(content) = std::fs::read_to_string("/sys/class/dmi/id/sys_vendor") {
        let vendor = content.trim().to_lowercase();
        if vendor.contains("qemu")
            || vendor.contains("vmware")
            || vendor.contains("virtualbox")
            || vendor.contains("xen")
        {
            return true;
        }
    }

    // 检查 hypervisor 标志
    if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
        if content.contains("hypervisor") {
            return true;
        }
    }

    false
}

/// 打印IPMI信息
fn print_ipmi_info() {
    println!("\n{}", "=".repeat(80));
    println!("{:^80}", "IPMI/BMC Information");
    println!("{}", "=".repeat(80));

    match linux_ipmi::collect_ipmi_info() {
        Ok(Some(ipmi)) => {
            // 显示IPMI状态
            let status_text = match ipmi.status {
                common::entity::hardware::IpmiStatus::Available => "Available",
                common::entity::hardware::IpmiStatus::NotConfigured => "Not Configured",
                common::entity::hardware::IpmiStatus::NotAvailable => "Not Available",
                common::entity::hardware::IpmiStatus::AccessDenied => "Access Denied",
                common::entity::hardware::IpmiStatus::Error(ref msg) => {
                    println!("IPMI Status:    Error - {}", msg);
                    return;
                }
            };

            println!("IPMI Status:    {}", status_text);

            if let common::entity::hardware::IpmiStatus::Available = ipmi.status {
                // 显示网络配置
                if let Some(ref ip) = ipmi.ip_address {
                    println!("IP Address:     {}", ip);
                }
                if let Some(ref mac) = ipmi.mac_address {
                    println!("MAC Address:    {}", mac);
                }
                if let Some(ref subnet) = ipmi.subnet_mask {
                    println!("Subnet Mask:    {}", subnet);
                }
                if let Some(ref gateway) = ipmi.gateway {
                    println!("Gateway:        {}", gateway);
                }

                println!("Channel:        {}", ipmi.channel);

                // 显示设备信息
                if let Some(ref device_id) = ipmi.device_id {
                    println!("Device ID:      {}", device_id);
                }
                if let Some(ref firmware) = ipmi.firmware_version {
                    println!("Firmware:       {}", firmware);
                }
                if let Some(manufacturer_id) = ipmi.manufacturer_id {
                    println!("Manufacturer:   0x{:06x}", manufacturer_id);
                }

                // 显示用户信息
                if !ipmi.users.is_empty() {
                    println!("\nBMC Users:");
                    println!(
                        "{:<8} {:<16} {:<8} {:<12}",
                        "User ID", "Username", "Enabled", "Privilege"
                    );
                    println!("{}", "-".repeat(50));

                    for user in &ipmi.users {
                        let privilege_text = match user.privilege_level {
                            1 => "Callback",
                            2 => "User",
                            3 => "Operator",
                            4 => "Administrator",
                            15 => "No Access",
                            _ => "Unknown",
                        };

                        println!(
                            "{:<8} {:<16} {:<8} {:<12}",
                            user.user_id,
                            user.username,
                            if user.enabled { "Yes" } else { "No" },
                            privilege_text
                        );
                    }
                }
            } else {
                println!("IPMI/BMC is not available or not configured");
            }
        }
        Ok(None) => {
            println!("No IPMI device detected");
        }
        Err(e) => {
            println!("Failed to collect IPMI information: {}", e);
        }
    }
}
